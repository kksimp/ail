# 0035. Integer overflow traps; wrapping, saturating and checked arithmetic are explicit

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 4 — Type system; 8 — Error handling; 25 — Optimization
- **Spec rules:** [expr.arith.1–4](../spec/05-expressions.md)
- **Resolves:** OQ-009

## Context

Overflow behaviour affects both safety and optimisation. With single-mode,
always-optimised builds (ADR 0015), "check in debug, wrap in release" is not
available.

## Options considered

### Option A — Trap always, with explicit wrapping/saturating/checked operations (chosen)
- **Pros:** Safe. Bugs surface immediately. Intent is visible in the code.
  The checks give the optimiser value-range facts.
- **Cons:** A small cost where a check can't be removed. Remaining checks may
  block vectorisation in some loops (visible in the performance report, and
  fixable with explicit wrapping operations).

### Option B — Always wrap
- **Pros:** Zero cost.
- **Cons:** Silent wrong results and a source of security bugs.

### Option C — Trap with a per-module opt-out
- **Pros:** An escape hatch.
- **Cons:** The same code means different things in different places.

### (Excluded) Undefined behaviour on overflow, as in C
Contradicts safety by default and ADR 0019's determinism.

## Decision

Arithmetic on fixed-width integers that overflows **traps**. Explicit
operations provide wrapping, saturating and checked (returning an optional or
error value) arithmetic. The compiler removes checks it can prove
unnecessary, and reports every check that remains.

## Consequences

- What a trap does (stop the task or the program, and whether it can be
  observed) is decided in the error-handling topic.
- The syntax for explicit wrapping, saturating and checked operations is to
  be designed (operators vs. methods), measured with the token benchmark.
- The performance report (brief §20) gains "overflow checks: N eliminated,
  M remaining", mirroring bounds checks.
- Division by zero and `MIN / -1` also trap (`expr.arith.3`).

## Rejected alternatives

- **Always wrap:** silent bugs.
- **Per-module opt-out:** non-local meaning. The explicit operations cover the need.
- **Undefined behaviour:** unsafe.
