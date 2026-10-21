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
6. **Register writes** use `assign`, which does not call the drop function
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
| collatz | 0.811 | 0.398 (0.49x) | 0.382 (0.47x) |
| nbody | 0.294 | 0.869 (2.96x) | 0.376 (1.28x) |
| spectralnorm | 0.385 | 0.717 (1.86x) | 0.513 (1.33x) |
| mandelbrot | 0.228 | 0.661 (2.90x) | 0.228 (1.00x) |
| fannkuch | 0.752 | 1.026 (1.36x) | 0.682 (0.91x) |
| binarytrees | 0.438 | 1.655 (3.78x) | 1.008 (2.30x) |
| matmul | 0.387 | 0.815 (2.10x) | 0.443 (1.14x) |
| sieve | 0.437 | 1.168 (2.67x) | 0.550 (1.26x) |
| geometric mean | | 1.97x | 1.12x |

Callgrind instruction counts (millions; stable where wall time is not):
sieve 300,000: 615 to 351; binarytrees 10: 725 to 452; nbody 20,000: 3,614
to 1,830 (ADR 0618 and this ADR); matmul 60: 202 to 145.

## Rejected alternatives

- **`unsafe` unchecked indexing in `get`/`put`**: measured 10 to 12% fewer
  instructions on sieve, binarytrees, nbody and matmul; not adopted because a
  compiler bug would become silent memory corruption instead of a panic. The
  per-instruction cost is mostly operand decoding; the lasting fix is a
  frame layout with locals in the register file (see the performance notes).
- **`panic = "abort"` with one codegen unit**: 5 to 8% more instructions.
- **Fat LTO** (ADR 0618): no gain.
