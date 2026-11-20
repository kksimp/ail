# tokbench: token measurement harness

Counts how many tokens source files cost under several real model tokenizers.
It implements layers 1–2 of AIL's token benchmark
([ADR 0029](../../docs/decisions/0029-token-efficiency-benchmarks.md)).
It's a measurement instrument, not part of AIL
([ADR 0070](../../docs/decisions/0070-token-bench-tool.md)).

## Run (one command)

**macOS / Linux**, from the repository root:
```sh
tools/tokbench/run.sh
```

**Windows (PowerShell)**, from the repository root:
```powershell
powershell -ExecutionPolicy Bypass -File tools\tokbench\run.ps1
```

That's all. The first run sets everything up automatically:
- a private Python environment in `tools/tokbench/.venv`, so nothing is
  installed system-wide;
- the tokenizer files, downloaded into `tools/tokbench/.cache`.

Later runs skip setup. It needs Python 3.10 or newer. To choose a specific
interpreter, set `PYTHON=/path/to/python`.

Options (the same for both scripts):

| Command | Does |
|---|---|
| `run.sh` | Measure every file under `corpus/` |
| `run.sh FILE ...` | Measure specific files (paths relative to where you run it) |
| `run.sh --json` | Machine-readable output |
| `run.sh --claude` | Also count with the **current** Claude tokenizer (optional, see below) |
| `run.sh --check-budgets` | Check the language card and quick reference against their token budgets (see below) |

### Budget check

`run.sh --check-budgets` measures `docs/language-card.md`, `docs/language-card-core.md` and
`docs/quick-reference.md` on the offline five-tokenizer mean (the `mean` column)
and prints each number with its budget. It exits 1 if any is over, so the gate
can run it. It always uses the offline tokenizers, even when `--claude` is given.

The budgets are in one place, the `BUDGETS` table at the top of `tokbench.py`:

| Page | Budget | Source |
|---|---:|---|
| `docs/language-card.md` | 16,000 | [ADR 1110](../../docs/decisions/1110-language-card-budget-16000.md) |
| `docs/language-card-core.md` | 3,000 | [ADR 1110](../../docs/decisions/1110-language-card-budget-16000.md) |
| `docs/quick-reference.md` | 12,518 | [ADR 0817](../../docs/decisions/0817-token-budgets-for-agent-pages.md) |

The mean rule is [ADR 0817](../../docs/decisions/0817-token-budgets-for-agent-pages.md).
ADR 0276 stated the card budget per tokenizer ("on every tokbench tokenizer"); the
mean is looser, so a card that passes the mean can still fail one tokenizer. ADR 0703
is about `ail api --card` trimming levels and does not state a budget. Change a budget
only here, and cite its ADR next to it. Any quick-reference addition is paid for by a
trim in the same change (ADR 0817).

### Optional: the current Claude tokenizer

The offline `claude-legacy` column is an **older** Claude tokenizer. Current
Claude tokenizers are only available through Anthropic's token-counting API,
which needs an Anthropic API key (`ANTHROPIC_API_KEY`, or `ant auth login`).
The counting calls aren't billed, but getting a key means creating an Anthropic
developer account. This is entirely optional. All AIL syntax decisions so far
were made with the offline tokenizers, which have agreed with each other on
every comparison. The Anthropic SDK is installed only when you pass `--claude`.

The API counts a whole message, so the tool subtracts the framing overhead.
Results may be off by about one token per file.

## Tokenizers

| Column | Source | Notes |
|---|---|---|
| `cl100k` | OpenAI `cl100k_base`, via the pinned `litellm` wheel | OpenAI, 2023 |
| `o200k` | OpenAI `o200k_base`, via the same wheel | GPT-4o era |
| `claude-legacy` | `anthropic_tokenizer.json`, via the same wheel | Old Claude tokenizer. Use `--claude` for current models. |
| `mistral-tekken` | `tekken_240911.json`, via the pinned `mistral-common` wheel | Recent Mistral |
| `mistral-spm-v1` | `tokenizer.model.v1`, via the same wheel | Older SentencePiece |
| `claude:<model>` | Anthropic API `messages.count_tokens` | Only with `--claude` |

