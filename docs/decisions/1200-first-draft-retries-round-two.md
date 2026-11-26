# 1200. First-draft retries, round two: one note for `expr.match.3` on run, a worker-pool how-to, text-join help

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); output and docs only, no language change
- **Topic:** 22 — Tools
- **Spec rules:** [check.out.1](../spec/22-tools.md#check-output), `trait.op.1`, `conc.task.5`

## Context

The second pass of the Opus trial (main 78e27c83, report `opus-ail-trial-rerun.md`) replayed the same nine first drafts and
hit the same diagnostics, so cycles to working fell only from 17 to about 15. Rules already owned elsewhere
(`err.type.3` fix lines, float patterns, `--fix` behind syntax errors, standalone `check` and `test`) are not touched
here. What was left:

- `ail run` of a program with catch-all arms printed one `warning[expr.match.3]` block per arm (8 lines for one
  `calc` run), before the program's own output.
- `ail ask how "worker pool"` found nothing, and the `conc.task.5` help covered only one call per element.
- `" " + xs.map(|t| "#{t}").join(" ")` has no certain fix: a part that holds an interpolation cannot sit inside
  `{}`. The help said only "use interpolation".
- `if c then "" else " " + x` was never rewritten; [ADR 1199](1199-public-error-set-fix.md) fixed that in the same
  week, so this change only adds the case `tests/fix/plus_else`.

## Decision

- The checks `run`, `build` and `test` make first print the `expr.match.3` warnings as one line, `note: N style
  warning(s) [expr.match.3] not shown; ail check FILE lists them`. `ail check` prints them in full. Errors and every
  other warning are unchanged (check.out.1).
- `docs/how-to.md` gains "worker pool", adapted from the trial's passing program; the `conc.task.5` help points to it.
- The `trait.op.1` help for `+` on text says a part holding a quoted string or its own interpolation is bound to a
  name first.

## Options considered

- Hide `expr.match.3` on run entirely: rejected, a reader would not know a check was skipped.
- Make it an error or remove it: rejected, a language and ADR 0379 change.
- A certain fix that binds the right part to a new local: rejected for now, it adds a line and invents a name.

## Consequences

Untested at writing; the Mac job `first-draft-2/job.sh` is the first run.
