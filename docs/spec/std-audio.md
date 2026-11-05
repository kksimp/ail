# Standard library: `std.audio` sound

Chapter key: `std`

> **Status:** Draft, no ADR yet (Fable allocates the number). Part 1 (clips,
> mixing, WAV) is written in AIL in `std/audio/audio.ail` and tested by
> `tests/cases/std_audio.ail`. Part 2 (devices) is specified here and not built.

**std.audio.1** — `use std.audio` imports a standard module written in AIL.
A `Clip` has `rate` (Hz), `channels` and `samples`: interleaved floats in
`-1.0..1.0`. Sound is plain data, so an agent can build, inspect and test it
without a device.

**std.audio.2** — `frames(c)` is `samples.len() / channels`; `duration_ms(c)`
is `frames * 1000 / rate`.

**std.audio.3** — `silence(rate, channels, ms)` and `tone(freq, ms, rate, amp)`
(mono sine) make clips.

**std.audio.4** — `gain(c, g)` scales; `peak(c)` is the largest absolute sample.

**std.audio.5** — `concat(a, b)` and `mix(a, b)` need equal rate and channels,
else `AudioError`. `mix` sums, clamps to `-1.0..1.0`, and lasts as long as the longer clip.

**std.audio.6** — `to_mono(c)` averages channels; `resample(c, rate)` is linear.

**std.audio.7** — `decode_wav(b bytes) Clip ! AudioError` reads RIFF/WAVE 16-bit
PCM; other chunks are skipped; 32-bit float is refused for now.
`encode_wav(c) bytes` writes 16-bit PCM, clamping.

## Devices (specified, not built)

**std.audio.8** — `std.audio.device`: `play(clip) Handle`, `Handle.stop()`,
`Handle.wait()`, `open_output(rate, channels) Out` with `write(clip)`, and
`open_input(rate, channels) In` with `read(ms) Clip`. A `Handle` is unique and
must be waited, as tasks are (ADRs 0095-0096). Effect: `audio`.

**std.audio.9** — Backends follow `std.app` (ADR 0838): a small host program per
platform owns the real-time audio thread and exchanges PCM with AIL over its
stdin and stdout, so no AIL code runs on an audio callback. Hosts: macOS
(CoreAudio, AudioQueue/AUHAL, first), Windows (WASAPI shared mode, event
driven), Linux (PipeWire, ALSA fallback). Mixing of several `play` calls happens
in AIL's mixer thread, off the audio thread, with a ring buffer in the host.
Target latency is under 20 ms. Mixing and DSP are SIMD and multi-core
candidates for the auto-parallel pass.

## Rejected alternatives

- Callbacks from the audio thread into AIL: break `ffi.decl.3` and risk GC pauses in real time.
- Linking SDL, miniaudio or PortAudio: custom tooling by default (ADR 0009).
- Integer sample clips: float keeps mixing and DSP simple; WAV encode converts.

## Open for Fable

ADR number; MP3/OGG/FLAC decoders (FLAC first, lossless and simple, then
Vorbis, then MP3); spatial audio; f32 sample storage for tokens and speed
once `List[f32]` is native-fast.
