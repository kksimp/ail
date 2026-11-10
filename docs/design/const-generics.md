# Const generics (proposed)

Status: proposed by lane H (language gaps); [ADR 0954](../decisions/0954-const-generics.md). Gap 11 of `/mnt/project-files/gaps/replace-c-cpp-rust-gaps-2026-11-05.md`. Spec 09 lists const generics as "not yet designed"; ADR 0090 defers fixed arrays and SIMD vectors to them.

## Why

Fixed-size arrays, matrices, SIMD vectors and ring buffers need a length in the type. Without it a `Mat4` is a `List[float]` with a runtime check, so the checker cannot reject a 3x4 times 3x4 and the backend cannot keep the value in registers. Rust and C++ users expect `[T; N]` and `template<int N>`.

## Proposed rules (slice 1)

- A type parameter list may hold const parameters: `struct Vec[T, const N int]`, `fn dot[const N int](a Vec[float, N], b Vec[float, N]) float`. A const parameter is named, then its type. Slice 1 allows `int` only.
- A const argument is an int literal or a module constant: `Vec[float, 4]`, `Vec[float, DIM]`. No arithmetic on arguments in slice 1 (`N + 1` is rejected, with a diagnostic naming the rule).
- Inside the declaration `N` is a compile-time `int` value, usable in expressions and in other types' arguments (`Vec[T, N]`).
- Inference unifies const arguments: `dot(a, b)` fixes `N` from `a` and requires `b` to match; a mismatch is a type error that names both values (`expected Vec[float, 4], found Vec[float, 3]`).
- Compilation is by monomorphisation (ADR 0536): each distinct `N` is one instance, so `N` is a literal in the generated code and loops over `0..N` unroll and vectorise as for any literal bound.
- Fixed arrays are then library code: `Array[T, N]` is a std struct with `N` slots, constant-indexed access is bounds-checked at compile time, and a literal `[a, b, c]` types as `Array[T, 3]` where an `Array` is expected.

## Rejected alternatives

- Bare `N int` without `const`: hides the bad-bound error `gen.bound.1` for `[T int]`. See ADR 0954.

- Const expressions in arguments (`Vec[T, N * 2]`): needs a solver for equality of expressions. Deferred to a later slice; no program needs it yet.
- Runtime-checked length (`List` plus assert): no compile-time dimension errors and no register-resident vectors. This is the status quo.
- `[T; N]` syntax: a new token form. Square-bracket generics (ADR 0041) already carry the argument list, and `Array[T, N]` costs about the same tokens (measure with tokbench before the compiler slice).
- Template-style duck-typed parameters: unclear diagnostics, against AI-first.

## Slices

1. Spec text and ADR (this change).
2. Parser and checker: const parameter declaration, arguments, inference, diagnostics with rule IDs. Native-only tests with `.out` goldens.
3. Lowering by monomorphisation in its own `compiler/mir/lower_constgen.ail` (ADR 0823).
4. `std` `Array[T, N]` and SIMD vector types on top.
