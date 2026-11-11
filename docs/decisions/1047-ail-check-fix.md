# 1047. `ail check --fix`: repair, then check, in one command

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076); scope into 0.1.0 by the designer (2026-11-10)
- **Builds on:** [ADR 0203](0203-ail-fix-command.md) (`ail fix`), [ADR 0917](0917-ail-task-protocol.md) (`ail task`), [ADR 0918](0918-ail-mcp-compiler-tools.md) (MCP)
- **Topic:** 20 — AI semantic API

## Context

165 of the 251 programs in `tests/mistakes` have a machine fix (66%). Applying one costs an agent a second command (`ail fix`), a second read of its output, and usually a third command to check again. A slip the compiler already knows how to repair should cost no extra round.

## Decision

- `ail check --fix FILE` applies every safe fix to a fixed point exactly as `ail fix` does (`fix.apply.1-3`), writes the file, then checks it and prints only what remains plus one line: `fixed 3: expr.not.1, gram.fn.4 x2` (rules in first-seen order, `xN` when a rule fixed more than once). Nothing is printed when no fix applied. With `--json` that line is one object `{"api":1,"fixed":3,"line":"..."}` on standard output; the diagnostics stay on standard error as before.
- The exit status is that of the check after the fixes (`fix.check.1`).
- The same flag on `ail task verify --fix` (check stage) and as `fix: "true"` on the MCP tools `ail_check` and `task_verify` (`fix.check.2`).
- Plain `ail check` never edits files.
- The coverage figure, fixed programs of the corpus, is printed by the `mistakes` suite so it can't drift. The target of 85% is a series of small lanes, each adding `.fixed` files; it is not part of this change.

## Consequences

One call for a repairable slip; the agent reads one diagnostic's worth less per slip. `ail fix` stays the tool for a whole-file repair with a diff (`--dry-run`).

## Rejected alternatives

- **Making plain `ail check` rewrite files:** a check that edits surprises people and agents alike.
- **`--fix` on `ail run` and `ail build`:** the same surprise, and it hides the edit among program output.
- **Printing every applied fix:** the per-fix lines of `ail fix` cost tokens the agent does not need; the one summary line names the rules, and the diff is in version control.
