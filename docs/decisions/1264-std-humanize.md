# 1264. `std.humanize`: locale-aware numbers, sizes and relative times

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.humanize.1 to std.humanize.12](../spec/std-humanize.md)

## Context

Programs print counts, sizes and ages for people: `1,234,567`, `1.5 MB`, `3 minutes ago`, `22nd`. Agents write these by hand
and get the thousands loop, the rounding up to the next unit (`1000K`) and the plural wrong. The format spec `{n:,}` only
groups with commas. `std.time.date` has `duration_text` and `Date.format` but nothing for sizes, compact numbers or
relative times, and `std.textwrap` is about columns, so no overlap.

## Decision

- A pure AIL module, `std/humanize/humanize.ail`, in scope as `humanize` (std.use.4). Registration lines only.
- A `Locale` struct of group, decimal, percent separator and minimum grouping digits (the CLDR data for `en`, `de`, `fr`,
  `es`, `ja`) and `custom(group, decimal)`. The functions are methods on `Locale`; the free functions are the English
  style, the common case.
- `int_text`, `float_text`, `percent`, `compact` (`K M B T`), `size_si` (powers of 1000) and `size_iec` (powers of 1024),
  `ordinal` (English) and `relative(then_ms, now_ms)`.
- Rounding is half away from zero on the float scaled by `10^decimals`; a value that rounds up to the next unit moves to it.
- No function traps on its input; `nan` and `inf` print as words.
- Tests: `tests/cases/std_humanize`, with expected output from a Python mirror of the rules.

## Rejected alternatives

- **A full CLDR or ICU port** (currency, Indian grouping, plural rules per language, localized unit names): large tables
  for rare use; `Locale` is a public struct, so a program can build any separator style.
- **Locale lookup from the environment** (`LANG`): hidden global state, and a program's output would change with the
  machine. The locale is always an argument.
- **Localized suffixes and relative times** (`vor 3 Minuten`): needs per-language plural data; English only, and said so.
- **One `size(n, base)` function:** the two bases have different unit names, so two names are clearer than a flag.
- **`parse_size` in this lane:** the reverse direction needs its own error type and rules; a follow-up.
- **Extending the `{n:,}` format spec:** a language change.
- **Putting sizes and relative times in `std.time.date`:** that module is about calendars; sizes are not dates.
