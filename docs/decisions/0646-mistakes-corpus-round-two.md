# 0646. Mistakes corpus, round two: programs on the batch's new features

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** gram.help.1, gram.help.2, gram.rec.1 (ADR 0262)
- **Builds on:** [ADR 0262](0262-report-foreign-syntax-once-with-help.md) (corpus format), [ADR 0202](0202-more-machine-applicable-fixes.md) and [ADR 0264](0264-anchored-call-rewrite-fixes.md) (fixes), [ADR 0647](0647-round-two-diagnostics-help-and-anchors.md) (diagnostics), [ADR 0648](0648-round-two-fix-rules-and-the-corpus-target.md) (fix rules)

## Context

The corpus held 154 programs, and `ail fix` resolved 96 of them (62%). Since
then the batch added bytes, named arguments, function values, match guards
and ranges, tasks and `par for`, regex, dates, hashing and compression, the
network, tensors, typed `none`, bit methods and float conversions. Agents
trained on Python and Rust make specific slips with each of them, and the
corpus did not cover them.

## Decision

1. **96 new programs** in `bootstrap/tests/mistakes/`, each with `.rules` (the
   rule IDs of its errors, in report order). The brief asked for 60; the
   extra programs cover the rename mistakes that agents make across the
   standard library, which the fix target needs (ADR 0648).
2. **Programs that compile are not mistakes.** Six candidates were dropped
   after `ail check` accepted them: `bytes.hex()`, raw strings `r"..."`,
   `tensor.from` with `t / 2` (an `int` literal beside a `Tensor[float]` is a
   `float`, type.num.10), `compress.gzip` without `use` (standard modules
   are in scope by name, std.use.4), and a `match` arm with `return`.
3. **`.fixed` for every program `ail fix` resolves.** Each `.fixed` checks
   clean and a second `ail fix` changes nothing (the existing harness).
4. **Measured:** the corpus is 250 programs; `ail fix` resolves 150 of them
   (60.0%, the ADR 0262 target). The 96 round-two programs alone: 54 with a
   fix (56%), after the fix rules of ADR 0648.

## Rejected alternatives

- **Exactly 60 programs.** Too few for the new features: several had no
  program at all. The count is a floor in the harness, not a size.
- **Keep programs whose diagnostics are silent.** A silent program records
  no rule, so it asserts nothing. Two found silent gaps (a call's silent
  acceptance of `t[0][1]`, see ADR 0647) are left as open questions rather
  than encoded as passing.
- **Rewrite old programs to make them fixable.** Changes existing expected
  output; new programs only.

## Consequences

The harness (`bootstrap/tests/mistakes.rs`) runs 250 programs. `ail fix`
coverage and the `fixable.txt` list (ADR 0648) grew with it.
