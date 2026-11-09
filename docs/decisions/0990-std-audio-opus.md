# 0990. `std.audio.opus`: an Opus decoder in AIL

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076); codec choice by the designer (VP9 + Opus first, 2026-11-08)
- **Topic:** 17 — Standard library; media
- **Spec rules:** `std.opus.1`-`std.opus.5` so far ([std-opus.md](../spec/std-opus.md))
- **Builds on:** [ADR 0949](0949-std-audio.md), [ADR 0988](0988-std-media-webm.md)

## Context

YouTube sends Opus audio to browsers without proprietary codecs. AIL had no lossy audio decoder.

## Decision

`std.audio.opus` decodes Opus (RFC 6716) in AIL to `audio.Clip` at 48 kHz. It lands in parts, each tested: packet framing and the range decoder (part 1), then CELT (part 2, what most music at YouTube's rates uses), then SILK and hybrid (part 3). Code is written from the RFC; any constants or code taken from libopus (BSD-3-Clause) keep its notice in `NOTICE`. Output must match libopus's float decoder within the RFC 6716 §6 conformance tolerance; reference outputs come from synthetic clips encoded and decoded with libopus.

## Rejected alternatives

- **Linking libopus.** ADR 0009 (custom tooling) and ADR 0010.
- **Fixed-point decoding.** All targets have fast floats; the float decoder is simpler and the reference for conformance.
- **SILK first.** YouTube's Opus streams are mostly CELT.

## Consequences

Packet loss concealment and the encoder are out of scope for now.

Part 2 (2026-11-09): CELT decoding (std.opus.4-5). It was first written as a
Python model of RFC 6716 §4.3 and checked against libopus 1.5.2 on stereo,
mono, 10 ms, 20 ms and transient clips (within 1 LSB at 16 bits, 104-108 dB
SNR), then ported; the test's expected samples come from that model. The
inverse MDCT is a DCT-IV through a mixed-radix FFT of n/2 (radices 4, 2, 3,
5), with its plans computed once as constants. Tables carried from libopus (band edges, allocation,
energy model, pulse cache) keep its BSD-3-Clause notice in `NOTICE`.

Part 3 (2026-11-09): SILK, hybrid and mode switches (std.opus.4-6). The
decoder state becomes `Stream`, which holds the CELT and SILK states; the
CELT-only `CeltState` API of part 2 is gone. SILK follows libopus's
fixed-point decoder (the 64-bit forms of its multiply macros, 32-bit wrapping
kept explicitly), so it can match bit for bit: a Python model of it matched
libopus 1.5.2 exactly on narrow, medium and wide band, 10, 20 and 60 ms mono
clips, and within 1 LSB on stereo and hybrid clips, before the port. SILK's
tables are stored as hex strings decoded once as constants, because the
formatter puts each element of a long list literal on its own line.
Low-bitrate redundancy (LBRR) is read and skipped: it only matters for loss
recovery. A switch with no redundant CELT frame fails instead of running
packet-loss concealment; libopus's encoder sends that frame on most switches,
so normal streams rarely hit this.
Rejected: SILK in floating point. It is shorter, but it can't match libopus
exactly, and the fixed-point form makes the tests deterministic down to the
bit.
