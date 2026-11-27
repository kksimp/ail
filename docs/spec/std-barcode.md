# Standard library: `std.barcode`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR TBD).

One-dimensional barcodes: Code 128, Code 39, EAN-13, UPC-A and Interleaved 2
of 5. A barcode is a list of bar and space widths in modules; the module
encodes text or digits to that list, reads such a list back, adds and checks
the check digits, and writes the bars as text art or SVG. Every function is
pure (`eff.infer.1`) and needs no capability. The module is written in AIL and
is in scope without `use` (`std.use.4`); `barcode.ean13(...)` names it. 2D
symbols are in [std.qr](std-qr.md).

## The module

**std.barcode.1** — `std.barcode` provides `code128`, `code39`, `ean13`,
`upca`, `itf`, `decode`, `check_digit`, `is_valid`, `code39_check`, `to_text`
and `to_svg`, the methods `Barcode.pattern` and `Barcode.modules`, and the
types `Barcode`, `Symbology` (`Code128`, `Code39`, `Ean13`, `UpcA`, `Itf`),
`BarcodeError` and `BarcodeKind`. A `Barcode` has `kind`, `text` and
`widths`. `widths` alternates bar, space, bar, and so on, starts and ends with
a bar, is in modules (1 is the narrowest element) and has no quiet zone.
`b.modules()` is the sum of the widths; `b.pattern()` is one character per
module, `1` for a bar and `0` for a space. *(ADR TBD)*

```ail tool=run,FILE expect=Ean13,5901234123457,95 rules=std.barcode.1
fn main() !
    b := barcode.ean13("590123412345")?
    print("{b.kind.name()},{b.text},{b.modules()}")
```

## Encoding

**std.barcode.2** — `code128(text)` holds every ASCII character, 0 to 127, in
subsets A, B and C, which it chooses itself. It starts in subset C for text
that is only digits (two or more) or that begins with four or more digits,
else in A when the first character is a control character, else in B. In A
or B a run of four or more digits switches to C (an odd run puts its first
digit in the current subset), in C a lone digit or any other character
switches back to B (A for a control character), and in A a character from 96
up switches to B, in B a control character to A. Each symbol is 11 modules
(six elements), the stop is 13 modules (seven), and the check symbol is the
start value plus each later value times its position, modulo 103. A symbol
needs no quiet zone in `widths`. Not provided: FNC1 to FNC4 and the shift
symbol. *(ADR TBD)*

```ail tool=run,FILE expect=68,101 rules=std.barcode.2
fn main() !
    a := barcode.code128("123456")?
    b := barcode.code128("abcdef")?
    print("{a.modules()},{b.modules()}")
```

**std.barcode.3** — `code39(text, checksum)` holds the 43 characters `0-9`,
`A-Z`, space and `- . $ / + %`, upper case only, between the `*` start and
stop, a narrow gap between characters, and a wide element three modules wide.
With `checksum: true` the modulo-43 check character (the sum of the character
values in that order) is appended; `code39_check(text)` returns it. `text` of
the result is the data with the check character and without the `*`.
*(ADR TBD)*

```ail tool=run,FILE expect=HELLOB,127 rules=std.barcode.3
fn main() !
    b := barcode.code39("HELLO", checksum: true)?
    print("{b.text},{b.modules()}")
```

**std.barcode.4** — `ean13(digits)` takes 12 digits and appends the check
digit (std.barcode.7), or takes 13 and requires the check digit to be right.
The symbol is 95 modules: a 3-module guard, six left digits, a 5-module centre
guard, six right digits and a guard. The right digits use set R. The left
digits use set L or G as the first digit says (the parity table of the
standard), so the first digit is not drawn. *(ADR TBD)*

```ail tool=run,FILE expect=5901234123457 rules=std.barcode.4
fn main() !
    print(barcode.ean13("590123412345")?.text)
```

**std.barcode.5** — `upca(digits)` takes 11 digits and appends the check
digit, or takes 12 with the right one. The symbol is the EAN-13 symbol of a 0
followed by the 12 digits, and `kind` is `UpcA`. *(ADR TBD)*

```ail tool=run,FILE expect=036000291452,true rules=std.barcode.5
fn main() !
    a := barcode.upca("03600029145")?
    b := barcode.ean13("0036000291452")?
    print("{a.text},{a.widths == b.widths}")
```

**std.barcode.6** — `itf(digits, checksum)` is Interleaved 2 of 5: a start
of four narrow elements, each pair of digits as five bars (the first digit)
interleaved with five spaces (the second), and a stop of a wide bar and two
narrow elements; wide is three modules. The count of digits, with the check
digit when `checksum: true` (the GTIN check digit, as ITF-14 has it), must be
even. *(ADR TBD)*

```ail tool=run,FILE expect=45,1236 rules=std.barcode.6
fn main() !
    a := barcode.itf("1234")?
    b := barcode.itf("123", checksum: true)?
    print("{a.modules()},{b.text}")
```

**std.barcode.7** — The GTIN check digit of a digit string is the number that
makes the weighted sum a multiple of 10, the weights being 3 and 1 from the
right, starting with 3 at the last digit of the data. `check_digit(digits)`
returns it (an `int`) for the 12 data digits of an EAN-13, the 11 of a UPC-A,
the 13 of an ITF-14 or any such string; `is_valid(digits)` is true when the
last digit is the check digit of the rest, and false for less than two digits
or a non-digit. *(ADR TBD)*

