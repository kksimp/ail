# 0490. Editor find bar: regex search on Alt-R

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Tooling: the editor (ADR 0180)
- **Spec rules:** none (editor behaviour; the matching is `std.regex`, ADR 0438)
- **Confirms:** [ADR 0489](0489-editor-find-references.md)

## Context

The find bar (Ctrl-F) searches plain text. `std.regex` (ADR 0438) is now on
the batch branch, so the find bar can take patterns too.

## Decision

Alt-R toggles regex search in the find bar. The label shows `find (regex):`
while on. Each line is searched from the cursor with `regex.find` on the
rest of the line, so the match is leftmost and the cursor lands on its
start. Plain search stays the default. Replace (Ctrl-R, Ctrl-Shift-H) stays
plain text in this change.

## Options considered

### Option A — Alt-R toggles regex in the find bar (chosen)
- **Pros:** one key, the same bar, and the query is re-searched as the
  toggle flips.
- **Cons:** another key in the table.

### Option B — a separate regex find command
- **Pros:** no mode.
- **Cons:** a second bar to learn and a second prompt state.

### Option C — always regex
- **Pros:** one behaviour.
- **Cons:** plain text with `.` or `(` would need escaping every time.

## Criteria

Token use (ADR 0076 ranking): the editor's text is the same API output as
before; regex only adds one toggle and one function. Understanding: one
toggle, one label. Readability: the label says which mode is on. Speed: the
pattern compiles once per line search, and the interpreter caches compiled
programs by pattern text (std.regex.2), so the cost is small.

## Consequences

- The key table (`--keys`, README, palette) gains Alt-R.
- Headless tests: `regex_find_matches_patterns` and
  `regex_find_reports_no_match` in `tools/edit/edit.ail`.
- Replace with regex is a follow-up, not in this change.

## Rejected alternatives

- A separate regex find command (option B): rejected because it means a second bar to learn and a second prompt state.
- Regex as the only mode (option C): rejected because plain text with `.` or `(` would need escaping every time.
