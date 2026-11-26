# 1207. `std.net.websocket`: a WebSocket client and server

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** [std.net.33–41](../spec/17-std.md)
- **Extends:** [ADR 0421](0421-raw-tcp-and-tls-conn.md) (raw `Conn`), [ADR 0360](0360-in-house-tls-for-std-net.md) (TLS client)

## Context

`std.net` has HTTP and raw TCP but no WebSocket, and an agent audit (2026-11-26) listed it as a place agents get
stuck: they reach for a library that doesn't exist and write the framing by hand. `docs/design/not-in-std.txt`
said "not in std".

## Decision

- A module written in AIL, `std/net/websocket.ail`, used with `use std.net.websocket` (like `std.net.jar`, the
  `use` is always written). It needs nothing new in the compiler or runtime: SHA-1 (`hash.sha1_legacy`, the
  handshake's one legitimate use), `bytes.base64`, `crypto.random_bytes`, `Conn.read_bytes` and `write_bytes`.
- Client: `connect("ws://..." | "wss://...")`. Server: `listen(addr)`, `Server.accept()` answering the handshake.
- A message API, not a frame API: `send_text`, `send_binary`, `send_text_parts` and `send_binary_parts` (explicit
  fragmentation), `ping`, `pong`, `recv`, `close`. `recv` joins fragments, answers pings and closes, and enforces the
  RFC's rules (masking, reserved bits, control frames, UTF-8, close codes, a size limit of 16 MiB by default),
  closing with the code the RFC names.
- `recv` returns pings and pongs as messages (after answering), so one loop sees everything; `match m.kind`.
- No syntax change; one error type, `WsError { kind, message }`, with a `WsKind` enum as `NetError` has.

## Rejected alternatives

- **A frame-level API** (`read_frame`, `write_frame`) as the main surface. Every caller would repeat the control-frame
  and fragmentation logic, which is where the mistakes are. The frame code is private.
- **Callbacks** (`on_message`). Colorless tasks and a plain `recv` loop read more locally and need no new feature.
- **Skipping pings in `recv`.** It would hide heartbeats a program may want to count or time.
- **permessage-deflate and subprotocol negotiation now.** Rare in the first uses; both can be added to the handshake
  later without changing these signatures.
- **A TLS server for `wss` serving.** No TLS server exists (`std.net.13`); a `wss` server sits behind a proxy.

## Consequences

Tests are native programs over loopback (`tests/native/run/net_websocket.ail`); nothing touches the network.
Client and server share one frame codec, so a bug shows in both. Windows and Linux use the same code over `Conn`.
