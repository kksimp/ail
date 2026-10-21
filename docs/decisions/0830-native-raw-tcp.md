# 0830. Native raw TCP: sockets by system calls, handles in a table

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture; standard library
- **Spec rules:** [native.subset.1](../spec/23-native.md), std.net.4, std.net.12
- **Builds on:** [ADR 0804](0804-native-error-values-are-structs-and-enums.md), [ADR 0580](0580-os-calls-by-system-calls-with-a-status.md), [ADR 0421](0421-raw-tcp-and-tls-conn.md)

## Context

`std.net` is the largest standard module the native subset lacks. The examples
that use it (`fetch`, `poller`, `digest`, `loopback`, Garlic) need an HTTP
client and server, which rest on raw TCP. The interpreter implements it with the
Rust standard library; the native runtime has no libc and no socket code.

## Decision

`compiler/rt/native/net.ail` implements `net.listen`, `net.connect`, and the
methods of `net.Listener` (`accept`, `addr`, `close`) and `net.Conn` (`read`,
`read_line`, `write`, `set_timeout`, `peer`, `close`) by system calls (`socket`,
`connect`, `bind`, `listen`, `accept`, `getsockname`, `getpeername`,
`setsockopt`, `getsockopt`, `fcntl`, `poll` or `ppoll`, `sendto`, `read`), the
numbers per target in `net_sys_*` so a libSystem switch changes one place.
`compiler/mir/lower_net.ail` lowers the calls (no change to `lower.ail`).

- **Handles.** A `Conn` and a `Listener` are structs of one `int` field, as in
  the checker's `std/types.ail`. The field is an index into a table of socket
  objects, a list that global slot 40 keeps alive. Entries are never reused,
  so a stale handle cannot reach a new socket. A socket object holds its
  descriptor, timeout, read buffer (16 KiB, doubled for a longer line), flags,
  peer and address.
- **Errors.** `net.NetError(url, kind, message)` is a standard error struct like
  `fs.IoError` (ADR 0804), kinds `bad_url`, `not_found`, `refused`,
  `timed_out`, `closed`, `bind`, `other`. A system call's `errno` maps to a kind
  by target (Linux and macOS number them differently). The message wording is
  the runtime's own; the `kind` matches the interpreter.
- **Waiting.** Every socket is non-blocking and a wait is `poll` with the
  connection's timeout (default 30000 ms); `accept` waits without a limit. A
  wait does not yet let other tasks run (std.net.8); it will when native tasks
  land (#194).
- **Text.** `read` returns up to `max` bytes without splitting a character and
  turns bytes that aren't UTF-8 into U+FFFD by the same maximal-subpart rule as
  the interpreter; `read_line` strips `\n` or `\r\n`.
- **Addresses.** IPv4 only: `A.B.C.D:PORT`, `localhost`, and names in
  `/etc/hosts`. Any other host fails with kind `not_found`: there is no DNS
  client, no IPv6 and no TLS yet.

## Rejected alternatives

- **Blocking sockets with a `SO_RCVTIMEO`.** A timeout then lacks the poll's
  resolution for `connect` and cannot yield to tasks later.
- **The descriptor as the handle.** A stale handle could reach a new socket
  after the kernel reuses the descriptor, and the buffer and timeout need a
  home.
- **Writing the module in AIL source, like `std.log`.** `net` is a builtin
  module with builtin structs; the checker and the interpreter know its
  functions by name, and the lowering already maps standard modules to
  runtime calls.
