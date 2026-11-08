# 0949. std.audio: sound as data, devices through a host program

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.audio.1-9](../spec/std-audio.md)
- **Builds on:** [ADR 0838](0838-std-app-native-windows.md) (host-program pattern), [ADR 0009](0009-custom-tooling-policy.md)

## Context

AIL has no audio, which blocks games and media apps from replacing C and C++.

## Decision

1. `std.audio` is written in AIL. A `Clip` is interleaved float samples with a rate and channel count; mixing, resampling and WAV are pure functions (part 1, built).
2. Devices (`std.audio.device`, part 2) use one host program per platform, talking PCM over stdin/stdout as `std.app` does: CoreAudio on macOS first, then WASAPI, then PipeWire with ALSA fallback. No AIL code runs on an audio callback.
3. Decoders follow in the order FLAC, Vorbis, MP3. FLAC is built: [ADR 0974](0974-flac-decoding.md). Streaming output: [ADR 0993](0993-audio-streaming-output.md).

## Rejected alternatives

- Audio callbacks into AIL: breaks `ffi.decl.3`, risks GC pauses on the real-time thread.
- SDL, miniaudio or PortAudio: custom tooling by default (ADR 0009).
- Integer sample clips: float keeps DSP simple.
