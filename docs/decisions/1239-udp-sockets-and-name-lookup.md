# 1239. `std.net` gains UDP sockets and a public name lookup

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); lane 20 of the standard library plan of 2026-11-26
- **Topic:** Standard library
- **Spec rules:** std.net.73, std.net.74, std.net.75, std.net.76; std.net.14 amended
- **Builds on:** [ADR 0421](0421-raw-tcp-and-tls-conn.md), [ADR 0422](0422-net-serve-capability.md), [ADR 0598](0598-windows-imports-and-system-calls.md)

## Context

`std.net` had TCP, HTTP and WebSocket but no datagrams, so a program could not
speak DNS, syslog, StatsD, NTP, mDNS or a game protocol, and could not ask what
addresses a name has. The runtime already had both halves privately: the DNS
client of `net_dns.ail` opens a UDP socket, and `net_resolve` returns the first
address of a name. Windows needs `sendto` and `recvfrom`, which the PE import
table already names.

## Options considered

### Option A — a `Udp` handle with `send_to` and `recv(timeout_ms)`, and `lookup` (chosen)
- `net.udp(addr) Udp ! NetError`, `u.send_to(addr, data) ! NetError`,
  `u.recv(timeout_ms) Datagram? ! NetError`, `u.addr()`, `u.close()`;
  `net.lookup(host) List[str] ! NetError`.
- **Pros:** one handle type shaped like `Conn` (same `addr`, `close`, error
  kinds); the timeout is in the call, so a polling loop needs no second call;
  "nothing arrived" is an absent value, not an error a program must tell apart
  from a real failure; `Datagram` names its two parts (`data`, `from`).
- **Cons:** a timeout argument on every call, even for a server that waits
  forever (it passes `-1`).

### Option B — `recv` fails with kind `TimedOut`
- **Pros:** the same as `Conn.read`.
- **Cons:** a timeout is the normal outcome of a UDP wait (retries, polling); an
  error for it pushes every caller into an `else err` that must test the kind.

### Option C — a tuple `(bytes, str)` from `recv`, as the plan sketched
- **Cons:** positional parts are easy to swap; a named struct reads better and
  costs no tokens at the use site (`d.data`, `d.from`).

### Option D — `net.udp` needs `net.serve` like `listen`
- **Cons:** every client (a DNS or StatsD client binds port 0) would need the
  server capability. Instead the static effects are `net` and `io`, and the
  runtime policy asks for `net.serve` only when a fixed port is bound.

## Decision

Option A. `recv` keeps one 64 KiB buffer per socket. `lookup` returns every
`A` record: the DNS client now collects all addresses of an answer, and on
macOS the whole `getaddrinfo` list is walked; `connect` and the rest still use
the first. IPv4 only, like the rest of `std.net`. Windows uses Winsock
`sendto` and `recvfrom` (`win_net.ail`), and ignores the `WSAECONNRESET` that
a port-unreachable reply to an earlier send leaves on the socket.

## Rationale

AI understanding first: the names are the ones agents type (`udp`,
`send_to`, `recv`, `lookup`), the shapes match `Conn`, and an optional result
says exactly when nothing came. No syntax changes; the runtime pieces already
existed.

## Consequences

- `net_sys_dgram` now sets macOS's own `O_NONBLOCK` value (it passed Linux's,
  which macOS ignores; the DNS client is not used on macOS, so it never showed).
- A recording marks UDP and `lookup` calls as uncaptured, so a replay stops at
  them (replay.cap.2), as for any call without a recorder.
- Not done: IPv6, multicast and broadcast options, connected UDP sockets,
  `AAAA`, `MX` or `TXT` queries, a TCP fallback for long DNS answers.

## Rejected alternatives

- `recv` failing with `TimedOut`: makes the common outcome an error (Option B).
- A tuple result: unnamed parts (Option C).
- `net.serve` for every UDP socket: wrong for clients (Option D).
- Only the first address from `lookup`: a program choosing between addresses
  or checking a name's records needs them all.
