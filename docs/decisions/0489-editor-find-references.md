# 0489. Editor find references on Shift-F12

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Tooling: the editor (ADR 0180)
- **Spec rules:** none (editor behaviour, not language behaviour)
- **Confirms:** [ADR 0244](0244-editor-rename-symbol.md), [ADR 0312](0312-editor-agent-panel.md)

## Context

The editor already has find as you type (Ctrl-F), replace in the buffer
(Ctrl-R), replace in files (Ctrl-Shift-H), go to definition (F12), rename
(F2) and a problems list (Ctrl-Shift-M) whose rows carry rule ids. The
missing piece was a list of every use of a name. `ail api --refs` already
answers that, so the editor only needs to show it.

## Decision

Shift-F12 runs `ail api --refs` for the name under the cursor and fills the
pane with one row per place: `FILE:LINE:COL: declaration` or `reference`.
Enter jumps, Escape closes. The rows go through the existing pane list, so
they reuse the jump and paths-shortening code (`shown_path`).

## Options considered

### Option A — a references pane on Shift-F12 (chosen)
- **Pros:** one key, one list, no new state; the same API answer as F2.
- **Cons:** one more key in the palette table.

### Option B — show the count in the F12 status line only
- **Pros:** no new key.
- **Cons:** the user can't reach the places; it adds nothing F12 doesn't do.

### Option C — a regex find bar now
- **Pros:** more search power.
- **Cons:** needs `std.regex`, which is not in the tree yet. Deferred.

## Criteria

Token use (ADR 0076 ranking): the list is the API's output, not new code
paths, so nothing extra for an agent to read. Understanding: one rule per
key. Readability: one row per place. Speed: one `ail api` call per press.

## Consequences

- The key table (`--keys`, README, palette) gains Shift-F12.
- Headless tests: `references_list_declaration_and_uses` and
  `reference_rows_stay_short_for_long_paths` in `tools/edit/edit.ail`.
- `docs/design/cards/edit.md` is regenerated.

## Rejected alternatives

- A count in the F12 status line only (option B): rejected because the user cannot reach the places from a count, and it adds nothing F12 does not already do.
- A regex find bar now (option C): deferred, since `std.regex` was not in the tree yet. Added later by ADR 0490.
