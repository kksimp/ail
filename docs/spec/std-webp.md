# Standard library: `std.image.webp`

Chapter key: `std`

> **Status:** Drafted. Written in AIL in `std/image/webp.ail` (ADR 1289, and ADR 1298 for `std.webp.11-20`) and tested by
> `tests/cases/std_webp_decode.ail`, `std_webp_lossy.ail`, `std_webp_alpha.ail`, `std_webp_anim.ail` and `std_webp_errors.ail`.

WebP decoding into the `image.Image` of [std-image](std-image.md): lossless (the WebP Lossless Bitstream specification,
VP8L), lossy (VP8 key frames, RFC 6386) with an alpha plane (`ALPH`), and animation (`ANIM` and `ANMF`). Every function is
pure (`eff.infer.1`); the module is in scope without `use` and is named `webp`. It reads no files: pass it
`fs.read_bytes(path)?`. There is no encoder. The decoded pixels equal what libwebp gives (`dwebp`, and `anim_dump` for the
canvas of an animation) for every file the tests and the Python mirror used to build them cover.

## The module

**std.webp.1** — `webp` provides `decode`, `decode_animation`, `info`, `is_webp`, and the types `WebpError`, `WebpKind`, `Info`, `Frame` and `Animation`. Details: [below](#more-on-stdwebp1). *(ADR 1289)*

**std.webp.2** — `info(data bytes) Info ! WebpError` reads the RIFF container and the first image header and gives `width`,
`height`, `lossless`, `alpha` and `animated`. Details: [below](#more-on-stdwebp2). *(ADR 1289)*

```ail tool=run,FILE expect=true,3x2,true,false,false rules=std.webp.1,std.webp.2
fn main() !
    b := bytes.from_hex("524946461c000000574542505650384c0f0000002f02400000000862ddbb89fe47ff5300") else return
    inf := webp.info(b)?
    print("{webp.is_webp(b)},{inf.width}x{inf.height},{inf.lossless},{inf.alpha},{inf.animated}")
```

**std.webp.3** — `decode(data bytes) image.Image ! WebpError` returns the image as RGBA8 (`std.image.1`), alpha included
whatever the alpha hint says. The chunks after the 12-byte header are walked in order (a chunk's data is padded to an even
size) until the first `VP8L`, `VP8 ` or `ANMF` chunk; `VP8X` (its flags and canvas size) and every other chunk (`ICCP`,
`EXIF`, `XMP `, `ANIM`) are skipped, and a chunk past the RIFF size fails with `Truncated`. A file with no image chunk fails
with `BadChunk`; a VP8X canvas that differs from the image size with `Corrupt`; an image of more than `image.MAX_PIXELS`
pixels with `Unsupported`. A lossy image is decoded by `std.webp.11` to `std.webp.17`, its `ALPH` chunk by `std.webp.18`, and
for an animated file `decode` returns the first frame as `std.webp.19` composes it. *(ADR 1289)*

```ail tool=run,FILE expect=ff8000ffff8000ffff8000ffff8000ffff8000ffff8000ff rules=std.webp.3
fn main() !
    b := bytes.from_hex("524946461c000000574542505650384c0f0000002f02400000000862ddbb89fe47ff5300") else return
    print(webp.decode(b)?.pixels.hex())
```

**std.webp.4** — A VP8L stream is read as bits, low bit first: the signature byte `0x2f`, 14 bits of width - 1 and of
height - 1, the alpha hint bit and 3 version bits (0). Details: [below](#more-on-stdwebp4). *(ADR 1289)*

**std.webp.5** — Pixels are coded with prefix codes. Details: [below](#more-on-stdwebp5). *(ADR 1289)*

**std.webp.6** — The pixel stream has an optional color cache of 1 to 11 bits, and (in the main image only) a meta prefix
image whose blocks of 2^k pixels pick one of the groups of five prefix codes (green with length and cache symbols, red,
blue, alpha, distance). Details: [below](#more-on-stdwebp6). *(ADR 1289)*

**std.webp.7** — The predictor transform adds, to every pixel, a prediction from its neighbours L, T, TR and TL, chosen per
block by the mode in the green channel of its sub-image: 0 black (`0xff000000`), 1 L, 2 T, 3 TR, 4 TL, 5 to 10 averages of
the neighbours, 11 the closer of L and T to the gradient, 12 and 13 the clamped gradients, and 14 and 15 black. Details: [below](#more-on-stdwebp7). *(ADR 1289)*

**std.webp.8** — The cross-color transform adds to red `(g2r * green) >> 5` and to blue `(g2b * green) >> 5` plus
`(r2b * red) >> 5`, with every factor read as a signed byte and red the new value; the three factors are the
channels of the block's sub-image pixel. Details: [below](#more-on-stdwebp8). *(ADR 1289)*

**std.webp.9** — The color-indexing transform replaces each index, in the green channel and packed 1, 2, 4 or 8 to a
pixel, by its palette entry; the palette is stored with each entry added to the one before, and an index past it gives
transparent black. *(ADR 1289)*

```ail tool=run,FILE expect=fafafaff000000ff rules=std.webp.4,std.webp.9
fn main() !
    b := bytes.from_hex("524946461c000000574542505650384c0f0000002f07c00000c5e847f43f669867980100") else return
    img := webp.decode(b)?
    print("{img.pixels[0..4].hex()}{img.pixels[8..12].hex()}")
```

**std.webp.10** — A file that is not valid fails with `WebpError`, never a trap: `NotWebp` (no `RIFF`/`WEBP` header), `Truncated`
(a chunk, the VP8L data, a VP8 partition or an alpha plane ends early), `BadChunk` (a VP8X chunk below 10 bytes, no image
chunk, an ANMF chunk that is too short, has no VP8X or ANIM chunk before it, or has a frame that does not fit the canvas),
`Corrupt` (a bad VP8L signature, a bad prefix code, a reference or cache index out of range, a repeated transform, a
canvas that differs from the image, a VP8 frame that is not a key frame or lacks its start code, a bad `ALPH` header) and
`Unsupported` (`std.webp.2`, `std.webp.3`, a VP8 frame that is not meant to be shown). Details: [below](#more-on-stdwebp10). *(ADR 1289)*

```ail tool=run,FILE expect=NotWebp: rules=std.webp.10
fn main()
    img := webp.decode(b"not a webp") else err
        print("{err.kind.name()}: {err.why}")
        return
    print(img.width)
```

### `std.webp` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

## Lossy images

**std.webp.11** — A `VP8 ` chunk holds one VP8 key frame (RFC 6386). Its 3-byte frame tag gives the key-frame bit (0), a version
of 0 to 3, the show-frame bit and the size of the first partition; then come the start code `9d 01 2a` and the 14-bit width
and height (the top 2 bits of each, the scale, are ignored). The image size is the one in the frame header and equals
the VP8X canvas when there is one (`Corrupt` otherwise). An inter frame, a missing start code or a zero size fails with
`Corrupt`; a frame that is not to be shown with `Unsupported`; a first partition, a partition table or a token
partition that runs past the chunk with `Truncated`. Each partition is read by the boolean arithmetic decoder (an
8-bit range, bytes shifted in one at a time, a bit is 0 with the probability `p / 256`); it gives zero bits past the end
of its partition, and once every macroblock is decoded a partition that was read past its end fails with `Truncated`. *(ADR 1298)*

**std.webp.12** — The first partition begins with the frame header: the color space and clamping type (which do not change
the output), the optional segment header (an update of the segment map probabilities and of the four segment quantizers and
filter levels, absolute or relative), the loop filter type, level and sharpness with the optional reference and mode
deltas, the number of token partitions (1, 2, 4 or 8), the base quantizer index with its five deltas, the token
probability updates and the optional skip probability. The index of each segment is clamped to 0 to 127 and read through
the DC and AC tables of the specification; the second-order DC is doubled, the second-order AC is scaled by 155 / 100 and
at least 8, and the chroma DC index is at most 117. Macroblock row `r` reads its tokens from partition `r mod n`; the last
partition runs to the end of the chunk. *(ADR 1298)*

**std.webp.13** — Per macroblock the first partition holds its segment (when the map is updated), its skip flag (when the
skip probability is present), the 16x16 luma mode (DC, V, H, TM, or the ten 4x4 sub-block modes with one mode for each
block, each coded with the modes above and to the left as context), and the chroma mode (DC, V, H, TM). The context of
the sub-block modes of a macroblock that is not in 4x4 mode is the mode that equals its 16x16 mode. *(ADR 1298)*

**std.webp.14** — The residual is read from the macroblock's token partition as tokens of 4x4 blocks: for the second-order
luma DCs (16x16 modes only), the 16 luma blocks, and the 4 + 4 chroma blocks. Each token is coded by the probability table
of its plane, its band (the table of the 17 positions), and its context (the number of nonzero neighbours above and to
the left, then 0, 1, or more than 1 for the previous token), in zigzag order, with the extra-bit categories for large
values and a sign bit. A skipped macroblock has no tokens and clears its neighbour contexts, except that a macroblock with
second-order data keeps the second-order context. The second-order block goes through the inverse Walsh-Hadamard
transform to the DC of each luma block, and each block through the inverse DCT (35468 and 85627 fixed point, with the
shortcut `(dc + 4) >> 3` for a block that has only a DC), and is added to the prediction and clamped to 0 to 255. *(ADR 1298)*

**std.webp.15** — A macroblock is predicted from its already decoded neighbours. The row above the first macroblock row reads
127 and the column left of the first macroblock column 129 (the corner as in libwebp). The 16x16 luma and the 8x8 chroma blocks
use DC (the mean of the available neighbours, 128 with none), TM (the clamped gradient `left + above - above_left`), V (copy
the row above) or H (copy the column to the left). A 4x4 luma block uses one of the ten sub-block modes (B_DC, B_TM, B_VE, B_HE,
B_RD, B_VR, B_LD, B_VL, B_HD, B_HU) of RFC 6386. The four pixels above and to the right of a block come from the row above the
macroblock (the last pixel repeated at the right edge of the frame), also for the blocks below the first row of the
macroblock. *(ADR 1298)*

**std.webp.16** — After the whole frame is reconstructed, the loop filter runs macroblock by macroblock in raster order, as
libwebp does: the type is the simple or the normal filter of the frame header; the level is the segment's or the frame's, plus
the reference and mode deltas, clamped to 0 to 63; the interior limit comes from the level and the sharpness; and a
macroblock with no residual and a whole-block prediction mode skips its inner edges. The filter changes the luma (and, in the
normal filter, the chroma) planes in place, before the conversion to RGB. A frame with level 0 is not filtered. *(ADR 1298)*

**std.webp.17** — Colors are converted from `Y`, `U`, `V` to RGB with libwebp's fixed-point constants: with `yy = (19077 y) >> 8`,
red is `(yy + ((26149 v) >> 8) - 14234) >> 6`, green `(yy - ((6419 u) >> 8) - ((13320 v) >> 8) + 8708) >> 6` and blue
`(yy + ((33050 u) >> 8) - 17685) >> 6`, each clamped to 0 to 255. The chroma planes are upsampled by libwebp's "fancy" triangle
filter, as `dwebp` does by default: two luma rows at a time, a sample is `(9a + 3b + 3c + d + 8) >> 4` of the four nearest chroma
samples (`a` the nearest), as two rounded halves in libwebp's order, and the first and last columns use `(3a + b + 2) >> 2`. The
alpha of an image without an `ALPH` chunk is 255. *(ADR 1298)*

```ail tool=run,FILE expect=8ef409ff8ef409ff8ef409ff8ef409ff rules=std.webp.11,std.webp.12,std.webp.13,std.webp.14,std.webp.15,std.webp.16,std.webp.17
fn main() !
    b := sample()
    print(webp.decode(b)?.pixels.hex())

fn sample() bytes
    return bytes.from_hex("524946463c000000574542505650382030000000d001009d012a0200020001402625a00274ba01f80003b000fef3c267ff15bfe39b7c736ff8adfff8b39188ecbc000000") else b""
```

## Alpha

**std.webp.18** — An `ALPH` chunk next to a `VP8 ` chunk (in a VP8X file) holds the alpha plane of a lossy image: a header
byte and then the data. The header's bits 0 to 1 are the compression method (0 raw bytes, 1 a lossless VP8L stream without its
header, coding the plane as the green channel of a `width x height` image), bits 2 to 3 the prediction filter (0 none, 1
horizontal, 2 vertical, 3 gradient), bits 4 to 5 the pre-processing (level quantization, which does not change the
decoding) and bits 6 to 7 are reserved and must be 0. A method above 1 or a reserved bit set fails with `Corrupt`, an
empty chunk or a raw plane with fewer than `width * height` bytes with `Truncated`. The prediction filters are undone per
pixel and modulo 256: the first pixel adds 0, the first row adds the pixel to the left, the first column adds the pixel above,
and the others add (horizontal) the pixel to the left, (vertical) the pixel above, or (gradient)
`clamp(left + above - above_left, 0, 255)`. Only the first `ALPH` chunk is used. The alpha is written unchanged into the RGBA
pixels; the RGB values of a transparent pixel are kept as the lossy decoder made them, not zeroed and not premultiplied. A
lossless (`VP8L`) image has its alpha in its own stream (`std.webp.4`) and an `ALPH` chunk next to it is ignored. *(ADR 1298)*

```ail tool=run,FILE expect=127 rules=std.webp.18
fn main() !
    b := sample()
    img := webp.decode(b)?
    print("{img.pixels[3]} {img.pixels[27]} {img.pixels[51]} {webp.info(b)?.alpha}")

fn sample() bytes
    return bytes.from_hex("52494646e000000057454250565038580a000000100000000c0000080000414c5048760000000000152a3f556a7f94aabfd4e9ff00152a3f556a7f94aabfd4e9ff00152a3f556a7f94aabfd4e9ff00152a3f556a7f94aabfd4e9ff00152a3f556a7f94aabfd4e9ff00152a3f556a7f94aabfd4e9ff00152a3f556a7f94aabfd4e9ff00152a3f556a7f94aabfd4e9ff00152a3f556a7f94aabfd4e9ff5650382044000000d001009d012a0d00090002804225b00274010ed38b200000fee87c34b684d7d2969e8865dafc3a85bff98c7c3468731918848a03f5967b9affc29577fff079917f7cd000") else b""
```

## Animation

**std.webp.19** — `decode_animation(data bytes) Animation ! WebpError` gives every frame of the file as the whole canvas.
`Animation` has `width` and `height` (the VP8X canvas), `loops` (the `ANIM` chunk's loop count, 0 for forever) and
`frames`, a `List[Frame]`; a `Frame` has `img`, an `image.Image` of the canvas size, and `delay_ms`, the frame's duration in
milliseconds. A file that is not animated gives one frame of the image with a delay of 0 and `loops` 1. The `ANIM`
background color is ignored, as libwebp's `AnimDecoder` does. An `ANMF` chunk is the frame x and y offsets (24 bits each, in
units of 2 pixels), its width - 1 and height - 1 (24 bits each), its duration (24 bits) and a flags byte (bit 0 dispose to
background, bit 1 do not blend), followed by the frame's own chunks: an optional `ALPH` and a `VP8 `, or a `VP8L`. The frame
image must be the size the `ANMF` says (`Corrupt`) and the frame must fit the canvas (`BadChunk`); the frames together may
not hold more than `image.MAX_PIXELS` pixels (`Unsupported`). A file with `ANMF` chunks needs a `VP8X` chunk and an `ANIM`
chunk before the first of them (`BadChunk`; `info` fails the same way without `VP8X`). The canvas starts transparent black.
Each frame is drawn onto it and then copied to the result. If the previous frame was disposed to the background, its
rectangle is cleared to transparent black first. A frame is drawn without blending when it has the no-blend flag, or when
it is a key frame: the first frame, or a frame that covers the whole canvas and either has no alpha or follows a frame
disposed to the background that covered the whole canvas (libwebp's rule). A frame has alpha when its `VP8L` header says so
or it has an `ALPH` chunk. `decode` of an animation returns the first of these canvases. *(ADR 1298)*

**std.webp.20** — A blended frame pixel with alpha 0 leaves the canvas pixel as it is. Otherwise, with the source `s` over the
canvas `d`, not premultiplied, `da = d.a * (256 - s.a) >> 8`, `a = s.a + da` and each color channel is
`(s.c * s.a + d.c * da) * (16777216 / a) >> 24` (libwebp's `BlendPixelNonPremult`). Inside the rectangle that the previous
frame disposed to the background there is nothing to blend with, and the frame's pixels are copied. A frame with the
no-blend flag is copied. *(ADR 1298)*

```ail tool=run,FILE expect=698690623 rules=std.webp.19,std.webp.20
fn main() !
    b := sample()
    a := webp.decode_animation(b)?
    print("{a.frames.len()} {a.loops} {a.width}x{a.height} {hash.crc32(a.frames[3].img.pixels)}")

fn sample() bytes
    return bytes.from_hex("524946469801000057454250565038580a00000012000000070000070000414e494d06000000ffffffff0100414e4d46540000000000000000000700000700001e0000015650384c3b0000002f07c00100993244f43f3611d1ff8042b611a0319d3fc5317d0a0281249cfdb1a650d3b611d4f53d1ce53f1fb89800ff842c00f384c31206a6f80100414e4d46640000000100000100000300000300001e0000015650384c4b0000002f03c000107f40906d53a6f9339ce53884d946c3e9f3fc490e474ddb465030dcfaf3e77518b20e607c170050c9e572150aa42502a1923f1004402c68f1a0459b03311eec44f43f40abcb0100414e4d465e0000000000000000000300000300001e0000005650384c450000002f03c000105740986d54a62338ffc7f808b26d8a737f9a519cc1840035a769dae37b5d3a9dce7f80bf2774c6343581499e016a028060ec75e9a090565ef123fa1f5fd7362b00414e4d463e0000000300000300000100000100001e0000025650384c260000002f014000001f20164cf180ba12081072fcd7081022164cf6e62f19c6fc475f0c0cd621a2ff21") else b""
```
=======
### `std.webp` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.webp.1

`is_webp(data bytes) bool` says whether `data` is at least 12 bytes and starts with `RIFF` and has `WEBP` at byte 8.

#### More on std.webp.10

It has the fields `kind`, a `WebpKind`, and `why`, the reason as text, and shows as `webp: ` and the reason. A file of 2^27 pixels can need no bits at all, so a small file can ask for a long decode; the limit is `image.MAX_PIXELS`.

#### More on std.webp.2

For a lossless image the size and the alpha hint come from the VP8L header; for a lossy one from the VP8 frame header (`lossless` is false); for an animation, from the VP8X canvas (`animated` is true). `alpha` is also true when the VP8X flags say so. A bad VP8L signature byte fails with `Corrupt`, a VP8L version other than 0 with `Unsupported`.

#### More on std.webp.4

Then come the transforms, each led by a 1 bit and a 2-bit type (predictor 0, cross color 1, subtract green 2, color indexing 3), each used at most once (`Corrupt` otherwise), ended by a 0 bit, and then the pixels. The transforms are undone in the opposite order to the one they were read in. Color indexing narrows the width of the later steps to `ceil(width / 2^k)`, with `k` 3, 2, 1 or 0 for palettes of up to 2, 4, 16 or more colors. A read past the end of the data gives zero bits and, at the next check, a `Truncated` failure.

#### More on std.webp.5

A code is simple (1 or 2 symbols of 1 or 8 bits) or normal (code lengths 0 to 15 coded by a code of 19 symbols, in the order 17, 18, 0 to 5, 16, 6 to 15, with repeats 16, 17 and 18 and an optional maximum symbol count). A code with one symbol takes no bits; any other code must fill the code space exactly, or the file fails with `Corrupt`. Codes are canonical, shorter first and by symbol within a length, and the first bit read is the first bit of the code.

#### More on std.webp.6

A green symbol below 256 is a literal pixel (then red, blue and alpha), 256 to 279 a backward reference (a length and a distance, each a prefix symbol with extra bits; a distance code of 1 to 120 is one of the 120 two-dimensional offsets `x + y * width`, at least 1, and above 120 it is the distance minus 120), and the rest a color-cache entry. A reference that starts before the first pixel or runs past the last fails with `Corrupt`.

#### More on std.webp.7

The first pixel predicts from black, the rest of the first row from L, the first column from T, and the right-most pixel's TR is the first pixel of its own row. All sums are per channel, modulo 256.

#### More on std.webp.8

The subtract-green transform adds green to red and blue.
>>>>>>> bc575fcf8f118dc20fb32e73e79108f29d1289e7

#### More on std.webp.1

`is_webp(data bytes) bool` says whether `data` is at least 12 bytes and starts with `RIFF` and has `WEBP` at byte 8.

#### More on std.webp.10

It has the fields `kind`, a `WebpKind`, and `why`, the reason as text, and shows as `webp: ` and the reason. A file of 2^27 pixels can need no bits at all, so a small file can ask for a long decode; the limit is `image.MAX_PIXELS`.

#### More on std.webp.2

For a lossless image the size and the alpha hint come from the VP8L header; for a lossy one from the VP8 frame header (`lossless` is false); for an animation, from the VP8X canvas (`animated` is true). `alpha` is also true when the VP8X flags say so. A bad VP8L signature byte fails with `Corrupt`, a VP8L version other than 0 with `Unsupported`.

#### More on std.webp.4

Then come the transforms, each led by a 1 bit and a 2-bit type (predictor 0, cross color 1, subtract green 2, color indexing 3), each used at most once (`Corrupt` otherwise), ended by a 0 bit, and then the pixels. The transforms are undone in the opposite order to the one they were read in. Color indexing narrows the width of the later steps to `ceil(width / 2^k)`, with `k` 3, 2, 1 or 0 for palettes of up to 2, 4, 16 or more colors. A read past the end of the data gives zero bits and, at the next check, a `Truncated` failure.

#### More on std.webp.5

A code is simple (1 or 2 symbols of 1 or 8 bits) or normal (code lengths 0 to 15 coded by a code of 19 symbols, in the order 17, 18, 0 to 5, 16, 6 to 15, with repeats 16, 17 and 18 and an optional maximum symbol count). A code with one symbol takes no bits; any other code must fill the code space exactly, or the file fails with `Corrupt`. Codes are canonical, shorter first and by symbol within a length, and the first bit read is the first bit of the code.

#### More on std.webp.6

A green symbol below 256 is a literal pixel (then red, blue and alpha), 256 to 279 a backward reference (a length and a distance, each a prefix symbol with extra bits; a distance code of 1 to 120 is one of the 120 two-dimensional offsets `x + y * width`, at least 1, and above 120 it is the distance minus 120), and the rest a color-cache entry. A reference that starts before the first pixel or runs past the last fails with `Corrupt`.

#### More on std.webp.7

The first pixel predicts from black, the rest of the first row from L, the first column from T, and the right-most pixel's TR is the first pixel of its own row. All sums are per channel, modulo 256.

#### More on std.webp.8

The subtract-green transform adds green to red and blue.
