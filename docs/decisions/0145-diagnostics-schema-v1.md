# 0145. Version the JSON diagnostics as schema v1, with spans, notes and fixes

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API ([ADR 0024](0024-semantic-api-from-stage-2.md))
- **Spec rules:** none (tool output; documented in [semantic-api.md](../design/semantic-api.md))
- **Plan:** [ai-first-plan.md](../design/ai-first-plan.md) §2.1

## Context

`ail check --json` printed one object per diagnostic with a position and a
message. An agent fixing an error had to re-read the source to find what to
replace, and a "did you mean" help was prose to parse. Tools reading the
output had no version to check.

## Options considered

### Option A — schema v1 on every line, fixes as anchored edits (chosen)
Every object starts with `"api":1`; it gains `end_line`/`end_col`, and
optional `notes` and `fixes` (`{title, edits: [{line, col, end_line,
end_col, text}]}`). Existing keys keep their order; new keys follow them.
- **Pros:** each line stands alone (no header to track), old readers keep
  working, an edit applies without reading the file.
- **Cons:** `"api":1` repeats on every line (about 4 tokens each).

### Option B — a header line `{"ail_api":1}` then diagnostics
- **Pros:** the version once.
- **Cons:** a reader that filters lines (`grep`, the test harness) loses it;
  an empty report still needs the header.

### Option C — fixes as prose in `help` only (status quo)
- **Cons:** every agent must parse English to apply a fix.

## Decision

1. Each JSON line carries `"api":1` first, then the existing keys in their
   order (`severity rule file line col message help`), then `end_line`,
   `end_col` (exclusive; the end of the token at the position when the
   checker gives none), then `notes` and `fixes` when non-empty.
2. Positions are 1-based lines and columns counted in characters.
3. A fix is all-or-nothing. The checker may anchor an edit on the text it
   replaces; if that text isn't found at the reported place, the fix is
   dropped rather than reported wrong.
4. Diagnostics are sorted by position (stable).
5. Text output shows `note: FILE:LINE:COL: ...` and `fix: TITLE` lines.
6. First fixes: every "did you mean" suggestion (replace the name),
   `expr.bind.4` on a `mut` variable (`mut x := e` → `x = e`), and
   `expr.bind.3` on an undeclared name (`y = e` → `mut y := e`). First
   note: `mod.unit.1` on a type declared twice points at the first one.

## Rationale

**Tokens** are a tie for the program text (the schema doesn't change source)
and the per-line tag is small next to a message. **AI understanding** decides:
an agent applies a fix by position without re-reading or parsing prose, and
a self-describing line is safe to filter. Byte-stable output (sorted, fixed
key order) keeps diffs of tool output meaningful.

## Consequences

- `ail api` (lane 2) and later commands use the same `"api"` version.
- A breaking change to a key means `"api":2`; adding optional keys doesn't.
- More checker diagnostics should gain fixes over time.

## Rejected alternatives

- Header line (B): lost by line filters; per-line tags are cheap.
- Prose-only fixes (C): the cost this ADR exists to remove.
- Byte offsets instead of columns: agents and editors work in lines and
  columns; byte offsets need the file's encoding.
