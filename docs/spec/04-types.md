# 04. Types

Chapter key: `type`

> **Status:** Drafted (partial). Specified: numeric types (integers and
> floats), structs, enums, optional types, strings, characters, the standard
> collections, and maps and sets. Not yet designed: fixed-size arrays, SIMD
> vector types, and const generics.

## Numeric types

**type.num.1** — The signed integer types are `i8`, `i16`, `i32` and `i64`.
The unsigned integer types are `u8`, `u16`, `u32` and `u64`. The
floating-point types are `f32` and `f64`. *(ADR 0055)*

**type.num.2** — `int` is an alias for `i64`, and `float` is an alias for
`f64`. *(ADR 0055)*

**type.num.3** — An integer literal with no other type constraint has type
`int`. A floating-point literal with no other type constraint has type
`float`. *(ADR 0055)*

**type.num.4** — `int` and `i64` are both canonical spellings, as are `float`
and `f64`. The formatter MUST NOT rewrite one into the other. *(ADR 0059)*

**type.num.5** — Integer types support `+ - * / %`, unary `-` and the
comparisons `== != < <= > >=`. Sizes, lengths and indexes have type `int`
(this is the size and index type of `plat.ptr.2`). *(ADR 0116)*

**type.num.6** — `float` (`f64`) supports `+ - * /`, unary `-`, `+= -= *= /=`
and `== != < <= > >=`, all as IEEE 754 binary64 (round to nearest, ties to
even). It has no `%` or `%=`. Nothing traps: overflow gives `inf` or `-inf`,
and `x / 0.0` gives `inf`, `-inf` or `nan`. `nan` is unequal to everything,
itself included. Every `<`, `<=`, `>` and `>=` with a `nan` operand is false,
and `-0.0 == 0.0`. These operators are IEEE wherever they are written,
including in generic code through an `Eq` or `Ord` bound (an amendment of
`trait.std.1–2` for `float` only). There is no implicit conversion between
`int` and `float` (`trait.op.3`): write `n.to_float()` or `x.to_int()`
(`type.num.9`). `float` implements `Eq`, `Ord` and `ToStr`. It isn't `Hash`
(`trait.auto.5`). *(ADR 0130)*

```ail fragment
a := 0.1 + 0.2          # 0.30000000000000004
b := 1.0 / 0.0          # inf: no trap
c := 0.0 / 0.0          # nan; c == c is false
```

**type.num.7** — The text form of a `float` (`ToStr`, interpolation, and
inside other values) is Python's `repr`. The digits are the shortest decimal
that reads back as the same value; if several are equally short, the one
closest to the exact value. Write the value as d.ddd × 10^e. If
`-4 <= e < 16`, use fixed notation with at least one digit after the `.`.
Otherwise use `d[.ddd]e±XX`: the sign is always written, the exponent has at
least two digits, and the mantissa has no `.0`. The special forms are `inf`,
`-inf`, `nan` (never `-nan`) and `-0.0`. *(ADR 0130)*

```ail fragment
"{1000.0}"              # 1000.0
"{0.0001}"              # 0.0001
"{-0.0}"                # -0.0
"{1e16}"                # 1e+16
"{0.00001}"             # 1e-05
"{1.5e300}"             # 1.5e+300
"{5e-324}"              # 5e-324
"{123456789012345678.0}"  # 1.2345678901234568e+17
```

**type.num.8** — `s.to_float()` returns a `float?`. `s` MUST be an optional
`-`, then digits, then an optional `.` and digits, then an optional exponent
(`e` or `E`, an optional sign, digits); or exactly `inf`, `-inf` or `nan`, so
every text form of `type.num.7` reads back. A `+`, a `_`, spaces, and a leading
or trailing `.` make the result `none`. The result is correctly rounded
(nearest, ties to even). Overflow gives `inf` or `-inf` (as Python does), and
underflow gives a subnormal or `±0.0`; `"-0"` gives `-0.0`. Float literals
are correctly rounded too; a literal that overflows is a `lex.num.3` error.
*(ADR 0130)*

