# XXXX. `std.net.smtp`: an SMTP client and MIME builder

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** [smtp.client.1–9, smtp.msg.1–7](../spec/std-net-smtp.md), [std.net.42](../spec/17-std.md)
- **Extends:** [ADR 0421](0421-raw-tcp-and-tls-conn.md) (raw `Conn`), [ADR 0360](0360-in-house-tls-for-std-net.md) (TLS client)

## Context

Programs that send mail (alerts, reports, sign-in codes) had to write the protocol and the MIME rules by hand over
`net.connect`, and the two places they go wrong are non-ASCII headers and dot-stuffing. std has TCP and an in-house
TLS client but no way to upgrade an open connection, which STARTTLS (port 587, the common submission port) needs.

## Decision

- A module in AIL, `std/net/smtp.ail`, used with `use std.net.smtp` (always written, like `std.net.websocket`).
- `Conn.start_tls(host)`: one runtime function (`net_start_tls`) that runs the existing TLS client handshake over an
  open `Conn`. It refuses when bytes are buffered and unread, so nothing sent in the clear can pass for encrypted data.
- A `Client` session: `connect` with `Security.Implicit | StartTls | Plain`, `auth` (PLAIN, else LOGIN), `send`,
  `send_raw`, `quit`. A password is refused on an unencrypted connection unless `allow_insecure_auth` is set.
- A `Message` of plain data and `build`: RFC 2047 encoded words, 7bit or base64 bodies, `multipart/alternative` and
  `multipart/mixed`, a boundary checked against the parts, no header injection (line breaks refused).
- Failures are values: `SmtpError { kind, code, message }` with an enum `kind` (type.enum.7).
- No syntax change.

## Rejected alternatives

- **STARTTLS only inside the module, with no `Conn` method.** The upgrade needs the socket object the runtime owns;
  a method is the one honest place for it, and IMAP, POP3 and FTP need it too.
- **A transport trait now.** One transport exists; add the trait when a second does.
- **Sending a password in the clear by default.** The failure is silent to the user; an opt-in flag is cheap.
- **Quoted-printable output.** One encoder is less code and no mail client needs readable raw bodies.
- **SMTPUTF8.** An ASCII address is enough for the first use; the check can widen later without a signature change.
- **A pipelined `send`.** One command at a time reads simply and works against every server; the cost is latency
  per recipient.

## Consequences

Tests are a native program over loopback with a scripted server (`tests/cases/std_net_smtp.ail`); STARTTLS and
`Implicit` need a TLS server, which std lacks (`std.net.13`), so those two paths are checked only by their failure
cases. The module was written without a build; the first gate run is its first compile. ADR number and the index
row come from the integrator; replace `ADR TBD` in `std/net/smtp.ail`, `docs/spec/std-net-smtp.md` and
`docs/status.md`.
