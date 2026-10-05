# 0143. An empty `[]` or `{}` declared with `:=` takes its type from its first use

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** Designer
- **Topic:** 21 — Standard library; 4 — Type system
- **Spec rules:** [type.coll.3](../spec/04-types.md), [type.coll.13](../spec/04-types.md)
- **Amends:** [ADR 0090](0090-collections.md) (empty literals needed a known expected type)
- **Evidence:** [program-measurements-r2.md](../design/program-measurements-r2.md) (Python parity round)

## Context

`mut out := []` was an error with no expected type, so programs wrote
`mut out := List[str]()`: 25 times in the three example programs, 0.8% of
their tokens. Python and Rust programmers write the bare form first.

## Decision

1. When a local is declared with `:=` (or `mut ... :=`) from `[]` or `{}`
   with no expected type, its type is fixed by the first statement that
   determines it, in program order:
   - `x.push(e)`: `List[T]` with `e`'s type;
   - `x.add(e)`: `Set[T]`;
   - `x[k] = v`: `Map[K, V]`;
   - `x = e`, passing `x` where a parameter's type is known, returning `x`,
     or using it as a field's value: that type.
2. Any other use before the type is known (reading it, iterating it,
   `len()`) is a compile-time error whose help suggests `List[T]()`,
   `Map[K, V]()` or `Set[T]()`. So is a declaration that is never used in a
   way that fixes the type.

```ail
mut out := []
for x in xs
    out.push("{x}")     # out is List[str]
mut seen := {}
seen.add(3)             # seen is Set[int]
```

## Consequences

- The type of such a local is found by reading forward to its first use,
  not at the declaration. The explicit constructors remain for when the
  type should be visible up front.

## Rejected alternatives

- **Keep the error (ADR 0090):** the type is always visible at the
  declaration, but a Python-style first attempt fails.
- **Full type inference** (Hindley–Milner) for every local: far more machinery
  for no further gain in the measured programs.
