# 0292. Editor project search: Ctrl-Shift-F, matches in the pane, F4 for the next

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour)
- **Extends:** [ADR 0241](0241-editor-tabs-and-running.md), [ADR 0291](0291-editor-command-palette-one-key-list.md)

## Context

People and agents work on directories, not one file. The editor needs to
find a name or text across the project and go to each place.

## Options

### Option A — plain substring search in the editor, matches in the output pane (chosen)
Ctrl-Shift-F opens a prompt on the tab bar, prefilled with the word under
the cursor. Enter searches every `.ail` file under the directory (the
Ctrl-P walk: hidden and `target` directories skipped), reading open tabs
from their buffers so unsaved edits count. The matches, as
`FILE:LINE:COL: text` (at most 1,000), fill the pane and take the keys:
Up/Down select, Enter opens the file in a tab and jumps, Escape closes.
F4 jumps to the next match from anywhere. The rows are run-output
locations, so a click jumps too.
- **Pros:** no new tool; unsaved buffers are searched; one jump path
  (`Loc`) shared with run output.
- **Cons:** substring only; a large tree is read on each search.

### Option B — run `grep` through `std.proc`
- **Cons:** an external tool (ADR 0009), not on Windows by default, and it
  misses unsaved buffers.

### Option C — semantic search through `ail api --refs` across files
- **Cons:** only names, only when everything parses; it complements text
  search (F2/F12 already use it) rather than replacing it.

## Decision

Option A. Case-sensitive, first match per line.

## Consequences

- A regex or case-insensitive mode can be added behind the same prompt.
- When `std.fs` gains `walk` (lane 54), `list_files` can use it.

## Rejected alternatives

- **B:** external tool and misses unsaved edits. **C:** names only.
