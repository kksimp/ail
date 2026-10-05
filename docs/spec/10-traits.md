# 10. Methods and traits

Chapter key: `trait`

> **Status:** Drafted (partial). Specified: methods, associated functions,
> placement, traits, default methods, explicit implementations, coherence,
> call resolution, `Self`, operator traits, `any` dynamic dispatch, and the
> standard traits `Eq`, `Ord`, `ToStr` and `Hash`. Not yet designed: associated
> types (OQ-037), supertraits and associated functions in traits (OQ-039), the
> arithmetic and indexing operator trait names, and custom hashing.

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
A qualified call such as `Shape.area(x)` selects one explicitly. `trait.res.2`
makes an exception for standard traits' default methods. *(ADR 0082, ADR 0137)*

**trait.res.2** — A built-in or inherent method takes precedence over a
default method with the same name of a standard (prelude) trait, so
`trait.res.1` reports no ambiguity between them. A list's `xs.min()`
(`type.coll.11`) therefore hides `Ord`'s `min(other)`, and a user type's own
`min` keeps working when the type implements `Ord`. Methods of user traits
stay ambiguous under `trait.res.1`. *(ADR 0137)*

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
`a == b` calls `a.eq(b)`, and `a != b` is `not a.eq(b)`. For `float`, `==` and
`!=` are the IEEE 754 operators of `type.num.6` wherever they are written,
including in generic code through an `Eq` bound: NaN is unequal to itself and
`-0.0 == 0.0`, so `float` equality isn't reflexive. *(ADR 0127, ADR 0130)*

**trait.std.2** — `trait Ord` has the method `fn cmp(self, other Self)
Ordering`, where `enum Ordering` has the variants `Less`, `Equal` and
`Greater`. `a < b` is true when `a.cmp(b)` is `Less`, `a <= b` when it isn't
`Greater`, and likewise for `>` and `>=`. A type that implements `Ord` also
implements `Eq`, and a constraint `Ord` provides `==`. For `float`, the four
operators are instead the IEEE comparisons of `type.num.6`, also through an
`Ord` bound, so each is false when an operand is NaN and they don't follow
`cmp`; `float`'s `cmp` is the total order of `type.num.11`. `Ord` also has
the default methods `a.min(b)` and `a.max(b)`: `a.min(b)` is `b` if
`b.cmp(a)` is `Less`, else `a`; `a.max(b)` is `b` if `b.cmp(a)` is `Greater`,
else `a`, so ties give `a`. Every `Ord` type has them, in generic code too.
For `float` they use `cmp`, so NaN counts as the greatest value and
`x.min(nan)` is `x`. *(ADR 0127, ADR 0130, ADR 0137)*

```ail
fn clamp[T Ord](x T, lo T, hi T) T
    x.max(lo).min(hi)
```

**trait.std.3** — `trait ToStr` has the method `fn to_str(self) str`. A value's
text form, used by `print`, interpolation and error reports, is its
`to_str()`. *(ADR 0127)*

**trait.std.4** — `int`, `str`, `float` and `char` implement `Ord`. `str`
compares UTF-8 bytes lexicographically, and `char` compares by code point.
`float`'s `cmp` is the total order of `type.num.11`: numbers in numeric order,
`-0.0` equal to `0.0`, and NaN equal to NaN and greater than every number.
`List[T]` implements `Ord` when `T` does, comparing lexicographically with
`T`'s `cmp`, so a `List[float]` compares by the total order; a derived `Ord`
(`trait.derive.1`) likewise compares `float` fields by `cmp`. *(ADR 0127,
ADR 0130, ADR 0134)*

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
`int`, `bool`, `str`, `char` and `float` (whose `==` is IEEE, `trait.std.1`);
`List[T]`, `Set[T]` and `T?` when `T` does; `Map[K, V]` when `V` does (`K` is
always `Hash`, so `Eq`); and struct, enum and `error` types whose fields all
do. Two values are equal when they are the same variant with equal fields;
lists are equal element by element; maps are equal when they have the same
keys with equal values, and sets when they have the same elements, in any
order (`type.map.4`). Parts are compared with their own `==`, so a value
holding a NaN is unequal to itself. *(ADR 0126, ADR 0130, ADR 0131,
ADR 0134)*

**trait.auto.2** — Every type implements `ToStr` automatically. The text form
of an `int`, `bool` or `str` is given by `type.str.3`, of a `float` by
`type.num.7`, and of a `char` by `type.char.1`. A struct's is its name and
fields, `Point(x: 1, y: 2)`; a variant's is its enum and variant names and
fields, `Dir.North` or `Shape.Circle(r: 1)`; a list's is `[1, 2]`; a map's is
`{"a": 1, "b": 2}`; a set's is `{1, 2}`; an empty map and an empty set are
both `{}`. Maps and sets follow insertion order (`type.map.1`). An absent
optional's is `none`, and a present one's is its value's. Inside these forms,
a `str` is quoted with `"` and a `char` with `'`, and other values use their
own text form (a `float` as at top level). Within the quotes, `\\`, the
delimiting quote, and the control characters `\n`, `\t`, `\r` and `\0` are
escaped with a backslash, every other control character (category Cc) is
written `\u{hex}` in lowercase hex without padding, and everything else is
kept as is, including `{` and the other kind of quote. *(ADR 0126,
ADR 0135, ADR 0130, ADR 0131, ADR 0134)*

```ail
print(["a\"b", "x\ty"])     # ["a\"b", "x\ty"]
print(['x', '\'', '{'])      # ['x', '\'', '{']
print({"k": [1.5, 2.0]})     # {"k": [1.5, 2.0]}
```

**trait.auto.3** — An explicit `impl Eq for T` or `impl ToStr for T` replaces the
automatic implementation. For the effect on `Hash`, see `trait.auto.5`.
*(ADR 0126, ADR 0133)*

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

**trait.auto.5** — `Hash` is a prelude trait with no methods. A type is
`Hash` when its `Eq` is automatic (`trait.auto.1`) or derived by an empty
`impl Eq` (`trait.derive.1`), and all its parts are `Hash`: `int`, `bool`,
`str`, `char`, `Ordering`, `T?` and `List[T]` when `T` is, and struct, enum
and `error` types whose fields all are. A generic type is `Hash` per instance
(`Box[int]` is, `Box[float]` isn't). `float`, `Map`, `Set` and `any Trait`
aren't `Hash`. A hand-written `fn eq` (`trait.auto.3`) removes `Hash`. An
`impl Ord` doesn't change `Eq` (`==` never calls `cmp`), so a type with
`impl Ord` and an automatic `Eq` stays `Hash`. `impl Hash` is a compile-time
error, since custom hashing isn't designed. A constraint `[T Hash]` implies
`Eq`. Map keys and set elements MUST be `Hash` (`type.map.2`). Hash values are
never visible to programs, so output can't depend on the hash function.
*(ADR 0133)*
