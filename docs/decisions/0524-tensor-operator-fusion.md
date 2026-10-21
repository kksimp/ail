# 0524. Operator fusion for elementwise tensor chains

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 11 — Operators; standard library (`std.tensor`)
- **Spec rules:** `std.tensor.20` (fusion) — new

## Context

A chain such as `(a * b + a).relu()` builds one tensor per operator, so a
chain of five operations on a million elements writes and reads five
temporary arrays. The batch's speed goal is to beat CPython on tensor code,
and per-operator temporaries are the cost a chain adds over the kernels.

## Decision

- The checker finds maximal chains of two or more elementwise operations
  over names and float literals (`+ - * /`, unary `-`, `relu()`, `sigmoid()`,
  `tanh()` with no arguments) and marks the outermost node (`resolved.fused`).
  The interpreter runs a marked chain only when its plan still exists, so the
  two agree.
- At run time, when every tensor input has the same shape and is a `float`
  tensor (floats act as scalars), the chain runs chunk by chunk: 1024 elements
  at a time, one operator per tight loop over the chunk, using a stack of
  chunk-sized buffers. Each element gets the same IEEE operation in the same
  order as the unfused calls, so results are bit-identical.
- Otherwise the chain runs unfused, one operator at a time, each with its own
  position, so traps name the same element and line.
- Both tiers use the one plan in `bootstrap/src/fuse.rs`. The bytecode tier
  has one new instruction, `Fused`. The test `tensor_fusion` holds the tree
  walker and the bytecode tier to identical output, equal to the unfused
  program.

## Options considered

- **Chunked fusion over a checker-marked chain (chosen).** Shared plan, both
  tiers consult it, small code, and the fallback is the existing code.
  Measured faster than unfused (see below).
- **One pass per element with a postfix stack.** Built first and rejected on
  measurement: 1.01 s against 0.67 s unfused, because the per-element match
  on each step costs more than the saved arrays. Chunking fixed it.
- **Lazy tensors that build a graph and run on `print` or `to_list`.**
  Rejected: a tensor is a value (ADR 0060), and a lazy result is hard to
  explain in diagnostics.
- **Fusion across statements (`x := a * b; y := x + a`).** Deferred: the
  intermediate `x` is visible to the program, and fusing it needs a liveness
  analysis the checker does not do.
- **`fma` or reassociation for speed.** Rejected: it changes rounding, and
  std.tensor.20 promises identical results.
- **A generic interpreter for mixed shapes.** Rejected: broadcasting already
  materializes its expansion (ADR 0522); a second path would add code without
  a measured gain.

## Consequences

- A chain of two or more tensor operations allocates one result (plus chunk
  buffers) instead of one array per operator, when inputs share a shape.
- Diagnostics and traps are unchanged: the fallback keeps each operator's
  position, and the fused path traps at the same element as the unfused one.

## Measurements

Linux x86-64, release build, `tests` chain `(a * b + a - b / 2.0).relu().tanh()`
summed over 20 passes on 1,000,000 elements (times are the best of two runs
and include the start-up of `ail run`):

| Program | Time |
|---|---|
| AIL, fused (default tier) | 0.29 s |
| AIL, fused (tree walker) | 0.28 s |
| AIL, unfused (one operator per statement) | 0.41–0.46 s |
| AIL, per-element fusion (rejected first attempt) | 1.01 s |
| CPython, pure lists (reference only) | 4.58 s |

The CPython figure is a pure-Python loop, not NumPy, so it shows the
language gap rather than the best Python can do. The fused and unfused
outputs print identically. tokbench was not run: no syntax changed.

## Rejected alternatives summary

Per-element stack loop (slow, measured), lazy tensors, cross-statement
fusion (deferred), `fma` or reassociation (changes results), and a generic
mixed-shape interpreter.
