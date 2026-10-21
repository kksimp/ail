# Python twins for the examples, measured (lane 228)

> Non-normative measurement, 2026-10-21, batch head `fcc7f33d`. Twins are in
> `tools/tokbench/corpus/programs/NAME/NAME.py` (the corpus layout, lane 143), and
> each AIL file is the example without its comment lines. Tokens are the mean of
> the five tokenizers (`tools/tokbench/run.sh`). `tools/bench/bench.py` finds the
> corpus pairs, so they appear in the before/after table as `agent/NAME`.

## Coverage

- 59 programs in `examples/`. 51 have a Python twin with byte-identical output
  (checked with `cmp` against `examples/NAME.out`).
- 8 have no twin, with the reason:
  - `ui_hello`, `dashboard`: need a window or a `std.ui.widgets` layout (the
    non-window screen is a port of the widget library, not a twin).
  - `native_collatz`, `native_fib`, `native_mandel`, `native_sieve`, `native_words`:
    written for the native backend (`ail build`).
  - `config`: its output shows an `int` field as a float (`"replicas":3.0`, from
    `ToJson`). A twin would have to copy that. Filed here as a finding:
    `S(n: 3).to_json()` gives `{"n":3.0}` on this head.
- Twins use the same inputs (the `agent-data/` and `data/` files) and print the
  same bytes. `fetch` and `loopback` run offline; `fetch` reads the local copy the
  example also falls back to.

## Totals

- 51 pairs. Sum of tokens: AIL 25,461, Python 26,597 (AIL −4%).
- Geometric mean of AIL / Python: **0.911** (AIL 9% smaller). AIL is smaller in 36
  pairs and larger in 15.
