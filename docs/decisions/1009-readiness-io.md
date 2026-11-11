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

## Amendment (2026-11-10): the Windows set

- **Decided by:** AI (delegated, ADR 0076). **Criterion:** speed at many connections; feature parity of the three hosts for 0.1.0.

The second slice. On Windows a set is an I/O completion port (`compiler/rt/native/win_ready.ail`). Each armed socket gets one `IOCTL_AFD_POLL` request, issued with `NtDeviceIoControlFile` on a helper handle (`\Device\Afd\Ail`, opened with `NtCreateFile`) that is bound to the port. The socket driver completes the request when the socket is readable, has a connection to accept, was shut down or was closed, and `GetQueuedCompletionStatusEx` returns the completions, so a wait is O(ready) as on the other hosts. A request completes once, which is the one-shot arming; the listener's request is issued again at the start of each wait, and a request on a socket that is already readable completes at once, so the listener is level-triggered. A socket closed while armed completes with `AFD_POLL_LOCAL_CLOSE` and is not reported.

Consequences:

- A Windows program that serves imports two functions from `ntdll.dll` (`compiler/obj/pe.ail` lists the DLL). Programs that do not serve import nothing new.
- The interface is the one `select` uses inside Winsock and that libuv, wepoll and mio build on; Microsoft does not document it. If `NtCreateFile` fails, `rdy_new` fails and the server keeps the `WSAPoll` loop (std.net.32).
- Records the kernel writes into live in the process heap, not the collected heap. A set is closed with `rdy_close`; on Windows it is an address, not a descriptor. Closing cancels the outstanding requests and collects them; the 96-byte set itself is kept, because another task may be inside a wait.
- A wait sits in the system for at most 100 ms at a time, so a set closed by another task ends it.
- `tools/native-net/ready_many.ail` parks N connections (default 1,000) for the measurement.

Rejected for Windows:

- **A zero-byte overlapped `WSARecv` per parked connection.** Documented, but a listening socket cannot take one: the listener would need `AcceptEx` (a second DLL and a different accept path from the other hosts) or a second wait that the port cannot join.
- **`WSAPoll` over all parked connections** (what Windows did until now): O(parked) on every wait, the cost this ADR removes.
- **`WSAEventSelect` with `WaitForMultipleObjects`:** 64 handles per wait.
- **Registered I/O (RIO):** a different send and receive path for every socket, not a readiness question.

## Amendment (2026-11-11): the Windows set waits for a seed

**Decided by:** AI (delegated, ADR 0076).

The published seed's PE writer predates the `ntdll.dll` entry in `pe_dll_of`, so it would import `NtCreateFile` and `NtDeviceIoControlFile` from `KERNEL32.dll`, and the seed-built compiler would not load on Windows (0xC0000139). Until a seed that has the mapping is published, `wrdy_new` returns -38 (`WRDY_OFF`) and the two calls are stubbed, so the server takes its `poll` loop on Windows as before. The `pe_dll_of` mapping lands now, so that the next seed carries it; the calls come back in the change that pins that seed.

Rejected alternatives:

- **Publishing a seed before this batch:** a seed upload needs the designer's OK and a gate of its own, and that would hold up the rest of the batch.
- **Looking the calls up at run time with `GetProcAddress`:** the runtime has no indirect C call.
- **Opening AFD through `CreateFileW` on `\\.\GLOBALROOT\Device\Afd` and using `DeviceIoControl`:** this was not tested on ail-win. It also departs from the design that wepoll, mio and libuv share, and that this ADR chose.
