# Standard library: `std.image.webp`

Chapter key: `std`

> **Status:** Drafted. Written in AIL in `std/image/webp.ail` (ADR-TBD) and tested by `tests/cases/std_webp_decode.ail`
> and `std_webp_errors.ail`.

Lossless WebP decoding (the WebP Lossless Bitstream specification, VP8L) into the `image.Image` of
[std-image](std-image.md). Every function is pure (`eff.infer.1`); the module is in scope without `use` and is named
`webp`. It reads no files: pass it `fs.read_bytes(path)?`. There is no encoder, and lossy (VP8) and animated files are not
decoded.

## The module

**std.webp.1** — `webp` provides `decode`, `info`, `is_webp`, and the types `WebpError`, `WebpKind` and `Info`.
`is_webp(data bytes) bool` says whether `data` is at least 12 bytes and starts with `RIFF` and has `WEBP` at byte 8. *(ADR-TBD)*

**std.webp.2** — `info(data bytes) Info ! WebpError` reads the RIFF container and the first image header and gives `width`,
`height`, `lossless`, `alpha` and `animated`. For a lossless image the size and the alpha hint come from the VP8L header;
for a lossy one from the VP8 frame header (`lossless` is false); for an animation, from the VP8X canvas (`animated` is
true). `alpha` is also true when the VP8X flags say so. A bad VP8L signature byte fails with `Corrupt`, a VP8L version
other than 0 with `Unsupported`. *(ADR-TBD)*

```ail tool=run,FILE expect=true 3x2 true false false rules=std.webp.1,std.webp.2
fn main() !
    b := bytes.from_hex("524946461c000000574542505650384c0f0000002f02400000000862ddbb89fe47ff5300") else return
    inf := webp.info(b)?
    print("{webp.is_webp(b)} {inf.width}x{inf.height} {inf.lossless} {inf.alpha} {inf.animated}")
```

**std.webp.3** — `decode(data bytes) image.Image ! WebpError` returns the image as RGBA8 (`std.image.1`), alpha included
whatever the alpha hint says. The chunks after the 12-byte header are walked in order (a chunk's data is padded to an even
size) until the first `VP8L`, `VP8 ` or `ANMF` chunk; `VP8X` (its flags and canvas size) and every other chunk (`ICCP`,
`EXIF`, `XMP `, `ANIM`) are skipped, and a chunk past the RIFF size fails with `Truncated`. A file with no image chunk fails
with `BadChunk`; a VP8X canvas that differs from the VP8L size with `Corrupt`; a lossy or animated file, or an image of
more than `image.MAX_PIXELS` pixels, with `Unsupported`. *(ADR-TBD)*

```ail tool=run,FILE expect=ff8000ffff8000ffff8000ffff8000ffff8000ffff8000ff rules=std.webp.3
fn main() !
    b := bytes.from_hex("524946461c000000574542505650384c0f0000002f02400000000862ddbb89fe47ff5300") else return
    print(webp.decode(b)?.pixels.hex())
```

**std.webp.4** — A VP8L stream is read as bits, low bit first: the signature byte `0x2f`, 14 bits of width - 1 and of
height - 1, the alpha hint bit and 3 version bits (0). Then come the transforms, each led by a 1 bit and a 2-bit type
(predictor 0, cross color 1, subtract green 2, color indexing 3), each used at most once (`Corrupt` otherwise), ended by a 0 bit,
and then the pixels. The transforms are undone in the opposite order to the one they were read in. Color indexing narrows
the width of the later steps to `ceil(width / 2^k)`, with `k` 3, 2, 1 or 0 for palettes of up to 2, 4, 16 or more colors.
A read past the end of the data gives zero bits and, at the next check, a `Truncated` failure. *(ADR-TBD)*

**std.webp.5** — Pixels are coded with prefix codes. A code is simple (1 or 2 symbols of 1 or 8 bits) or normal (code
lengths 0 to 15 coded by a code of 19 symbols, in the order 17, 18, 0 to 5, 16, 6 to 15, with repeats 16, 17 and 18 and an
optional maximum symbol count). A code with one symbol takes no bits; any other code must fill the code space exactly,
or the file fails with `Corrupt`. Codes are canonical, shorter first and by symbol within a length, and the first bit read
is the first bit of the code. *(ADR-TBD)*

**std.webp.6** — The pixel stream has an optional color cache of 1 to 11 bits, and (in the main image only) a meta prefix
image whose blocks of 2^k pixels pick one of the groups of five prefix codes (green with length and cache symbols, red,
blue, alpha, distance). A green symbol below 256 is a literal pixel (then red, blue and alpha), 256 to 279 a backward
reference (a length and a distance, each a prefix symbol with extra bits; a distance code of 1 to 120 is one of the 120
two-dimensional offsets `x + y * width`, at least 1, and above 120 it is the distance minus 120), and the rest a
color-cache entry. A reference that starts before the first pixel or runs past the last fails with `Corrupt`. *(ADR-TBD)*

**std.webp.7** — The predictor transform adds, to every pixel, a prediction from its neighbours L, T, TR and TL, chosen per
block by the mode in the green channel of its sub-image: 0 black (`0xff000000`), 1 L, 2 T, 3 TR, 4 TL, 5 to 10 averages of
the neighbours, 11 the closer of L and T to the gradient, 12 and 13 the clamped gradients, and 14 and 15 black. The first pixel
predicts from black, the rest of the first row from L, the first column from T, and the right-most pixel's TR is the first
pixel of its own row. All sums are per channel, modulo 256. *(ADR-TBD)*

**std.webp.8** — The cross-color transform adds to red `(g2r * green) >> 5` and to blue `(g2b * green) >> 5` plus
`(r2b * red) >> 5`, with every factor read as a signed byte and red the new value; the three factors are the
channels of the block's sub-image pixel. The subtract-green transform adds green to red and blue. *(ADR-TBD)*

**std.webp.9** — The color-indexing transform replaces each index, in the green channel and packed 1, 2, 4 or 8 to a
pixel, by its palette entry; the palette is stored with each entry added to the one before, and an index past it gives
transparent black. *(ADR-TBD)*

```ail tool=run,FILE expect=fafafaff000000ff rules=std.webp.4,std.webp.9
fn main() !
    b := bytes.from_hex("524946461c000000574542505650384c0f0000002f07c00000c5e847f43f669867980100") else return
    img := webp.decode(b)?
    print("{img.pixels[0..4].hex()}{img.pixels[8..12].hex()}")
```

**std.webp.10** — A file that is not valid fails with `WebpError`, never a trap: `NotWebp` (no `RIFF`/`WEBP` header), `Truncated`
(a chunk or the VP8L data ends early), `BadChunk` (a VP8X chunk below 10 bytes, or no image chunk), `Corrupt` (a bad VP8L
signature, a bad prefix code, a reference or cache index out of range, a repeated transform, a canvas that differs
from the image) and `Unsupported` (`std.webp.2`, `std.webp.3`). It has the fields `kind`, a `WebpKind`, and `why`, the reason
as text, and shows as `webp: ` and the reason. A file of 2^27 pixels can need no bits at all, so a small file can ask
for a long decode; the limit is `image.MAX_PIXELS`. *(ADR-TBD)*

```ail tool=run,FILE expect=NotWebp: the data does not start with RIFF and WEBP rules=std.webp.10
fn main()
    img := webp.decode(b"not a webp") else err
        print("{err.kind.name()}: {err.why}")
        return
    print(img.width)
```
