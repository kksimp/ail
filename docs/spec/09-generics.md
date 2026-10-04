# 09. Generics

Chapter key: `gen`

> **Status:** Drafted (partial). Bracket syntax (`gram.generic.1`) and
> constraints are specified. The compilation strategy, variance, and const
> generics are not yet designed.

## Constraints

**gen.bound.1** — A generic type parameter MAY be followed by one or more
constraints, joined with `+`, inside the generic parameter brackets:
`[T Shape]`, `[K Hash + Eq, V]`. *(ADR 0074)*

**gen.bound.2** — The body of a generic declaration is type-checked using only
the operations its declared constraints provide. Using any other operation on a
value of a type parameter is a compile-time error at the declaration.
*(ADR 0074)*

```ail
fn largest[T Shape](items List[T]) T
    mut best := items[0]
    for item in items
        if item.area() > best.area()
            best = item
    best
```

> **Note:** `for` loop syntax in the example is illustrative.
