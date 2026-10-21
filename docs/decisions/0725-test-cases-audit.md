# 0725. Test-case audit: one audit script, duplicates removed, one-theme cases

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 231)
- **Topic:** Tests (bootstrap/tests/cases)
- **Spec rules:** none; this is test hygiene, not language behaviour.

## Context

`bootstrap/tests/cases/` has over 500 cases, written by about 60 lanes. Three
problems came up in the audit: some programs are the same program under another
name, some diagnostic cases have no `.rules` file so their rule IDs are not
checked, and some vague names or multi-theme files hide what a failure means.

## Decision

1. **`tools/cases-audit.py`** (about 60 lines, run with `python3 -I`) reports two
   things: duplicate programs (the same program modulo identifier names, with the
   same companion files) and diagnostic cases with no `.rules`, `.test` or `.trap`
   file whose `ail check --json` reports a diagnostic. `tools/rules-audit.sh` runs
   it as check 6.
2. A duplicate is removed and the better name kept. `shadowing` was the same program
   as `fix_notes`; `fix_notes` stays because its notes field is asserted in
   `cases.rs`.
3. Single-word names that say nothing about the case are renamed to
   `area_feature_aspect`: `arithmetic`, `errors`, `generics`, `interpolation`,
   `optionals`, `supertraits`, `traits`, `variants`.
4. A case whose header lists separate features is split, and each part's output is
   checked to be a partition of the original output. `std_traits` is the first split.
5. A `.out` is never regenerated to make a case pass. The ten largest `.out` changes
   in history were read; each was an intended change or a value now checked by hand.

## Measured

Before: 541 cases. Duplicates 1, missing `.rules` 0 after excluding `.test` and
`.trap` cases and the `mod.main.1` "no main" report for test modules.

## Rejected alternatives

- **Split all 42 cases over 60 lines in one change.** Too much churn to review.
  Generated data tests (float and int conversions, with Python-derived values) are
  left whole, since their length is data, not several tests.
- **Merge the `--json` and `--deterministic` variants** of a program (`trace_calls`
  and `trace_json`, `par_comp` and `par_comp_deterministic`). They run different
  options and check different output, so they are not duplicates.
- **Keep `shadowing` and rename `fix_notes`.** `fix_notes` already names the notes
  feature under test (fix.note.1); `shadowing` is misleading, because the rule is a
  redeclaration in one scope (expr.bind.4).
