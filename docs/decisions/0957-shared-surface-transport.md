# 0957. Shared-surface transport for `std.gfx`

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI; GPU
- **Spec rules:** `std.gfx.6`-`std.gfx.8` ([std-gfx.md](../spec/std-gfx.md)), `std.app.21`
- **Requested by:** the designer, who wants AIL to replace C, C++ and Rust, with real-time rendering that uses modern hardware well. Design: [gpu-frame-loop.md](../design/gpu-frame-loop.md). The macOS host and the CPU-drawn loop ship first; Linux and Windows hosts follow.

## Context

Pixels and draw data must reach the host without JSON.

## Decision

The program creates a shared memory region of two or three frame buffers (RGBA8, row stride in bytes) and sends one control line `{"v":1,"t":"surface","id","w","h","stride","count",...}`; the host maps it. Per frame the program fills the buffer not shown and sends `{"t":"present","buf":K}`; the host swaps and answers `{"kind":"vsync","t","dt","buf"}`. macOS and Linux: a file in `$TMPDIR` with mode 0600 and a random name, unlinked once both sides have mapped it. Windows: `CreateFileMapping` by handle, with no file path. The program makes the mapping with the C library (`mkstemp`, `ftruncate`, `mmap`, `unlink`) through `extern c fn`, so v1 needs no runtime file; the effects are those of `app.run`. A Windows host will use `CreateFileMapping` and a runtime piece in its own file (ADR 0823).

## Rejected alternatives

- **Base64 in the pipe.** About 11 MB of text per 1080p frame.
- **A world-readable path.** Any local user could read or write the frame.
- **A general `mmap` for user code.** Wider than needed; breaks memory safety (`safe.*`).

## Consequences

Needs file-backed shared mapping in the native runtime on all three targets.
