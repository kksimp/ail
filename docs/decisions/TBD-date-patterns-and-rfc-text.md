# TBD. Date patterns, RFC 3339 and RFC 2822 text in `std.time.date`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** std.date.23, std.date.24, std.date.25, std.date.26, std.date.27
- **Amends:** [ADR 0451](0451-format-patterns-use-strftime-tokens.md) (more `format` tokens)

## Context

`std.time.date` could write a date by a `strftime` pattern and read only ISO 8601 and durations. Agents write
`strptime` code, RFC 2822 headers (mail, HTTP `Date`) and RFC 3339 API timestamps from habit, and found nothing to
read a `%d/%m/%Y`, `Tue, 20 Oct 2026 17:01:52 GMT` or `...t...z` input. ISO week numbers and `%-d` were also missing.

## Decision

- `format` gains `%C %k %l %P %h %u %w %U %W %G %g %V`, the shorthands `%F %T %R %D %r %c %x %X %n %t` (C locale), and
  the `-` and `_` flags for number tokens. Unknown tokens stay as written, as before.
- `dt.rfc2822()` and `dt.http_date()` write the two header forms. RFC 3339 text is the existing `dt.iso()`; no alias.
- `parse_rfc3339` (strict: seconds and an offset required, `T`/`t`, `Z`/`z`) and `parse_rfc2822` (also reads the HTTP
  `Date` form; optional weekday and seconds, `Z`/`UT`/`GMT`/`UTC`/`±HHMM`, North American names, trailing comment).
- `parse_format(text, pattern)` and `parse_format_in(text, pattern, zone)` read by the same tokens (those that can be
  read are listed in std.date.26). Failures are `DateError` with a `why` per cause (std.date.27). An instant needs an
  offset: from `%z`/`%s`, or `parse_format_in`; this matches `parse_iso` and `parse_in` (std.date.14) rather than
  guessing UTC.
- The weekday read by `%a`/`%A` is not checked against the date (as `strptime`), and the locale is always C.

## Rejected alternatives

- **Read with no offset as UTC.** A silent wrong instant for local-time input; `parse_iso` already refuses it.
- **Check the weekday against the date.** Real inputs carry wrong weekdays (hand-written mail); failing hurts more than it helps.
- **Locale names and `%x`/`%c` from the system locale.** Output would differ per machine; a program must give the same text everywhere.
- **`%Z` and `%E`/`%O` modifiers in `parse_format`.** A zone abbreviation is ambiguous (`IST`); refused with a named `why`.
- **Make `rfc3339()` an alias of `iso()`.** Two names for one thing cost tokens; the synonyms file points agents to `iso`.
- **A `Date`-only `parse_format`.** `parse_format(...).date` gives it.
