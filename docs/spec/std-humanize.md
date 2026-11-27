# Standard library: `std.humanize`

Chapter key: `std`

> **Status:** Drafted (ADR TBD). Written in AIL (`std/humanize/humanize.ail`).

Numbers, sizes and times written for people to read. Every function is pure
(`eff.infer.1`) and needs no capability. The module is in scope without `use`
(`std.use.4`); `humanize.int_text(n)` names it. Durations as `1h30m` stay in
`std.time.date` (`date.duration_text`); this module does not repeat them.

## The module

**std.humanize.1** — `Locale` is a struct of four public fields: `group` (between
groups of three integer digits), `decimal` (before the fraction), `percent_sep`
(between a number and `%`) and `min_digits` (the integer part is grouped only
when it has at least that many digits, and never below four). Build one with
`Locale(group: ..., decimal: ..., percent_sep: ..., min_digits: ...)`. Every
formatting function below is a method of `Locale`; the same name as a free
function is the English style. *(ADR TBD)*

**std.humanize.2** — `en()`, `de()`, `fr()`, `es()` and `ja()` give the styles of
those languages, after the CLDR data: English `1,234.5`; German `1.234,5` and
`12 %` with a no-break space; French `1 234,5` with a narrow no-break space
(U+202F) and `12 %`; Spanish `1234,5` but `12.345,6` (`min_digits` 5);
Japanese as English. `locale(tag str) Locale?` finds one by tag: the language
part before `-` or `_`, in any case (`de-AT`, `fr_CA`); `none` for any other
language. *(ADR TBD)*

**std.humanize.3** — `custom(group, decimal) Locale` is a style with your own
separators: groups from four digits up, no space before `%`. *(ADR TBD)*

**std.humanize.4** — `int_text(n int) str` writes an integer with the style's
group separator: `1,234,567`, `-1,234,567`. It works for every `int`. *(ADR TBD)*

**std.humanize.5** — `float_text(x float, decimals int) str` writes `x` with
exactly `decimals` decimals, rounded half away from zero, grouped, with the
style's decimal mark. `decimals` is cut to 0 to 9. A result that is all zeros
has no sign (`-0.001` to two decimals is `0.00`). `nan`, `inf` and `-inf` give
those words. *(ADR TBD)*

**std.humanize.6** — `percent(ratio float, decimals int) str` writes `ratio * 100`
as `float_text` does, then the style's `percent_sep` and `%`: `0.256` with one
decimal is `25.6%`. *(ADR TBD)*

**std.humanize.7** — `compact(n int, decimals int) str` writes `n` in thousands
(`K`), millions (`M`), billions (`B`) or trillions (`T`) with at most `decimals`
decimals and no trailing zeros: `1234` is `1.2K`, `2000000` is `2M`. A value
that rounds up to the next step moves to it (`999950` is `1M`, not `1000K`).
Below 1000 the number is whole; past 999 trillion the count of trillions grows.
The suffixes are the English ones in every style. *(ADR TBD)*

**std.humanize.8** — `size_si(n int, decimals int) str` writes a byte count in
powers of 1000: `B`, `kB`, `MB`, `GB`, `TB`, `PB`, `EB`, a space before the unit,
trailing zeros dropped, the rounding of std.humanize.7. A count below 1000 is
whole and ungrouped (`999 B`). *(ADR TBD)*

**std.humanize.9** — `size_iec(n int, decimals int) str` is the same in powers of
1024: `B`, `KiB`, `MiB`, `GiB`, `TiB`, `PiB`, `EiB`. *(ADR TBD)*

**std.humanize.10** — `ordinal(n int) str` is `n` with the English suffix: `st`,
`nd`, `rd`, otherwise `th`, and `th` for 11, 12 and 13 in every hundred
(`11th`, `112th`, `21st`); a negative number keeps its sign (`-2nd`). *(ADR TBD)*

**std.humanize.11** — `relative(then_ms int, now_ms int) str` says how far `then_ms`
is from `now_ms`, both in milliseconds: `3 minutes ago`, or `in 2 hours` when
`then_ms` is later. Under a second either way is `just now`. The largest unit
that fits is used and the count is cut down, never rounded: seconds below 60,
minutes below 60, hours below 24, days below 7, weeks below 30 days, months
(30 days) below 360 days, then years (365 days, at least 1). One is singular
(`1 hour ago`). English only. *(ADR TBD)*

**std.humanize.12** — No function traps on its input. The one `int` arithmetic is
`now_ms - then_ms` in `relative`, which traps on overflow like any `int`
subtraction. Float rounding follows `float` arithmetic: a value that sits a
hair under a half in binary (`1.005` to two decimals) rounds down. *(ADR TBD)*

```ail rules=std.humanize.1,std.humanize.2,std.humanize.3,std.humanize.4
fn main()
    print(humanize.int_text(1234567))                 # 1,234,567
    print(humanize.de().int_text(1234567))            # 1.234.567
    print(humanize.es().int_text(1234))               # 1234
    print(humanize.es().int_text(12345))              # 12.345
    print(humanize.custom("_", ",").int_text(1234567))  # 1_234_567
    print(humanize.locale("fr_CA") != none)           # true
    print(humanize.locale("xx") == none)              # true
```

```ail rules=std.humanize.5,std.humanize.6,std.humanize.7
fn main()
    print(humanize.float_text(1234.5, 2))             # 1,234.50
    print(humanize.de().float_text(1234.5, 2))        # 1.234,50
    print(humanize.float_text(-0.001, 2))             # 0.00
    print(humanize.percent(0.256, 1))                 # 25.6%
    print(humanize.compact(1234, 1))                  # 1.2K
    print(humanize.compact(999950, 1))                # 1M
```

```ail rules=std.humanize.8,std.humanize.9,std.humanize.10,std.humanize.11,std.humanize.12
fn main()
    print(humanize.size_si(1500000, 1))               # 1.5 MB
    print(humanize.size_iec(1572864, 1))              # 1.5 MiB
    print(humanize.size_iec(1023, 1))                 # 1023 B
    print(humanize.ordinal(112))                      # 112th
    print(humanize.relative(0, 180000))               # 3 minutes ago
    print(humanize.relative(120000, 0))               # in 2 minutes
```

## Rejected alternatives

See [ADR TBD](../decisions/XXXX-std-humanize.md).