## Corpus

The full language card has a budget of 16,000 tokens and the core card 3,000 ([ADR 1110](../../docs/decisions/1110-language-card-budget-16000.md)); check them with `run.sh --check-budgets` (or `run.sh docs/language-card.md` for the per-tokenizer table).

| Directory | Contents |
|---|---|
| `corpus/r1/` | Round-1 block-style samples (A–D) plus Rust and Go references, as measured in [syntax-measurements-r1.md](../../docs/design/syntax-measurements-r1.md). They use candidate syntax from before later decisions. |
| `corpus/r2/` | Methods, trait conformance, generic constraints, and receiver-mutation variants. See [traits-generics-options.md](../../docs/design/traits-generics-options.md). |
| `corpus/r3/` | Operators, trait-typed parameters, dynamic dispatch, and attribute variants (ADRs 0077–0083). |
| `corpus/r4/` | Printing, interpolation, collection types and literals, `elif`, optional wrapping, and ranges (ADRs 0088–0093). |
| `corpus/r5/` | Concurrency (colorless vs. async/await), visibility (`pub` vs. capitalisation), and import styles. |
| `corpus/r8/` | Format specifications (`{x:>8}`, `{r:.2}`) vs `pad_left`/`to_hex`/`to_fixed` calls on a table-and-money program (ADR 0173). |
| `corpus/programs/NAME/` | Whole programs with Python twins (and Go, Rust, TypeScript, C or Swift ports for some), compared in [program-measurements-r2.md](../../docs/design/program-measurements-r2.md), [-r3.md](../../docs/design/program-measurements-r3.md) and [-r5.md](../../docs/design/program-measurements-r5.md). The `.ail` files are `examples/NAME.ail` without comments, blank lines or `@test` functions (the agent-tool programs `logscan`, `tasks`, `build`, `snapdiff` and `csvreport` keep their tests, as their twins do): regenerate them with `python3 tools/tokbench/corpus_from_examples.py`. |
| `corpus/r9/` | One 4-dependency package manifest in seven formats: line per dependency, indented, dotted keys, TOML (two forms), JSON, AIL literal (ADR 0230). |
| `corpus/current/` | Samples in the syntax decided so far (as of 2026-10-04). `tokenize.ail` predates ADR 0089 (it still indexes a `str` directly) and also uses constructs that aren't decided yet: `while`, `char`, slicing, `+=`, `and`, methods that mutate a `mut` binding, and returning an error value. Corpus files hold no explanatory comments, because comments would distort the counts. |

## Attribution

The `.rs`, `.go`, `.ts`, `.c` and `.swift` files under `corpus/` are measurement
twins, not part of the toolchain: the Rust bootstrap is gone
([ADR 0904](../../docs/decisions/0904-rust-bootstrap-deleted.md)) and nothing builds them.


`attribute.py` (measurement only) says where the tokens of the
`corpus/programs/` go: for each construct class (signature types, `fn`,
`self`, `:=`, `mut`, `use`, `else`, `?`, indentation, ...) it deletes that
class from every program and reports the difference, for AIL and for Python.
Run it after `run.sh` has set up once:

```sh
tools/tokbench/.venv/bin/python tools/tokbench/attribute.py [--json]
```


## Known print difference: csv2json (lane 252)

`corpus/programs/csv2json` prints the same JSON in both twins except for number text. AIL prints a whole float without a fraction (`12`, json.print.1 under ADR 0631), and Python's `json` prints `12.0`. The difference is inherent to the two languages' printing rules, so the twin is kept as it is, and the difference is not a bug. Its output is therefore not byte-identical to AIL's, and the token rows for this pair are unchanged by it.
## Corpus change log

