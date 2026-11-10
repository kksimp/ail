# Standard library: `std.audio.opus` Opus decoding

Chapter key: `std`

> **Status:** Draft (ADR 0990). Written in AIL in `std/audio/opus.ail` and
> tested by `tests/cases/std_audio_opus.ail` and `std_audio_opus_celt.ail`.
> Framing, the range decoder and CELT are written; SILK and hybrid follow.

**std.opus.1** — `use std.audio.opus` imports a standard module written in AIL
that decodes Opus (RFC 6716). Failures are `OpusError`, whose text starts `opus:`.

**std.opus.2** — `parse_packet(bytes) Packet ! OpusError` splits a packet by its
TOC byte and frame-count code (RFC 6716 §3.1-3.2): `config`, `mode` (`Silk`,
`Hybrid`, `Celt`), `bandwidth` in Hz, `samples` per frame at 48 kHz, `stereo`
and `frames`. A packet that breaks §3.4's rules fails. `p.duration()` is the
samples of all frames together.

**std.opus.3** — `decoder(bytes) RangeDecoder` reads one frame with the range
decoder of RFC 6716 §4.1: `decode(ft)` then `update(fl, fh, ft)`,
`decode_bin(bits)`, `bit_logp(logp)`, `icdf(table, ftb)`, `uint(ft)`, raw
`bits(n)` from the frame's end, `tell()` and `tell_frac()` (eighth bits).
`failed` is set when `uint` reads a value out of range. `ilog(x)` is the bits
needed for `x`.

**std.opus.4** — `celt_state(channels) CeltState` is the state a stream keeps
from packet to packet: band energies of the last three frames, each channel's
overlap and post-filter history, the de-emphasis memory and the noise seed.
One state decodes one stream from its first packet; `channels` is 1 or 2.

**std.opus.5** — `decode(mut CeltState, bytes) audio.Clip ! OpusError` decodes
one packet to interleaved 48 kHz samples in -1..1 with `st.channels` channels:
a mono stream feeds both channels of a stereo state and a stereo stream is
averaged for a mono one. CELT frames (RFC 6716 §4.3: energy, bit allocation,
PVQ band shapes, anti-collapse, inverse MDCT, pitch post-filter and
de-emphasis) match libopus's float decoder within 1 LSB at 16 bits. SILK and
hybrid packets fail.

```ail skip=context
mut st := opus.celt_state(2)
clip := opus.decode(mut st, packet) else err
    return err
```
