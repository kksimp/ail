# 0270. What the fuzz harness checks: crash, determinism, fmt, api, and a differential on contracts

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 28 — Testing strategy (tool)
- **Spec rules:** none (a tool); harness rule IDs `fuzz.prop.1–4`, `fuzz.mut.1`
- **Related:** [ADR 0269](0269-fuzz-generator-design.md), [ADR 0170](0170-ail-fmt-command.md), [ADR 0203](0203-ail-fix-command.md)

## Context

The reference interpreter has no second implementation to compare with, so
differential testing has to compare the tool against *itself* under
transformations that must not change the answer. Criterion **AI
understanding**: each property is one line of English that a failure message
names.

## Decision

A crash is a Rust panic, exit status 101, an `internal error` report, a
signal or a timeout (10 s; 3 s for mutants, where a changed loop bound may
legitimately hang and is ignored). A trap (exit 1) is never a crash.

| ID | Property |
|---|---|
| `fuzz.prop.1` | `ail check --json` never crashes and gives byte-identical output twice; every diagnostic line is valid JSON; no `bootstrap.*` rule on a generated program |
| `fuzz.prop.2` | `ail fmt` output re-formats to itself, keeps `ail api` identical apart from positions (the parse is preserved), and a run prints the same |
| `fuzz.prop.3` | `ail run --deterministic` never crashes and repeats exactly; `--no-contracts` prints the same for a program whose contracts hold |
| `fuzz.prop.4` | every `ail api` form (`--pretty`, `--symbols`) is valid JSON; a program that checks clean raises no `bootstrap.*` error while running |
| `fuzz.mut.1` | on a mutant, `check` diagnoses without crashing, repeats, and an exit-2 result carries an error diagnostic; a mutant that checks clean runs without a panic; `ail fix` doesn't crash, and if the mutant parsed (`fmt` accepts it) the fixed file still parses |

"Parses" means `ail fmt --stdout` exits 0, since `fmt` refuses unparsable
files (`fmt.safe`). Parse equality is checked on the `ail api` outline with
`line`, `col`, `end_line`, `end_col` and `file` removed, plus equal run
output; comparing full ASTs would need a library crate.

## Rejected alternatives

- Comparing full ASTs: the front end is a binary crate; the outline plus
  output equality catches the realistic regressions at no cost to its layout.
- Requiring that a mutant be rejected: many token edits are valid programs.
- Treating a trap as a failure: traps are the specified response (`err.trap.3`).
