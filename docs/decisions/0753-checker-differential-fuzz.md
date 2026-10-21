# 0753. Differential fuzzing of the self-hosted checker

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 28 — Testing strategy; 23 — Compiler architecture
- **Spec rules:** every rule `ail check` reports (ADR 0740)

## Context

ADR 0740 gates the AIL checker (`compiler/check/`) against `ail check` on a
fixed corpus. A corpus only covers what people wrote. Random programs with one
typed mistake reach combinations nobody wrote.

## Decision

- `tools/fuzz/gen/gen.ail` (AIL) generates programs from a grammar: a fixed
  prelude (struct, enum, error enum, fallible and optional helpers), 0-2 helper
  functions and a `main` of typed statements (bindings, loops, `if`, `match`,
  `?`, optionals, `mut` calls). A quarter of the programs get one mistake of 19
  kinds at a random statement (wrong arity, missing `mut`, moved value reused,
  unhandled failure, unknown method or variant, bad `else`, and so on).
  Program `k` of a run uses seed `SEED+k`, so a run reproduces.
- `tools/fuzz/check-diff.sh [N] [SEED] [DIR]` generates N programs, runs both
  checkers, and compares rule, line, column and message. As in
  `self_hosted_check`, only rules the AIL side reports and lines it checks
  count, and eff.cap.4-5 are ignored. Each disagreement is minimised by
  deleting runs of 8, 4, 2 and 1 lines while the same difference stays, then
  saved under `tools/fuzz/found/check/`.
- The bootstrap is the oracle (ADR 0032). A disagreement is fixed in
  `compiler/check/`, unless the bootstrap is wrong by the spec; then it is
  written up with the rule ID and both are left alone.

## Result

10,000 programs (seeds 10000-12499, 20000-22499, 30000-32499, 40000-42499):
9,902 passed the front end and were compared, and there were **0
disagreements** (9,902 of 9,902 agree on rule, line, column and message). The
other 98 are rejected by the parser before either checker runs. About a quarter
carry a mistake.

## Limits

The generator is small: no generics, traits, closures, modules or tasks. Zero
disagreements measures agreement on this grammar only. Widen the grammar before
reading more into it.

## Rejected alternatives

- **Reusing the Rust fuzz generator** (`bootstrap/tests/fuzz/progs.rs`): it
  cannot run in the AIL toolchain, and the point of the AIL generator is that it
  is also a program the AIL checker accepts.
- **Comparing only rule and line:** the message and column are cheap to compare
  and caught the one harness bug found during development.
