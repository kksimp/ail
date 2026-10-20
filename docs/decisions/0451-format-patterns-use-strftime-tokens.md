# 0451. `format` patterns use strftime tokens

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.date.10](../spec/17-std.md#calendar-stdtimedate)

## Context

`dt.format(pattern)` needs a small token language. Two families exist: strftime
(`%Y-%m-%d %H:%M:%S`, from C, Python, Go's Unix `date`, Ruby, PHP) and letter
patterns (`YYYY-MM-DD HH:mm:ss`, from Moment, Java, Unicode CLDR).

## Options considered

### Option A — strftime tokens (chosen)
- **Pros:** the most widely known set, so a model writes them without checking;
  every token is one character after `%`, with no case trap: `%m` month and `%M`
  minute are different letters and both are always two digits.
- **Cons:** about 10% more tokens in a format string than letters.

### Option B — letter patterns
- **Pros:** fewer tokens in the pattern.
- **Cons:** `mm` is minutes and `MM` months, `hh` is 12-hour and `HH` 24-hour, a
  known source of bugs; quoting literal letters needs an escape syntax.

Measured with `tools/tokbench/run.sh --json` on three `format` helpers
(`%Y-%m-%d %H:%M:%S %z`, `%a %b %e, %Y`, `%I:%M %p` against
`YYYY-MM-DD HH:mm:ss ZZ`, `ddd MMM D, YYYY`, `hh:mm A`):

| Tokenizer | strftime | letters |
|---|---|---|
| cl100k | 62 | 51 |
| o200k | 65 | 54 |
| claude-legacy | 70 | 65 |
| mistral-tekken | 65 | 58 |
| mistral-spm-v1 | 73 | 74 |
| mean | 67.0 | 60.4 |

The gap is 10% of a format string, which is a rare construct in
representative programs (under 2% of any program's tokens), so token usage
is a tie (ADR 0076). The tiebreaker is AI understanding.

## Decision

Option A: `%Y %y %m %d %e %j %H %I %p %M %S %f %a %A %b %B %z %:z %Z %s %%`.
An unknown token is kept as written (`%q` prints `%q`) so a typo shows in the
output. Names are English.

## Consequences

- Locale-aware names are out of scope; a program that needs them formats
  the fields itself.
- Adding a token later changes what a pattern with that `%x` prints, so each
  addition is recorded in a new ADR.
