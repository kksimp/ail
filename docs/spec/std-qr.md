# Standard library: `std.qr`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR TBD).

QR Code Model 2 symbols (ISO/IEC 18004): numeric, alphanumeric and byte modes,
versions 1 to 40, the four error correction levels, Reed-Solomon codewords and
the eight masks with the standard penalty scoring. The result is a module
matrix, which the module also writes as text art or SVG. Every function is
pure (`eff.infer.1`) and needs no capability. The module is written in AIL and
is in scope without `use` (`std.use.4`); `qr.encode(...)` names it. There is
no PNG writer: `std` has no image module yet, and the matrix (`Code.dark`)
is what one would draw.

## The module

**std.qr.1** — `std.qr` provides `encode`, `encode_bytes`, `to_text` and
`to_svg`, the method `Code.dark`, and the types `Code`, `Level`, `QrError`
and `QrKind`. A `Code` has the `int` fields `version`, `size` and `mask`, and
`level`, a `Level`. `c.dark(x, y)` is true when the module in column `x` and
row `y`, counted from the top left, is dark; it is false outside the symbol
(so a quiet zone needs no special case). *(ADR TBD)*

```ail tool=run,FILE expect=1,21,true rules=std.qr.1
fn main() !
    c := qr.encode("HELLO WORLD")?
    print("{c.version},{c.size},{c.dark(0, 0)}")
```

**std.qr.2** — `Level` is an enum: `Low`, `Medium`, `Quartile` and `High`,
which restore about 7%, 15%, 25% and 30% of the codewords. `encode` and
`encode_bytes` take `level`, by name, and default to `Medium`. A higher level
holds less data, so the same text can need a larger version. *(ADR TBD)*

```ail tool=run,FILE expect=1,2 rules=std.qr.2
fn main() !
    a := qr.encode("HELLO WORLD", level: qr.Level.Low)?
    b := qr.encode("HELLO WORLD", level: qr.Level.High)?
    print("{a.version},{b.version}")
```

**std.qr.3** — `encode(text, level, min_version)` and `encode_bytes(data, level,
min_version)` choose the smallest version, from `min_version` (default 1) up
to 40, whose data capacity holds the mode bits, the character count, the
data, and at least the four terminator bits at that level. The symbol is
`version * 4 + 17` modules on a side. A larger `min_version` gives a larger
symbol even for short data. *(ADR TBD)*

```ail tool=run,FILE expect=13,69 rules=std.qr.3
fn main() !
    c := qr.encode("a".repeat(300))?
    print("{c.version},{c.size}")
```

**std.qr.4** — `encode(text)` picks the densest mode that holds all of the
text: numeric when it is only digits, alphanumeric when it is only the
characters of `std.qr.5`, else byte mode over the UTF-8 bytes of the text
(no ECI header is written; scanners read UTF-8). `encode_bytes(data)` always
uses byte mode and the bytes as they are. A version 1 symbol at level `Low`
holds 41 digits, 25 alphanumeric characters or 17 bytes. *(ADR TBD)*

```ail tool=run,FILE expect=1112 rules=std.qr.4
fn main() !
    a := qr.encode("1".repeat(41), level: qr.Level.Low)?
    b := qr.encode("A".repeat(25), level: qr.Level.Low)?
    c := qr.encode("a".repeat(17), level: qr.Level.Low)?
    d := qr.encode("a".repeat(18), level: qr.Level.Low)?
    print("{a.version}{b.version}{c.version}{d.version}")
```

**std.qr.5** — Alphanumeric mode is the 45 characters `0-9`, `A-Z`, space and
`$ % * + - . / :`, upper case only; one lower case letter makes the whole
text byte mode. Numeric mode packs three digits into 10 bits, alphanumeric
two characters into 11 bits. *(ADR TBD)*

```ail tool=run,FILE expect=2,1 rules=std.qr.5
fn main() !
    a := qr.encode("a".repeat(25), level: qr.Level.Low)?
    b := qr.encode("A".repeat(25), level: qr.Level.Low)?
    print("{a.version},{b.version}")
```

**std.qr.6** — The data codewords are split into blocks by the version and
level tables of ISO/IEC 18004, each block gets its Reed-Solomon error
correction codewords over GF(256) with the polynomial 0x11D, and the blocks are
interleaved. The codewords are placed in the zigzag order from the lower right
corner, two columns at a time, skipping the function modules. Placement is
the same for every symbol of one version, so equal data at equal settings
gives an equal matrix. *(ADR TBD)*

