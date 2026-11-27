# Standard library: `std.image.png`

Chapter key: `std`

> **Status:** Drafted. Written in AIL in `std/image/png.ail` (ADR 1250; APNG, ADR TBD) and tested by `tests/cases/std_png_decode.ail`,
> `std_png_errors.ail`, `std_png_roundtrip.ail`, `std_apng_decode.ail`, `std_apng_errors.ail` and `std_apng_roundtrip.ail`.

PNG decoding and encoding (ISO 15948, the W3C PNG Third Edition) into the `image.Image` of [std-image](std-image.md).
Zlib comes from `compress` ([std.compress.7](17-std.md)) and CRC-32 from `hash` (`std.hash.3`). Every function is pure
(`eff.infer.1`); the module is in scope without `use` and is named `png`. It reads no files: pass it `fs.read_bytes(path)?`.

## The module

**std.png.1** — `png` provides `decode`, `encode`, `info`, `is_png`, `decode_animation`, `encode_animation`, `is_animated`, and the types `PngError`, `PngKind`, `Info`, `ColorType`, `Frame` and
`Animation`. `is_png(data bytes) bool` says whether `data` starts with the 8-byte PNG signature. *(ADR 1250)*

**std.png.2** — `info(data bytes) Info ! PngError` reads only the signature and the IHDR chunk, checks IHDR's CRC-32 and
header, and gives `width`, `height`, `color` (`Gray`, `Rgb`, `Palette`, `GrayAlpha`, `Rgba`), `depth` (1, 2, 4, 8 or 16) and
`interlaced`. A header outside the specification fails with `BadHeader`: a side of 0 or above 2^31 - 1, a color
type other than 0, 2, 3, 4 or 6, a bit depth the color type does not allow (gray 1 to 16, RGB 8 and 16, palette 1 to 8,
gray with alpha and RGBA 8 and 16), a compression or filter method other than 0, or an interlace method other than 0 and 1.
An image of more than `image.MAX_PIXELS` pixels fails with `Unsupported`. *(ADR 1250)*

```ail tool=run,FILE expect=2x1 rules=std.png.1,std.png.2
fn main() !
    b := bytes.from_hex("89504e470d0a1a0a0000000d4948445200000002000000010806000000f4227f8a0000001149444154789c63f8cfc0f09fa1e1bf030010ba03be479620a00000000049454e44ae426082") else return
    inf := png.info(b)?
    print("{inf.width}x{inf.height} {inf.color.name()} {inf.depth}")
```

