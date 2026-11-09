# Standard library: `std.video.vp9` VP9 decoding

Chapter key: `std`

> **Status:** Draft (ADR 0989). Written in AIL in `std/video/vp9.ail` and
> tested by `tests/cases/std_video_vp9.ail`.

**std.vpx.1** — `use std.video.vp9` imports a standard module written in AIL
that decodes VP9 profile 0 (8-bit, 4:2:0). A stream it cannot decode fails
with `Vp9Error`, whose text starts `vp9:`: a malformed packet, a profile other
than 0, a missing reference, or a feature not yet supported (std.vpx.5).

**std.vpx.2** — `decoder()` makes a `Decoder`, which keeps the reference
frames, probability contexts and segmentation state between packets.
`d.decode(packet) List[Frame] ! Vp9Error` takes one packet as WebM and IVF
carry it: one frame, or a superframe (several frames with an index at the
end). It returns the frames the packet shows, in order: usually one, none for
a hidden frame, and a copy of a stored frame for `show_existing_frame`.

**std.vpx.3** — A `Frame` has `width`, `height` and I420 planes without
padding: `y` (`width * height` bytes) and `u` and `v` (`cw() * ch()` bytes
each, where `cw()` is `(width + 1) / 2` and `ch()` is `(height + 1) / 2`). Rows
are top to bottom. `space` is the stream's colour space (1 BT.601, 2 BT.709,
the VP9 numbering) and `full_range` its colour range.

**std.vpx.4** — Decoding follows the VP9 bitstream specification: the output
planes equal those of the reference decoder (libvpx) for every profile 0
stream it decodes.

**std.vpx.5** — A key frame or intra-only frame of a new size starts a new
set of references. An inter frame whose references differ in size from it
fails ("references of another size are not supported yet"): scaled motion
compensation is not implemented.

**std.vpx.6** — `f.rgba() bytes` converts a frame to RGBA, 4 bytes a pixel,
rows top to bottom, alpha 255. It uses BT.709 when `space` is 2 and BT.601
otherwise, in studio range (16-235) unless `full_range`, with 14-bit integer
coefficients; each chroma sample covers 2 x 2 pixels.

## Rejected alternatives

- Decoding into RGBA directly: video layers and encoders want I420, and the
  conversion is cheap to call separately.
- Returning frames by callback: a list is simpler to test and fits `par`
  decoding later.

## Examples

```ail rules=std.vpx.1,std.vpx.2,std.vpx.3,std.vpx.4,std.vpx.5,std.vpx.6
use std.video.vp9

fn main()
    mut d := vp9.decoder()
    frames := d.decode(b"\x88") else err
        print(err)                      # vp9: shows a missing frame
        return
    for f in frames
        print("{f.width}x{f.height} {f.y.len()} {f.rgba().len()}")
```
