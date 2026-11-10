# Standard library: `std.media.webm` WebM demuxing

Chapter key: `std`

> **Status:** Draft (ADR 0988). Written in AIL in `std/media/webm.ail` and
> tested by `tests/cases/std_media_webm.ail`.

**std.webm.1** — `use std.media.webm` imports a standard module written in AIL
that reads WebM and Matroska (RFC 8794 EBML). Malformed input fails with
`WebmError`, whose text starts `webm:`.

**std.webm.2** — A `Track` has `number`, `kind` (`"video"`, `"audio"` or
`"other"`), `codec` (the Matroska codec ID, such as `"V_VP9"` or `"A_OPUS"`),
`private` (CodecPrivate), `width` and `height`, `rate` and `channels`,
`default_ns`, `delay_ns` and `preroll_ns` (0 when absent) and `encrypted`.
`blank_track()` has every field empty.

**std.webm.3** — A `Frame` has `track`, `ns` (presentation time in
nanoseconds: the cluster's timecode plus the block's, times the timecode
scale), `key` and `data` (the coded bytes). A SimpleBlock's key flag is its
own; a BlockGroup's frame is key when it has no ReferenceBlock. Xiph, EBML
and fixed lacing are split into one frame each; frame `i` of a lace is at the
block's time plus `i` times the track's `default_ns`.

**std.webm.4** — `new()` makes a `Demuxer`; `d.push(bytes) List[Frame] !
WebmError` takes the next bytes and returns the frames completed, in stream
order. Feeding the same bytes in any split gives the same frames. Segment and
Cluster may have unknown sizes; any other element with an unknown size fails.
`ready` is true once Tracks is read; `tracks`, `scale`, `duration_ns`,
`consumed` (bytes taken) and `segment_at` (offset of the segment's data) are
fields. `d.track(n)` gives the track numbered `n`.

**std.webm.5** — The Cues become `cues`: each `Cue` has `ns`, `track` and
`pos` (cluster offset from `segment_at`).

**std.webm.6** — `read(bytes) Media ! WebmError` reads a whole file: `tracks`,
`duration_ns`, `cues` and `frames`; it fails when there is no Tracks element.
`m.of(n)` is track `n`'s frames and `m.first(kind)` the first track of a kind.

**std.webm.7** — Unsigned, signed, float (4 or 8 bytes) and string elements
are big-endian as RFC 8794 says; strings drop trailing zero bytes.

**std.webm.8** — `write(tracks, frames) bytes` makes a WebM file: millisecond
timecodes, one SimpleBlock per frame in the order given, and a new cluster at
each video key frame and whenever a block's time would not fit 16 bits.
`read(write(t, f))` gives back `t`'s fields that WebM carries and `f` at
millisecond precision.

```ail skip=context
mut d := webm.new()
for f in d.push(chunk)?
    if (d.track(f.track)?).codec == "A_OPUS"
        audio_frames.push(f.data)
```
