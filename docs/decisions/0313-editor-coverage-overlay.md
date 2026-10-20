# 0313. Editor coverage overlay (Ctrl-Shift-C) until the next edit

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour); reads `test.cover.1–3` output
- **Extends:** [ADR 0312](0312-editor-agent-panel.md)

## Context

`ail test --coverage=lines --json` reports the line ranges no test ran.
Seeing them beside the code says where a test is missing.

## Options

### Option A — mark unrun lines in the gutter, cleared by any edit (chosen)
Ctrl-Shift-C runs the buffer's tests with line coverage. Each unrun line's
number turns red with a `!` after it, and the status line gives
`lines hit/total, functions hit/total`. The first edit clears the marks,
since line numbers no longer match the run.
- **Pros:** one glance; never stale.
- **Cons:** an edit loses the marks; rerun to see them again.

### Option B — keep marks and shift them with edits
- **Cons:** needs edit-tracking of line numbers; a stale mark misleads.

### Option C — list unrun ranges in the pane
- **Cons:** the code itself doesn't show them.

## Decision

Option A.

## Rejected alternatives

- **B:** stale marks mislead. **C:** not beside the code.
