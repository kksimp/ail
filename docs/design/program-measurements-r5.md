# Program measurements, round 5: the agent corpus at parity with Python

> **Non-normative design evidence,** 2026-10-20. Where the agent-tool corpus
> still cost more tokens than Python after round 4 (ADR 0255), the candidates
> measured against it, and the result of ADRs 0277–0280.

## Method

The agent-tool corpus is `logscan`, `tasks` and `build` in
`tools/tokbench/corpus/programs/` (ADR 0217), each with an idiomatic Python
twin that prints the same bytes. This round adds two programs of the same kind
with twins: `snapdiff` (compare two snapshots of a directory, the "fetch and
diff" an agent does on local files) and `csvreport` (a revenue report from a
CSV export with a malformed row). The AIL corpus files are the examples
without comment lines; like the twins, they keep their tests.

Counts are the mean of the five offline tokenizers (`tools/tokbench/run.sh`).
Candidates were applied to copies of the corpus by script or by hand, and the
"what each change saves" table reverts one change at a time from the final
files, as ADR 0255 did.

## Where the gap was (start of round 5)

Per top-level item, AIL minus Python (`+` is AIL heavier), on the corpus as
lane 41 left it (AIL 1,731.4, Python 1,620.4, +6.9%):

| Item | logscan | tasks | build | Total | Cause |
|---|---|---|---|---|---|
| `use std.x` lines vs. imports | +2.8 | +6.4 | +7.8 | +17.0 | AIL needs one line per module; Python's path root is a separate line, counted below |
| Python's path root constant (`DATA`, `ROOT`) | 0 | −18.8 | −13.4 | −32.2 | AIL builds the path in `main` |
| Declared structs and constants vs. tuples and dicts | +13.8 | +1.8 | +15.0 | +30.6 | static types; not a target |
| Helper functions | +15.6 | +20.2 | −46.4 | −10.6 | named fields in `Entry(...)`, `Todo(id: id, ...)` |
| `main` | +13.0 | +48.6 | +27.4 | +89.0 | four-line `if`/`else` where Python has a conditional expression; `Map[str, int]()` where Python has `Counter()` |
| Tests | +17.4 | +21.2 | +14.8 | +53.4 | `else return` per optional, one `assert_eq` per field, escaped `\{` in JSON |
| `if __name__ == "__main__"` | −12.0 | −12.0 | −12.0 | −36.0 | Python only |
| **Total** | **+50.4** | **+67.4** | **−6.8** | **+111.0** | |

Items are counted separately, so the rows add up to the totals within 0.2
of rounding. The `use` lines cost 54.4 tokens in all (3.1% of the corpus); the rest of the
gap was spread over constructs that each cost under 1%.

## Candidates

Each applied alone to the corpus as it stood (1,731.4) and to
`examples/*.ail` (18,181.2):

| Candidate | Agent corpus | Change | Examples | Change | Decision |
|---|---|---|---|---|---|
| Standard modules without `use` (OQ-071) | −51.4 | −3.0% | −71.8 | −0.39% | adopted, ADR 0277 |
| Messages on error variants (OQ-072) | 0.0 | 0.0% | −115.4 | −0.63% (`sheet` −1.6%) | adopted, ADR 0278 |
| `?` in tests and on optionals in `!` functions | −8.8 | −0.51% | −8.8 | −0.05% | adopted, ADR 0279 |
| One-line `if c then a else b` | −11.4 | −0.66% | −42.4 | −0.23% | adopted, ADR 0280 |
| `{}` typed by `m[k] += n` | −9.6 | −0.55% | — | — | adopted, ADR 0280 |
| Return-type inference for one-expression private functions | −9.2 | −0.53% | −46.4 | −0.26% | rejected, ADR 0280 |
| `print` with several arguments | 0.0 | 0.0% | 0.0 | 0.0% | rejected: no program prints values separated by single spaces |
| String `+` (78 interpolations rewritten) | +8.6 | +0.37% | +28.8 | +0.15% | kept rejected: costs tokens |

Rewrites that needed no language change, found while measuring: field
shorthand (`Todo(id, title, status, hours: ...)`, `type.struct.3`), comparing
a whole struct in a test (`assert_eq(parse_line(...)?, want)`), `s.len()` for
`s.chars().len()`, and `{}` for a set filled with `add`.

## Results

| Program | Python | AIL before | AIL after | Change | After vs. Python |
|---|---|---|---|---|---|
| logscan | 482.2 | 532.6 | 497.2 | −6.6% | +3.1% |
| tasks | 675.2 | 742.6 | 688.8 | −7.2% | +2.0% |
| build | 463.0 | 456.2 | 430.4 | −5.7% | −7.0% |
| **Three programs** | **1,620.4** | **1,731.4** | **1,616.4** | **−6.6%** | **−0.25%** |
| snapdiff (new) | 309.8 | — | 324.2 | | +4.6% |
| csvreport (new) | 398.6 | — | 408.6 | | +2.5% |
| **Five programs** | **2,328.8** | | **2,349.2** | | **+0.9%** |

The three-program corpus is now at parity with Python (−0.25%, from +6.9%).
The two new programs were written after the changes and land within 5% of
their twins; both lose tokens to Python built-ins that need no import or
declaration (`old.keys() | new.keys()`, `csv.reader`, `Counter`, tuple
unpacking) and win them back on path handling.

What each change saves on the final five programs (the corpus without that one
change, minus the final corpus, 2,349.2):

| Change | ADR | Tokens | Share |
|---|---|---|---|
| Standard modules without `use` | 0277 | 77.6 | 3.3% |
| `{}` typed by `m[k] += n` | 0280 | 24.0 | 1.0% |
| `?` on optionals in `!` functions and tests | 0279 | 16.8 | 0.7% |
| One-line `if c then a else b` | 0280 | 11.4 | 0.5% |

On the eight-program corpus of rounds 1–3 (regenerated from the examples),
the changes save 151.4 tokens: `sheet` 4,654.4 to 4,545.2 (messages, one-line
`if`), `json` 3,312.2 to 3,290.6, `interp` 5,399.8 to 5,392.8, `tool` 318.0 to
304.4 (no `use` lines). That corpus is now 4.1% below Python (15,643.0 against
14,997.2; it was 3.2% below). `commands` is left as round 3 measured it: its
example has since gained contracts, which aren't part of this comparison.

Messages on error types save nothing on this corpus (no program prints an
error it declared) and 115.4 tokens on the examples, almost all of it in
`sheet`.

## Speed

The changes are in the parser and checker; the interpreter only gained a check
on the `none` path of `?`. Best of 7 wall times on Linux x86-64, the binary run
directly, two alternating rounds, base branch against this branch, on the base
branch's programs: `loops` 0.669/0.679 s against 0.647/0.648 s, `parsum`
0.652/0.694 against 0.649/0.640, `interp` 0.195/0.183 against 0.199/0.188,
`json` 0.010/0.011 against 0.010/0.011, `sheet` 0.022/0.020 against
0.022/0.020, `logscan` 0.006/0.005 against 0.005/0.005, `tasks` 0.013/0.013
against 0.012/0.013. Unchanged within noise.
