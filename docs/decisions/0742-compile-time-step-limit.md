# 0742. A step limit on computing constants (ct.eval.3)

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 15 — Compile-time execution
- **Spec rules:** ct.eval.3
- **Builds on:** [ADR 0226](0226-compile-time-limits-determinism-and-caching.md), [ADR 0364](0364-bytecode-tier-limits-coverage-and-traces.md)

## Context

`ct.eval.3` limits each constant to 100,000,000 evaluation steps and 1 GiB of live
values. The bootstrap computes a constant on first use with no limit: a constant
that loops for ever hangs the program with no message.

## Decision

While the outermost constant is computed, the interpreter turns on its statement
hook (both tiers already test it once per statement) and counts statements.
Past 100,000,000 the run stops with `trap[ct.eval.3]`, naming the constant, with the
interpreter's stack at that point. A step is one statement run, not one expression:
the hook is per statement, and an expression-level count would add a counter to every
expression of every program. `AIL_CT_MAX_STEPS` lowers the limit, for the test that
exercises it. A constant computed inside another counts toward the outer one.

Not done: the 1 GiB memory limit (the interpreter has no live-value count; `--max-memory`
covers the process), and derive generators (they run in their own interpreter,
`interp/generate.rs`). Both stay listed in the chapter's status note.

## Rejected alternatives

- **Count expressions.** Closer to the rule's wording, but costs every expression of
  every run; statements bound loops and calls, which is what the limit is for.
- **Report at `ail check`.** The bootstrap computes constants only when the program runs.
- **A timer.** Not deterministic: the same constant would pass on a fast machine.
