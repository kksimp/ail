# Standard library: `std.image.ops`

Chapter key: `std`

> **Status:** Drafted. Written in AIL in `std/image/ops.ail` (ADR 1261) and tested by `tests/cases/std_image_ops.ail`.

Operations on the `image.Image` of [std-image](std-image.md): resize, crop, rotate, flip, grayscale, alpha compositing,
rectangles and lines. Every function is pure (`eff.infer.1`) and gives a new image; the module is in scope without
`use` and is named `imageops` (`std.use.4`). The heavy operations compute their rows on all cores, in order, and give the
result the sequential loop gives (`conc.par.12`). A failure is an `image.ImageError` with the text in `err.why`.

**std.imageops.1** — `imageops` provides `resize`, `crop`, `rotate90`, `rotate180`, `rotate270`, `flip_horizontal`,
`flip_vertical`, `grayscale`, `composite`, `fill_rect`, `draw_line`, `draw_rect` and the enum `Filter`. Details: [below](#more-on-stdimageops1). *(ADR 1261)*

```ail tool=run,FILE expect=2 rules=std.imageops.1
fn main() !
    img := image.solid(4, 2, 0xff0000ff)?
    print(imageops.rotate90(img).width)
```

**std.imageops.2** — `resize(img, w, h, how Filter) Image ! ImageError` scales to `w` by `h` pixels. Details: [below](#more-on-stdimageops2). *(ADR 1261)*

```ail tool=run,FILE expect=true rules=std.imageops.2
fn main() !
    img := image.solid(4, 4, 0x336699ff)?
    small := imageops.resize(img, 2, 2, imageops.Filter.Lanczos3)?
    print(small.pixel(1, 1) == 0x336699ff)
```

**std.imageops.3** — `crop(img, x, y, w, h) Image ! ImageError` is the `w` by `h` part with its top left pixel at column `x`
and row `y`. It fails when `w` or `h` is below 1 or the part is not wholly inside the image. *(ADR 1261)*

```ail tool=run,FILE expect=2x3 rules=std.imageops.3
fn main() !
    img := image.solid(4, 4, 0)?
    c := imageops.crop(img, 1, 1, 2, 3)?
    print("{c.width}x{c.height}")
```

**std.imageops.4** — `rotate90` turns the image a quarter turn clockwise, `rotate270` a quarter turn counterclockwise, and
`rotate180` half a turn. The quarter turns swap `width` and `height`. *(ADR 1261)*

```ail tool=run,FILE expect=2x4 rules=std.imageops.4
fn main() !
    img := image.solid(4, 2, 0)?
    r := imageops.rotate270(img)
    print("{r.width}x{r.height}")
```

**std.imageops.5** — `flip_horizontal` mirrors left to right and `flip_vertical` top to bottom. *(ADR 1261)*

```ail tool=run,FILE expect=0506070801020304 rules=std.imageops.5
fn main() !
    img := image.image(2, 1, b"\x01\x02\x03\x04\x05\x06\x07\x08")?
    print(imageops.flip_horizontal(img).pixels.hex())
```

**std.imageops.6** — `grayscale` sets red, green and blue to the luma `(299 r + 587 g + 114 b + 500) / 1000` (integer
division; ITU-R BT.601 weights, rounded) and keeps alpha. *(ADR 1261)*

```ail tool=run,FILE expect=4c4c4cff rules=std.imageops.6
fn main() !
    img := image.solid(1, 1, 0xff0000ff)?
    print(imageops.grayscale(img).pixels.hex())
```

**std.imageops.7** — `composite(dst, src, x, y) Image` lays `src` over `dst` with its top left pixel at column `x`, row `y`.
Each covered pixel is the source-over blend of the two: with source alpha `sa` and destination alpha `da` (0 to 255), the
result alpha is `(sa * 255 + da * (255 - sa) + 127) / 255` and each color is `(sc * sa * 255 + dc * da * (255 - sa) +
oa * 255 / 2) / (oa * 255)`, where `oa` is the result alpha (a result alpha of 0 gives the source pixel). An opaque source
pixel replaces, and a transparent one keeps, the destination pixel. `x` and `y` may be negative or past the edge; the part
outside `dst` is dropped. The result has the size of `dst`. *(ADR 1261)*

```ail tool=run,FILE expect=7f0080ffff0000ff rules=std.imageops.7
fn main() !
    red := image.solid(2, 1, 0xff0000ff)?
    glass := image.solid(1, 1, 0x0000ff80)?
    print(imageops.composite(red, glass, 0, 0).pixels.hex())
```

**std.imageops.8** — `fill_rect(img, x, y, w, h, rgba) Image ! ImageError` paints the rectangle at column `x`, row `y`, `w` wide
and `h` high with `rgba`, blended as in std.imageops.7 (an opaque color replaces). Details: [below](#more-on-stdimageops8). *(ADR 1261)*

```ail tool=run,FILE expect=000000ffffffffffffffffff rules=std.imageops.8
fn main() !
    img := image.solid(3, 1, 0x000000ff)?
    print(imageops.fill_rect(img, 1, 0, 5, 5, 0xffffffff)?.pixels.hex())
```

**std.imageops.9** — `draw_line(img, x0, y0, x1, y1, rgba) Image ! ImageError` paints a one-pixel line from (`x0`, `y0`)
to (`x1`, `y1`), both ends included, by Bresenham's algorithm, blended as in std.imageops.7. Details: [below](#more-on-stdimageops9). *(ADR 1261)*

```ail tool=run,FILE expect=4278190335 rules=std.imageops.9
fn main() !
    img := image.solid(3, 3, 0x000000ff)?
    print(imageops.draw_line(img, 0, 0, 2, 2, 0xff0000ff)?.pixel(1, 1))
```

**std.imageops.10** — `draw_rect(img, x, y, w, h, rgba) Image ! ImageError` paints the one-pixel outline of the
rectangle, each pixel once, blended as in std.imageops.7. Details: [below](#more-on-stdimageops10). *(ADR 1261)*

```ail tool=run,FILE expect=255 4294967295 rules=std.imageops.10
fn main() !
    img := image.solid(3, 3, 0x000000ff)?
    r := imageops.draw_rect(img, 0, 0, 3, 3, 0xffffffff)?
    print("{r.pixel(1, 1)} {r.pixel(0, 0)}")
```

### `std.imageops` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.imageops.2

`Filter.Nearest` copies, for each new pixel, the source pixel under its center. `Filter.Bilinear` (a triangle of radius 1) and `Filter.Lanczos3` (a windowed sinc of radius 3) weigh the source pixels around the new pixel's center, with the radius widened by the scale when the image shrinks, so every source pixel counts; the weights are normalised to sum to 1. The weighted filters multiply the colors by alpha before weighing and divide after, so a transparent pixel leaves no fringe, and round to the nearest byte. A solid image stays the same color under every filter.

### `std.imageops` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.imageops.1

Colors are `0xRRGGBBAA` ints with straight (not multiplied) alpha; one outside 0 to 2^32 - 1 is an error where a function takes one.

#### More on std.imageops.10

The part outside the image is dropped, and a rectangle with `w` or `h` below 1 changes nothing.

#### More on std.imageops.2

It fails when a side is below 1 or the result would have more than `image.MAX_PIXELS` pixels; the same size gives `img`. Details: [below](#more-on-stdimageops2).

#### More on std.imageops.8

The part outside the image is dropped, and a rectangle with `w` or `h` below 1 changes nothing.

#### More on std.imageops.9

The part outside the image is dropped. It fails when a coordinate is more than 2^30 from 0.