- **2026-10-21, data argument, `ail` lookup and repository depth** (21 AIL programs, 19 tools-path programs, and their twins; output checked against the Python twin):
  - `programs/days_left`, `digest_lines`, `gz_check`, `redact_mail`, `regex_codes`, `shift_dates`: the data directory is an optional first argument, defaulting to `agent-data` beside the program. Both twins read it the same way. No copies of `agent-data` in the program folders.
  - All 21 programs that run `ail` find it with `proc.which("ail")`, falling back to the repository root (`{here}/../../../../../ail`). The twins use `shutil.which("ail")`.
  - 19 programs read `{here}/../tools/...` data; they now use `{here}/../../../../../tools/...`, so they resolve from the repository layout as well as from the fixture.
  - Measured on the same 72 pairs (144 files) in `corpus/programs`: AIL 28,845.4 → 29,251.2 (+1.41%); Python 32,262.0 → 32,643.0 (+1.18%); AIL/Python 0.8941 → 0.8961 (+0.2%, inside the 2% tie band). The AIL rise is the `ail` lookup and data-argument lines, one per program.

- **2026-10-21, twin output check** (no corpus file changed; no token count changed): `twin_outputs.py` runs every `corpus/programs/NAME` pair and diffs the AIL program's output against its Python twin's. Each pair runs in a mirror tree laid out like the repository (`ail` wrapper, `tools/` links, `examples/` and `agent-data/` links), because the programs find their tools and data by path. `tools/bench/bench.py` prints the result. Result: 70 of 72 pairs match exactly, `csv2json` matches apart from the documented whole-float difference, and `fetch` is skipped (it needs network access). `scan` agreed once the mirror linked `examples/`.
Day-over-day totals move when a corpus program changes, not only when the language does. Each entry
says which program changed and why, with its five-tokenizer mean before and after.
- **2026-10-21, idiomatic rewrites** (four programs; output checked byte-identical to the Python twin):
  - `programs/report_dates`: `date.utc(ms).format(...)` (std.time.date) replaces the hand-written civil-date function, with `use std.time.date`. 409.2 → 217.6.
  - `programs/hexdump`: reads bytes with `fs.read_bytes` and formats rows in-process, instead of shelling out to `od`. 147.0 → 124.2.
  - `agent-r4/t01_wordcount`: `counts.top(3)` replaces the sort-by-count pipeline. 57.0 → 43.6.
  - `programs/bits`: `mode_text` uses one-line `if ... then ... else`. 425.4 → 414.6.
  Total for the four: 1,038.6 → 800.0 (−238.6). These programs are now idiomatic, so the AIL/Python ratio for them falls for a reason that is not a language change.
- **2026-10-21, tuples (ADR 0791-0794)** (two twins; output checked byte-identical to the Python twin):
  - `agent-r4/t12_sort_ties`: the people are `(name, age)` pairs sorted by the key `(-p.1, p.0)`, as the Python twin does, instead of a `struct P` and two `sorted` passes. 91.4 → 57.6.
  - `agent-r4/t03_table`: the rows are `(str, int)` pairs taken apart by `for name, qty in rows`, as in Python. 38.8 → 35.2.
  Total for the 25 std twins: 1,128.6 → 1,091.2 (−37.4), AIL against Python +13.9% → +10.1%. This change is the language's.
- **2026-10-21, day 3** (output checked byte-identical to the Python twin where one runs):
  - `agent-r4/t11_json_transform`: `load_json[List[U]](p)?` and `print_json(v)` replace the `from_json`/`parse_json`/`to_text` chain (ADR 0774). 89.6 → 76.4.
  - `programs/hexdump`: `data[a..b].hex(" ")` replaces the row comprehension and join (ADR 0775). 124.2 → 104.2.
  - Removed the superseded duplicates `agent-r4/t11b_json_typed`, `t20b_grep_methods` and `t23b_copy_tree_std`; t11, t20 and t23 are those forms.
- **2026-10-21, `agent-r4/t05_args`** (lane 192, ADR 0777): the options are declared once as a struct with `impl FromArgs` and read with `Opts.parse`, instead of a spec list. Output checked equal for the same command lines. 95.4 → 85.6.
