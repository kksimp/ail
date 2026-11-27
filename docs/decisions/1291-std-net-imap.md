# 1291. `std.net.imap`: an IMAP client and MIME parser

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** [imap.client.1–14, imap.mime.1–9](../spec/std-net-imap.md)
- **Extends:** [ADR 0421](0421-raw-tcp-and-tls-conn.md) (raw `Conn`), [ADR 0360](0360-in-house-tls-for-std-net.md) (TLS client)

## Context

A program that reads mail (a support inbox, a verification-code reader, a backup of a mailbox) had to speak IMAP
over `net.connect_tls` by hand. The two places that go wrong are the reply grammar (literals, parenthesised
lists, `NIL`) and the message format (folded headers, encoded words, transfer encodings, charsets, nested
multiparts). std has sockets and a TLS client, and SMTP sends mail (std wave 3), but nothing reads it.

## Decision

- A module in AIL, `std/net/imap.ail`, used with `use std.net.imap` (always written, like `std.net.websocket`). No
  runtime change: `Conn.read_line` and `read_bytes` are enough, and literals are read as exact byte counts.
- One tagged-command engine (`exec`) serves every command. It reads replies as items: a line with the literals it
  announces kept inline as bytes, so a parser sees one value per reply. A cursor type (`Scan`) parses an item and
  records a malformed reply in a flag, so each parser reads straight through and fails once.
- Replies become values: `MailboxInfo`, `Selected`, `Fetched` with `Envelope`, `Event`. Request arguments are
  checked before sending (message sets, flags, criteria, items), so no argument can add a command line.
- `idle(wait_ms)` is one round of the wait, not a callback: it returns the reports, so the caller's loop decides
  what to do between rounds and nothing needs a second task.
- A MIME parser in the same file, `parse_mail`, which flattens nested multiparts into leaves with section numbers
  and decodes transfer encodings, RFC 2047 words, RFC 2231 parameters and four charsets. It never fails.
- A password is refused on an unencrypted connection unless `allow_insecure_auth` (as `std.net.smtp`).
- Failures are values: `ImapError { kind, message }` with an enum `kind` (type.enum.7). No syntax change.

## Rejected alternatives

- **A separate `std.net.mime` module.** The MIME parser is the half of this work that mail readers need whether or
  not they use IMAP, but a second module costs a registration set and a collision with the SMTP lane's builder
  today. It can be split out later by moving the functions; the names stay.
- **STARTTLS waiting for a later PR.** Done here instead, stacked on the SMTP lane's `Conn.start_tls` (`std.net.50`):
  `Security.StartTls` upgrades after the greeting and asks for the capabilities again (imap.client.15). A server
  that lacks the capability fails the connect; there is no silent fall back to the clear.
- **A tree of nested parts.** A recursive struct is awkward in AIL today and most readers want the text and the
  attachments; a flat list with section numbers answers both and matches what `BODY[1.2]` takes.
- **`idle` with a callback or a `Seq`.** Both hide a timeout loop that the caller usually wants to own; a round
  that returns a list is the smallest thing that works.
- **Parsing `BODYSTRUCTURE` into values.** `parse_mail` on the whole message answers the same questions with one
  fetch; the structure is kept as text for callers that want to pick a part before downloading it.
- **Charsets beyond four.** Tables for the CJK charsets are large; UTF-8, US-ASCII, ISO-8859-1 and Windows-1252
  cover most mail, and `Part.charset` tells the caller when the text is lossy.
- **Mailbox names as UTF-8 only.** RFC 3501 servers send modified UTF-7; decoding it is 40 lines and makes `INBOX`
  and `Entwürfe` look the same to the caller.

## Consequences

Tests are a native program over loopback with a scripted server (`tests/cases/std_net_imap.ail`); TLS needs a TLS
server, which std lacks (`std.net.13`), so `Security.Implicit` is untested here. IDLE over TLS also relies on a read
that times out leaving the TLS stream usable; it is checked only in the clear. The module was written without a
build; the first gate run is its first compile. ADR number and the index row come from the integrator; replace
`ADR 1291` in `std/net/imap.ail`, `docs/spec/std-net-imap.md` and `docs/status.md`.
