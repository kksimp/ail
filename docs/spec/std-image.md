# Standard library: `std.image`

Chapter key: `std`

> **Status:** Drafted. Written in AIL in `std/image/image.ail` (ADR 1250) and tested by `tests/cases/std_image.ail`.

The pixel buffer that the image codecs share. [std-png](std-png.md) decodes and encodes PNG into it. Every
function is pure (`eff.infer.1`); the module is in scope without `use` (`std.use.4`).

**std.image.1** — `image.Image` has the `int` fields `width` and `height` and the `bytes` field `pixels`: `width * height`
pixels of 4 bytes each (red, green, blue, alpha; alpha 255 is opaque), rows from the top, each left to right, with no padding.
`==` compares the fields. *(ADR 1250)*

**std.image.2** — `image(width int, height int, pixels bytes) Image ! ImageError` makes an image. It fails, with the
text in `err.why`, when a side is below 1, when the image has more than `image.MAX_PIXELS` (2^27) pixels, or when
`pixels` is not `width * height * 4` bytes. A codec that fills the fields itself holds to the same limits. *(ADR 1250)*

```ail tool=run,FILE expect=2x1 rules=std.image.1,std.image.2
fn main() !
    img := image.image(2, 1, b"\x01\x02\x03\x04\x05\x06\x07\x08")?
    print("{img.width}x{img.height}")
```

**std.image.3** — `solid(width int, height int, rgba int) Image ! ImageError` is an image of one color, `0xRRGGBBAA`; it
fails like `image` and when `rgba` is outside 0 to 2^32 - 1. *(ADR 1250)*

**std.image.4** — `img.pixel(x, y)` is the pixel at column `x` and row `y` as an `int` `0xRRGGBBAA`. It traps when
`x` or `y` is outside the image (a contract, `contract.req.1`). *(ADR 1250)*

```ail tool=run,FILE expect=287454020 rules=std.image.3,std.image.4
fn main() !
    img := image.solid(4, 4, 0x11223344)?
    print(img.pixel(3, 3))
```
