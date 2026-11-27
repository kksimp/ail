# 1292. `compress.deflate_flush` and compressed WebSocket sends

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** [std.compress.17](../spec/17-std.md), [std.net.52](../spec/17-std.md)
- **Extends:** [ADR 0828](0828-native-compress.md) (native compress), ADR 1292 (router hijack, receive-only permessage-deflate)

## Context

ADR 1292 left permessage-deflate receive only: RFC 7692 7.2.1 needs the sender's DEFLATE stream to end at a sync flush
(no final block) and may keep its window between messages (context takeover). `compress.deflate` always ends with a
final block and starts with an empty window.

## Decision

- **`compress.deflate_flush(data, history) bytes`** (`std.compress.17`). A pure function, no new type: it encodes
  `data` as non-final blocks plus an empty stored block (`00 00 ff ff`), and its matcher also searches the last 32 KiB of
  `history`, the bytes the stream already carries. `cz_deflate` became `cz_deflate_core(data, n, start, final)`, so the
  matcher, block choice and Huffman code are shared with `deflate`; the history is hashed but not written. Pieces joined
  and closed with a final block are one valid stream.
- **Window kept by the caller.** `WebSocket` keeps the last 32 KiB it sent (`hist`) and passes it back; no compressor
  state lives in the runtime.
- **Sends** (`std.net.52`): on a socket that agreed the extension, `send_text` and `send_binary` send one frame with RSV1
  and the flushed bytes less their last four. Context takeover is the default and ends when the client offers
  `server_no_context_takeover` (then `hist` stays empty). The answer no longer carries `server_no_context_takeover`
  unless offered. An offer of `server_max_window_bits` below 15 is passed over, since the window is fixed at 32 KiB.
  The `_parts` senders send one compressed frame.

## Rejected alternatives

- **A streaming `Deflater` handle with `write(data, flush)`.** A new runtime object, and a lifetime for the GC to track,
  for what a pure function and a 32 KiB slice do; the copy of the history per message is cheap next to the match search.
- **Always no context takeover.** Valid and simpler, but a chat of small messages compresses poorly without the window,
  which is the reason to compress.
- **Honouring `server_max_window_bits < 15`.** Needs a distance limit in the matcher for the message too; passing the
  offer over is allowed by RFC 7692 7.1.2.1 and costs only compression.
- **Fragmenting the compressed message by the caller's parts.** The extension compresses messages; one frame is
  correct and the parts' boundaries carry no meaning.

## Consequences

- Never built (no cloud builds); the first gate may need fix-forward. `tests/native/run/compress_flush.ail` has a
  hand-derived golden; `tools/native-net/ws-deflate-check.sh` checks the server against the Python `websockets` client
  and `zlib` and must be run on the Mac or ail-linux.
- The test `std_http_router_upgrade` changed with the answer header and the compressed replies.
