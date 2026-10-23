# 0740. The self-hosted checker must match `ail check` with nothing skipped

- **Status:** Accepted
- **Amended by:** [ADR 0865](0865-no-double-work.md)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture (Stage 3, ADR 0022); 28 — Testing strategy
- **Spec rules:** every rule `ail check` reports, except eff.cap.4–5

## Context

Slice 4 of lane 114 reports how far the AIL checker (`compiler/check/`)
agrees with the Rust one. Until slice 3 the selftest `self_hosted_check`
compared only the rules and lines the AIL checker covered (ADRs 0377–0378).
With `std.tensor`, `std.ui` and named colors ported, the AIL checker types
every function body. With nothing skipped it reports every diagnostic
`ail check` does, at the same rule, file, line and column, in order, and
with the same message. The only exceptions are eff.cap.4–5, which
`bootstrap/src/packages.rs` reports from package manifests before the
checker runs. The AST dump doesn't carry them.

The corpus is every example, test case, package case, `ail fix` mistake,
`perfbench` program, agent-corpus program (`tools/tokbench/corpus/programs`),
`tools/edit`, and `compiler` itself: 522 programs and 2,615 function bodies.

## Options considered

### Option A — gate on the unfiltered comparison (chosen)
The selftest fails unless the two agree with nothing skipped, apart from
eff.cap.4–5. It still prints the covered-rule comparison and the bodies
count, and it counts matching messages without gating on them.
- **Pros:** the checkers can't drift apart. A change to the Rust checker
  shows up at once as a failure that names the program and both diagnostic
  lists.
- **Cons:** a change to `bootstrap/src/check*` must port the change to
  `compiler/check/` in the same branch. Since the AIL checker mirrors the
  Rust one function for function (ADR 0377), the port is usually a few
  lines.

### Option B — keep the covered-rule gate, and report the rest
- **Cons:** a new Rust rule falls outside `COVERED_RULES` and is skipped
  silently, so the checkers drift apart with the test still green. That is
  the stale list ADR 0378 rejected.

### Option C — gate messages too
- **Cons:** wording changes more often than placement. It is reported
  instead, and all 536 messages agree today.

## Decision

Option A. The checkers agree on 522 of 522 programs and 536 of 536
diagnostics, and all 536 messages match.

eff.cap.4–5 stay outside the checker. When the self-hosted pipeline reads
packages itself, its package loader will report them.

## Rationale

ADR 0076 criterion 2 (AI understanding): a single checker behaviour, with
one place that says where the two differ (nowhere, today).

## Consequences

- AGENTS.md: a change to the Rust checker ports to `compiler/check/` in the
  same branch.
- The gap mechanism (ADR 0378) stays, for porting larger features in
  slices. A gapped function with diagnostics now fails the gate, so a gap
  can last only as long as its branch.
- The AIL checker takes about 15 s for `compiler` and `tools/edit` under
  the tree-walking interpreter, against under 1 s for `ail check`. The
  bytecode tier (ADR 0359) and native code are expected to close that gap.

## Rejected alternatives

Options B and C, above.
