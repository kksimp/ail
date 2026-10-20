# 0377. The checker in AIL is ported pass for pass, and says what it covers

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture (Stage 3, ADR 0022); 28 — Testing strategy
- **Spec rules:** every rule the Rust checker reports (slice 1: the declaration rules listed in `compiler/check/coverage.ail`)

## Context

Lane 114 writes the type and effect checker in AIL (`compiler/check/`),
reading the program from `ail api --ast --program` (ADR 0376). It must
report the same diagnostics as `ail check`: rule, file, line and column, in
order (ADR 0032 keeps the Rust checker as the oracle). The Rust checker is
about 11,000 lines; the AIL one arrives in slices.

## Options considered

### Option A — a port that mirrors `check.rs` pass for pass (chosen)
The same passes in the same order (`collect_traits`, `collect_types`,
`collect_impls`, `resolve_fields`, ..., `check_impls`), the same type
representation (`Ty`), the same messages.
- **Pros:** agreement follows from structure, not luck; a difference points
  at one function on each side; the Rust code documents the AIL code.
- **Cons:** inherits the Rust checker's shape, which a fresh design might
  improve (for instance two passes over inferred error sets).

### Option B — a fresh design (constraint solving, a separate inference pass)
- **Cons:** months before diagnostics match; every difference is a design
  question. Better done once Stage 4 no longer needs the Rust oracle.

## Decision

Option A. Two consequences make the slices testable:

1. **Coverage is data.** `compiler/check/coverage.ail` lists the rules the
   AIL checker reports and computes the lines it doesn't check yet
   (function bodies until slice 2, contracts and invariants until slice 3).
   With several dumps, `ail run compiler check A.json B.json ...` prints
   each program's diagnostics after `== PATH`, then `rules ...` and
   `skip FILE FIRST LAST` lines.
2. **The selftest compares.** `bootstrap/tests/self_hosted_check.rs` dumps
   every example, case, mistake, package and `tools/edit` the front end
   accepts, runs the AIL checker on them in four batches, and requires its
   diagnostics to equal `ail check --json`'s restricted to the covered rules
   and lines. Each slice widens the coverage; when nothing is skipped the
   comparison is total.

Diagnostics are sorted by position, stably, as `diag::settle` sorts them.

## Rationale

Matching an oracle exactly is a structural property; the cheapest way to
get it is the same structure (ADR 0076 criterion 2, AI understanding: a
reader of either checker finds the other's counterpart by name).

## Consequences

- A rule change lands in `check.rs` and in `compiler/check/` together once
  the AIL checker covers it; the selftest fails otherwise.
- Writing slice 1 found language gaps, recorded in
  [self-hosting-status.md](../design/self-hosting-status.md), and one
  bootstrap fault fixed here: a module's locals clashed with the root
  module's items (`expr.bind.5`); they now meet only their own module's.

## Rejected alternatives

Option B, above. Comparing only rule IDs without positions: it would hide
diagnostics reported at the wrong place, which an agent applying fixes
depends on.
