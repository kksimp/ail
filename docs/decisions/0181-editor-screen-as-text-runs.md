# 0181. The editor's screen is a list of colored text runs plus plain rows

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling
- **Spec rules:** none

## Context

The editor's view (ADR 0180) is a pure function to a `Screen` that a front
end draws. The brief suggested a grid of cells `{text, fg, bg}`.
`std.ui`'s `Window.put(col, row, text, fg, bg)` writes runs of text.

## Options considered

### Option A — runs plus plain rows (chosen)
`Screen` has `spans: List[Span]` (`row`, `col`, `text`, `fg`, `bg`), drawn in
order so later runs cover earlier ones, `text: List[str]` (each row as
plain text), and the caret position.
- **Pros:** about 60 runs a screen instead of 3,200 cells, so rendering is
  cheap in the interpreter; one `put` per run; tests and agents read rows
  as strings (`--print` prints them).
- **Cons:** two views of the same screen to keep in step (one `Screen.put`
  writes both).

### Option B — a cell grid
- **Pros:** simple to diff and to address one cell.
- **Cons:** 3,200 values per 100x32 frame and one `put` call each; tests
  assert on cells instead of text.

## Decision

`Screen` is a list of colored runs plus each row's plain text and the caret.

## Rationale

ADR 0076 criterion 2 (AI understanding: a test reads `s.text[7]` as the
status line) and criterion 4 (speed: 6 ms a render) agree.

## Consequences

- A front end draws a screen in one loop over `spans`.
- A diffing front end can compare runs between frames if needed later.

## Rejected alternatives

- **Cell grid:** 50 times more values per frame and harder to read in tests.
