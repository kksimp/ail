# Standard library: `std.image.png`

Chapter key: `std`

> **Status:** Drafted. Written in AIL in `std/image/png.ail` (ADR 1250) and tested by `tests/cases/std_png_decode.ail`,
> `std_png_errors.ail` and `std_png_roundtrip.ail`.

PNG decoding and encoding (ISO 15948, the W3C PNG Third Edition) into the `image.Image` of [std-image](std-image.md).
Zlib comes from `compress` ([std.compress.7](17-std.md)) and CRC-32 from `hash` (`std.hash.3`). Every function is pure
(`eff.infer.1`); the module is in scope without `use` and is named `png`. It reads no files: pass it `fs.read_bytes(path)?`.

## The module

**std.png.1** — `png` provides `decode`, `encode`, `info`, `is_png`, and the types `PngError`, `PngKind`, `Info` and
`ColorType`. `is_png(data bytes) bool` says whether `data` starts with the 8-byte PNG signature. *(ADR 1250)*

**std.png.2** — `info(data bytes) Info ! PngError` reads only the signature and the IHDR chunk, checks IHDR's CRC-32 and
header, and gives `width`, `height`, `color` (`Gray`, `Rgb`, `Palette`, `GrayAlpha`, `Rgba`), `depth` (1, 2, 4, 8 or 16) and
`interlaced`. A header outside the specification fails with `BadHeader`: a side of 0 or above 2^31 - 1, a color
type other than 0, 2, 3, 4 or 6, a bit depth the color type does not allow (gray 1 to 16, RGB 8 and 16, palette 1 to 8,
gray with alpha and RGBA 8 and 16), a compression or filter method other than 0, or an interlace method other than 0 and 1.
An image of more than `image.MAX_PIXELS` pixels fails with `Unsupported`. *(ADR 1250)*

```ail tool=run,FILE expect=2x1 Rgba 8 rules=std.png.1,std.png.2
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

```ail tool=run,FILE expect=NotPng: the data does not start with the PNG signature rules=std.png.8
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
