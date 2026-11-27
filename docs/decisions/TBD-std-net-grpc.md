# TBD. `std.net.grpc`: gRPC over HTTP/2

- **Status:** Proposed (number from the integrator)
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** std.net.60-65 (more with the client and server)

## Context

Agents asked for gRPC services get stuck: there is no gRPC in std. Message bodies come from the protobuf module and
the transport from the HTTP/2 module (`serve_streams`: `read`, `write`, `finish(trailers)`, `reset(code)`).

## Decision

- `std/net/grpc.ail`, in AIL, used with `use std.net.grpc` (always written, like `std.net.websocket`).
- Layer 1 (this change): codes, metadata, 5-byte message framing with a chunk-joining `Decoder`, `grpc-timeout`,
  `grpc-message` escaping, request, response and trailer blocks, mapping of non-gRPC HTTP statuses.
- Layer 2 (follows the HTTP/2 stream API): a client (unary and the three streaming shapes, deadlines) and a server
  (a handler per `/pkg.Service/Method`, status from the handler's error), bound to `http2.Stream`.
- No compression in this version: a compressed message fails with `Unimplemented`.

## Rejected alternatives

- Writing a private HTTP/2 inside the module: duplicates the HTTP/2 lane.
- Code generation from `.proto` files: the protobuf module's scope; services take a path and two byte functions.
