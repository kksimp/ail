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

**type.num.5** — Integer types support `+ - * / %`, unary `-` and the
comparisons `== != < <= > >=`. Sizes, lengths and indexes have type `int`
(this is the size and index type of `plat.ptr.2`). *(ADR 0116)*

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

**type.struct.4** — A struct literal MUST name every field of the struct
exactly once, in any order. Missing, unknown and repeated fields are
compile-time errors. *(ADR 0113)*

**type.struct.5** — A struct's field names MUST be distinct. Each value in a
struct literal MUST have its field's type. *(ADR 0117)*

**type.struct.6** — `e.f` reads field `f` of `e`. The type of `e` MUST be a
struct that declares `f`. *(ADR 0117)*

> **Note:** field shorthand and defaults are not yet decided.

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

**type.enum.5** — An enum's variant names MUST be distinct. *(ADR 0117)*

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

## Booleans

**type.bool.1** — `bool` is the type of the literals `true` and `false`.
Logical operators take and produce `bool` (`expr.op.1`). *(ADR 0110)*

## Strings and characters

**type.str.1** — `str` is the type of immutable UTF-8 text. `char` is the type
of a single Unicode scalar value. *(ADR 0089)*

**type.str.2** — `str` doesn't support the indexing operator. Byte and
character access go through explicit views (`s.bytes`, `s.chars()`).
*(ADR 0089)*

**type.str.3** — The text form of an `int` is its decimal representation,
with a leading `-` when negative. The text form of a `bool` is `true` or
`false`, and of a `str` is the string itself. `bool` and `str` support `==`
and `!=`; `str` equality compares UTF-8 bytes. *(ADR 0116)*

## Collections

**type.coll.1** — The standard library provides `List[T]`, `Map[K, V]` and
`Set[T]`, all with value semantics (`own.val.1`). *(ADR 0090)*

**type.coll.2** — `[a, b, ...]` is a list literal. `{k: v, ...}` is a map
literal. `{a, b, ...}` is a set literal. *(ADR 0090)*

**type.coll.3** — The empty literals `[]` and `{}` require a known expected
type. Otherwise they are a compile-time error. *(ADR 0090)*

**type.coll.4** — `xs[i]` traps if `i` is out of bounds. `xs[a..b]` yields
the sub-list from `a` up to but excluding `b`, as a value. *(ADR 0090)*

**type.coll.5** — `xs.len()` returns the number of elements of a list as an
`int`. *(ADR 0116)*

**type.coll.6** — `xs.push(v)` appends `v` to a list. Its receiver is
`mut self` (`trait.method.3`). *(ADR 0116)*

```ail
ports := {"http": 80, "https": 443}
primes := [2, 3, 5, 7]
```