- `regex_codes` is 127.0 here. With `regex.group_all` (PR #243) it is 106.2.

## Table

| Example | AIL | Python | AIL vs Python |
|---|---|---|---|
| `backup` | 177.2 | 175.6 | +1% |
| `bits` | 425.4 | 378.2 | +12% |
| `build` | 429.2 | 463.0 | -7% |
| `calendar` | 420.2 | 697.2 | -40% |
| `chan_pipe` | 94.2 | 129.2 | -27% |
| `checksums` | 103.6 | 158.8 | -35% |
| `commands` | 524.8 | 677.2 | -23% |
| `compress` | 140.2 | 152.0 | -8% |
| `csv2json` | 187.8 | 164.8 | +14% |
| `csvreport` | 407.4 | 398.6 | +2% |
| `days_left` | 111.2 | 132.0 | -16% |
| `digest` | 288.2 | 357.0 | -19% |
| `digest_lines` | 62.2 | 103.0 | -40% |
| `fetch` | 162.6 | 183.8 | -12% |
| `ffi_buffers` | 366.2 | 254.2 | +44% |
| `ffi_libc` | 208.8 | 189.4 | +10% |
| `frames` | 117.4 | 134.4 | -13% |
| `gradebook` | 214.4 | 270.0 | -21% |
| `gz_check` | 110.6 | 130.0 | -15% |
| `hello` | 10.8 | 23.8 | -55% |
| `hex_base64` | 87.2 | 119.8 | -27% |
| `hexdump` | 147.0 | 117.6 | +25% |
| `interp` | 5419.0 | 5565.2 | -3% |
| `json` | 3306.8 | 3233.2 | +2% |
| `logscan` | 496.4 | 482.2 | +3% |
| `logwatch` | 219.4 | 235.4 | -7% |
| `loopback` | 160.6 | 285.0 | -44% |
| `matrix` | 120.4 | 213.4 | -44% |
| `merge` | 364.8 | 369.0 | -1% |
| `parallel` | 331.8 | 311.6 | +6% |
| `perms` | 375.8 | 267.8 | +40% |
| `pipeline` | 225.8 | 240.8 | -6% |
| `poller` | 234.0 | 267.8 | -13% |
| `redact_mail` | 112.6 | 126.4 | -11% |
| `regex` | 186.6 | 210.2 | -11% |
| `regex_codes` | 127.0 | 94.2 | +35% |
| `report_dates` | 409.2 | 181.6 | +125% |
| `route` | 313.4 | 256.8 | +22% |
| `runner` | 290.0 | 318.6 | -9% |
| `scan` | 285.6 | 326.0 | -12% |
| `shapes` | 448.6 | 569.2 | -21% |
| `sheet` | 4572.0 | 4674.0 | -2% |
| `shift_dates` | 90.4 | 115.6 | -22% |
| `snapdiff` | 313.4 | 309.8 | +1% |
| `supervise` | 265.6 | 267.8 | -1% |
| `tail` | 317.4 | 332.4 | -5% |
| `task_sum` | 157.2 | 162.2 | -3% |
| `tasks` | 596.0 | 675.2 | -12% |
| `tensors` | 224.4 | 331.8 | -32% |
| `tool` | 305.6 | 343.0 | -11% |
| `workers` | 394.2 | 421.2 | -6% |

## The ten largest AIL losses

Each has the construct responsible and a proposal. Proposals marked *usage* use
library code that already exists; the program does not use it.

1. **`report_dates`, +125%** (409 vs 182). The program writes its own
   civil-from-days calendar (13 lines). *Usage:* `std.time.date` already gives
   `date.utc(ms).format("%Y-%m-%d")`. Rewritten, it is 217 tokens with the same
   output (checked). Library proposal: none needed.
2. **`ffi_buffers`, +44%** (366 vs 254). Raw `mem.alloc`, `write`, `read`, `free` and
   `unsafe` blocks around each C call; Python's `ctypes` arrays are one line. Proposal:
   `mem.buffer(n, fill)` that frees itself at scope end (no syntax change).
3. **`perms`, +40%** (376 vs 268). `has` is written `(mode / bit) % 2 == 1`, because
   AIL's bit operators are methods. *Usage:* `mode.bit_and(bit) != 0` is the existing
   method, and it is shorter. The file modes are written in decimal (`420`), where
   Python has `0o644`; that is a literal syntax question, not a library one.
4. **`regex_codes`, +35%** (127 vs 94; +13% with PR #243). Fixed by
   `regex.group_all` (ADR 0667). Remaining cost: the `fs.read_text` line with its
   path and context string.
5. **`hexdump`, +25%** (147 vs 118). The program shells out to `od` and then splits
   the rows. Library proposal: `bytes.hex(" ", 16)`, giving the spaced hex rows of a
   file in one call, so no process is needed.
6. **`route`, +22%** (313 vs 257). Two `if` guards and a `match` on the status, plus
   the `\r\n` string with `.replace("\r", "")`. No library gap: the cost is
   `match`/`if` structure, and a rewrite is the same length.
7. **`csv2json`, +14%** (188 vs 165). A hand-built `Json` tree per row, with a type
   rule per column. Library proposal: `std.csv` (PR #238, not yet merged) gives the rows; `Json` from a `Map[str, str]` with typed columns is a
   one-liner proposal for `Json.from_map`.
8. **`bits`, +12%** (425 vs 378). Multi-line `if` expressions for each flag, where
   Python uses one-line conditionals. *Usage:* `if c then "r" else "-"` on one line is
   already valid AIL (see `report_dates`). No library gap.
9. **`ffi_libc`, +10%** (209 vs 189). `unsafe` and `mem.cstr`/`mem.free` around each
   call, as in item 2. Proposal: `mem.with_cstr(text, fn)` removes the free.
10. **`parallel`, +6%** (332 vs 312). The `rows.push("")` placeholder and the
    `x.to_float()` conversions. No library gap; the difference is the builder shape.

Items 1, 3 and 8 are usage fixes, not library changes. Items 2, 5, 7 and 9 each
need a library addition, and each is small enough for a lane of its own. No
syntax change is proposed here, so `docs/open-questions.md` is not changed.
