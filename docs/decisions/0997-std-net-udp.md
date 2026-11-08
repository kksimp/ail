# 0997. std.net: UDP sockets

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** `std.net.33` to `std.net.37`
- **Amends:** [ADR 0421](0421-raw-tcp-and-tls-conn.md) and [ADR 0423](0423-net-platforms-and-deferred.md), which deferred UDP
- **Builds on:** ADR 0960 (IPv6 text addresses; on its own branch when this was written)

## Context

WebRTC (ICE, DTLS, SRTP, SCTP: ADRs 0998 to 1002), QUIC, DNS servers, game
networking and service discovery all need datagrams. `std.net` has TCP and HTTP
only; the runtime has a v4-only datagram socket for the DNS client
(`net_sys_dgram`). C, C++, Rust and Python all have UDP in their standard
libraries.

## Decision

Addresses stay text, as in `listen` and `Conn.peer()`: `"HOST:PORT"`, with
brackets for IPv6 (`"[::1]:5000"`). No public address struct is added: it would
cost tokens at every call and ADR 0960 has none.

1. `bind_udp(addr str) Udp ! NetError` opens a datagram socket bound to `addr`
   (`std.net.33`). Port 0 picks a free port; `u.addr()` returns the bound
   `ip:port` text. `"HOST:PORT"` resolves v4 first, then v6, as `connect` does.
   A `"[::]:PORT"` socket accepts v4 as well where the OS allows it.
2. `u.send_to(data bytes, addr str) int ! NetError` sends one datagram and
   returns the byte count (`std.net.34`). A datagram over 65507 bytes fails with
   kind `too_large`. A full send buffer waits, bounded by the timeout.
3. `u.recv_from(max int) Datagram ! NetError` waits for one datagram
   (`std.net.35`). The result is `Datagram { data bytes, from str }`,
   `from` in the text form of rule 1. A datagram longer than `max` is cut to
   `max` and the rest is dropped (the OS's behaviour, made explicit).
4. `u.set_timeout(ms int)` (default 30000; 0 means do not wait) bounds
   `send_to` and `recv_from`; an expired wait fails with kind `timeout`
   (`std.net.36`). A wait is a task yield like TCP's, so other tasks run.
5. `u.close()` ends the socket; later calls and calls blocked on it fail with
   kind `closed` (`std.net.37`). `connect_udp` (a default peer) is not added:
   ICE needs many peers on one socket.

Runtime: `net_sys_socket_fam` gains a type parameter (stream or datagram);
`net_sys_dgram` becomes its v4 datagram case, so the DNS client is unchanged.
Windows v6 sockets stay unsupported until the v6 work lands there (kind
`unsupported` with a diagnostic). Tests use loopback peers only.

## Consequences

- `NetError.kind` gains `too_large`.
- ADR 0998 (STUN/ICE) builds on `Udp`; ADR 1000 (DTLS) wraps it.
- Needs the IPv6 runtime (`net_resolve6`, `net_sockaddr6`) from the branch of
  ADR 0960. Until it lands, the v4 half can ship alone.

## Rejected alternatives

- A public `Addr` struct with host and port fields: rejected on tokens (a field
  access per use) and because ADR 0960 keeps text addresses.
- A connected-only `Udp` (`connect_udp`, `send`, `recv`): rejected because ICE
  and servers talk to many peers on one socket.
- Returning `from` as a separate out parameter: rejected for local reasoning;
  a struct result is one value.
- Waiting for `bytes`-free text datagrams: rejected, datagrams are binary.
