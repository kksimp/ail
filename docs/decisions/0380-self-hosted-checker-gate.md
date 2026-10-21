# 0380. Checker changes are ported to the AIL checker in the same branch

- **Status:** Accepted
- **Date:** 2026-10-20 (in effect from commit fed6b45, the first checker slice the gate tests)
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); recorded as a gate in the batch 2 rules of 2026-10-20
- **Topic:** 23 — Compiler (self-hosting, Stage 3)
- **Spec rules:** none yet (a process gate; it defines no language behaviour)
- **Builds on:** [ADR 0377](0377-checker-in-ail-ported-pass-for-pass.md) (the checker in AIL, ported pass for pass), [ADR 0085](0085-interim-local-testing-gate.md) (the local selftest gate)

## Context

The checker exists twice: the Rust checker in `bootstrap/src/check*` and the
AIL checker in `compiler/check/` (ADR 0377). Until the Rust one is retired,
the two must agree. A change that lands in only one of them makes the
disagreement invisible: the Rust checker's behaviour moves on, and the AIL
copy quietly falls behind.

ADR 0377 already makes the comparison a test. `bootstrap/tests/self_hosted_check.rs`
dumps every example, case, mistake, package and `tools/edit` the front end
accepts, runs the AIL checker on them, and requires the same diagnostics
(rule, file, line, column, in order) as `ail check --json`, restricted to the
rules and lines the AIL checker covers (`compiler/check/coverage.ail`). That
test runs in `./ail selftest`, so it runs before any merge (ADR 0085).

What the test cannot do is notice a change that was never ported. A checker
change that only touches Rust keeps the test green, because the AIL checker
still agrees with the old Rust behaviour, and nothing says the port is owed.

## Decision

1. A change to a checker source file (`bootstrap/src/check*`) must be ported
   to `compiler/check/` in the same branch, so the two agree again.
2. The port is complete when `./ail selftest` passes with `self_hosted_check`
   green on Linux x86-64 (the interim gate, ADR 0085).
3. Coverage grows by slices (ADR 0377) and is not narrowed to hide a difference.

## Consequences

- Checker changes are a little larger: every rule change carries its AIL
  twin, with tests on both sides.
- The gate is a review rule backed by the selftest. It is not a separate
  check: a Rust-only change can still pass `self_hosted_check` when it falls
  outside the covered rules, so the reviewer also checks `coverage.ail`.
- The batch rules apply the same porting rule to the lexer and parser, with
  `compiler/syntax/` as the target. Those are checked by comparing syntax
  trees with `ail api --ast` (ADR 0376), not by this gate.

## Rejected alternatives

- **Porting the checker only when Stage 3 completes.** The record gives no
  debate. Reconstructed 2026-10-21: the two copies would drift for the whole
  port, and the comparison in ADR 0377 is useful only while both exist.
- **Comparing on the examples alone.** Rejected because the test in
  `bootstrap/tests/self_hosted_check.rs` also covers the mistakes corpus,
  packages and `tools/edit`, which the examples do not exercise.
