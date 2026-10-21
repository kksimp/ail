# 0425. A `Child` is a unique value, consumed by `wait` or `kill`, and ended with the program

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 7 — Ownership; 21 — Standard library architecture
- **Spec rules:** [std.proc.5, std.proc.7](../spec/17-std.md#processes-stdproc)
- **Builds on:** [ADRs 0095–0096](0096-scope-bound-tasks.md), [ADR 0191](0191-cleanup-with-drop.md)

## Context

A task handle must be waited (`conc.task.5`). A child process is the same
kind of resource: forgetting it leaves a process running and a zombie.

## Options considered

### Option A — `unique struct Child`, `wait` and `kill` take `move` (chosen)
A second `wait` or a use after `kill` is a compile error (`own.move.2`).
- **Pros:** reuses the ownership checker; nothing special in the checker but
  the `move` flags of two standard signatures.
- **Cons:** the checker doesn't yet insist that a child *is* consumed
  (there is no must-use rule for unique values in general, `own.uniq.3`
  only runs a cleanup). A leaked child is therefore ended at program exit.

### Option B — a runtime check that traps at scope end if not waited
- **Cons:** needs a per-scope hook for values in structs; `end_tasks` does it
  for task handles only. Costs on every block for one rare value.

### Option C — `wait` and `kill` as methods on `Child`
- **Cons:** a method on a std struct needs the plumbing `ui.Window` has
  (`check/ui.rs`); `move self` on a standard method adds more.

## Decision

Option A, plus: a child neither waited nor killed when the program ends is
killed and reaped (`std.proc.7`), including at `io.exit`. `kill` returns the
status (137 on Unix); killing then waiting is one call. The interpreter's
`has_unique` fast-path switch ignores standard unique types, so declaring
`proc.Child` in `std/types.ail` doesn't slow programs that never use it.

## Consequences

- Follow-up: a general "unique value must be consumed" rule would make the
  exit-time kill unnecessary; it would change `own.uniq.*`, so it is the
  designer's call (not made here).

## Rejected alternatives

- A runtime check that traps at scope end if a child is not waited (option B): rejected because it needs a per-scope hook for values in structs, and `end_tasks` does it only for task handles. It would cost every block for one rare value.
- `wait` and `kill` as methods on `Child` (option C): rejected because a method on a standard struct needs plumbing that `ui.Window` has, and `move self` on a standard method adds more.
- A general must-use rule for unique values now: left as the designer's call, since it would change `own.uniq.*`. Until then a leaked child is killed at program exit.
