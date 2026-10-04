# 04. Types

Chapter key: `type`

> **Status:** Drafted (partial). Specified: numeric types, structs, enums,
> optional types, strings and characters, and the standard collections. Not
> yet designed: fixed-size arrays, SIMD vector types, and const generics.

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

## Structs

**type.struct.1** — A struct type is declared with the keyword `struct`, its
name, and an indented block containing one field declaration per line.
*(ADR 0052)*

**type.struct.2** — A field declaration is written `name Type`. *(ADR 0052,
ADR 0037)*

```ail
struct User
    id i64
    name str
    email str?
```

**type.struct.3** — A struct value is constructed by the type name followed
by a parenthesised, comma-separated list of `field: value` pairs.
*(ADR 0058)*

```ail
u := User(id: 7, name: "Ada", email: none)
```

> **Note:** field shorthand, defaults, and field ordering are not yet decided.
> The spelling of the absent optional value (`none` above) is illustrative.

## Enums

**type.enum.1** — An enum (sum) type is declared with the keyword `enum`, its
name, and an indented block containing one variant per line. *(ADR 0053)*

**type.enum.2** — A variant either carries no data or carries a parenthesised
list of fields, each written `name Type`. *(ADR 0053)*

```ail
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

## Optional types

**type.opt.1** — AIL has no null value. *(ADR 0048)*

**type.opt.2** — `T?` denotes an optional value of type `T`. It is exactly
equivalent to `Option[T]`. *(ADR 0048)*

**type.opt.3** — The canonical form of an optional type is `T?`.
*(ADR 0048)*

```ail
fn find(id i64) User?
    ...
```

> **Note:** see also `plat.ptr.2` (the 64-bit size and index type) and
> `expr.arith.*` (integer arithmetic).

**type.opt.4** — `none` is the absent value of every optional type.
*(ADR 0093)*

**type.opt.5** — Where a value of type `T?` is expected, a value of type `T`
converts implicitly to a present optional. *(ADR 0093)*

## Strings and characters

**type.str.1** — `str` is the type of immutable UTF-8 text. `char` is the type
of a single Unicode scalar value. *(ADR 0089)*

**type.str.2** — `str` doesn't support the indexing operator. Byte and
character access go through explicit views (`s.bytes`, `s.chars()`).
*(ADR 0089)*

## Collections

**type.coll.1** — The standard library provides `List[T]`, `Map[K, V]` and
`Set[T]`, all with value semantics (`own.val.1`). *(ADR 0090)*

**type.coll.2** — `[a, b, ...]` is a list literal. `{k: v, ...}` is a map
literal. `{a, b, ...}` is a set literal. *(ADR 0090)*

**type.coll.3** — The empty literals `[]` and `{}` require a known expected
type. Otherwise they are a compile-time error. *(ADR 0090)*

**type.coll.4** — `xs[i]` traps if `i` is out of bounds. `xs[a..b]` yields
the sub-list from `a` up to but excluding `b`, as a value. *(ADR 0090)*

```ail
ports := {"http": 80, "https": 443}
primes := [2, 3, 5, 7]
```
