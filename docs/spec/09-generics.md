# 09. Generics

Chapter key: `gen`

> **Status:** Drafted (partial). Specified: bracket syntax (`gram.generic.1`),
> constraints, anonymous generic parameters, and the compilation-strategy
> freedom. Not yet designed: variance and const generics.

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

## Anonymous generic parameters

**gen.anon.1** — A parameter whose type is written as a trait name declares an
anonymous type parameter constrained by that trait: `fn draw(s Shape)` is
equivalent to `fn draw[T Shape](s T)`. *(ADR 0078)*

**gen.anon.2** — The canonical form uses the anonymous form when the type
parameter would be referenced exactly once, and the named form otherwise.
*(ADR 0078)*

## Compilation

**gen.impl.1** — The semantics of generic code MUST NOT depend on whether an
instance is specialised or shares code with other instances. The
implementation chooses per instance and MUST report its choices in the
performance report. *(ADR 0080)*
