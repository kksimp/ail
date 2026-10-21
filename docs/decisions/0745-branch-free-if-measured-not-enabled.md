# 0745. Branch-free `if` with deferred overflow checks: built, measured, not enabled

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** expr.arith.1 (behaviour unchanged by construction)
- **Builds on:** [ADR 0623](0623-branch-and-codegen-peepholes.md)

## Context

collatz is 1.3-1.65x behind `gcc -O2`; its step is `if n % 2 == 0 then n / 2
else 3 * n + 1`, which gcc turns into `cmov`. Evaluating both arms is only
allowed if the program can't tell, and a checked `3 * n + 1` must trap only
when that arm is taken (expr.arith.1).

## What was built

Experiment branch `lane/241-select-experiment` (commit 8b67aa59):

- `compiler/mir/select.ail`: a diamond `branch c -> A, B -> J` whose arms hold
  up to 6 computing instructions becomes both arms in the branching block and
  a `select` per join parameter. A checked `+ - *` in an arm is cloned as
  `addo`/`subo`/`mulo` plus `ovf` (the overflow flag), and the block ends in a
  branch on it to the original diamond, so a taken arm that overflows
  re-executes and traps exactly as before. An untaken arm that overflows only
  sends the program through the original code.
- `select`, `addo`/`subo`/`mulo`, `ovf` in MIR; `cmov`/`csel` and the fused
  `jo`/`b.vs`/`b.ne` on the flag in `x64/` and `a64/`, with encoder tests;
  comparisons fused into the `cmov`/`csel` flags.
- Conformance cases: both arms taken, the untaken arm overflowing (no trap), and
  a trap in each arm, equal to `ail run` on x86-64 and aarch64 (qemu).

## Measurements (Linux x86-64, 21 runs, best / median seconds)

| | base | select | gcc -O2 |
|---|---:|---:|---:|
| collatz 1M | 0.333 / 0.373 | 0.335 / 0.389 | 0.258 |
| nbody | 0.109 / 0.117 | 0.110 / 0.131 | 0.085 |
| spectralnorm | 0.088 / 0.115 | 0.089 / 0.099 | 0.058 |
| fannkuch | 0.619 / 0.677 | 0.621 / 0.672 | 0.236 |
| sieve | 0.582 / 0.620 | 0.565 / 0.627 | 0.482 |
| matmul | 0.141 / 0.167 | 0.151 / 0.200 | 0.081 |
| binarytrees | 0.922 / 0.996 | 0.902 / 0.994 | 0.127 |
| 40M random-data `if`s (microbenchmark) | 0.349 / 0.388 | 0.366 / 0.419 | |

No program gains, and a loop built to mispredict gets 5% slower. The
speculated arm adds an `imul`, `add` and two flag branches to the loop's
dependency chain (`n` feeds the next step), and in a latency-bound loop the
misprediction was already overlapped with that chain. gcc wins on collatz by
using `lea` for `3 * n + 1`, which has no overflow check to carry.

## Decision

Do not enable the pass: it adds a MIR pass, four ops and two code generators for
no measured gain. The experiment branch keeps the work for a time the cost model
changes (for example a `lea`-class unchecked fast path proven by ranges, or a
throughput-bound loop with random data).

## Rejected alternatives

- **Materialised overflow conditions** (range compares with constants, `bxor`
  tricks for variable operands): 25 instructions on the collatz hot path; 0.46 s
  against 0.39 s for the branchy loop.
- **A hidden `jo` edge to the original code from the middle of a block:** the
  register allocator's liveness doesn't see an edge that isn't in the CFG.
- **Dropping the check on the speculated arm:** loosens expr.arith.1, which is
  the designer's to change.
