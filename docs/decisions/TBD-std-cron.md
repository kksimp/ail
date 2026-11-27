# TBD. `std.cron`: cron expressions with next and previous firing in a time zone

- **Status:** Accepted (the integrator assigns the number)
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); part of the designer's std library push (2026-11-26)
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.cron.1 to std.cron.12 in [std-cron.md](../spec/std-cron.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)

## Context

The task corpus has `cron_field_parser`, and schedulers, job runners and log tools all need "when does this
expression fire next". Agents write the field expansion each time and get the same parts wrong: the day-of-month and
day-of-week rule, Sunday as 0 and 7, steps that start at a value, and above all clock changes. The std plan lists it
as a pure AIL module on top of `std.time.date` and `std.tz`.

## Decision

A new module written in AIL, `std/cron/cron.ail`, in scope without `use` (std.use.4). No syntax change, no runtime or
checker work; it uses `std.time.date` for the calendar, zones and the zone database.

- `cron.parse(text) Schedule ! CronError` reads five fields, six with seconds first, or an `@` macro (`@yearly`,
  `@annually`, `@monthly`, `@weekly`, `@daily`, `@midnight`, `@hourly`). Fields take `*`, values, ranges, lists, `/step`
  and the month and weekday names; weekday 7 is Sunday. `?` counts as `*` in the day fields.
- Day fields follow Vixie cron: when neither begins with `*`, either matching is enough; otherwise both must match.
- `s.next(after_ms) int?` and `s.prev(before_ms) int?` are strict and give `none` after a nine-year search, so
  `0 0 30 2 *` ends instead of looping. `s.upcoming(after_ms, n)` is the list form; `s.matches(dt)` the test form.
- A schedule is read in UTC until `s.in_zone(zone)` (a new value, `BadZone` for an unknown zone). A wall time a
  clock change skips does not fire; one that happens twice fires once, the first time. Firings are exact wall-clock
  times of the zone.
- Failures are `CronError` with an enum `kind` (`NoText`, `FieldCount`, `BadValue`, `OutOfRange`, `BadStep`,
  `BadRange`, `BadMacro`, `BadZone`) and a text naming the field (type.enum.7).
- The search walks days, then the schedule's own hours, minutes and seconds inside a day that matches, so a sparse
  schedule costs days, not minutes.

## Rejected alternatives

- **A zone argument on every call (`s.next(ms, zone)`).** Repeats a value that belongs to the schedule, and an
  unknown zone would need an error on a function whose result is an optional.
- **`next` returning `int ! CronError`.** "Never fires" is an ordinary answer, not a failure; the optional says it.
- **Firing the second pass of a repeated wall time as well.** Surprising for a daily job (it runs twice that day);
  Vixie cron and most schedulers run it once.
- **Running a skipped wall time just after the gap.** Moves a 02:30 job to 03:00 on one day, which can collide with a
  03:00 job; skipping is predictable and documented.
- **Quartz extensions (`L`, `W`, `#`, years).** Different grammar, rarely needed, and a separate dialect can be added
  later without changing the five-field form.
- **`@reboot`, `@every 5m`.** Not expressions of times; `@reboot` needs a process lifetime this module does not have.
- **A module named `schedule`.** A common local name, which hides the module (std.use.4); `cron` clashes with none.