```ail tool=run,FILE expect=7,true,false rules=std.barcode.7
fn main() !
    d := barcode.check_digit("590123412345")?
    print("{d},{barcode.is_valid("5901234123457")},{barcode.is_valid("5901234123458")}")
```

## Decoding

**std.barcode.8** — `decode(widths)` reads a barcode from measured widths:
alternating bar and space, starting and ending with a bar, in any unit and
with no quiet zone. It tries Code 128, EAN-13, Interleaved 2 of 5 and Code 39
in that order, the widths as given and then reversed (so a barcode scanned
from the right reads too). The result is a `Barcode` whose `text` is the data
and whose `widths` are the widths as given. The kind is `Ean13` for an
EAN-13 or UPC-A symbol (a first digit of 0 is a UPC-A). Each symbol is
rescaled to its own width, so a uniform scale or a slow drift of the unit
reads; a width that is not positive is `NotFound`. Code 128 reads subsets A,
B and C and checks the check symbol (`BadCheck` when it fails). *(ADR TBD)*

```ail tool=run,FILE expect=Code128,Hello 123456 rules=std.barcode.8
fn main() !
    w := barcode.code128("Hello 123456")?.widths
    mut scaled := List[int]()
    for x in w
        scaled.push(x * 5)
    b := barcode.decode(scaled)?
    print("{b.kind.name()},{b.text}")
```

**std.barcode.9** — An EAN-13 symbol is 59 elements: the guards must be three,
five and three equal elements, each digit is four elements rescaled to seven
modules and found in set L, G or R, and the parity of the left half gives the
first digit. The check digit must be right (`BadCheck`). *(ADR TBD)*

```ail tool=run,FILE expect=Ean13,0036000291452 rules=std.barcode.9
fn main() !
    b := barcode.decode(barcode.upca("03600029145")?.widths)?
    print("{b.kind.name()},{b.text}")
```

**std.barcode.10** — An Interleaved 2 of 5 symbol is 7 elements plus 10 for
every two digits; narrow and wide are told apart per group, the two widest of
the five bars and of the five spaces being wide, and a group where the second
and third widest are equal is not read. There is no check digit to test, so
`text` has every digit, including a check digit that was added. *(ADR TBD)*

```ail tool=run,FILE expect=Itf,98765432101234 rules=std.barcode.10
fn main() !
    b := barcode.decode(barcode.itf("98765432101234")?.widths)?
    print("{b.kind.name()},{b.text}")
```

**std.barcode.11** — A Code 39 symbol is ten elements per character, nine of
them the character and one a gap; the three widest of the nine are wide. The
first and last characters must be `*` and none other may be. `text` is the
characters between, with a check character if one was added. *(ADR TBD)*

```ail tool=run,FILE expect=Code39,A.B $/+% rules=std.barcode.11
fn main() !
    b := barcode.decode(barcode.code39("A.B $/+%")?.widths)?
    print("{b.kind.name()},{b.text}")
```

## Errors

**std.barcode.12** — `BarcodeError` is an error with `kind BarcodeKind` and
`why str`; its text is `why`. The kinds are `NoData` (empty text, no digits or
no widths), `BadChar` (a character the code cannot hold), `BadLength` (a
digit count the code does not take), `BadCheck` (a check digit that is wrong,
or in `decode` a symbol read whose check does not match) and `NotFound`
(`decode` found no barcode). Match on `err.kind`, never on `why`. *(ADR TBD)*

```ail tool=run,FILE expect=BadCheck rules=std.barcode.12
fn main()
    b := barcode.ean13("5901234123458") else err
        print(err.kind.name())
        return
    print(b.text)
```

## Output

**std.barcode.13** — `to_text(b, quiet, height, label)` is the bars as text art:
`height` (default 3) rows of `█` for a bar module and a space for a light one,
with `quiet` (default 10) light modules on each side, then, when `label` is
true, a line with `text` centred under them. `to_svg(b, scale, height, quiet,
label)` is an SVG document: a white rectangle and one black path with one
rectangle subpath per bar, in a `viewBox` of `modules + 2 * quiet` units by
`height` units (plus 12 for the label), drawn `scale` pixels per unit
(defaults 2, 60 and 10); the label is a `text` element with the characters
`& < > "` escaped. The output has no newline and no external references.
*(ADR TBD)*

```ail tool=run,FILE expect=2,95 rules=std.barcode.13
fn main() !
    b := barcode.ean13("590123412345")?
    lines := barcode.to_text(b, quiet: 0, height: 2, label: false).lines()
    print("{lines.len()},{lines[0].chars().len()}")
```

## Limits

**std.barcode.14** — Not provided: Code 93, Codabar, EAN-8, EAN and UPC
add-ons, UPC-E, GS1-128 and FNC1 to FNC4 in Code 128, extended Code 39, GS1
DataBar, and 2D symbols other than [std.qr](std-qr.md); and no PNG writer, as
`std` has no image module yet. `decode` reads measured widths; finding the
bars in a picture is not part of it. *(ADR TBD)*

```ail tool=run,FILE expect=NoData rules=std.barcode.14
fn main()
    b := barcode.decode(List[int]()) else err
        print(err.kind.name())
        return
    print(b.text)
```
