# 1196. `ail mv`: move declarations into a module

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer chose to build it (2026-11-25); a new command, no language change
- **Topic:** 22 — Tools
- **Spec rules:** [mv.run.1-3, edit.op.10](../spec/22-tools.md)

## Context

In the final run's refactor task (split one file into modules) every trial did the split with a hand-written
script that sliced lines and added `pub` by pattern, and two of ten failed once on a missing `use` or a local that
hid a module. The compiler already has the move as a structured edit (`move`, edit.op.10, ADR 1054), which
requalifies every reference from the checked program. No agent called it, and it could not do this task:

- it takes JSON on standard input, a shape no agent typed; the agents typed `git mv`, `sed` and `python3`;
- it moves one declaration at a time, so a function and the type it names can never leave the root module
  (each move fails because the other is still there);
- it refuses a module that has no file yet, which is the usual case when splitting;
- a moved struct became `pub` with private fields, so the code left behind no longer compiled.

## Decision

- `ail mv [--dry-run] [--in DIR] NAME... MODULE`: the shape of the shell's `mv`, names first, destination last.
- The `move` edit takes `"ids"`: several declarations of one file move in one step and may name each other.
- `ail mv` makes `MODULE/NAME.ail` when the package has no such module.
- A struct made `pub` by a move has all its fields made `pub`: code of its old module could read and build it
  by field, and must still be able to.
- All or nothing: the edit is applied only when the result checks, and a file made for the move is removed
  when it is not.
- `ail mv` is `tools/ailmv/ailmv.ail`, a program the launcher runs; the analysis stays in `compiler/edit/`.

## Options considered

- **Teach the card to use `ail edit` with a `move` op.** Rejected: the transcripts show agents do not write
  JSON edits when a shell verb exists; and the three limits above would still stop the task.
- **`ail mv FILE.ail DIR/` (move files).** Rejected as the main form: the task is to move declarations out of a
  file, not to move a file; a file path as the last word gets an error that shows the module form.
- **Make only the fields used outside `pub`.** Rejected: it needs a second pass over the edited program, and a
  later use of another field would fail for no reason the author can see. All fields were reachable before.
- **Move the declarations a moved body needs automatically.** Rejected: it would move code the author did not
  name; the error names the declaration to add to the command.

## Consequences

- Declarations in one `ail mv` come from one file; a second file takes a second command.
- A type with an `impl` is still moved by hand (edit.op.10).
- The gate suite `agent_tools` splits a one-file program and runs the result.
