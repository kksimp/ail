# 0954. Const generics: `int` parameters on types and functions

- **Status:** Proposed
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 09 — Generics
- **Spec rules:** [gen.const.1-5](../spec/09-generics.md)
- **Builds on:** [ADR 0041](0041-square-bracket-generics.md), [ADR 0090](0090-collections.md) (defers fixed arrays and SIMD vectors to const generics), [ADR 0536](0536-native-generics-by-monomorphisation.md), [ADR 0937](0937-grid-flat-2d-list.md)

## Context

Spec 09 lists const generics as not designed. Fixed arrays, matrices, SIMD vectors and ring buffers need a length in the type, so dimension errors are compile-time errors and the backend sees a literal bound. Rust and C++ both have it. Design detail: [const-generics.md](../design/const-generics.md).

## Decision

1. A type parameter list may declare const parameters, name then type: `struct Vec[T, N int]`. Slice 1 allows `int` only (gen.const.1).
2. A const argument is an int literal or a module constant. Expressions on parameters (`N + 1`) are rejected (gen.const.2).
3. `N` is a compile-time `int` inside the declaration and in other types' arguments (gen.const.3).
4. Inference unifies const arguments; a mismatch names both values (gen.const.4).
5. Compilation is by monomorphisation, one instance per argument list (gen.const.5).
6. **Fit with `Grid[T]` and `std.tensor`.** Both keep runtime shapes. Const generics are additive: a later `Grid[T, R int, C int]` or `Tensor[T, N int]` is a new type, or a sized view of the runtime one, and a runtime-shaped value converts to a sized one with one checked call (`g.sized[3, 4]()`, a trap on mismatch). Nothing in this decision changes ADR 0937 or the tensor API.

## Rejected alternatives

- Const expressions in arguments: needs an expression-equality solver; deferred until a program needs it.
- Runtime-checked length (status quo): no compile-time dimension errors, no register-resident vectors.
- `[T; N]` syntax: a new form; square-bracket generics already carry the list.
- Duck-typed template parameters: unclear diagnostics.

## Consequences

Checker and lowering land in later slices (parser and checker in `compiler/check/`, lowering in its own `compiler/mir/lower_constgen.ail`, ADR 0823). Status moves to Accepted when slice 2 passes the gate.
