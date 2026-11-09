# 0988. `std.media.webm`: WebM and Matroska demuxing in AIL

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; media
- **Spec rules:** `std.webm.1`-`std.webm.8` ([std-webm.md](../spec/std-webm.md))
- **Builds on:** [ADR 0949](0949-std-audio.md), [ADR 0009](0009-custom-tooling-policy.md)
- **Requested by:** the designer, who wants Garlic to play YouTube and every format Chromium plays. The designer chose VP9 + Opus in WebM first (2026-11-08).

## Context

YouTube sends open-source Chromium VP9 video and Opus audio in WebM, fed to Media Source Extensions in pieces. AIL had no container reader, and every later codec (VP9, Opus, AV1, Vorbis) needs one.

## Decision

`std.media.webm` is written in AIL. A `Demuxer` takes bytes in pieces (`push`) and returns the coded frames now complete, each with track, time in nanoseconds and key flag; it enters Segment and Cluster without their whole data, so unknown sizes (live streams) and MSE media segments work, and skips unwanted elements without buffering them. `ready` marks the end of the init segment. `read` handles a whole file. All three lacings are read. `write` builds a WebM file, so tests and programs can make media with no outside tool. Decoding is left to the codec modules.

## Rejected alternatives

- **A DOM-like tree of all elements.** Holds a whole file in memory and cannot read a stream in pieces.
- **Demuxing inside Garlic.** Other programs (media player, editor, ADR 0899) need the same reader.
- **libwebm or ffmpeg.** ADR 0009 (custom tooling) and ADR 0010.

## Consequences

Encrypted tracks are flagged, not decrypted (EME is out of scope). CRC-32 elements are skipped, not checked.
