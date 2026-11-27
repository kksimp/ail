# Standard library: `std.image.jpeg`

Chapter key: `std`

> **Status:** Drafted. Written in AIL in `std/image/jpeg.ail` (ADR-TBD) and tested by `tests/cases/std_image_jpeg.ail`,
> `std_image_jpeg_progressive.ail` and `std_image_jpeg_info.ail`.

JPEG decoding (ITU-T T.81, JFIF, Adobe and EXIF markers) into the `image.Image` of [std-image](std-image.md). Every
function is pure (`eff.infer.1`); the module is in scope without `use` and is named `jpeg`. It reads no files: pass it
`fs.read_bytes(path)?`. There is no encoder yet.

## The module

**std.jpeg.1** — `jpeg` provides `decode`, `decode_upright`, `info` and `orient`, and the types `JpegError`,
`JpegKind` and `Info`. It reads baseline and extended Huffman frames (SOF0, SOF1) and progressive frames (SOF2) with
8-bit samples: 1 component (gray), 3 (YCbCr or RGB) or 4 (CMYK or YCCK), each sampled by whole factors 1 to 4 of the
largest, with restart intervals. Markers it does not use (comments, other APP segments) are skipped. *(ADR-TBD)*

**std.jpeg.2** — `info(data bytes) Info ! JpegError` reads the headers up to the frame header and gives `width`,
`height`, `components`, `progressive` and `orientation`: the EXIF orientation tag (0x0112) of an APP1 `Exif` segment
before the frame, 1 to 8, or 1 when there is none. *(ADR-TBD)*

```ail tool=run,FILE expect=2x1 1 false 1 rules=std.jpeg.1,std.jpeg.2
fn main() !
    b := bytes.from_hex("ffd8ffdb00430001010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101ffc0000b080001000201011100ffc40014000100000000000000000000000000000009ffc4001a1000010501000000000000000000000000080006073878b7ffda0008010100003f0068021a5c21e5f8079434d7ffd9") else return
    i := jpeg.info(b)?
    print("{i.width}x{i.height} {i.components} {i.progressive} {i.orientation}")
```

**std.jpeg.3** — `decode(data bytes) image.Image ! JpegError` gives the pixels as stored, without the EXIF
orientation. Its output equals libjpeg-turbo's default decode to RGBA byte for byte: the integer inverse DCT (the
13-bit Loeffler-Ligtenberg-Moschytz factorization of libjpeg's `islow`), triangle-filter ("fancy") upsampling of
components sampled 2x1, 1x2 or 2x2 when they are wider than 2 samples and repeated samples otherwise, and JFIF's
YCbCr matrix in 16-bit fixed point. Gray is repeated into red, green and blue. Three components are RGB when an
Adobe marker says transform 0, or when there is no Adobe marker and they are named `R`, `G`, `B`; otherwise YCbCr.
Four components are YCCK when an Adobe marker says transform 2, else CMYK; with an Adobe marker the inks are stored
inverted, as Adobe writes them. CMYK becomes RGB as Pillow converts it: `r = (255 - k) - (c * (255 - k)) / 255`,
rounded. Alpha is 255. A missing end-of-image marker after whole scans is accepted. *(ADR-TBD)*

```ail tool=run,FILE expect=673720575 3368601855 rules=std.jpeg.3
fn main() !
    b := bytes.from_hex("ffd8ffdb00430001010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101ffc0000b080001000201011100ffc40014000100000000000000000000000000000009ffc4001a1000010501000000000000000000000000080006073878b7ffda0008010100003f0068021a5c21e5f8079434d7ffd9") else return
    img := jpeg.decode(b)?
    print("{img.pixel(0, 0)} {img.pixel(1, 0)}")
```

**std.jpeg.4** — `decode_upright(data bytes) image.Image ! JpegError` decodes like `decode` and then applies the EXIF
orientation with `orient`, giving the picture as a photo viewer shows it. *(ADR-TBD)*

**std.jpeg.5** — `orient(img image.Image, orientation int) image.Image` turns an image by an EXIF orientation: 1 leaves
it, 2 mirrors it left to right, 3 turns it 180 degrees, 4 mirrors it top to bottom, 5 mirrors it about the
top-left to bottom-right diagonal, 6 turns it 90 degrees clockwise, 7 mirrors it about the other diagonal and 8 turns it 90
degrees counterclockwise. 5 to 8 swap the width and height. Any other value leaves the image. *(ADR-TBD)*

```ail tool=run,FILE expect=1x2 287454020 rules=std.jpeg.4,std.jpeg.5
fn main() !
    img := image.solid(2, 1, 0x11223344)?
    t := jpeg.orient(img, 6)
    print("{t.width}x{t.height} {t.pixel(0, 1)}")
```

**std.jpeg.6** — A failure is a `JpegError` whose text starts `jpeg:`, with `why` and an enum `kind` (`type.enum.7`):
`NotJpeg` (no start-of-image marker), `Truncated` (the file ends inside a marker segment or inside a scan's data, or before any
scan), `Corrupt` (a bad
table, header, Huffman code or coefficient run, a scan before the frame, a missing table) and `Unsupported`
(arithmetic, lossless and hierarchical coding, 12- and 16-bit samples, 2 components, fractional sampling factors, a
height given by a DNL marker, more than `image.MAX_PIXELS` pixels). Decoding never traps. *(ADR-TBD)*

```ail tool=run,FILE expect=NotJpeg rules=std.jpeg.6
fn main()
    img := jpeg.decode(b"GIF89a") else err
        print(err.kind.name())
        return
    print(img.width)
```
