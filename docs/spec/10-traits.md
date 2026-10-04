# 10. Methods and traits

Chapter key: `trait`

> **Status:** Drafted (partial). Specified: methods, associated functions,
> placement, traits, default methods, explicit implementations, coherence,
> call resolution, operator traits, and `any` dynamic dispatch. Not yet
> designed: associated types (OQ-037), object safety, and the standard
> operator trait names.

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

## Call resolution

**trait.res.1** — If a method call `x.f(...)` could refer to more than one
method (inherent or from any implemented trait), it is a compile-time error.
A qualified call such as `Shape.area(x)` selects one explicitly.
*(ADR 0082)*

## Operators

**trait.op.1** — The operators `+ - * / %`, unary `-`, `== != < <= > >=`
and indexing `a[i]` MAY be implemented for a type by implementing the
corresponding standard-library trait. *(ADR 0077)*

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
