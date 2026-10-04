# 10. Methods and traits

Chapter key: `trait`

> **Status:** Drafted (partial). Method declarations, trait declarations, and
> explicit implementations are specified. Not yet designed: functions without
> `self`, method placement and coherence (OQ-036), default methods, associated
> types, operator traits, and dynamic dispatch.

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

## Traits

**trait.decl.1** — A trait is declared with the keyword `trait`, its name, and
an indented block of method signatures. *(ADR 0073)*

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
