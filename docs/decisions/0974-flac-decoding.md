# 0974. std.audio: decode FLAC in AIL

- **Status:** Accepted
- **Date:** 2026-11-07
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.audio.11](../spec/std-audio.md)
- **Amends:** [ADR 0949](0949-std-audio.md)

## Context

ADR 0949 planned decoders in the order FLAC, Vorbis, MP3. Only WAV was built.

## Decision

`audio.decode_flac(bytes) Clip ! AudioError` decodes FLAC in AIL: STREAMINFO,
every frame, constant, verbatim, fixed and LPC subframes, rice residuals,
wasted bits and the three stereo modes, at 8 to 24 bits. Samples become floats
in `-1.0..1.0`. CRCs and the MD5 are not checked, since a damaged stream
already fails on its sync or structure and checking costs time. The test
decodes a mono and a stereo stream from a reference encoder and compares sums
with the reference decoder's output.

## Rejected alternatives

- Linking libFLAC: custom tooling by default (ADR 0009).
- Verifying CRC16 per frame: slower for little gain in a local decoder; can be added later.