```ail tool=run,FILE expect=234 rules=std.qr.6
fn main() !
    c := qr.encode("HELLO WORLD", level: qr.Level.Low)?
    mut n := 0
    for y in 0..c.size
        for x in 0..c.size
            if c.dark(x, y)
                n += 1
    print(n)
```

**std.qr.7** — Each of the eight masks is tried, and the one with the lowest
penalty is used (the lowest mask number on a tie); `c.mask` says which. The
penalty is the sum of the four rules of the standard in the form of the
Nayuki reference encoder: runs of five or more equal modules in a row or
column (3 for five, plus 1 for each more), each 2x2 block of one color (3),
each finder-like 1:1:3:1:1 run with four light modules on a side (40), and
10 for each 5% the share of dark modules is away from 50%. *(ADR TBD)*

```ail tool=run,FILE expect=7,0 rules=std.qr.7
fn main() !
    a := qr.encode("HELLO WORLD", level: qr.Level.Low)?
    b := qr.encode("HELLO WORLD", level: qr.Level.Medium)?
    print("{a.mask},{b.mask}")
```

**std.qr.8** — The function modules are written as the standard says: three
finder patterns with their separators, the timing lines, the alignment
patterns (version 2 and up), the dark module, the two copies of the 15
format bits (level, mask and a BCH check, XORed with 0x5412), and, from
version 7, the two copies of the 18 version bits (a BCH check of the
version). *(ADR TBD)*

```ail tool=run,FILE expect=7,45 rules=std.qr.8
fn main() !
    c := qr.encode("1", level: qr.Level.Low, min_version: 7)?
    print("{c.version},{c.size}")
```

## Errors

**std.qr.9** — `QrError` is an error with `kind QrKind` and `why str`; its text
is `why`. The kinds are `NoData` (empty text or no bytes), `TooLong` (the data
does not fit a version 40 symbol at that level) and `BadVersion`
(`min_version` outside 1 to 40). Match on `err.kind`, never on `why`.
*(ADR TBD)*

```ail tool=run,FILE expect=NoData rules=std.qr.9
fn main()
    c := qr.encode("") else err
        print(err.kind.name())
        return
    print(c.version)
```

## Output

**std.qr.10** — `to_text(code, quiet, invert)` is the symbol as text art, two
module rows to a line with the characters `█`, `▀`, `▄` and a space, with
`quiet` light modules around it (default 4, the standard minimum; 0 or less
means none). Lines are joined with `\n`, with none at the end. With
`invert: true` dark and light swap, for a light-on-dark terminal. A symbol with
`quiet: 0` and `size` 21 is 11 lines of 21 characters. *(ADR TBD)*

```ail tool=run,FILE expect=11,21 rules=std.qr.10
fn main() !
    c := qr.encode("HELLO WORLD")?
    lines := qr.to_text(c, quiet: 0).lines()
    print("{lines.len()},{lines[0].len()}")
```

**std.qr.11** — `to_svg(code, scale, quiet)` is an SVG document: a white
rectangle and one black path, in a `viewBox` of `size + 2 * quiet` units, drawn
`scale` pixels per module (defaults 8 and 4; `scale` below 1 counts as 1).
Each horizontal run of dark modules is one `M x,y h n v1 h -n z` subpath, and
the output has no newline and no external references. *(ADR TBD)*

```ail tool=run,FILE expect=true,true rules=std.qr.11
fn main() !
    c := qr.encode("HELLO WORLD")?
    s := qr.to_svg(c)
    print("{s.starts_with("<svg xmlns")},{s.ends_with("</svg>")}")
```

## Limits

**std.qr.12** — The largest symbol is version 40, 177 by 177 modules. Its
capacity at level `Low` is 7089 digits, 4296 alphanumeric characters or 2953
bytes, and at level `High` 3057, 1852 or 1273. Not provided: Kanji mode, ECI,
structured append, Micro QR, FNC1, and a PNG writer. *(ADR TBD)*

```ail tool=run,FILE expect=40 rules=std.qr.12
fn main() !
    c := qr.encode("1".repeat(7089), level: qr.Level.Low)?
    print(c.version)
```