```ail fragment
"2.5E-3".to_float()     # 0.0025
"1e999".to_float()      # inf
"-0".to_float()         # -0.0
"+1".to_float()         # none
".5".to_float()         # none
```

**type.num.9** — `x.abs()`, `x.floor()`, `x.ceil()`, `x.round()` and
`x.sqrt()` return a `float`. `round` rounds halves away from zero (Rust, Go
and Swift; Python's `round(2.5)` is 2). `floor`, `ceil`, `round` and `sqrt`
keep the sign of zero, so `(-0.4).round()` is `-0.0`. `sqrt` of a negative
number is `nan`. `x.to_int()` returns an `int`: it truncates toward zero, and
traps on `nan` or when the truncated value is outside `int`. `n.to_float()`
rounds to nearest, ties to even, and never traps. *(ADR 0130)*

**type.num.10** — An integer literal whose expected type is `float` is a
float literal. Only literals (and `-` applied to a literal) convert: an
expression of type `int` never does. *(ADR 0130)*

```ail skip=context
struct Stats
    total float

s := Stats(total: 0)    # 0 is the float 0.0
mut t := 0.0
t += 1                  # 1 is the float 1.0
y := t * 2              # 2 is the float 2.0
z := t * n              # compile-time error if n is an int: write n.to_float()
```

**type.num.11** — `cmp` on `float` is a total order: numbers compare in
numeric order, `-0.0` is `Equal` to `0.0`, and `nan` is `Equal` to `nan` and
`Greater` than every number (`nan` sorts last). `sort`, `sorted`, `min`, `max`
(`type.coll.11`), and the ordering of composite types (`List[float]`, and
derived `Ord` on a struct with a `float` field) use this `cmp`, so their
results are deterministic. For `float` alone, `a.cmp(b) == Equal` doesn't
imply `a == b`. *(ADR 0130)*

```ail fragment
n := 0.0 / 0.0
[2.0, n, 1.0].sorted()  # [1.0, 2.0, nan]
n < 1.0                 # false: the operator is IEEE
```

> **Note:** float literal patterns are an error (`expr.match.7`). The same
> holds for `f32`.

**type.num.12** — `int` has these methods. *(ADR 0137)*
- `checked_add`, `checked_sub`, `checked_mul`, `checked_div`, `checked_rem`
  and `checked_neg` return an `int?`: `none` on overflow or a zero divisor.
  (`MIN % -1` doesn't overflow: it is `0`, `expr.arith.5`.)
- `wrapping_add`, `wrapping_sub`, `wrapping_mul` and `wrapping_neg` wrap in
  two's complement and return an `int`.
- `saturating_add`, `saturating_sub` and `saturating_mul` clamp to the
  minimum or maximum `int` and return an `int`.
- `abs()` traps on the minimum `int`.
- `to_float()` (`type.num.9`) and `to_hex()` (`type.str.8`).

These satisfy `expr.arith.2` for addition, subtraction, multiplication and
negation. Division and remainder have the checked forms only; wrapping and
saturating division are deferred. `a.min(b)` and `a.max(b)` of two `int`
values are the default methods of `Ord` (ADR 0137).

```ail fragment
x := 9223372036854775807
x.checked_add(1)        # none
x.wrapping_add(1)       # -9223372036854775808
x.saturating_add(1)     # 9223372036854775807
```

**type.num.13** — A sized number is a value of one of the types of
`type.num.1` other than `int` and `float`. Its literals are typed by context: an
integer literal whose expected type is a sized integer type (a parameter, a
field, a return value, or the other operand of an operator, written on either
side) has that type; it MUST fit, or the program is rejected. A `-` applied to
a literal belongs to the literal, so `-128` is an `i8`. An integer or float
literal whose expected type is `f32` is an `f32`. With no expected type a
literal is an `int` or a `float` (`type.num.3`). *(ADR 0194)*

**type.num.14** — Arithmetic, unary `-` and comparison on sized numbers are
defined only between two values of the same type; there is no implicit
widening or conversion (`trait.op.3`). The sized integers support what `int`
does (`type.num.5`), and a result outside the type traps (`expr.arith.1`,
`expr.arith.3`); unary `-` isn't defined for the unsigned types. `f32` follows
`type.num.6` in single precision and has no `%`. Every sized number is `Eq` and
`Ord`; every one but `f32` is `Hash`. A sized integer has the methods of
`type.num.12` (`checked_*`, `wrapping_*`, `saturating_*`, `abs`, `to_hex`,
computed in the type; `checked_neg`, `wrapping_neg` and `abs` for the signed
types only), and `f32` has `abs`, `floor`, `ceil`, `round` and `sqrt` in
single precision. Text forms follow `type.num.7` (an `f32` uses the shortest
digits that read back as an `f32`). *(ADR 0194)*

**type.num.15** — Conversions are explicit methods named for the target:
`to_i8`, `to_i16`, `to_i32`, `to_u8`, `to_u16`, `to_u32`, `to_u64` and `to_f32`
on any integer type (including `int`), `to_f32` on `float`, and `to_int` and
`to_float` back to `int` and `float` from any sized number. An integer
conversion is exact and traps (`type.num.15`) when the value doesn't fit; a
float conversion rounds to nearest, ties to even, and never traps, except
`to_int` of an `f32`, which is `type.num.9`'s. *(ADR 0195)*

```ail fragment
a := 300.to_u16()       # a u16
b := a.to_u32() * 2     # a u32: 600
c := 0.5.to_f32() + 1   # an f32: 1 is a float literal
```

## Structs

**type.struct.1** — A struct type is declared with the keyword `struct`, its
name, and an indented block containing one field declaration per line.
*(ADR 0052)*

**type.struct.2** — A field declaration is written `name Type`. *(ADR 0052,
ADR 0037)*

```ail fragment
struct User
    id i64
    name str
    email str?
```

**type.struct.3** — A struct value is constructed by the type name followed
by a parenthesised, comma-separated list of arguments. An argument is named
(`field: value`), shorthand (a bare local name equal to a field's name, which
sets that field), or positional (any other expression). *(ADR 0058,
ADR 0142)*

```ail skip=context
u := User(id: 7, name: "Ada", email: none)
```

**type.struct.4** — A struct literal MUST set every field of the struct
exactly once. Named and shorthand arguments may come in any order. Missing,
unknown and repeated fields are compile-time errors. *(ADR 0113, ADR 0142)*

**type.struct.5** — A struct's field names MUST be distinct. Each value in a
struct literal MUST have its field's type. *(ADR 0117)*

**type.struct.6** — `e.f` reads field `f` of `e`. The type of `e` MUST be a
struct that declares `f`. *(ADR 0117)*

**type.struct.7** — Positional arguments fill the fields not set by named or
shorthand arguments, in declaration order. They are allowed only when the
fields they fill have pairwise different declared types, so that a swap is
always a type error; two fields whose types are type parameters count as the
same type. Otherwise it's a compile-time error whose help names the fields.
*(ADR 0142)*

```ail error=type.struct.7
struct Pos
    row int
    col int

Pos(row, col)          # shorthand: locals named row and col
Pos(row: r, col: c)    # named
Pos(r, c)              # error: row and col are both int
```

> **Note:** field defaults are not yet decided.

## Enums

**type.enum.1** — An enum (sum) type is declared with the keyword `enum`, its
name, and an indented block containing one variant per line. *(ADR 0053)*

**type.enum.2** — A variant either carries no data or carries a parenthesised
list of fields, each written `name Type`. *(ADR 0053)*

```ail fragment
enum Shape
    Circle(r f64)
    Rect(w f64, h f64)
    Empty
```

**type.enum.3** — Where the expected type of an expression or pattern is a
known enum type, its variants MAY be written without qualification
(`Circle(r)`). Elsewhere they MUST be qualified with the enum name
(`Shape.Circle(r)`). *(ADR 0057)*

**type.enum.4** — The canonical form omits the qualification wherever
`type.enum.3` permits it. *(ADR 0057)*

**type.enum.5** — An enum's variant names MUST be distinct. *(ADR 0117)*

**type.enum.6** — A variant with data is constructed like a struct literal
(`type.struct.3–4`, `type.struct.7`): `Circle(r: 1.0)`, or `Circle(1.0)`.
*(ADR 0123, ADR 0142)*

## Optional types

**type.opt.1** — AIL has no null value. *(ADR 0048)*

**type.opt.2** — `T?` denotes an optional value of type `T`. It is exactly
equivalent to `Option[T]`. *(ADR 0048)*

**type.opt.3** — The canonical form of an optional type is `T?`.
*(ADR 0048)*

```ail skip=context
fn find(id i64) User?
    ...
```

> **Note:** see also `plat.ptr.2` (the 64-bit size and index type) and
> `expr.arith.*` (integer arithmetic).

**type.opt.4** — `none` is the absent value of every optional type.
*(ADR 0093)*

**type.opt.5** — Where a value of type `T?` is expected, a value of type `T`
converts implicitly to a present optional. *(ADR 0093)*

**type.opt.6** — `T??` is an optional whose content is an optional `T?`; it
is a different type from `T?`. Any number of `?` may follow a type. A value
of type `T??` arises from collection methods on optional elements, for
example `m.get(k)` on a `Map[K, V?]` and `xs.last()` on a `List[T?]`.
*(ADR 0209)*

**type.opt.7** — `none` is the absent value of the outermost level. `e else
d`, `e?` and `if v := e` act on that level only and give an optional of the
inner type: `m.get(k) else none` flattens a `V??` to a `V?`. A present value
whose content is `none` is not `none`: `==` compares every level, and the
text form of the present-but-empty value of a `T??` is `some(none)`. A `T`
converts to a `T??` (`type.opt.5`, twice). *(ADR 0209)*

**type.opt.8** — A `T?` does not convert to a `T??`, because its `none` would
be ambiguous between the two levels. The error is the ordinary mismatch with
a help that says so. *(ADR 0209)*

```ail fragment
fn lookup(m Map[str, int?], k str) int??
    m.get(k)

fn flat(m Map[str, int?], k str) int?
    lookup(m, k) else none
```

```ail error=expr.block.2
fn lift(v int?) int??
    v

fn main()
    print(lift(none))
```

## Booleans

**type.bool.1** — `bool` is the type of the literals `true` and `false`.
Logical operators take and produce `bool` (`expr.op.1`). *(ADR 0110)*

## Strings and characters

**type.str.1** — `str` is the type of immutable UTF-8 text. `char` is the type
of a single Unicode scalar value. *(ADR 0089)*

**type.str.2** — `str` doesn't support the indexing operator or `for`
iteration. Character access goes through the explicit view `s.chars()`
(`type.str.7`); `s.bytes()` is not implemented yet. The diagnostic for `s[i]`
or `for c in s` suggests `s.chars()`. Length is `s.len()` (`type.str.12`).
*(ADR 0089, ADR 0135, ADR 0208)*

**type.str.3** — The text form of an `int` is its decimal representation,
with a leading `-` when negative. The text form of a `bool` is `true` or
`false`, and of a `str` is the string itself; those of a `float` and a `char`
are given by `type.num.7` and `type.char.1`. `bool` and `str` support `==`
and `!=`; `str` equality compares UTF-8 bytes. `str` also supports `<`, `<=`,
`>` and `>=` (`trait.std.4`). *(ADR 0116, ADR 0127, ADR 0130, ADR 0134)*

**type.str.4** — `s.split(sep)` returns a `List[str]` of the pieces of `s`
between occurrences of `sep`, keeping empty pieces, so `n` occurrences give
`n + 1` pieces. An empty `sep` traps. `sep` is a `str` or a `char`
(`type.str.6`). *(ADR 0124, ADR 0135)*

**type.str.5** — `s.to_int()` returns an `int?`. It holds the value of `s`
when `s` is an optional `-` followed by one or more decimal digits and the
value fits in `int`; otherwise it is `none`. *(ADR 0124)*

**type.str.6** — The `str` methods that search for text (`contains`,
`starts_with`, `ends_with` and `split`) accept a `char` as well as a `str`;
a `char` means the one-character string. `strip_prefix` and `strip_suffix`
take a `str` only. *(ADR 0135)*

**type.str.7** — A `str` has these methods. *(ADR 0135)*
- `s.chars()` returns a new `List[char]`. Strings stay unindexable
  (`type.str.2`).
- `s.lines()` returns a `List[str]`: `s` split on `\n`, with one trailing
  empty piece dropped and a `\r` before each `\n` removed (Python's
  `splitlines` for `\n` and `\r\n`).
- `s.trim()` removes the leading and trailing characters for which
  `is_space()` is true (`type.char.2`).
- `s.contains(sub)`, `s.starts_with(p)` and `s.ends_with(p)` return a `bool`.
- `s.strip_prefix(p)` and `s.strip_suffix(p)` return the rest of `s` as a
  `str?`, or `none` when `s` doesn't start (end) with `p`.

```ail fragment
"a\r\nb\n".lines()             # ["a", "b"]
"a\n\n".lines()                # ["a", ""]
"  hi ".trim()                # hi
"key=v".strip_prefix("key=")  # v
"h\u{e9}".chars()              # ['h', 'é']
```

**type.str.8** — `s.to_float()` returns a `float?` (`type.num.8`).
`n.to_hex()` on an `int` returns its hexadecimal digits, lowercase and without
a prefix, with a `-` before the magnitude of a negative value, so the minimum
`int` is `-8000000000000000`. *(ADR 0135, ADR 0137)*

**type.str.9** — `s.pad_left(width, fill)` and `s.pad_right(width, fill)`
add copies of the `char` `fill` on the left (right) until `s` has `width`
characters (scalar values). A string already that wide, or any string when
`width <= 0`, is returned unchanged. The fill is required: AIL has no default
arguments. *(ADR 0135)*

```ail fragment
"7".pad_left(3, '0')        # 007
"ab".pad_right(4, ' ')      # "ab  "
```

**type.str.13** — `s.slice(a, b)` returns the characters of `s` from index
`a` up to but excluding `b` (indexes count characters, as `len()` does:
`type.str.12`). It traps unless `0 <= a <= b <= s.len()`; bounds are never
clamped. `s.char_at(i)` returns the character at `i` as a `char?`, `none` if
`i` is out of range or negative. *(ADR 0215)*

**type.str.14** — `s.find(sub)` returns the character index of the first
occurrence of `sub` as an `int?`, or `none`. `s.replace(from, to)` returns a
copy with every non-overlapping occurrence of `from` replaced by `to`, left
to right; it traps if `from` is empty. `s.to_lower()` and `s.to_upper()`
return the Unicode lower- and upper-case forms. `sub`, `from` and `to` are
`str`s. *(ADR 0215)*

```ail fragment
s := "Héllo"
a := s.slice(1, 3)       # "él"
i := s.find("llo")       # 2
t := s.replace("l", "L") # "HéLLo"
```

**type.str.10** — `s.repeat(n)` returns `n` copies of `s` joined together.
It traps if `n < 0`; AIL doesn't clamp. *(ADR 0135)*

**type.str.12** — `s.len()` returns the number of characters (Unicode scalar
values) of `s`, the same as `s.chars().len()` without building the list; it
takes time proportional to the length. `s.byte_len()` returns the number of
UTF-8 bytes. Grapheme clusters are not counted: `"e\u{301}".len()` is 2.
*(ADR 0208)*

```ail fragment
n := "héllo".len()        # 5
b := "héllo".byte_len()   # 6
```

**type.str.11** — In `{x:spec}` (`lex.str.7`), `.N` formats a `float` with
exactly `N` decimal places, rounding the exact value half to even (`{2.5:.0}`
is `2`); `nan` and `inf` print as `{x}` does. `x` formats an `int` in lowercase
hex without `0x`, with a leading `-` for a negative number (as `n.to_hex()`).
Using `.N` on a non-float or `x` on a non-int is a compile-time error. Width,
fill and alignment apply to the text form of any value. *(ADR 0173)*

```ail error=type.str.11
n := 1
print("{n:.2}")      # compile-time error: write {n.to_float():.2}
```

> **Note:** joining is `xs.join(sep)` on any list (`type.coll.9`). There is
> no `str.from_chars`. `s.len()` is absent until bytes versus characters is
> decided (open questions).

## Characters

**type.char.1** — `char` is a Unicode scalar value (`type.str.1`). It
implements `Eq`, `Ord` (by code point), `Hash` and `ToStr`. Its text form at
the top level is the character itself. Inside another value it is a character
literal in single quotes, escaped as in `trait.auto.2`. *(ADR 0134)*

**type.char.2** — A `char` has these predicates and mappings. *(ADR 0134)*
- `is_digit()` is true for ASCII `0` to `9` only, so it implies that
  `to_digit(10)` is some value.
- `is_alpha()` is true for the Unicode `Alphabetic` property, and `is_alnum()`
  for `is_alpha()` or `is_digit()`.
- `is_space()` is true for the Unicode `White_Space` property.
- `is_upper()` and `is_lower()` test the Unicode `Uppercase` and `Lowercase`
  properties.
- `to_upper()` and `to_lower()` apply the single-character mapping. A
  character whose mapping is several characters (such as `ß`) is unchanged.

The Unicode version is the implementation's (the bootstrap's is that of the
pinned Rust toolchain).

**type.char.3** — `c.to_digit(radix)` returns an `int?`. The characters `0`
to `9`, `a` to `z` and `A` to `Z` are worth 0 to 35. The result is `none` if
`c` isn't one of those or its value is `>= radix`. A `radix` outside `2..36`
traps. *(ADR 0134)*

**type.char.4** — `c.code()` returns the scalar value of `c` as an `int`.
`char.from_code(n)` returns a `char?`: `none` for surrogates and values
outside the scalar range. The diagnostics for `ord(c)`, `chr(n)` and
`c.to_int()` suggest `c.code()`, `char.from_code(n)` and "`c.code()` or
`c.to_digit(10)`". *(ADR 0134)*

```ail fragment
c := 'f'
c.is_alpha()            # true
c.to_digit(16)          # 15
c.to_digit(10)          # none
c.code()                # 102
char.from_code(0x41)    # 'A'
```

## Collections

**type.coll.1** — The standard library provides `List[T]`, `Map[K, V]` and
`Set[T]`, all with value semantics (`own.val.1`). *(ADR 0090)*

**type.coll.2** — `[a, b, ...]` is a list literal. `{k: v, ...}` is a map
literal. `{a, b, ...}` is a set literal. *(ADR 0090)*

```ail fragment
ports := {"http": 80, "https": 443}
primes := [2, 3, 5, 7]
```

**type.coll.3** — The empty literals `[]` and `{}` require a known expected
type, except when they declare a local (`type.coll.13`). Otherwise they are a
compile-time error, whose help suggests the constructors of `type.coll.8`.
*(ADR 0090, ADR 0131, ADR 0143)*

**type.coll.4** — `xs[i]` traps if `i` is out of bounds. `xs[a..b]` yields
the sub-list from `a` up to but excluding `b`, as a value; `xs[a..=b]` also
includes element `b` (it is `xs[a..b + 1]`). A slice traps unless
`0 <= a <= b <= xs.len()`: bounds are never clamped, unlike Python. A negative
integer literal as an index or a bound is a compile-time error, whose help
suggests `xs[xs.len() - 1]` or `xs.last()`. *(ADR 0090, ADR 0136)*

**type.coll.5** — `xs.len()` returns the number of elements of a list as an
`int`. *(ADR 0116)*

**type.coll.6** — `xs.push(v)` appends `v` to a list. Its receiver is
`mut self` (`trait.method.3`). *(ADR 0116)*

**type.coll.7** — `xs.swap(i, j)` exchanges the elements at indexes `i` and
`j`, and does nothing when `i == j`. It traps if either index is out of
bounds. Its receiver is `mut self`. *(ADR 0119)*

**type.coll.8** — `List[T]()`, `Map[K, V]()` and `Set[T]()` give an empty
collection anywhere, with no expected type needed. `{}` where the expected
type is a `Map` or `Set` is the empty map or set; with no expected type it is
the error of `type.coll.3`. *(ADR 0131)*

```ail fragment
seen := Set[int]()
counts := Map[str, int]()
names := List[str]()
```

**type.coll.9** — A list has these methods. *(ADR 0136)*
- `xs.pop()` returns a `T?`: it removes and returns the last element, or is
  `none` if the list is empty. Its receiver is `mut self`.
- `xs.last()` returns the last element as a `T?`, or `none` if empty.
- `xs.contains(x)` returns a `bool`. `T` MUST be `Eq`.
- `xs.index_of(x)` returns the index of the first element equal to `x` as an
  `int?`, or `none` if absent. `T` MUST be `Eq`. (`xs.index_of(x) == -1` is an
  error: `expr.cmp.3`.)
- `xs.join(sep)` returns a `str`. `T` may be any type: each element
  contributes its text form, with `str` and `char` elements unquoted at the
  top level and nested values in their quoted forms (`trait.auto.2`).

```ail fragment
xs := ["a", "b"]
xs.join(", ")           # a, b
[1, 2, 3].join("")      # 123
[["a"], ["b"]].join("+")  # ["a"]+["b"]
xs.index_of("b")        # 1
```

**type.coll.14** — A list has these further methods. *(ADR 0214)*
- `xs.insert(i, x)` puts `x` at index `i`, moving later elements up. `i` MAY
  equal `xs.len()` (append). Receiver `mut self`.
- `xs.remove_at(i)` removes and returns the element at `i` as a `T`.
  Receiver `mut self`.
- `xs.extend(ys)` appends the elements of the `List[T]` `ys`. Receiver
  `mut self`.
- `xs.clear()` removes every element and `xs.reverse()` reverses in place.
  Receivers are `mut self`.
- `xs.first()` returns the first element as a `T?`, or `none` if empty.

`insert` traps unless `0 <= i <= xs.len()` and `remove_at` traps unless
`0 <= i < xs.len()`, both under rule `type.coll.4`. There is no `xs[a..b]`
method: slicing is `type.coll.4`.

```ail fragment
mut xs := [1, 2, 3]
xs.insert(1, 9)       # [1, 9, 2, 3]
n := xs.remove_at(0)  # 1
xs.extend([4, 5])     # [9, 2, 3, 4, 5]
```

**type.coll.10** — Open-ended slices `xs[a..]`, `xs[..b]` and `xs[..=b]` follow
the bounds rule of `type.coll.4`. Open ranges exist only inside `[]`;
`xs[..]` is a compile-time error (write `xs`). *(ADR 0136)*

**type.coll.11** — `xs.sorted()` returns a new `List[T]` and `xs.sort()`
(`mut self`) sorts in place. `T` MUST be `Ord`, both are stable, and both use
`cmp` (so `float` follows `type.num.11`). `xs.min()` and `xs.max()` return a
`T?`: the first least (greatest) element, or `none` if the list is empty. An
inherent method takes precedence over a method of the same name from `Ord`
(`trait.res.2`), so `xs.min()` isn't `Ord`'s `min(other)`. A binding of
`xs.sort()`'s result gets a help suggesting `sorted()`. *(ADR 0136, ADR 0137)*

**type.coll.12** — `xs.sum()` exists only for `List[int]`, returning an `int`,
and `List[float]`, returning a `float`. It is a built-in special case that user
code can't declare (ADR 0128 is unchanged for user code). It starts at `0` or
`0.0` and adds left to right with `+`: an `int` overflow traps, and an empty
list gives `0` or `0.0`. For `float` it matches a Python loop, not Python
3.12's compensated `sum()`. *(ADR 0136)*

**type.coll.13** — A local declared with `:=` from `[]` or `{}` with no
expected type takes its type from the first statement that determines it:
`x.push(e)` (a `List`), `x.add(e)` (a `Set`), `x[k] = v` (a `Map`), `x = e`,
passing `x` where a parameter's type is known, returning `x`, or using it as
a field's value. Any other use before then, or none at all, is a
compile-time error whose help suggests `List[T]()`, `Map[K, V]()` or
`Set[T]()`. *(ADR 0143)*

```ail fragment
mut out := []
out.push("a")          # out is List[str]
```

## Maps and sets

**type.map.1** — Iteration, `keys()`, `values()` and the text form of a map or
set follow insertion order, and `s.sorted()` is a stable sort of a set's
elements in that order. Assigning to an existing key keeps its
position. `remove` keeps the order of the other entries, and a key added again
later goes last. In a literal, a later duplicate replaces the value and keeps
the first position; duplicate keys or elements that are equal constant values
are a compile-time error. *(ADR 0131)*

**type.map.2** — Map keys and set elements MUST be `Hash` (`trait.auto.5`).
This is checked where the type is written or inferred: `Map[float, int]` is a
compile-time error that names the cause. *(ADR 0131, ADR 0133)*

**type.map.3** — `m[k]` reads the value for `k` and traps if `k` is missing;
the message shows the key's text form. An entry `m[k]` of a map place is a
place (`own.place.1` amended), but only when `m[k]` is the whole target of `=`
does it insert or replace. Every other use traps if `k` is missing:
- the target of `op=` (`m[k] += 1`);
- a step before the last one in a longer place (`m[k].f = v`, `m[k][j] = v`);
  `m[a][b] = v` needs `m[a]` to exist and inserts `b`;
- the receiver of a `mut self` method (`m[k].push(x)`);
- a `mut` argument, and `for mut x in m[k]`.

`k` is evaluated once, before the right-hand side. *(ADR 0131)*

```ail fragment
mut m := Map[str, int]()
m["a"] = 1              # inserts
m["a"] = 2              # replaces; "a" keeps its position
m["a"] += 1             # 3
m["z"] += 1             # traps: key "z" is missing
mut g := Map[int, Map[int, str]]()
g[1] = Map[int, str]()
g[1][2] = "x"           # inserts key 2 into g[1]
g[3][4] = "y"           # traps: g has no key 3
```

**type.map.4** — Two maps are `==` when they have the same keys with equal
values (`V` MUST be `Eq`), and two sets when they have the same elements. Order
is ignored, so equal maps can print differently. Maps and sets aren't `Ord` or
`Hash`. The text form is `{"a": 1, "b": 2}` for a map, `{1, 2}` for a set, and
`{}` for an empty map or set. *(ADR 0131)*

**type.map.5** — `for x in s` iterates a set in insertion order, over the
value `s` had when the loop started (`expr.loop.4`, amended to allow a
`Set`). `for k in m` on a map is a
compile-time error whose help says to iterate `m.keys()`: a key and a pair are
both plausible readings. `for mut` over a set, or over `m.keys()` or
`m.values()`, is a compile-time error. *(ADR 0131)*

**type.map.6** — Maps and sets have these methods. *(ADR 0131)*
- Map: `m.len()` returns an `int`; `m.get(k)` returns a `V?`;
  `m.contains(k)` returns a `bool`; `m.remove(k)` (`mut self`) removes the
  entry and returns its value as a `V?`; `m.keys()` returns a `List[K]` and
  `m.values()` a `List[V]`, both new lists in insertion order.
- Set: `s.len()` returns an `int`; `s.add(x)` (`mut self`) returns whether `x`
  was added; `s.remove(x)` (`mut self`) returns whether `x` was there;
  `s.contains(x)` returns a `bool`; `s.sorted()` returns a `List[T]` (`T` MUST
  be `Ord`); `s.is_subset(t)` returns a `bool`.

The diagnostics for `m.get(k, d)`, `contains_key` and `items()` suggest
`v := m.get(k) else d`, `contains`, and iterating `keys()` and indexing.

```ail fragment
mut seen := Set[str]()
first := seen.add("a")  # true
again := seen.add("a")  # false: already there
mut ages := {"ann": 31, "bob": 27}
ages.get("cy")          # none
for k in ages.keys()
    print("{k}: {ages[k]}")
```
