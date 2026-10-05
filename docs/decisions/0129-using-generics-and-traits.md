# 0129. Type arguments are inferred; values convert to `any Trait` implicitly; rules for `impl` blocks and `any` calls

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 9 — Generics; 10 — Traits
- **Spec rules:** [gen.infer.1–2](../spec/09-generics.md), [gen.bound.3](../spec/09-generics.md), [trait.impl.5–6](../spec/10-traits.md), [trait.any.3–5](../spec/10-traits.md), [expr.call.2](../spec/05-expressions.md), [expr.cmp.1](../spec/05-expressions.md)
- **Extends:** [ADR 0041](0041-square-bracket-generics.md), [ADR 0073](0073-explicit-trait-impls.md), [ADR 0079](0079-any-dynamic-dispatch.md)

## Deciding criterion

**Token usage** decides inference and implicit conversion to `any`: writing
`largest(xs)` rather than `largest[int](xs)`, and `[c, r]` rather than
`[any(c), any(r)]`, on every use. The remaining rules **tie** on tokens, and
**AI understanding** decides: each error is reported where the mistake is,
and no call depends on information that isn't in the signatures.

## Decision

1. **Inference.** The type arguments of a generic function call, struct
   literal or variant are inferred from the arguments and, failing that, from
   the expected type. If they can't be, it's an error that suggests the
   explicit form.
2. **Explicit type arguments** follow the name of a generic function or type:
   `parse[int](s)`, `Stack[int].new()`, `Pair[int, str](first: 1, second: "a")`.
3. **Bounds** are checked at each use. The error names the type and the
   trait it lacks.
4. **Conversion to `any`.** Where `any Trait` is expected, a value of any type
   implementing `Trait` converts implicitly, like optional wrapping.
5. **Calls through `any`.** A trait method can be called on an `any Trait`
   value only if `Self` appears in its signature only as the receiver: a
   method taking `other Self` can't know that `other` has the same concrete
   type. Calling it is an error at the call. `any Trait` itself is always a
   valid type, and it satisfies a bound `Trait` only when every method of
   `Trait` can be called through it.
6. **`impl` blocks** define only the trait's methods, each with the trait's
   signature (with `Self` replaced). A type implements a trait at most once.
7. **Comparisons.** In `a == b` and `a != b`, `b`'s expected type is `a`'s, so
   `d == East` needs no qualification.

## Consequences

- An associated function with no arguments, such as `Stack.new()`, needs
  explicit type arguments unless the expected type is known: Rust and Swift
  infer them from later uses, which the bootstrap doesn't do.
- Associated functions declared inside traits (called `T.zero()`) aren't
  designed yet (OQ-039).

## Rejected alternatives

- **Always explicit type arguments:** tokens on every call.
- **Explicit `any(x)` conversion:** tokens on every use, while the target type
  already shows the cost (ADR 0079).
- **Rejecting traits with `Self` parameters as `any` types (Rust's object
  safety):** rejects whole traits for one method, and reports the problem at
  the type instead of at the call that can't work.
