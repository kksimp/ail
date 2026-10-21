# 0784. Std error families: compress and net client causes

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), on the coordinator's direction
- **Topic:** 17 — Standard library (`std.compress`, `std.net`, `err.type.3`)
- **Spec rules:** `std.compress.9`–`11`, `std.net.29`

## Context

[ADR 0783](0783-std-error-families.md) fixed the approach: `kind` stays a
`str` and each module's closed list is a set of spec rules with a checked
example. `std.compress.2` named the kinds but left the messages as "names the
defect", and `std.net.4` named `bad_url` and `bad_request` without a rule that
says when they fire.

## Decision

- `std.compress.9` lists the DEFLATE `corrupt` messages, `std.compress.10`
  the gzip and zlib wrapper messages, `std.compress.11` the zip messages.
  Each message is exactly one of the listed strings (a zip entry name or
  method number is the only variable part).
- `std.net.29` states that `bad_url` and `bad_request` fail before any
  connection is opened.
- The rule paragraphs are the `ail explain` text. Source comments cite them.

## Rejected alternatives

- **An enum for `kind` or the message.** Rejected in ADR 0783.
- **One rule for every compress message.** A reader of a zip failure would
  load the DEFLATE list too; three rules by layer keep each explain short.
- **Listing the server-side net causes here.** They belong to `serve` and are
  already in `std.net.10`.
