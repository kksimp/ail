# 0450. ISO 8601: an offset is required, `parse_in` supplies a zone, `iso()` prints round-trippable text

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.date.9, std.date.13–15](../spec/17-std.md#calendar-stdtimedate)

## Context

Agents and tools exchange timestamps as ISO 8601 text. Text without an
offset (`2026-10-20T13:00`) is ambiguous, and a silent guess of UTC or the
local zone is a common source of off-by-hours bugs.

## Options considered

### Option A — `parse_iso` demands an offset; `parse_in(text, zone)` reads offset-free text (chosen)
- **Pros:** the ambiguity is a visible, explained error (`no offset (write Z or
  +HH:MM, or use parse_in)`); the caller names the zone.
- **Cons:** one more function.

### Option B — treat no offset as UTC
- **Cons:** wrong for local timestamps, silently.

### Option C — treat no offset as the system zone
- **Cons:** the same program gives different answers on different machines, which
  breaks `--deterministic` runs and tests.

## Decision

Option A. `iso()` prints `YYYY-MM-DDTHH:MM:SS[.mmm]` with `Z` or `±HH:MM`, and
`parse_iso(dt.iso())` returns the same instant. Fractions longer than
milliseconds are cut, seconds go to 59, and `T` may be a space. The result is
in the text's own offset. Week dates (`2026-W43-2`), ordinal dates and basic
format without dashes are not accepted; the error names what was expected.

## Consequences

- `DateError` carries the text and a reason an agent can act on.
- Leap seconds (`:60`) are refused with a message saying so.
