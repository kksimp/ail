# Agent round 4: standard-library parity with Python (lane 192)

Date: 2026-10-21. Twins in `tools/tokbench/corpus/agent-r4/` (`tNN_*.py` and
`tNN_*.ail`), measured with `tools/tokbench/run.sh` (mean over the five
tokenizers). Ten of the planned 25 tasks are written so far; the table is
updated as more land. Before the lane's additions the same tasks needed
hand-written loops (see the `csv_manual`/`args_manual` pairs below).

## Result

| task | Python | AIL | AIL vs Python |
|---|---|---|---|
| t01_wordcount | 42.4 | 57.0 | +34% |
| t02_batches | 32.6 | 26.8 | -18% |
| t03_table | 36.8 | 38.8 | +5% |
| t04_csv_report | 37.2 | 53.6 | +44% |
| t05_args | 71.0 | 95.4 | +34% |
| t06_log | 24.8 | 22.6 | -9% |
| t07_flatten | 25.2 | 18.0 | -29% |
| t08_whitespace | 30.2 | 38.0 | +26% |
| t09_group | 37.8 | 41.6 | +10% |
| t10_retry | 85.2 | 84.8 | -0% |
| **total** | 423.2 | 476.6 | +13% |

By hand against the new modules (ADRs 0626, 0627):

| task | by hand | with std | change |
|---|---|---|---|
| CSV quoted-column sum | 236.4 | 69.6 | -71% |
| `[-v] [--out F] INPUT` | 240.0 | 113.0 | -53% |

## Where AIL still costs more than Python

- **Reading input.** `fs.read_text(env.args()[0])?` plus a `?` per fallible
  call is the bulk of t01, t04 (+34%, +44%). Python's `open(...).read()`
  has no error plumbing. This is the language's error-handling design, not a
  library gap.
- **`to_int() else 0`.** Python's `int(x)` raises; AIL needs a fallback.
- **`most_common`.** t01 needs `keys().sorted(by: |k| -counts[k]).take(n)`.
  A `counts().top(n)` is a candidate once tuples or pair structs exist.
- **argparse (+34%).** The help text and typed options are in the std version;
  Python's argparse has no `rest`.
- **No `zip`, `enumerate`, `items()`**: these need tuples or two-variable
  `for` (a language change, out of this lane's scope; recommendation: a
  `for k, v in map` and `for i, x in xs.enumerate()` form, to be measured).

## Added by this lane

`std.csv`, `std.args`, `std.log` (ADRs 0626-0628); list `chunks`, `windows`,
`counts`, `flatten`; string `center`, `words`, `trim_start`, `trim_end`,
`count`, `rfind` (ADR 0629). Language card 7,019 → 7,044 tokens (budget 8,000).
