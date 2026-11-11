# 1045. The attach listener on Windows, and how it stops

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 20 — AI semantic API
- **Spec rules:** `attach.wire.1`, `attach.wire.7`, `attach.cli.1`
- **Builds on:** [ADR 0884](0884-observe-field-and-attach-protocol.md) (attach), [ADR 0598](0598-windows-imports-and-system-calls.md) (Winsock imports)

## Context

Host parity for v0.1.0 lists `ail attach` as missing on Windows. Reading the tree showed the
socket was not the gap: `net.listen` and `accept` already reach Winsock. Three things were:

1. `std.attach.serve` published the discovery file under `%TEMP%` on Windows, but `ail attach`
   looked only in `$TMPDIR`, else `/tmp`, so it never found a program.
2. When the window closed, the listener task was left in `accept` and its socket was closed from
   another thread. Whether that ends the `accept` differs by system, and `run` waits for the task.
   A host that failed to start returned with the task still running.
3. No test drove the listener over a socket, on any host.

## Decision

1. **Directory.** Program and client use `TMPDIR`, else `TEMP`, else `/tmp`. On Windows `TEMP` is
   the temporary folder inside the user's profile; its access list is the owner-only protection,
   so no mode is set there. Unix keeps mode 0600.
2. **Stopping (`attach.wire.7`).** After the window closes or fails to open, `run` sets a stop
   flag, closes the reply channel (a waiting request gets an empty line) and connects to its own
   port once. The listener sees the flag at that connection, closes its socket and ends. If the
   connection can't be made, the socket is closed from outside as before.
3. **Two halves.** `serve.open()` gives the listener and its file, `serve.run_on(s, mut app, cmd,
   args, input)` runs the window with a given host program. `run` is the two with the built host.
   A case (`tests/cases/std_attach_listen`) uses them with a host written in AIL, so the listener
   is tested headless on the three hosts.

## Rejected alternatives

- **A Windows-only directory such as `%LOCALAPPDATA%\ail`.** Safer against a redirected `TEMP`,
  but one more rule to remember and a directory to create; `TMPDIR` can be redirected on Unix in
  the same way. Tie on tokens, worse for understanding.
- **`SO_EXCLUSIVEADDRUSE` on the Windows listener.** Windows already refuses another user's bind
  to a port bound without `SO_REUSEADDR`, which the Windows path never sets; the same user can
  read the token file anyway. It would also have changed the runtime during the freeze.
- **Ending `accept` by closing the socket only.** What the code did. It depends on the system and
  cannot be relied on from a second thread.
- **A shutdown request on the wire.** A client could then stop the listener; the flag is in
  process and needs no token.
- **A poll with a timeout in the listener.** Wakes an idle program several times a second for a
  case that happens once.

## Consequences

`ail attach` finds and drives a Windows program. A window's close no longer depends on how the
system treats a socket closed under `accept`. `std.attach.serve` gains `Served`, `open` and
`run_on`, and `Hub` a `stop` field.
