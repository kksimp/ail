# 0180. Ship a code editor written in AIL, with a headless core

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling
- **Spec rules:** none (a tool, not language behaviour)

## Context

The designer asked for an AIL code editor with a windowed GUI, written in
AIL, as a stretch goal of the 2026-10-20 buildout. It is the largest
interactive AIL program so far and the first client of `ail check --json`,
`ail explain` and `ail api` written in AIL itself. A window needs `std.ui`
(lane 11), which was being written in parallel; user modules (lane 3) had
not landed.

## Options considered

### Option A — one file, headless core plus thin front ends (chosen)
`tools/edit/edit.ail` holds an `Editor` value, an `enum Command`,
`Editor.apply` (pure editing), `Editor.run` (the commands that save or run
`ail`), and a pure `Editor.render` to a `Screen` (ADR 0181). Front ends map
input to commands and draw screens: `@test`s, `--print`, and the window.
- **Pros:** the whole editor is testable with no display, in about 20 ms; an
  agent can drive it from tests or `--print`; the window code is about 40
  lines.
- **Cons:** one file of about 1,000 lines until user modules land.

### Option B — a terminal (TTY) editor first
- **Pros:** no window dependency.
- **Cons:** AIL has no raw-mode terminal input; the designer chose a window.

### Option C — wait for `std.ui`, write the editor around the event loop
- **Cons:** untestable headless, and blocked on another lane.

## Decision

The editor is `tools/edit/edit.ail`, run as `./ail run tools/edit/edit.ail
FILE`. Its core is headless and its `@test` suite runs in `./ail selftest`
(`bootstrap/tests/tools.rs`). It drives the semantic API: Ctrl-K runs `ail
check --json` and marks spans, Ctrl-. applies a fix's edits, F1 runs `ail
explain`, Ctrl-O shows `ail api --text`. Checking unsaved text writes it to
`FILE.edit-check.ail` beside the file and removes it after.

## Rationale

ADR 0076 criterion 2 (AI understanding) decides: a pure core with commands
in and screens out is local reasoning an agent can test without a display,
and it exercises the machine interfaces AIL offers agents. Criterion 4
(speed) is met: a 1,000-line file renders a 100x32 screen in about 6 ms in
the interpreter, under the 50 ms target.

## Consequences

- `tools/` now holds AIL programs, not only scripts; the selftest checks them.
- The window front end is about 40 lines on `std.ui` (ADR 0177); a test drives it with hand-built `ui.Event`s and a never-opened `ui.Window`.
- Split into modules (`buffer`, `view`, `keys`, `window`) once user modules
  land.
- Writing it surfaced language gaps, recorded as OQ-047 and OQ-048.

## Rejected alternatives

- **Terminal editor first:** no raw terminal input in AIL; the designer chose a window.
- **Event loop first:** untestable without a display and blocked on lane 11.
- **Checking via standard input:** `ail check` reads files only; a copy beside the file keeps relative imports working.
