# 1009. Readiness I/O: epoll and kqueue sets for the HTTP server

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.net`), native runtime
- **Spec rules:** std.net.32
- **Criterion that decided:** speed at many connections (a tie on tokens and on AI understanding, since no API changes)

## Context

ADR 0420 chose non-blocking sockets and named a readiness API as the planned replacement for the sleep. On main (60fabc4b) a single wait is already one `poll(2)` call with the connection's timeout, and on a deterministic schedule it polls with 0 and 1 ms waits between task turns. What still grows with the number of connections is `Server.accept` (net_serve.ail): every pass rebuilt a `pollfd` array of every parked keep-alive connection, scanned the whole socket table for deadlines, and let the kernel scan the array again. With 10,000 idle connections every request paid for all of them.

## Options considered

### Option A — a readiness set per listener, armed once per park (chosen)
`epoll` on Linux (x86-64 and ARM64), `kqueue` on macOS. The listener is registered level-triggered; each parked connection is armed one-shot (`EPOLLONESHOT`, `EV_ONESHOT`) when it is parked and reported once. Deadlines are swept every 50 ms instead of every pass.
- **Pros:** a wait costs O(ready), not O(parked); one-shot reporting means two tasks sharing a server never take the same connection and a connection being read is not reported again; no API change.
- **Cons:** two system calls per park (arm) on top of the read; a reported connection that is not taken must be armed again (done in `serve_accept`); three per-OS structure layouts (`struct epoll_event` is packed on x86-64 only).

### Option B — one `epoll`/`kqueue` for every wait, including a single connection
- **Pros:** one code path.
- **Cons:** a one-off client wait pays to create or arm a set; `poll` on one descriptor is the cheaper call. Rejected.

### Option C — edge-triggered sets
- **Pros:** fewer wake-ups.
- **Cons:** a missed edge hangs a connection; the parked-connection protocol already knows when to re-arm. Rejected.

## Decision

Option A, in `compiler/rt/native/ready.ail` (`rdy_new`, `rdy_listen`, `rdy_arm`, `rdy_wait`). The set is made on the first `accept` that would block and holds the listener plus every connection already parked. `serve_park` arms its connection; a graceful shutdown uses the old `poll` path (it needs the count of idle connections and is rare). Windows keeps `poll` through `wnet_poll`; IOCP is the second slice of this ADR. Single-descriptor waits (`net_wait`) keep `poll(2)` and the deterministic-schedule relax loop (conc.sched.3), which `rdy_wait` copies.

## Consequences

- A socket object grows by two words, `NET_HANDLE` and `NET_POLLER`.
- `net_close` on a listener closes its set.
- No new runtime global slot.
- Not measured here: builds and tests run on the designer's machines. The 10,000-connection measurement belongs to the integrator's run.

## Rejected alternatives

- A set for every wait (option B): a set costs more than `poll` on one descriptor.
- Edge-triggered sets (option C): a missed edge hangs a connection.
- Keeping `poll` and only trimming the table scan: the kernel still copies and scans every parked descriptor on each wait.
