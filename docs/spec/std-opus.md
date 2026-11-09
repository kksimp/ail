# Standard library: `std.audio.opus` Opus decoding

Chapter key: `std`

> **Status:** Draft (ADR 0990). Written in AIL in `std/audio/opus.ail` and
> tested by `tests/cases/std_audio_opus.ail`. Part 1 (framing and the range
> decoder) is written; CELT, SILK and hybrid follow.

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
