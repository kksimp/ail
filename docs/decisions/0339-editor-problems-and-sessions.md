# 0339. Editor problems pane, replace in files, and remembered tabs

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour); reads `ail check --json`
- **Extends:** [ADR 0292](0292-editor-project-search.md) (the shared list)

## Context

The status line shows the active file's diagnostics, and saving checks
the other open tabs for their counts (ADR 0293). Missing were one place listing every
diagnostic of the programs being edited, and a way to come back to the
same tabs and cursors after a restart. Ctrl-R replaces in one buffer;
renaming across files needs a project-wide replace with a preview.

## Decision

### Problems (Ctrl-Shift-M)
For each open tab, the editor runs `ail check --json` on its buffer as
edited, which checks the whole program (modules included). Rows are
`FILE:LINE:COL: severity[rule] message`, errors first, deduplicated
across tabs (two tabs of one program report its diagnostics once). The
rows use the shared list: Up/Down, Enter opens the file and jumps, F4
next, Esc closes. With nothing to report it says so in the message line.

### Replace in files (Ctrl-Shift-H)
The search prompt asks what to find (the word at the cursor to start),
Enter, then the replacement, Enter. The pane previews every changed line
of the directory's `.ail` files (open tabs as edited, as in project
search) as `PATH:L:C: old → new`, with the line and file counts in the
title; Enter jumps to a row. Ctrl-Enter applies it to the buffers: each
file is opened as a tab if it isn't one and replaced as one undo step.
Nothing is saved, so the person reviews the tabs and saves.

### Status line
When the message wouldn't fit after a long path (a temporary directory on
macOS), the status line shows the file's name instead of its path.

### Remembered tabs
When the window closes, the editor writes the open tabs and cursors to
`~/.ail-edit` under the key `open:DIR`, where DIR is the working
directory, as `LINE,COL,PATH` entries (1-based) joined by `|`, the
active tab first. On the next start in the same directory it reopens the
remembered files that still exist at their cursors, then shows the file
it was started with. Tests and `--print` never read or write it.

## Rejected alternatives

- **Only the per-tab counts of ADR 0293:** show how many, not which or
  where.
- **A live problems pane updated on every keystroke:** a full program
  check per key is too slow in the interpreter; the list is refreshed on
  demand instead.
- **Replace writing the files directly:** no undo and no chance to
  review; the buffers keep both.
- **A per-row accept or skip in the preview:** more keys to learn; a
  more specific query or Ctrl-Z on a tab covers it. Left for later.
- **A session file per project (`.ail-edit` in the project):** writes
  into the user's repository; the settings file already exists and is
  per user.
- **Remembering on every change:** a write per keystroke for no gain;
  the window closing is the one moment that matters.
