# 0989. `std.video.vp9`: a VP9 decoder in AIL

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; media
- **Spec rules:** `std.vpx.1`-`std.vpx.6` ([std-vp9.md](../spec/std-vp9.md))
- **Builds on:** ADR 0988 (std.media.webm), [ADR 0009](0009-custom-tooling-policy.md), [ADR 0510](0510-licence-apache-2-with-runtime-exception.md)
- **Requested by:** the designer, who wants Garlic to play YouTube; VP9 + Opus in WebM first (2026-11-08).

## Context

YouTube serves VP9 in WebM to open-source Chromium. Garlic needs a VP9 decoder; AIL has none, and ADR 0010 rules out linking one.

## Decision

`std.video.vp9` decodes profile 0 (8-bit 4:2:0) in AIL, following the VP9 bitstream specification and libvpx (BSD-3-Clause, credited in NOTICE). The tables (default probabilities, scans, quantizers, filters) and the 1-D inverse transforms are taken from libvpx; the rest is written for AIL. The output matches libvpx bit for bit.

- A `Decoder` takes one packet at a time, as WebM blocks carry them (std.media.webm `Frame.data`), including superframes, and returns the shown frames.
- Frames are I420 planes (`bytes`) without padding; `Frame.rgba()` converts for drawing.
- Frames live in one pool of ten slots inside the decoder, so references are indices and no plane is copied until a frame is returned.
- Tile columns are decoded one after another; they share no state, so `par` tiles are the first speed step.
- Correctness was checked before any AIL build: a Python model of the decoder and a mechanical translation of the AIL text both matched libvpx on 18 synthetic clips (key, lossless, inter, odd sizes, AQ segments, two tile columns and rows, error resilient, frame parallel, sharpness, two-pass alternate references with superframes, compound prediction).

## Rejected alternatives

- **ffmpeg's VP9 decoder.** LGPL; AIL is Apache 2.0 (ADR 0510).
- **Calling libvpx through C.** ADR 0010 and the custom-tooling policy (ADR 0009); also no Windows/macOS library to rely on.
- **RGBA output only.** Players want I420 for the video layer and for scaling.
- **Copying planes into each reference slot.** Up to eight copies of every frame.

## Consequences

Not yet supported: profiles 1-3 (4:4:4, 10/12-bit) and references of a different size (scaled prediction); both fail with `Vp9Error`. Speed is the next step: 720p30 on the Mac needs `par` tiles and rows, SIMD prediction and transforms, and partial transforms by end of block. Test vectors are synthetic clips made with libvpx's encoder; no third-party video is in the repository.
