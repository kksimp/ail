# 0293. Editor file tree (Ctrl-B) and a check of every tab on save

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour)
- **Extends:** [ADR 0241](0241-editor-tabs-and-running.md), [ADR 0292](0292-editor-project-search.md)

## Context

With tabs, project search and a palette, the editor still had no view of
the directory, and a save could break another open file (a changed
signature) without any sign until that tab was checked.

## Options

### Option A — a left tree pane, and check every tab after a save (chosen)
Ctrl-B shows a tree of the working directory down the left side (28
columns; the tabs get the rest) and focuses it. In each directory,
`.ail` files come first, then directories, then other files, each by
name; hidden and `target` directories are skipped. Up/Down select,
Enter or Right opens a file in a tab (and gives the editor the keys) or
expands a directory, Left collapses or goes to the parent, Escape returns
to the editor; a click does the same. Ctrl-B on a focused tree hides it.
Directories are read when expanded, so a large tree costs nothing until
opened.

After a successful save, every other open tab runs `ail check` on its
buffer, and each tab label shows its counts: ` a.ail * E2 W1 `.
- **Pros:** cross-file breakage shows where the user is looking; no
  watcher or background process.
- **Cons:** a save with many tabs waits for each check (about 0.1 s each).

### Option B — the tree as an overlay like Ctrl-P
- **Cons:** the directory isn't visible while editing; Ctrl-P already
  covers "open by name".

### Option C — check other tabs in the background
- **Cons:** needs a task and result merging in the event loop; the cost
  is small enough to do inline today.

## Decision

Option A.

## Consequences

- When `std.fs` gains `walk` (lane 54), the tree and Ctrl-P can share it.
- A later `ail check` of a whole directory could replace the per-tab runs.

## Rejected alternatives

- **B:** hides the directory. **C:** complexity without a measured need.
