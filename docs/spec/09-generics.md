# 09. Generics

Chapter key: `gen`

> **Status:** Drafted (partial). Specified: bracket syntax (`gram.generic.1`),
> generic types, constraints, methods on generic types, inference, anonymous
> generic parameters, and the compilation-strategy freedom. Not yet designed:
> variance, const generics, and methods for one instantiation only.

## Generic declarations

**gen.decl.1** — Functions, `struct`, `enum` and `error` type declarations MAY
take type parameters in brackets after their name: `fn largest[T Ord](...)`,
`struct Stack[T]`. Every use of a generic type supplies its type arguments
(`Stack[int]`), except where `gen.method.1` and `gen.infer.1` allow them to be
left out. *(ADR 0041, ADR 0128)*

**gen.decl.2** — A type parameter MUST NOT reuse the name of a type, a trait,
or another type parameter in scope. *(ADR 0128)*

## Constraints

**gen.bound.1** — A generic type parameter MAY be followed by one or more
constraints, joined with `+`, inside the generic parameter brackets:
`[T Shape]`, `[K Hash + Eq, V]`. *(ADR 0074)*

**gen.bound.2** — The body of a generic declaration is type-checked using only
the operations its declared constraints provide. Using any other operation on a
value of a type parameter is a compile-time error at the declaration.
*(ADR 0074)*

```ail
fn largest[T Ord](items List[T]) T
    mut best := items[0]
    for item in items
        if item > best
            best = item
    best
```

**gen.bound.3** — Every type argument MUST satisfy its parameter's
constraints. The diagnostic names the type and the trait it doesn't
implement. *(ADR 0129)*

**gen.bound.4** — The constraints declared on a generic type's parameters
apply wherever the type is used, and are assumed in its methods and `impl`
blocks. *(ADR 0128)*

## Methods on generic types

**gen.method.1** — A method or associated function of a generic type is
declared with the type's bare name, `fn Stack.push(mut self, x T)`. The type's
parameters are in scope under their declared names, with their declared
constraints. The same holds for `impl Trait for Stack`. *(ADR 0128)*

**gen.method.2** — To add constraints for one method or `impl`, the type's
parameters are written after its name, in order, under their declared names,
with the added constraints: `fn Stack[T Ord].max(self) T?`. Such a method or
implementation exists only for instances whose type arguments satisfy them.
*(ADR 0128)*

**gen.method.3** — A method MAY declare type parameters of its own, after its
name: `fn Pair.with_first[C](self, c C) Pair[C, B]`. *(ADR 0128)*

```ail
struct Stack[T]
    items List[T]

fn Stack.new() Self
    Stack(items: [])

fn Stack.push(mut self, x T)
    self.items.push(x)
```

## Inference

**gen.infer.1** — The type arguments of a call to a generic function or
associated function, of a struct literal, and of a variant are inferred from
the arguments and, failing that, from the expected type. If any remains
unknown, it is a compile-time error, and the diagnostic SHOULD suggest
explicit type arguments. *(ADR 0129)*

**gen.infer.2** — Explicit type arguments are written in brackets after the
name of a generic function or type: `parse[int](s)`, `Stack[int].new()`,
`Pair[int, str](first: 1, second: "a")`. *(ADR 0041, ADR 0129)*

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
