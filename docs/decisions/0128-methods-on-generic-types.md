# 0128. Methods and `impl`s on a generic type use its bare name; `Self`

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 9 — Generics; 10 — Traits
- **Spec rules:** [gen.decl.1–2](../spec/09-generics.md), [gen.bound.4](../spec/09-generics.md), [gen.method.1–3](../spec/09-generics.md), [trait.self.1](../spec/10-traits.md)
- **Extends:** [ADR 0072](0072-dot-named-methods.md), [ADR 0073](0073-explicit-trait-impls.md), [ADR 0074](0074-inline-generic-bounds.md)
- **Evidence:** `tools/tokbench/corpus/r7/m*.ail`

## Context

`struct Stack[T]` needs methods. ADR 0072 declares methods as
`fn Type.name(...)`, which leaves open how a method names the type's
parameters.

## Options considered

A generic stack with six methods, a generic pair and an `impl`, plus a short
`main` (mean of five tokenizers):

| Option | Example | Tokens |
|---|---|---|
| Explicit parameters on every method (Rust, Go, Kotlin) | `fn Stack[T].push(mut self, x T)` | 267.4 |
| **Bare name; the type's parameters are in scope (Swift extensions) (chosen)** | `fn Stack.push(mut self, x T)` | **251.8** (−5.8%) |

Using `Self` for the return type saves another 1.6 in either form.

## Deciding criterion

**Token usage** decides: about 2.2 tokens per method header, 5.8% on library
code. **AI understanding** costs little:
- methods must be declared in the type's package (ADR 0082), so `T` is
  always a short lookup away;
- the brackets stay available, and are needed, to add bounds, so there's
  one form per meaning;
- the trait-parameter shorthand (ADR 0078) already uses this pattern: short
  by default, brackets when needed.

## Decision

1. `struct`, `enum` and `error` type declarations may take type parameters,
   with bounds: `struct Stack[T]`, `struct Sorted[T Ord]`. A type parameter
   can't reuse the name of a type or trait.
2. A method or associated function of a generic type is declared with the
   type's bare name: `fn Stack.push(mut self, x T)`. The type's parameters are
   in scope under their declared names, with their declared bounds.
3. To add bounds for one method or `impl`, write the type's parameters in
   order, under their declared names, with the extra bounds:
   `fn Stack[T Ord].max(self) T?`, `impl ToStr for Pair[A ToStr, B]`. The
   method exists only for instances that satisfy them.
4. A method may have type parameters of its own, after its name:
   `fn Pair.with_first[C](self, c C) Pair[C, B]`.
5. `Self` is the type a method, associated function or `impl` belongs to,
   with its parameters (`Stack[T]`), or the implementing type in a trait.
6. The bounds declared on a type's parameters apply wherever the type is
   used, and are assumed in its methods and `impl`s.

```ail excerpt
struct Stack[T]
    items List[T]

fn Stack.new() Self
    Stack(items: [])

fn Stack.push(mut self, x T)
    self.items.push(x)

fn Stack[T Ord].max(self) T?
    ...
```

## Consequences

- A method for one instantiation only (`fn Pair[int, int].sum`) isn't
  allowed yet.
- The formatter keeps the brackets only when they add bounds.

## Rejected alternatives

- **Explicit parameters on every method:** +5.8% tokens on library code, for
  information one lookup away.
- **`impl Type` blocks:** ADR 0072 chose dot-named functions.
