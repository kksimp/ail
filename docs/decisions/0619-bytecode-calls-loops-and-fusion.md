# 0619. Bytecode tier: lean calls, loop shapes and fused instructions

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — Interpreter
- **Spec rules:** expr.loop.1-4, expr.while.1, type.coll.4-6, expr.op.2, own.param.2
- **Builds on:** [ADR 0618](0618-bytecode-typed-fast-paths.md), [ADR 0361](0361-bytecode-tier-design.md)

## Context

After ADR 0618 the tier was at 1.33x of CPython (geometric mean over eight
compute programs). Callgrind on a trivial counting loop showed 60 to 80
machine instructions per bytecode instruction, spread over operand decoding,
bounds checks, a second `match` on the operator, a call to drop the old value
on every register write, and, per call, a recursive `exec`, a register vector
resized and cleared, and a code lookup.

## Decision

1. **Plain calls skip the generic path** (`run_call`): a compiled function
   with no extern body, contracts, receiver or fallible result runs without
   the tracing, contract and error-wrapping checks of `run_body_inner`.
   Programs with tracing or unique values use the old path.
2. **Counted `for` keeps its variable in place**: `RangeTest` binds it once,
   `RangeNext` updates the slot and tests the bound. `continue` leaves the
   variable (a separate depth on the loop); `break` and the end remove it.
3. **`while` tests after the body**: the loop enters at the test, so a turn
   is one conditional branch instead of a test and a jump. A step is still
   counted after the test and before the body (run.limit.3).
4. **`xs.push(v)` and `xs.len()` on a list local** are one instruction
   (`ListFast`) when the argument is a literal or an unmoved local; any other
   receiver falls through to the walker's call, so user types with such
   methods are unchanged.
5. **`a[i][k]`** with a local `a` and effect-free indexes is one instruction
   (`Index2`) that reads the row in place instead of cloning it; other values
   index in two steps with their own trap positions.
6. **`xs[i + k]` / `xs[i - k]`** with an int literal `k` is one instruction
   (`IndexOff`); the sum is checked as the operator would be, and compound
   assignment to a list element does float arithmetic in place as well.
7. **`x op= y` on a local** is one instruction (`OpAssign`, `OpAssignK`), and
   `to_float`, `sqrt` and `abs` on an `int` or `float` run in place (`NumFast`).
8. **Calls** keep the callee's code in the call instruction, and the register
   vectors come back all `Unit` instead of being cleared and resized.
9. **Cold arms leave `exec`**: the user-method call, traits, `?`, string
   interpolation, struct and variant construction, `match` arms, collection
   literals and hand-offs are separate non-inlined methods, so the hot loop
   has fewer values to keep live (4% fewer instructions on a counting loop,
   2 to 4% on the benchmarks).
10. **Register writes** use `assign`, which does not call the drop function
   for values that own nothing; `MethodGuard` skips the lookup for built-in
   receivers held in locals.

No `unsafe`; no language behaviour changed; the tier check keeps every
example, case and perfbench program byte-identical on both tiers.

## Measurements

Linux x86-64, min of 3 wall-clock runs, `ail run` against CPython 3, before =
the batch head, after = this change on top of ADR 0618 (ratio below 1 beats
CPython):

| Program | CPython s | Before s (ratio) | After s (ratio) |
|---|---:|---:|---:|
| collatz | 0.869 | 0.376 (0.43x) | 0.328 (0.38x) |
| nbody | 0.295 | 0.864 (2.93x) | 0.287 (0.97x) |
| spectralnorm | 0.444 | 0.694 (1.56x) | 0.331 (0.74x) |
| mandelbrot | 0.227 | 0.643 (2.83x) | 0.209 (0.92x) |
| fannkuch | 0.723 | 1.044 (1.44x) | 0.680 (0.94x) |
| binarytrees | 0.474 | 1.851 (3.90x) | 0.909 (1.92x) |
| matmul | 0.413 | 0.813 (1.97x) | 0.541 (1.31x) |
| sieve | 0.502 | 1.192 (2.37x) | 0.568 (1.13x) |
| geometric mean | | 1.87x | 0.95x |

Callgrind instruction counts (millions; stable where wall time is not),
batch head to this change: sieve 300,000: 615 to 322; binarytrees 10: 725 to
439; nbody 20,000: 3,614 to 1,489; spectralnorm 100: 1,240 to 772; matmul
60: 202 to 144; a counting loop of 1,000,000 turns: 247 to 157.

## Rejected alternatives

- **`unsafe` unchecked indexing in `get`/`put`**: measured 10 to 12% fewer
  instructions on sieve, binarytrees, nbody and matmul; not adopted because a
  compiler bug would become silent memory corruption instead of a panic. The
  per-instruction cost is mostly operand decoding; the lasting fix is a
  frame layout with locals in the register file (see the performance notes).
- **`panic = "abort"` with one codegen unit**: 5 to 8% more instructions.
- **Fat LTO** (ADR 0618): no gain.
