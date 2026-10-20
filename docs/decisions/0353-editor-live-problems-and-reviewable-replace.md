# 0353. Editor live problems list and reviewable replace

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour)
- **Extends:** [ADR 0339](0339-editor-problems-and-sessions.md)

## Context

ADR 0339's problems list (Ctrl-Shift-M) was a snapshot, and its replace
in files applied every previewed line or none. Both rejected the live and
per-row forms for time, not on merit.

## Decision

### Live problems
While the problems list is shown in the active tab's pane, it is rebuilt
after each save and once 500 ms pass with no edit. The window loop polls
instead of blocking while a refresh is pending. A refresh keeps the focus
where it is (typing goes on in the buffer) and keeps the selection on the
same diagnostic, matched by file and message rather than position, since
positions move as lines are added above. Closing the pane or showing
another list stops it. `--print --idle` acts as if the window had waited,
so tests can drive it.

### Reviewable replace
Each preview row starts with `[x]` (accepted) or `[ ]` (skipped). Space
toggles the selected row, A accepts every row, and the title counts the
accepted lines and files. Ctrl-Enter replaces only on the accepted rows'
lines, one undo step per file.

## Rejected alternatives

- **Refresh on every keystroke:** a whole-program check per key is too
  slow in the interpreter, and the list would jump while typing.
- **Keep the selection by row index:** a new diagnostic above shifts it
  onto a different problem.
- **Keep the selection by position:** edits above move every position.
- **Skip by occurrence instead of by line:** a line with two matches is
  rare, and rows are lines everywhere else in the editor.
