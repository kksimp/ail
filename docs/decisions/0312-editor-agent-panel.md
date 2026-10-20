# 0312. The editor's agent panel: info, trace and failure paths in one pane

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour); reads `api.out.*`, `trace.run.1`, `trace.snap.1–4` and `err.api.1` output
- **Extends:** [ADR 0292](0292-editor-project-search.md), [ADR 0244](0244-editor-rename-symbol.md)

## Context

The semantic API already answers what an agent or a person wants to know
about code: types, contracts, effects, error sets, references, a traced run
and a trap's frames. The editor showed only a name's type and signature.

## Options

### Option A — one key per question, answers as rows in the existing pane (chosen)
- **Ctrl-I (info):** `ail api --type` and `--refs` at the cursor. The pane
  shows the expression's type, the declaration's signature, its `requires`
  and `ensures`, effects, `fails with` error set, doc comment, and how
  many uses this file has. The declaration's facts come from the
  `--symbols` index (or the call's `callee`).
- **Shift-F5 (trace):** `ail run --trace=all --json`; each event is a row
  (calls indented by depth with their arguments, returns, contracts held or
  failed, the summary), a trap is a row followed by one row per frame with
  its arguments and locals, and the program's output follows as `> ` rows.
- **Ctrl-Shift-E (failure paths):** `ail check --errors`; each function's
  error set, then every `return`, `?` and handling `else` with its position.

Trace and failure rows use the project-search list (Up/Down, Enter jumps,
F4 next, Esc closes), so there is one navigation model for every list.
- **Pros:** no new UI; every answer is the CLI's own JSON, so tests can
  drive it headless (`--print --key=shift+f5`) against the real binary.
- **Cons:** the pane is a third of the screen; long traces scroll.

### Option B — a separate side panel
- **Cons:** another layout to maintain, and the tree already uses the left.

### Option C — show trace and errors as gutter decorations only
- **Cons:** loses the order of events and the frames' values.

## Decision

Option A. `--print` now drives the workspace, so every key is testable.

## Consequences

- A new semantic output gets a key in `key_rows()` and a row builder.

## Rejected alternatives

- **B:** layout cost. **C:** loses information.
