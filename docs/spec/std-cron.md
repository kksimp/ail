# Standard library: `std.cron`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1249, std-gap refill).

Cron expressions: parse one into a `Schedule`, then ask for the next or the
previous firing, or the next few, in a time zone. Every function is pure
(`eff.infer.1`) and needs no capability; the current time is the caller's to
pass (`date.now().unix_ms()`). The module is written in AIL and is in scope
without `use` (`std.use.4`); `cron.parse(...)` names it. Instants are
milliseconds since the Unix epoch, as in `std.time.date`.

## The module

**std.cron.1** — `std.cron` provides `parse`, `is_valid` and the types
`Schedule`, `CronError` and `CronKind`. A `Schedule` has the fields
`text str` (the expression it was made from, trimmed), `zone str` and
`seconds bool`, and the methods `in_zone`, `next`, `prev`, `upcoming` and
`matches`. *(ADR 1249)*

```ail tool=run,FILE expect=UTC,false rules=std.cron.1
fn main() !
    s := cron.parse("*/15 9-17 * * MON-FRI")?
    print("{s.zone},{s.seconds}")
```

**std.cron.2** — `parse(text str) Schedule ! CronError` reads an expression of
five fields, minute, hour, day of month, month and day of week; or six, with
a seconds field first; or an `@` macro (`std.cron.5`). Fields are separated
by runs of spaces or tabs, and the text is trimmed. A five-field expression
fires at second 0. `is_valid(text) bool` says whether `parse` accepts it.
*(ADR 1249)*

```ail tool=run,FILE expect=false,true,true,false rules=std.cron.2
fn main() !
    five := cron.parse("0 9 * * *")?
    six := cron.parse("30 0 9 * * *")?
    print("{five.seconds},{six.seconds},{cron.is_valid("0 9 * * *")},{cron.is_valid("0 9 * *")}")
```

**std.cron.3** — Each field is items joined by `,`. An item is `*` (every
value), `a` (one value), `a-b` (an inclusive range, `a` at most `b`), and any
of the three may take `/step` (a whole number of at least 1): `*/15`, `10-30/5`.
`a/step` runs from `a` to the field's top. The ranges are second 0-59, minute
0-59, hour 0-23, day of month 1-31, month 1-12 and day of week 0-7. A value
outside its range is `OutOfRange`, a range that runs backwards is `BadRange`,
and a step that is not a number of at least 1 is `BadStep`. `L`, `W`, `#` and
year fields are not supported. *(ADR 1249)*

```ail tool=run,FILE expect=2026-11-27T07:15:00Z,2026-11-27T07:05:00Z rules=std.cron.3
fn main() !
    from := date.parse_iso("2026-11-27T07:04:00Z")?
    a := cron.parse("15,45 * * * *")?
    b := cron.parse("5-55/10 * * * *")?
    x := a.next(from.unix_ms()) else 0
    y := b.next(from.unix_ms()) else 0
    print("{date.utc(x).iso()},{date.utc(y).iso()}")
```

**std.cron.4** — Months take the names `JAN` to `DEC` and days of the week
the names `SUN` to `SAT`, in any case, alone or as range ends (`MON-FRI`,
`jan,jul`). Day of week 0 and 7 are both Sunday, so `5-7` is Friday to Sunday.
A name in any other field is `BadValue`. *(ADR 1249)*

```ail tool=run,FILE expect=2026-11-29T10:15:00Z,2026-11-29T10:15:00Z rules=std.cron.4
fn main() !
    from := date.parse_iso("2026-11-27T07:04:00Z")?
    a := cron.parse("15 10 * * 7")?
    b := cron.parse("15 10 * * sun")?
    x := a.next(from.unix_ms()) else 0
    y := b.next(from.unix_ms()) else 0
    print("{date.utc(x).iso()},{date.utc(y).iso()}")
```

**std.cron.5** — The macros are `@yearly` and `@annually` (`0 0 1 1 *`),
`@monthly` (`0 0 1 * *`), `@weekly` (`0 0 * * 0`), `@daily` and `@midnight`
(`0 0 * * *`) and `@hourly` (`0 * * * *`). Any other `@` word, `@reboot`
included, is `BadMacro`. *(ADR 1249)*

```ail tool=run,FILE expect=2026-11-28T00:00:00Z rules=std.cron.5
fn main() !
    from := date.parse_iso("2026-11-27T07:04:00Z")?
    s := cron.parse("@daily")?
    x := s.next(from.unix_ms()) else 0
    print(date.utc(x).iso())
```

**std.cron.6** — A day is a run day when its month is in the month field and
the day fields allow it. When neither the day-of-month nor the day-of-week
field begins with `*` (`?` counts as `*`), a day matches when either field
matches, as in Vixie cron: `0 12 13 * 5` is every 13th and every Friday.
When one of them begins with `*`, both must match: `0 12 */2 * 5` is Fridays
on odd days. A day that does not exist, such as `30 2` (February 30), never
matches. *(ADR 1249)*