**std.png.3** — `decode(data bytes) image.Image ! PngError` returns the image as RGBA8 (`std.image.1`). Details: [below](#more-on-stdpng3). *(ADR 1250)*

```ail tool=run,FILE expect=ff0000ff0080ff40 rules=std.png.3
fn main() !
    b := bytes.from_hex("89504e470d0a1a0a0000000d4948445200000002000000010806000000f4227f8a0000001149444154789c63f8cfc0f09fa1e1bf030010ba03be479620a00000000049454e44ae426082") else return
    print(png.decode(b)?.pixels.hex())
```

**std.png.4** — Every color type and bit depth is read: gray (1, 2, 4, 8, 16 bits), RGB (8, 16), palette (1, 2, 4, 8),
gray with alpha (8, 16) and RGBA (8, 16). Details: [below](#more-on-stdpng4). *(ADR 1250)*

**std.png.5** — Transparency: a palette image takes each entry's alpha from the `tRNS` chunk (0 to 255, entries beyond it
are opaque, and more entries than `PLTE` has fail with `BadChunk`); a gray image takes the one gray sample of `tRNS` as fully transparent, and an RGB image the one color of `tRNS`.
The key is compared before scaling, at the file's bit depth. A `tRNS` chunk with a wrong size fails with
`BadChunk`; on gray-with-alpha and RGBA images it is ignored. A palette image needs a `PLTE` chunk of 3 to 768 bytes
in threes (`BadChunk`), and an index past it fails with `Corrupt`. A `PLTE` chunk in a gray image fails with `BadChunk`. *(ADR 1250)*

**std.png.6** — The five filter types (none, sub, up, average, Paeth) are undone with the byte distance of one pixel
(at least 1) as the specification gives. Details: [below](#more-on-stdpng6). *(ADR 1250)*

**std.png.7** — Adam7 interlaced images are read: the seven passes, each with its own filtered rows, are placed
in the image. A pass with no pixels (a small image) has no rows in the data. *(ADR 1250)*

**std.png.8** — A PNG that is not valid fails with `PngError`, never a trap: `NotPng` (no signature), `Truncated` (the
data ends inside a chunk or before IEND, or the image data is short), `BadChunk` (a chunk of the wrong order, size or number: IHDR not
first or repeated, a missing IDAT or PLTE, a chunk claiming over 2^31 - 1 bytes), `BadHeader` (`std.png.2`), `Checksum`
(a chunk's CRC-32), `Corrupt` (invalid zlib data, an unknown filter, a palette index out of range) and `Unsupported`. *(ADR 1250)*

```ail tool=run,FILE expect=NotPng: rules=std.png.8
fn main()
    img := png.decode(b"not a png") else err
        print("{err.kind.name()}: {err.why}")
        return
    print(img.width)
```

**std.png.9** — `encode(img image.Image) bytes ! PngError` writes a PNG file: IHDR, IDAT chunks of up to 1 MiB, IEND. The
image is written as 8-bit RGB when every pixel's alpha is 255, otherwise as 8-bit RGBA, not interlaced. Details: [below](#more-on-stdpng9). *(ADR 1250)*

```ail tool=run,FILE expect=true rules=std.png.9
fn main() !
    img := image.solid(3, 2, 0x336699cc)?
    print(png.decode(png.encode(img)?)? == img)
```

**std.png.10** — `PngError` has the field `kind`, a `PngKind` (`NotPng`, `Truncated`, `BadChunk`, `BadHeader`, `Checksum`,
`Corrupt`, `Unsupported`, `BadImage`) and `why`, the reason as text. It shows as `png: ` and the reason. *(ADR 1250)*

**std.png.11** — `decode_animation(data bytes) Animation ! PngError` returns the frames of an APNG file. `Animation` has `width`,
`height`, `loops` (the play count of `acTL`; 0 repeats forever) and `frames`, a `List[Frame]`; a `Frame` has `img`, the whole
canvas as an RGBA8 `image.Image` as it looks when the frame is shown, and `delay_ms`, `delay_num * 1000 / delay_den` (a
denominator of 0 means 100). A PNG with no `acTL` chunk before its image data is one frame with delay 0 and `loops` 1.
`is_animated(data bytes) bool` is true for a valid PNG with an `acTL` before its first `IDAT`, and false for anything else. `decode`
of an APNG gives its default image. Details: [below](#more-on-stdpng11). *(ADR TBD)*

```ail tool=run,FILE expect=2:2:100:ff0000ff00ff00ff:200:ff0000ff0000ffff rules=std.png.11
fn main() !
    b := bytes.from_hex("89504e470d0a1a0a0000000d4948445200000002000000010806000000f4227f8a000000086163544c00000002000000021d83f25c0000001a6663544c00000000000000020000000100000000000000000001000a0000f929b6790000001149444154789c63f8cfc0f09fe13fc37f0010f803fd417079e80000001a6663544c00000001000000010000000100000001000000000001000500000bde42f9000000116664415400000002789c636060f8ff1f00030201fff57ba5d70000000049454e44ae426082") else return
    anim := png.decode_animation(b)?
    a := anim.frames[0]
    b := anim.frames[1]
    print("{anim.frames.len()}:{anim.loops}:{a.delay_ms}:{a.img.pixels.hex()}:{b.delay_ms}:{b.img.pixels.hex()}")
```

**std.png.12** — The chunks are read as the APNG specification gives. `acTL` (8 bytes, at least 1 frame) must come before the
first `IDAT`; later it is ignored. `fcTL` (26 bytes) and `fdAT` carry sequence numbers that start at 0 and rise by 1 across both
chunk kinds. A frame's rectangle must lie inside the canvas, its dispose op is 0 to 2 and its blend op 0 or 1, the number of `fcTL`
chunks is `acTL`'s frame count, and every frame has image data. An `fcTL` before the `IDAT` makes the default image the
first frame, and it must cover the canvas; without one the default image is not part of the animation. Each frame is decoded
as a still image of its own size with the file's color type, depth, palette, `tRNS` and interlace (`std.png.3` to `std.png.7`).
Anything else fails with `BadChunk`, `Corrupt` or the errors of `std.png.8`, never a trap; frames times canvas pixels above
`image.MAX_PIXELS` fail with `Unsupported`. *(ADR TBD)*

**std.png.13** — The canvas starts transparent black. Each frame is drawn at its rectangle with blend op 0 (replace the pixels) or
1 (draw over, not premultiplied, rounded to nearest, see below), the canvas is shown as the frame, and then the frame's rectangle
is disposed: op 0 keeps it, 1 makes it transparent black, 2 restores what the canvas held before the frame was drawn (op 2 on the first
frame acts as op 1). Details: [below](#more-on-stdpng13). *(ADR TBD)*

**std.png.14** — `encode_animation(anim Animation) bytes ! PngError` writes an APNG file. Every frame must be `anim.width` by
`anim.height` with `width * height * 4` pixel bytes, `delay_ms` must be 0 to 65535, `loops` 0 to 4294967295, and there are 1 to 65535
frames, else `BadImage`. The first frame is also the default image, so a viewer without APNG shows it; each later frame is
cropped to the smallest rectangle that differs from the frame before it (1 x 1 if none does) with blend op 0 and dispose op 0.
The file is 8-bit RGB when every pixel of every frame is opaque and RGBA otherwise. `decode_animation(encode_animation(a)?)? == a`
for every valid `a`. *(ADR TBD)*

```ail tool=run,FILE expect=true rules=std.png.14
fn main() !
    a := image.solid(3, 2, 0xff0000ff)?
    mut px := List[u8]()
    for _ in 0..6
        px.push_u8(0)
        px.push_u8(0)
        px.push_u8(255)
        px.push_u8(255)
    b := image.image(3, 2, bytes.from_list(px))?
    anim := png.Animation(width: 3, height: 2, loops: 0, frames: [png.Frame(img: a, delay_ms: 40), png.Frame(img: b, delay_ms: 90)])
    back := png.decode_animation(png.encode_animation(anim)?)?
    print(back.frames[0].img == a and back.frames[1].img == b and back.frames[1].delay_ms == 90)
```

### `std.png` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.png.3

It checks the signature, walks every chunk up to IEND, and verifies each chunk's CRC-32 (`Checksum`). Chunks after IEND are ignored. The IDAT chunks are joined and inflated as one zlib stream, whose Adler-32 is checked. Ancillary chunks (`tEXt`, `pHYs`, `gAMA`, `iCCP` and the rest) are skipped: no gamma or color profile conversion is applied. A critical chunk other than IHDR, PLTE, IDAT and IEND fails with `Unsupported`.

#### More on std.png.4

Samples of fewer than 8 bits are scaled to 0 to 255 (`v * 255 / (2^depth - 1)`), and 16-bit samples keep their high byte, so the result is the 8-bit image a browser shows. Rows are not padded beyond the byte.

#### More on std.png.6

A filter byte above 4 fails with `Corrupt` and the row. Image data shorter than the header needs fails with `Truncated`; extra bytes are ignored.

#### More on std.png.9

Each row uses whichever of the five filters has the smallest sum of its bytes read as signed numbers, and the rows are deflated by `compress.zlib`. `decode(encode(img)?)? == img` holds for every image. An image with a side below 1, more than `image.MAX_PIXELS` pixels or a `pixels` length that is not `width * height * 4` fails with `BadImage`.

#### More on std.png.11

A frame is as wide and high as the canvas, so a program never composes frames itself. The chunks of an APNG are ancillary, so a
program that only knows PNG shows the default image, which is what `decode` returns. `info` reads the canvas size and the color
type of the file. The shape of `Frame` and `Animation` is that of `gif` and `webp`.

#### More on std.png.13

Blend op 1 with source alpha `sa`, canvas alpha `da`, `t = sa * 255 + da * (255 - sa)`: a source alpha of 0 keeps the canvas pixel, 255
replaces it, otherwise alpha is `(t + 127) / 255` and each color `(sc * sa * 255 + dc * da * (255 - sa) + t / 2) / t`. This is the specification's
non-premultiplied "over", rounded to nearest in integers. A `tRNS` or palette alpha is the frame pixel's alpha like any other.
