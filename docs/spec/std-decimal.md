# Standard library: `std.decimal`

Chapter key: `std`

> **Status:** Drafted (ADR 1245). Written in AIL (`std/decimal/decimal.ail`) on `std.bigint`.

Exact base-10 numbers for money and measurements: `0.1 + 0.2` is `0.3`, no digit is ever lost, and a division
says how many digits it keeps and how it rounds. Every function is pure (`eff.infer.1`) and needs no
capability. The module is in scope without `use` (`std.use.4`); `decimal.parse(...)` names it.

## The module

**std.decimal.1** — `std.decimal` provides `Decimal`, `Rounding`, `DecimalError`, `DecimalKind`, `from_int`,
`from_parts`, `parse` and `sum`. A `Decimal` is an integer and a scale: its value is `unscaled / 10^scale`, with
`scale >= 0` digits after the point. Details: [below](#more-on-stddecimal1). *(ADR 1245)*

```ail tool=run,FILE expect=1999,2 rules=std.decimal.1
fn main() !
    d := decimal.parse("19.99")?
    print("{d.unscaled()},{d.scale()}")
```

**std.decimal.2** — `from_int(n int) Decimal` has scale 0. `from_parts(unscaled BigInt, scale int) Decimal !
DecimalError` is `unscaled / 10^scale`. Details: [below](#more-on-stddecimal2). *(ADR 1245)*

```ail tool=run,FILE expect=1500,0.0025,0.000 rules=std.decimal.2
fn main() !
    a := decimal.parse("1.5e3")?
    b := decimal.parse("25e-4")?
    c := decimal.parse("+0.000")?
    print("{a},{b},{c}")
```

**std.decimal.3** — `Decimal.to_text() str` is the plain text: a leading `-` for a negative value, then the
integer part, then `.` and exactly `scale` digits (none for scale 0); there is no exponent form. Details: [below](#more-on-stddecimal3). *(ADR 1245)*

```ail tool=run,FILE expect=-0.05,12.50 rules=std.decimal.3
fn main() !
    a := decimal.parse("-0.05")?
    b := decimal.parse("12.50")?
    print("{a},{b}")
```

**std.decimal.4** — `a + b`, `a - b`, `a * b` and `-a` on two `Decimal`s are exact (`trait.op.1`): `+` and `-`
give the larger of the two scales, `*` the sum of the scales, `-a` keeps the scale. The method forms `a.add(b)`,
`a.sub(b)`, `a.mul(b)`, `a.negate()` are the same. There is no `a / b`: a quotient needs a scale and a rounding
mode, so use `divide` (`std.decimal.6`); it is a `trait.op.1` error. Mixing a `Decimal` and an `int` is an error
(`trait.op.3`): write `decimal.from_int(n)`. `Decimal` implements `Eq` and `Ord` by value, not by scale: `1.0 ==
1.00`, `1.5 < 2` and `xs.sorted()`, `max()` and `min()` work. Because the `Eq` is hand-written, a `Decimal` is not
`Hash` (`trait.auto.5`): key a map by `d.normalize().to_text()`. *(ADR 1245)*

```ail tool=run,FILE expect=3.75,0.600,0.3,true,true rules=std.decimal.4
fn main() !
    one_and_half := decimal.parse("1.5")?
    s := one_and_half + decimal.parse("2.25")?
    p := decimal.parse("1.20")? * decimal.parse("0.5")?
    t := decimal.parse("0.1")? + decimal.parse("0.2")?
    third := decimal.parse("0.3")?
    x := decimal.parse("1.0")?
    y := decimal.parse("1.00")?
    print("{s},{p},{t},{t == third},{x == y}")
```

```ail error=trait.op.1
fn main() !
    a := decimal.parse("1")?
    b := decimal.parse("3")?
    print(a / b)        # a quotient needs a scale and a rounding mode: use a.divide(b, 2, mode)
```

## Rounding and division

**std.decimal.5** — `round(self, scale int, mode Rounding) Decimal ! DecimalError` gives `self` with exactly
`scale` digits after the point, `0 <= scale <= 10000`. Details: [below](#more-on-stddecimal5).

| Variant | Result |
|---|---|
| `Down` | toward zero (truncate) |
| `Up` | away from zero if anything is discarded |
| `Floor` | toward negative infinity |
| `Ceiling` | toward positive infinity |
| `HalfUp` | nearest; a tie goes away from zero |
| `HalfDown` | nearest; a tie goes toward zero |
| `HalfEven` | nearest; a tie goes to the even last digit (banker's rounding) |

There is no default mode: the caller always names one. *(ADR 1245)*

```ail tool=run,FILE expect=1.00,1.01,-1.01 rules=std.decimal.5
fn main() !
    tie := decimal.parse("1.005")?
    a := tie.round(2, decimal.Rounding.HalfEven)?
    b := tie.round(2, decimal.Rounding.HalfUp)?
    c := decimal.parse("-1.001")?.round(2, decimal.Rounding.Floor)?
    print("{a},{b},{c}")
```

**std.decimal.6** — `divide(self, other Decimal, scale int, mode Rounding) Decimal ! DecimalError` is
`self / other` with exactly `scale` digits after the point (`0 <= scale <= 10000`), rounded by `mode`. A zero
divisor is `DivideByZero`. The result is the exact quotient rounded once, never a rounded intermediate.
*(ADR 1245)*

```ail tool=run,FILE expect=33.33,-4,6.00 rules=std.decimal.6
fn main() !
    total := decimal.parse("100.00")?
    share := total.divide(decimal.from_int(3), 2, decimal.Rounding.HalfEven)?
    half := decimal.from_int(-7).divide(decimal.from_int(2), 0, decimal.Rounding.HalfEven)?
    ratio := decimal.parse("1.50")?.divide(decimal.parse("0.25")?, 2, decimal.Rounding.Down)?
    print("{share},{half},{ratio}")
```

## Other operations

**std.decimal.7** — `pow(self, n int) Decimal ! DecimalError` is `self` to the power `n >= 0`, exactly; its scale
is `n` times the scale. `pow(0)` is 1. A negative `n` is `BadExponent`. *(ADR 1245)*

```ail tool=run,FILE expect=1.331,1 rules=std.decimal.7
fn main() !
    d := decimal.parse("1.1")?
    cube := d.pow(3)?
    one := d.pow(0)?
    print("{cube},{one}")
```

**std.decimal.8** — `normalize(self) Decimal` is the same value without trailing zeros in the fraction: `1.2500`
becomes `1.25` and `5.00` becomes `5`. Details: [below](#more-on-stddecimal8). *(ADR 1245)*

```ail tool=run,FILE expect=1.25,5,100 rules=std.decimal.8
fn main() !
    a := decimal.parse("1.2500")?.normalize()
    b := decimal.parse("5.00")?.normalize()
    c := decimal.parse("100")?.normalize()
    print("{a},{b},{c}")
```

**std.decimal.9** — `sum(xs List[Decimal]) Decimal` adds the list exactly; the empty list gives `0`. *(ADR 1245)*

```ail tool=run,FILE expect=23.99 rules=std.decimal.9
fn main() !
    xs := [decimal.parse("19.99")?, decimal.from_int(3), decimal.parse("1.0")?]
    print(decimal.sum(xs))
```

**std.decimal.10** — `as_int(self, mode Rounding) int?` rounds to a whole number by `mode`, and gives `none` when
the result does not fit an `int`. Details: [below](#more-on-stddecimal10). *(ADR 1245)*

```ail tool=run,FILE expect=2,3,2.5 rules=std.decimal.10
fn main() !
    d := decimal.parse("2.5")?
    even := d.as_int(decimal.Rounding.HalfEven) else -1
    up := d.as_int(decimal.Rounding.Up) else -1
    print("{even},{up},{d.as_float()}")
```

## Errors and small accessors

**std.decimal.11** — `DecimalError` is an error with `kind DecimalKind` and `why str`; its text is `why`. The
kinds are `NoDigits` (no digits in the text), `BadDigit` (a character that is not a digit, or a second `.`),
`BadExponent` (an exponent with no digits or a bad digit, or a negative `pow`), `BadScale` (a scale outside 0 to
10000), `DivideByZero` and `TooLarge` (an exponent of more than six digits, or one that needs a scale beyond 10000).
Match on `err.kind`, never on `why`. *(ADR 1245)*

```ail tool=run,FILE expect=BadDigit,DivideByZero rules=std.decimal.11
fn kind_of(text str) str
    d := decimal.parse(text) else err
        return err.kind.name()
    q := d.divide(decimal.from_int(0), 2, decimal.Rounding.Down) else err
        return err.kind.name()
    "{q}"

fn main()
    print("{kind_of("12x")},{kind_of("1")}")
```

**std.decimal.12** — `is_zero()` and `is_negative()` give `bool` (zero is never negative); `signum() int` is -1, 0
or 1; `abs()` and `negate()` give a `Decimal` of the same scale. *(ADR 1245)*

```ail tool=run,FILE expect=-1,true,2.5 rules=std.decimal.12
fn main() !
    d := decimal.parse("-2.5")?
    print("{d.signum()},{d.is_negative()},{d.abs()}")
```

### `std.decimal` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.decimal.1

`d.unscaled()` is the integer (a `BigInt`) and `d.scale()` the scale. The scale is part of the value's form: `12.50` has scale 2 and prints with its trailing zero, which is what money needs; `normalize` (`std.decimal.8`) removes it. The fields are private, so no value has a negative scale.

#### More on std.decimal.10

`as_float(self) float` is the nearest `float`; it is for showing or plotting, not for adding money.

#### More on std.decimal.2

`parse(text str) Decimal ! DecimalError` reads an optional `+` or `-`, digits with at most one `.` (`.5` and `7.` are allowed, `.` alone is not), and an optional exponent `e` or `E` with an optional sign and digits. The scale is the digits after the point less the exponent, and a negative one becomes scale 0 with the zeros written out: `1.5e3` is `1500`, `25e-4` is `0.0025`. Spaces, separators and other characters are errors. `-0` is zero, which is never negative.

#### More on std.decimal.3

It is also the text form (`trait.std.3`), so `print(d)` and `"{d}"` show the number.

#### More on std.decimal.5

Adding digits is exact; removing them rounds by `mode`. `Rounding` has seven variants. The discarded part is a tie when it is exactly one half.

#### More on std.decimal.8

A scale of 0 is left as it is (`100` stays `100`).
