# 0449. Calendar arithmetic: months clamp, days keep the wall clock, gaps and folds resolve forward

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.date.5](../spec/17-std.md#calendar-stdtimedate), std.date.7, std.date.12

## Context

Adding a month to January 31, a day across a daylight-saving change, and a
local time that does not exist or exists twice all need one documented answer.

## Options considered

### Option A — clamp the day; wall clock for days, months and years; elapsed time for `add_ms` (chosen)
- **Pros:** the same answer as Java `plusMonths`, Python `dateutil`, Rust
  `chrono` and Temporal (January 31 + 1 month = February 28 or 29); "tomorrow at
  noon" stays noon; a skipped time lands after the gap (the "compatible"
  disambiguation of Temporal) and a repeated time takes its first occurrence.
- **Cons:** `add_months(1).add_months(-1)` need not return the start date.

### Option B — overflow into the next month (JavaScript `Date`, `mktime`)
- **Pros:** reversible in some cases.
- **Cons:** January 31 + 1 month = March 2 or 3: a surprise that causes billing
  and scheduling bugs.

### Option C — fail on a missing day or a skipped time
- **Cons:** every call needs `else`, for rare cases; the clamp is what almost
  every caller wants.

## Decision

Option A: `std.date.5`, `std.date.7` and `std.date.12`.

## Consequences

- Property tests pin the invariants that do hold: `add_days` undoes itself,
  `add_years(k)` is `add_months(12 * k)`, the clamped day never grows.
- Vectors cover New York (2026-03-08 and 2026-11-01), Sydney, Berlin and
  Kolkata.
