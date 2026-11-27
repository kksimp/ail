# Standard library: `std.image.gif`

Chapter key: `std`

> **Status:** Drafted. Written in AIL in `std/image/gif.ail` (ADR 1289) and tested by `tests/cases/std_gif_decode.ail`
> and `std_gif_errors.ail`.

GIF decoding (the GIF89a specification) into the `image.Image` of [std-image](std-image.md). Every function is pure
(`eff.infer.1`); the module is in scope without `use` and is named `gif`. It reads no files: pass it
`fs.read_bytes(path)?`. There is no encoder.

## The module

**std.gif.1** — `gif` provides `decode`, `first_frame`, `info`, `is_gif`, and the types `GifError`, `GifKind`, `Info`,
`Frame` and `Animation`. `is_gif(data bytes) bool` says whether `data` starts with `GIF87a` or `GIF89a`. *(ADR 1289)*

**std.gif.2** — `info(data bytes) Info ! GifError` reads the first 13 bytes: `version` (`"87a"` or `"89a"`), and the canvas
`width` and `height`. A canvas side of 0 fails with `BadImage`, and one of more than `image.MAX_PIXELS` pixels in all fails
with `Unsupported`. *(ADR 1289)*

```ail tool=run,FILE expect=true 89a 5x4 rules=std.gif.1,std.gif.2
fn main() !
    b := bytes.from_hex("47494638396105000400810000ff000000ff000000ffffffff2c00000000050004000002064434869ad758003b") else return
    inf := gif.info(b)?
    print("{gif.is_gif(b)} {inf.version} {inf.width}x{inf.height}")
```

**std.gif.3** — `decode(data bytes) Animation ! GifError` returns every frame of the file. `Animation` has `width` and
`height` (the canvas), `loops` and `frames` (a `List[Frame]`). The canvas starts fully transparent (`0x00000000`). The
blocks are read in order up to the trailer: extensions, images, and comment or other extensions are skipped. A file with no
image fails with `BadBlock`. The frames of all images, at the canvas size, may hold at most `image.MAX_PIXELS` pixels in
all, else `Unsupported`. *(ADR 1289)*

**std.gif.4** — A `Frame` has `img`, an `image.Image` the size of the canvas that shows the canvas as it looks once the frame is
drawn (so a frame can be used on its own, with no compositing by the caller), and `delay_ms`, the Graphic Control
Extension's delay in hundredths of a second times 10 (0 when the frame has none; browsers show 0 and 10 ms as 100 ms, and
the field is not adjusted). *(ADR 1289)*

```ail tool=run,FILE expect=1 1 0 ff0000ff rules=std.gif.3,std.gif.4
fn main() !
    b := bytes.from_hex("47494638396105000400810000ff000000ff000000ffffffff2c00000000050004000002064434869ad758003b") else return
    anim := gif.decode(b)?
    first := anim.frames[0]
    print("{anim.frames.len()} {anim.loops} {first.delay_ms} {first.img.pixels[0..4].hex()}")
```

**std.gif.5** — `first_frame(data bytes) image.Image ! GifError` is `decode(data)?.frames[0].img`, the picture to show
for a still or for a thumbnail. *(ADR 1289)*

```ail tool=run,FILE expect=5x4 16711935 rules=std.gif.5
fn main() !
    b := bytes.from_hex("47494638396105000400810000ff000000ff000000ffffffff2c00000000050004000002064434869ad758003b") else return
    img := gif.first_frame(b)?
    print("{img.width}x{img.height} {img.pixel(1, 0)}")
```

**std.gif.6** — Image data is LZW with a minimum code size of 1 to 8 (anything else fails with `Unsupported`): codes of
`min + 1` to 12 bits, packed low bit first across the sub-blocks, a clear code resets the table, the end code (or the
end of the data) stops, and a full table (4096 entries) stays at 12 bits and adds nothing. A first code after a clear
that is not a color index, or a code beyond the next free table entry, fails with `Corrupt`. Data that ends before the frame
has `width * height` indexes fails with `Truncated`; extra indexes are ignored. *(ADR 1289)*

**std.gif.7** — A frame's color table is its local table when it has one, else the global table; with neither the frame
fails with `Corrupt`, and so does an index past the table. An interlaced frame stores its rows in four passes (every 8th
row from 0, every 8th from 4, every 4th from 2, every 2nd from 1) and they are put in their places. A frame whose rectangle
reaches past the canvas is clipped to it. The transparent index of the frame's Graphic Control Extension leaves the canvas
pixel as it was. *(ADR 1289)*

```ail tool=run,FILE expect=0 4278190335 rules=std.gif.7
fn main() !
    b := bytes.from_hex("47494638396104000300810000000000ff000000ff000000ff21f90401050000002c0000000004000300000206442063380205003b") else return
    img := gif.first_frame(b)?
    print("{img.pixel(0, 0)} {img.pixel(1, 0)}")
```

**std.gif.8** — The disposal method of a frame's Graphic Control Extension applies after the frame is shown: 0 and 1 leave
the canvas as it is, 2 clears the frame's rectangle to transparent, and 3 restores the canvas to what it was before the
frame was drawn. Other values leave it as it is. *(ADR 1289)*

```ail tool=run,FILE expect=0 0 rules=std.gif.8
fn main() !
    b := bytes.from_hex("47494638396104000400810000000000ff000000ff000000ff21ff0b4e45545343415045322e30030105000021f904040a0000002c00000000040004000002048c8f19050021f90409140000002c01000100020002000002031430050021f9040c1e0000002c000002000200020081ffffff00ff00000000ff000002039c32050021f90400000000002c0000000004000100000203943605003b") else return
    anim := gif.decode(b)?
    print("{anim.frames[3].img.pixel(1, 1)} {anim.frames[3].img.pixel(2, 2) & 255}")
```

**std.gif.9** — `loops` is the repeat count of the NETSCAPE2.0 application extension: 0 means forever. A file without the
extension plays once and has `loops` 1. *(ADR 1289)*

```ail tool=run,FILE expect=5 4 200 rules=std.gif.9
fn main() !
    b := bytes.from_hex("47494638396104000400810000000000ff000000ff000000ff21ff0b4e45545343415045322e30030105000021f904040a0000002c00000000040004000002048c8f19050021f90409140000002c01000100020002000002031430050021f9040c1e0000002c000002000200020081ffffff00ff00000000ff000002039c32050021f90400000000002c0000000004000100000203943605003b") else return
    anim := gif.decode(b)?
    print("{anim.loops} {anim.frames.len()} {anim.frames[1].delay_ms}")
```

**std.gif.10** — A GIF that is not valid fails with `GifError`, never a trap: `NotGif` (no signature), `Truncated` (the data ends
inside a block or table, before the trailer, or before the pixels of a frame), `BadBlock` (a block that starts with an
unknown byte, or a file with no image), `Corrupt` (`std.gif.6`, `std.gif.7`), `Unsupported` (`std.gif.2`, `std.gif.3`,
`std.gif.6`) and `BadImage` (a canvas side of 0). It has the fields `kind`, a `GifKind`, and `why`, the reason as text,
and shows as `gif: ` and the reason. *(ADR 1289)*

```ail tool=run,FILE expect=NotGif: the data does not start with GIF87a or GIF89a rules=std.gif.10
fn main()
    anim := gif.decode(b"not a gif") else err
        print("{err.kind.name()}: {err.why}")
        return
    print(anim.width)
```
