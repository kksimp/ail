# 0673. Language-only spec rules carry a `[definitional]` marker and need no citation

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 207, night run)
- **Topic:** Spec and tooling (lane 207)
- **Spec rules:** none new; adds a marker to [docs/spec/README.md](../spec/README.md) rule 2
- **Confirms:** [ADR 0483](0483-rule-id-audit.md) (the audit counts spec rules no code cites)

## Context

`tools/rules-audit.sh` check 2 lists every spec rule that no comment or string in `bootstrap/src` or `compiler/` cites. Some rules describe the language rather than a behaviour the code has to implement: there is no memory model to code (`conc.mm.1`), no run-time reflection (`refl.run.1`), no null value (`type.opt.1`) and no lifetime parameters (`own.borrow.2`). The audit had no way to say so, so those rules stayed on the list forever. Lane 207 asked for "the audit's convention for rules that need no code"; there was none.

## Decision

A spec rule that only describes the language carries the marker `[definitional]` in its heading line, after the rule ID, as `[removed in ...]` does. The audit skips such rules in check 2. The marker is for rules whose truth is a property of the language or of every target, not for a behaviour that is merely unbuilt. Unbuilt behaviour stays uncited, so it still shows on the audit.

## Rejected alternatives

- **A list of exceptions in the audit script.** Rejected: the list drifts from the spec, and the reader of the spec cannot see why a rule needs no code.
- **A test per definitional rule.** Rejected: a test that says "there is no reflection" checks nothing, and it costs tokens in every corpus that loads the test suite.
- **Marking unbuilt rules the same way.** Rejected: that would hide the gap the audit exists to show. Those rules are recorded in [docs/design/rule-coverage.md](../design/rule-coverage.md) for their owning lanes.

## Consequences

- Ten rules are marked: `conc.mm.1`, `conc.task.9`, `plat.ptr.1`, `plat.ptr.2`, `plat.ptr.3`, `plat.endian.1`, `plat.target.1`, `refl.run.1`, `type.opt.1`, `own.borrow.2`.
- Check 2 of the audit drops by the same ten.
