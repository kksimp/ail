# 10. Methods and traits

Chapter key: `trait`

> **Status:** Drafted (partial). Specified: methods, associated functions,
> placement, traits, default methods, explicit implementations, coherence,
> call resolution, `Self`, operator traits, `any` dynamic dispatch, and the
> standard traits `Eq`, `Ord` and `ToStr`. Not yet designed: associated types
> (OQ-037), supertraits and associated functions in traits (OQ-039), and the
> arithmetic and indexing operator trait names.

## Methods

**trait.method.1** — A method is declared as `fn Type.name(self, ...)` or
`fn Type.name(mut self, ...)`. With `self`, the receiver is read-only. With
`mut self`, the method may mutate its receiver. *(ADR 0072)*

**trait.method.2** — A method is called as `value.name(arguments)`.
*(ADR 0072)*

**trait.method.3** — The receiver of a method declared with `mut self` MUST be
a mutable place. The call is not marked with `mut` (an exception to
`own.param.2`). Exclusivity (`own.excl.*`) applies between the receiver and
the arguments. *(ADR 0075, ADR 0064)*

```ail
fn Circle.scale(mut self, k f64)
    self.r = self.r * k

mut c := Circle(r: 1.0)
c.scale(2.0)
```

**trait.method.4** — A declaration `fn Type.name(...)` whose first parameter is
not `self` declares an associated function, called as `Type.name(arguments)`.
*(ADR 0082)*

**trait.method.5** — Methods and associated functions of a type MUST be
declared in the package that declares the type. *(ADR 0082)*

```ail
fn Circle.new(r f64) Circle
    Circle(r: r)

c := Circle.new(1.0)
```

## Traits

**trait.decl.1** — A trait is declared with the keyword `trait`, its name, and
an indented block of method signatures. *(ADR 0073)*

**trait.decl.2** — A method in a trait MAY have a body (a default
implementation). Implementations inherit it unless they define the method
themselves. *(ADR 0081)*

**trait.impl.1** — A type implements a trait only through an
`impl Trait for Type` block containing definitions of the trait's methods.
*(ADR 0073)*

**trait.impl.2** — There is no structural conformance. Having methods whose
signatures match a trait does not make a type implement it. *(ADR 0073)*

**trait.impl.3** — An `impl` block that omits a required method, or defines
one with a mismatched signature, is a compile-time error reported at the
`impl` block. The diagnostic MUST list each missing or mismatched method.
*(ADR 0073)*

```ail
trait Shape
    fn area(self) f64

impl Shape for Circle
    fn area(self) f64
        pi * self.r * self.r
```

**trait.impl.4** — `impl Trait for Type` MAY appear only in the package that
declares `Trait` or the package that declares `Type`. *(ADR 0082)*

**trait.impl.5** — An `impl` block defines only methods of its trait, each with
the trait's signature, with `Self` replaced by the implementing type.
*(ADR 0129)*

**trait.impl.6** — A type implements a trait at most once. *(ADR 0129)*

## `Self`

**trait.self.1** — In a trait, `Self` is the implementing type. In a method,
associated function or `impl` block, `Self` is the type it belongs to, with
its type parameters. `Self` is not a type anywhere else. *(ADR 0081,
ADR 0128)*

## Call resolution

**trait.res.1** — If a method call `x.f(...)` could refer to more than one
method (inherent or from any implemented trait), it is a compile-time error.
A qualified call such as `Shape.area(x)` selects one explicitly.
*(ADR 0082)*

## Operators

**trait.op.1** — The operators `+ - * / %`, unary `-`, `== != < <= > >=`
and indexing `a[i]` MAY be implemented for a type by implementing the
corresponding standard-library trait: `Eq` for `==` and `!=`, `Ord` for
`< <= > >=` (`trait.std.1–2`). *(ADR 0077, ADR 0127)*

**trait.op.2** — No other operator can be defined or implemented. *(ADR 0077)*

**trait.op.3** — Operator calls dispatch statically and perform no implicit
conversion of their operands. *(ADR 0077)*

## Dynamic dispatch

**trait.any.1** — `any Trait` is a type whose values may hold a value of any
type implementing `Trait`. Method calls on it dispatch dynamically.
*(ADR 0079)*

**trait.any.2** — A bare trait name is not a type, except as a parameter's
top-level type (see `gen.anon.1`). *(ADR 0079, ADR 0078)*

```ail
struct Scene
    shapes List[any Shape]
```

**trait.any.3** — Where a value of type `any Trait` is expected, a value of any
type implementing `Trait` converts to it implicitly. *(ADR 0129)*

**trait.any.4** — A trait method can be called on an `any Trait` value only if
`Self` appears in its signature only as the receiver. Calling any other method
is a compile-time error at the call. *(ADR 0129)*

**trait.any.5** — `any Trait` satisfies a constraint `Trait` only when every
method of `Trait` can be called through it (`trait.any.4`). *(ADR 0129)*

## Standard traits

**trait.std.1** — `trait Eq` has the method `fn eq(self, other Self) bool`.
`a == b` calls `a.eq(b)`, and `a != b` is `not a.eq(b)`. *(ADR 0127)*

**trait.std.2** — `trait Ord` has the method `fn cmp(self, other Self)
Ordering`, where `enum Ordering` has the variants `Less`, `Equal` and
`Greater`. `a < b` is true when `a.cmp(b)` is `Less`, `a <= b` when it isn't
`Greater`, and likewise for `>` and `>=`. A type that implements `Ord` also
implements `Eq`, and a constraint `Ord` provides `==`. *(ADR 0127)*

**trait.std.3** — `trait ToStr` has the method `fn to_str(self) str`. A value's
text form, used by `print`, interpolation and error reports, is its
`to_str()`. *(ADR 0127)*

**trait.std.4** — `int` and `str` implement `Ord`. `str` compares UTF-8 bytes
lexicographically. `List[T]` implements `Ord` when `T` does, comparing
lexicographically. *(ADR 0127)*

**trait.derive.1** — An `impl` of `Eq`, `Ord` or `ToStr` with no method
definitions derives the implementation: structs compare field by field in
declaration order; enums compare by variant declaration order, then field by
field. Every field's type MUST implement the trait. *(ADR 0127)*

```ail
struct Version
    major int
    minor int

impl Ord for Version     # compares major, then minor
```

**trait.auto.1** — Every type implements `Eq` automatically when its parts do:
`int`, `bool`, `str`, `List[T]` and `T?` when `T` does, and struct, enum and
`error` types whose fields all do. Two values are equal when they are the
same variant with equal fields; lists are equal element by element.
*(ADR 0126)*

**trait.auto.2** — Every type implements `ToStr` automatically. The text form
of an `int`, `bool` or `str` is given by `type.str.3`. A struct's is its name
and fields, `Point(x: 1, y: 2)`; a variant's is its enum and variant names and
fields, `Dir.North` or `Shape.Circle(r: 1)`; a list's is `[1, 2]`; an absent
optional's is `none`, and a present one's is its value's. Inside these forms,
strings are quoted, and other values use their own text form. *(ADR 0126)*

**trait.auto.3** — An explicit `impl Eq for T` or `impl ToStr for T` replaces the
automatic implementation. *(ADR 0126)*

**trait.auto.4** — A method named `eq` or `to_str` MUST NOT be declared outside
an `impl` of `Eq` or `ToStr`. The diagnostic SHOULD suggest the `impl`.
*(ADR 0126)*

```ail
impl ToStr for Dir
    fn to_str(self) str
        match self
            North => "north"
            South => "south"
```
