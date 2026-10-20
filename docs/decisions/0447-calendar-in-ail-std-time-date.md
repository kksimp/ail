# 0447. The calendar is a standard module written in AIL: `std.time.date`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.date.1–15](../spec/17-std.md#calendar-stdtimedate)

## Context

`std.time` had only `now_ms()` and `sleep_ms(ms)` (ADR 0157). Programs that
log, schedule or parse timestamps need dates, times with a zone, arithmetic and
ISO 8601. The brief (lane 129) asked for `Date`, `DateTime` with a zone and
`Duration` arithmetic.

## Options considered

### Option A — a module written in AIL, `std.time.date` (chosen)
- **Pros:** the type methods, `Ord` and `ToStr` come from the language, so no
  new checker or interpreter code and nothing to port to `compiler/check/`
  beyond two names; it runs on both tiers and can be read by an agent with
  `ail api`; it is also a test of the language (pure integer math, `else`,
  result types).
- **Cons:** slower than native Rust (calendar math is a few dozen integer
  operations per call, far below the cost of anything the program then does
  with a date); `time.parse_iso` is spelled `date.parse_iso`.

### Option B — natives in Rust under `std.time`
- **Pros:** fastest; `time.parse_iso` as the brief spelled it.
- **Cons:** each struct type and each method needs a hard-coded entry in the
  checker, interpreter and effects table (as `ui.Window` has), and again in
  `compiler/check/`; a large edit in files that 27 lanes also touch.

### Option C — separate `Instant`, `Duration` and `Zone` types
- **Pros:** the type says what a value is.
- **Cons:** more names to learn and more tokens for every call; an `int` of
  milliseconds already is what `time.now_ms()` returns and `sleep_ms` takes.
  A zone as a `str` is checked at the one place a zone enters (`at`, `parse_in`).

## Decision

Option A. `use std.time.date` gives `date.Date`, `date.DateTime`, `date.at`,
`date.utc`, `date.now`, `date.now_local`, `date.parse_iso`, `date.parse_in`,
`date.parse_date`, `date.is_leap` and `date.days_in_month`. A duration is an
`int` of milliseconds with the constants `date.SEC`, `MIN`, `HOUR` and `DAY`.
A zone is a `str`. `time.now_ms()` stays as it is. Only the clock and named
zones are effects; everything else is pure.

## Consequences

- Rules `std.date.1–15`; tests `std_date`, `std_date_props`,
  `std_date_unhandled`, `std_date_deterministic`.
- `date.now()` reaches the `clock` capability, so `--deterministic` refuses it
  (`eff.cap.1` names the chain `main -> std.time.date.now -> time.now_ms`);
  programs that need a fixed "now" pass an instant in.
- Millisecond resolution; nanoseconds are not kept.
