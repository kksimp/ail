# 0226. Compile-time evaluation has step and memory limits, is deterministic, and is cached by what it ran

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 18 — Compile-time execution; 27 — Build system
- **Spec rules:** [ct.eval.1–4](../spec/15-compile-time.md)
- **Design note:** [compile-time-execution.md](../design/compile-time-execution.md) §4–5

## Context

Once constants and derive generators run code at compile time (ADR 0225,
ADR 0227), a runaway evaluation could hang `ail check`, the agent's main
loop, and a nondeterministic one would poison the shared cache (ADR 0015).
**AI understanding decides** (a stuck or flaky check is the worst feedback),
then speed.

## Options considered

### Option A — deterministic step and memory limits (chosen)
- **Pros:** the same program passes or fails on every machine; the error
  can name where evaluation stood.
- **Cons:** a step is an abstract unit; the limit needs a number.

### Option B — wall-clock timeout
- **Pros:** intuitive.
- **Cons:** a slow machine fails a build that a fast one passes.

### Option C — no limit
- **Pros:** nothing to tune.
- **Cons:** an infinite loop hangs the agent's check.

## Decision

1. Compile-time code has no capabilities (`ct.const.2`), so it sees no
   clock, environment, file or randomness; with insertion-ordered maps and
   sets, evaluation is a pure function of the program (`ct.eval.1`).
2. It uses the run-time semantics exactly (`ct.eval.2`).
3. Each constant and each derive may take at most **100 million evaluation
   steps** (one per expression evaluated) and **1 GiB** of live values, and
   the run-time call-depth limit. Exceeding one is a compile error naming
   the limit, the constant or `impl`, and the call stack (`ct.eval.3`). A
   package manifest (Topic 13) may raise the step limit for its package.
4. Results are cached by the `const_value(id)` and `derive(impl id)`
   queries, fingerprinted by the HIR of every function the evaluation
   entered plus the evaluator's version (`ct.eval.4`, ADR 0224).

## Consequences

- **Designer, look first at:** the two numbers in point 3 (OQ-095).
- The bootstrap doesn't count steps yet; it evaluates constants before
  `main` and a trap there is reported at the constant.

## Rejected alternatives

- **Timeouts (B):** nondeterministic.
- **No limit (C):** hangs the feedback loop.