```ail tool=run,FILE expect=2026-11-27T12:00:00Z,2026-11-30T00:00:00Z rules=std.cron.6
fn main() !
    from := date.parse_iso("2026-11-27T07:04:00Z")?
    either := cron.parse("0 12 13 * 5")?
    monday := cron.parse("0 0 ? * 1")?
    x := either.next(from.unix_ms()) else 0
    y := monday.next(from.unix_ms()) else 0
    print("{date.utc(x).iso()},{date.utc(y).iso()}")
```

**std.cron.7** — `s.next(after_ms) int?` is the first firing strictly after
`after_ms`, and `s.prev(before_ms) int?` the last strictly before it. Both
read the fields in the schedule's zone and give `none` when there is no
firing within nine years (3288 days) of the argument, as for `0 0 30 2 *`.
`s.upcoming(after_ms, count) List[int]` is up to `count` successive `next`
firings. *(ADR 1249)*

```ail tool=run,FILE expect=2026-11-27T07:30:00Z,2026-11-27T07:00:00Z,none rules=std.cron.7
fn main() !
    at := date.parse_iso("2026-11-27T07:15:00Z")?
    s := cron.parse("*/15 * * * *")?
    never := cron.parse("0 0 30 2 *")?
    n := s.next(at.unix_ms()) else 0
    p := s.prev(at.unix_ms()) else 0
    q := never.next(at.unix_ms()) else -1
    nothing := if q == -1 then "none" else "some"
    print("{date.utc(n).iso()},{date.utc(p).iso()},{nothing}")
```

**std.cron.8** — A wall-clock time that a clock change skips does not fire:
`30 2 * * *` in `America/New_York` has no firing on 2026-03-08. A wall-clock
time that happens twice fires once, the first time. A firing is always an
exact wall-clock time of its zone, so across a change the gap between
firings is 23 or 25 hours, not 24. *(ADR 1249)*

```ail tool=run,FILE expect=2026-03-09T02:30:00-04:00 rules=std.cron.8
fn main() !
    from := date.parse_iso("2026-03-07T12:00:00Z")?
    s := cron.parse("30 2 * * *")?.in_zone("America/New_York")?
    first := s.next(from.unix_ms()) else 0
    second := s.next(first) else 0
    z := date.at(second, "America/New_York")?
    print(z.iso())
```

**std.cron.9** — `s.in_zone(zone) Schedule ! CronError` is the same schedule
with its fields read in `zone`: `"UTC"` (the default), a fixed offset such as
`"+05:30"`, or an IANA name such as `"Asia/Kolkata"`. A zone the date module
does not know is `BadZone`. *(ADR 1249)*

```ail tool=run,FILE expect=2026-11-30T03:30:00Z rules=std.cron.9
fn main() !
    from := date.parse_iso("2026-11-27T04:00:00Z")?
    s := cron.parse("0 9 * * MON-FRI")?.in_zone("Asia/Kolkata")?
    x := s.next(from.unix_ms()) else 0
    print(date.utc(x).iso())
```

**std.cron.10** — `s.matches(dt) bool` is true when the date, hour, minute
and second of the `DateTime` `dt`, as they are written in it, are ones the
schedule fires at; milliseconds are ignored. Call `dt.in_zone(s.zone)` first
to read an instant in the schedule's zone. *(ADR 1249)*

```ail tool=run,FILE expect=true,false rules=std.cron.10
fn main() !
    s := cron.parse("*/15 9-17 * * MON-FRI")?
    a := date.parse_iso("2026-11-27T09:30:00Z")?
    b := date.parse_iso("2026-11-27T09:31:00Z")?
    print("{s.matches(a)},{s.matches(b)}")
```

**std.cron.11** — A `Schedule` shows as its `text`: `print(s)`, `"{s}"` and
`s.to_str()` give the expression as written, trimmed. `s.seconds` is true for
a six-field expression. *(ADR 1249)*

```ail tool=run,FILE expect=@daily,true rules=std.cron.11
fn main() !
    s := cron.parse("  @daily ")?
    t := cron.parse("*/5 * * * * *")?
    print("{s},{t.seconds}")
```

**std.cron.12** — Failures are a `CronError` with an enum `kind` and a `why`
text that names the field and the offending part (type.enum.7). The kinds are
`NoText` (empty), `FieldCount` (not five or six fields), `BadValue` (not a
number or name, or an empty item), `OutOfRange`, `BadStep`, `BadRange`,
`BadMacro` and `BadZone`. The first field in order that fails is the one
reported. *(ADR 1249)*

```ail tool=run,FILE expect=OutOfRange rules=std.cron.12
fn main()
    s := cron.parse("61 * * * *") else err
        print(err.kind.name())
        return
    print(s)
```

## Rejected alternatives

See the ADR for this module.
