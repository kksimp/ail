# 1286. `std.net.grpc`: gRPC over HTTP/2

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** std.net.60-72

## Context

Agents asked for gRPC services get stuck: there is no gRPC in std. Message bodies come from the protobuf module and
the transport from the HTTP/2 module (`serve_streams`: `read`, `write`, `finish(trailers)`, `reset(code)`).

## Decision

- `std/net/grpc.ail`, in AIL, used with `use std.net.grpc` (always written, like `std.net.websocket`).
- Layer 1 (this change): codes, metadata, 5-byte message framing with a chunk-joining `Decoder`, `grpc-timeout`,
  `grpc-message` escaping, request, response and trailer blocks, mapping of non-gRPC HTTP statuses.
- Layer 2 (this change): a server on `http2.serve_streams`: one handler shape `fn(Call) ! GrpcError` per
  `/pkg.Service/Method` covers unary and all streaming kinds; the status is the handler's error. Checked against
  Python grpcio by `tools/grpc-interop/run.sh`.
- A client (std.net.70-72) in the same file: the runtime's HTTP/2 client (`net_h2.ail`) does one whole request per
  TLS connection and exposes no streams or trailers, so the client is a small AIL HTTP/2 client over `net.Conn`
  (h2c; TLS needs a `Conn` connect that offers `h2`, a follow-up). One call at a time per client.
  Deadlines use `time.mono_ns()` on the client (RST_STREAM CANCEL, `DeadlineExceeded`).
- Not yet: server-side deadline cancellation (`Call.timeout_ms` is exposed; `time.mono_ns` exists, but cancelling a
  running handler needs a task API), compression, TLS client.
- No compression in this version: a compressed message fails with `Unimplemented`.

## Rejected alternatives

- Writing a private HTTP/2 inside the module: duplicates the HTTP/2 lane.
- Code generation from `.proto` files: the protobuf module's scope; services take a path and two byte functions.

## Verification

Built in a container with the tree's own stage-1 compiler and run natively: `net_grpc`, `net_grpc_server`,
`net_grpc_client` and `net_http2` match their goldens, and `tools/grpc-interop/run.sh` passes against Python grpcio
1.85 in both directions (AIL server with a grpcio client; AIL client with a grpcio server: unary, server, client
and bidirectional streaming, trailers, error status with a non-ASCII message, an unknown method, a 1 MB message, a
client deadline). Found on the way: `Channel` is a builtin type name (the client type is `Client`); a fallible
method called on a `ref` receiver or a field of one did not lower natively (`recv_ty` and `method_key` in
`compiler/mir/lower_err.ail` now see through `ref S`); `http2.answer` spawned a task per request and deadlocked
when `run`'s connection tasks held the pool, so it answers in order on the connection's task.
