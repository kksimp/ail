# Program measurements, agent round 4: ten more programs on the new modules

> **Non-normative design evidence,** 2026-10-21 (lane 203 of batch 2, night run).
> Ten agent-tool programs that use the modules added since round 3: `std.hash`,
> `std.regex`, `std.compress`, `std.time.date`, `bytes`, and `Channel` with
> `spawn`. Each has a Python twin that prints the same bytes. The programs are in
> `examples/` with their `.out`; the twins are in `tools/tokbench/corpus/programs/NAME/`.
> Earlier rounds: [round 3](program-measurements-agent-r3.md).

## Method

As in round 3: counts are the mean of the five offline tokenizers
(`tools/tokbench/run.sh`). The `.ail` files are the examples without comment
lines (`tools/tokbench/corpus_from_examples.py`); the `.py` twins are kept as
written. Every pair was run: `python3 NAME.py` and `ail run examples/NAME.ail`
print the same bytes (checked with `cmp`).

Reference treatment, so the comparison is fair: the card is
`docs/language-card.md` (7,019 tokens on this head); the Python reference is
`tools/tokbench/reference/python-reference.md` (6,813 tokens). "Both pay" counts
each language's card or reference once, on top of the programs.

## Results

| Program | Features exercised | AIL | Python | AIL vs Python |
|---|---|---|---|---|
| `digest_lines` | `hash.sha256`, `hash.crc32`, line split | 62.2 | 103.0 | −40% |
| `regex_codes` | `regex.find_all`, `m.text`, `strip_prefix`, `to_int` | 127.0 | 94.2 | +35% |
| `redact_mail` | `regex.is_match`, `regex.replace` | 112.6 | 126.4 | −11% |
| `gz_check` | `compress.gzip` / `gunzip`, `bytes.text()` | 110.6 | 130.0 | −15% |
| `shift_dates` | `date.parse_date`, `add_days`, `weekday`, `iso` | 90.4 | 115.6 | −22% |
| `days_left` | `date.parse_date`, `Date.since` | 111.2 | 132.0 | −16% |
| `frames` | byte indexing and slicing, `hex`, `text` | 117.4 | 134.4 | −13% |
| `hex_base64` | `bytes.from_hex`, `base64`, `from_base64` | 87.2 | 119.8 | −27% |
| `task_sum` | four `spawn` tasks, `wait()?` | 157.2 | 162.2 | −3% |
| `chan_pipe` | bounded `Channel`, producer task, `for v in ch` | 94.2 | 129.2 | −27% |
| **Total** | | **1,070.0** | **1,246.8** | **−14.2%** |

Programs alone, neither pays a card or reference: AIL 1,070 against Python
1,247, which is −14%. Nine of ten are cheaper in AIL; `regex_codes` is the
one that costs more.

Both pay (AIL card 7,019 plus programs; Python reference 6,813 plus programs):
AIL 8,089 against Python 8,060, which is **+0.4%**. The card is larger than the
Python reference, so on this set the card cost cancels the program savings. The
whole-task study has the same shape: it reports about −3.6% with both pay, on
nine tasks, a different set of programs (see
[whole-task study](#whole-task-study-on-this-head)).

## Whole-task study on this head

The study's numbers come from transcripts of agent runs, and a rerun needs fresh
agents, so this lane did not repeat the agent runs. What this lane checked:

- The card on this head is `docs/language-card.md`, 7,019 tokens. That is the
  same size as the study's last full rerun (82f0f194). PR #220 (card trim, 6,672)
  is not on the batch head at 220c9fb, so the trimmed-card numbers still stand
  as the study's own projection.
- The latest fixed drafts of the five AIL tasks that have them (`t1_c3`,
  `t2_c3`, `t3_c2`, `t4_c2`, `t5_c2`) pass `ail test` on this head: 1 + 2 + 2 + 2 + 1
  tests, all passing. The task files are read from a scratch copy.

Study totals as last measured (not changed by this lane): neither pays, nine
tasks, AIL 3,108 against Python 3,688 (−16%); both pay, AIL 10,128 with the
7,019 card against Python 10,501 (−3.6%).

## Top three remaining gaps (proposals)

Ranked by how much more AIL spends than Python, on this round and the study:

1. **Pulling a value out of a regex match** (`regex_codes`, +35%). The
   program finds each `code=NNN`, then strips the prefix and converts the rest.
   A capture accessor (`regex.captures` with a group index, or a `m.group(1)`
   that gives a `str?`) would drop the `strip_prefix` line and the `else "0"`.
   Python's `re.findall` returns the group directly. Proposed for the next
   regex lane; measure with this program.
2. **Converting and checking a file's text and its compressed bytes** (`gz_check`
   and `frames`, −13% to −15%). `text.to_bytes().len()` and `back.text()` are
   needed only to compare sizes and strings, and each `?` on a `bytes?` adds a
   line. Proposal: `bytes.len_of(str)` or a `str.byte_len()`, and a `==` between
   `bytes` and `str`. Cost is small per program; it recurs across byte programs.
3. **One handle per spawn** (`task_sum`, −3%, the closest to Python). Four
   `spawn` lines, four names, and a `.wait()?` on each. A `par map` over a range
   of chunks (`[sum_range(lo, hi) par for ...]`, see the quick reference's parallel
   map) would write this in one line, and the handle-per-name form would go.
   Proposal: check whether `par for` covers this task shape before adding syntax.

These are proposals for the integrator. They are not in `docs/open-questions.md`:
each needs an option list and a measurement before it becomes an open question.

## Syntax candidates, measured (lane 208)

[ADR 0675](../decisions/0675-round-4-syntax-candidates.md) measured the syntax
proposals on the 40 twinned programs. Proposal 3 (`task_sum`) needs no new
syntax: `[sum_range(lo, lo + 250) par for lo in [0, 250, 500, 750]].sum()` is
111.2 tokens against 157.2 for four `spawn` handles, so `task_sum` is −31%
against Python (162.2), and the round-4 total is 1,024.0 against 1,246.8
(−17.9%). `examples/task_sum.ail` now uses it. A two-variable `for` saves 0.2%
on the corpus and is rejected; pairs (`zip`, `enumerate`, `items`) go to
[OQ-124](../open-questions.md). `regex_codes` loses 17.4 tokens (127.0 to
109.6) by dropping the per-line loop and calling `regex.find_all` on the whole
text, with `strip_prefix("code=")?` in place of `else "0"`; lane 204 owns that
program and its library additions.

## Caveats

- Ten programs, one author, one run. The twins are idiomatic Python, not the
  shortest possible; a shorter twin would lower Python's count and widen AIL's gap.
- `gz_check` prints booleans, not sizes, because gzip bytes depend on the zlib
  build.
- Run time is not measured here. Only output equality with the twins was checked.
- Token counts are tokbench means. They are not per model, and the
  `claude-legacy` column is an older tokenizer.
# Agent round 4: standard-library parity with Python (lane 192)
Date: 2026-10-21. Twins in `tools/tokbench/corpus/agent-r4/` (`tNN_*.py` and
`tNN_*.ail`), measured with `tools/tokbench/run.sh` (mean over the five
tokenizers). All 25 planned tasks are written. Before the lane's additions the same tasks needed
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
| t11_json_transform | 57.6 | 107.0 | +86% |
| t12_sort_ties | 51.4 | 91.4 | +78% |
| t13_walk | 40.8 | 17.4 | -57% |
| t14_stdin_sum | 19.8 | 39.0 | +97% |
| t15_dates | 44.4 | 51.6 | +16% |
| t16_run_cmd | 40.8 | 39.6 | -3% |
| t17_http_json | 32.6 | 34.2 | +5% |
| t18_env_default | 22.0 | 21.2 | -4% |
| t19_kv_config | 63.6 | 96.2 | +51% |
| t20_grep_count | 42.2 | 62.6 | +48% |
| t21_digest | 27.6 | 23.4 | -15% |
| t22_dedup | 18.4 | 21.2 | +15% |
| t23_copy_tree | 14.4 | 27.4 | +90% |
| t24_report | 65.8 | 79.0 | +20% |
| t25_sleep_loop | 26.2 | 28.0 | +7% |
| **total** | 990.8 | 1215.8 | +23% |

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
## Findings from the second batch of twins (t11-t18)

- **Building JSON by hand (t11, +86%).** Reading with `as_arr`, `int(k)`, `str(k)`
  and writing through a derived `ToJson` struct still costs a struct
  declaration plus accessor fallbacks. `List[U].from_json(j)` is rejected
  (`List[...]` is a function, not a value), so the typed read path needs one
  `U.from_json` call per element. Candidate: let `List[T].from_json` work, or
  add `Json.to_list`. Recommendation: allow the generic associated call.
- **Sorting with ties (t12, +78%).** AIL has no tuples, so Python's
  `key=lambda p: (-p.age, p.name)` becomes two stable `sorted(by:)` calls
  (type.coll.11 guarantees stability) and a struct declaration. A `by:` key
  that is a list `[-p.age, p.name]` of ints would not type; recommendation:
  keep the two-pass idiom (already in the quick reference next to `counts`).
- **stdin (t14, +97%).** `io.read_all().lines()` plus `trim().to_int() else 0`
  is the whole cost; Python's `int(l)` raises. Not a library gap.
- **Where AIL wins:** `fs.glob` (t13, -57%), `flatten`, `chunks`, small
  logging.

## Findings from the last seven twins (t19-t25)

- **`key=value` lines (t19, +51%).** Splitting once at the first `=` takes
  `find`, `else continue` and two `slice` calls. A `partition`-style method needs
  a pair type; recommendation: `s.split_once(sep)` returning a two-element
  `List[str]?` is not worth a new idiom, so wait for tuples (open question)
  rather than add it.
- **Regex (t20, +48%).** `regex.compile(...)?` and `regex.is_match(re, l)` carry
  the module name twice per use; methods on `Regex` (`re.is_match(l)`) would cut
  ~6 tokens per call. Not done: it changes the std.regex surface (ADR needed).
- **Copying a tree (t23, +90%).** No recursive `fs.copy`; a loop over `fs.walk`
  with parent-directory creation is the agent's workaround. Candidate: `fs.copy`
  on a directory copies it recursively.
- **Formatting (t24, +20%).** Format specs cover Python's f-string cases
  except the percent suffix; the cost is one `* 100.0` and a literal `%`.
- **Wins:** digest (-15%), glob (-57%), flatten (-29%), batches (-18%).

## Added by this lane
`std.csv`, `std.args`, `std.log` (ADRs 0626-0628); list `chunks`, `windows`,
`counts`, `flatten`; string `center`, `words`, `trim_start`, `trim_end`,
`count`, `rfind` (ADR 0629). Language card 7,019 → 7,044 tokens (budget 8,000).

## Round 4 after (lane 204, ADR 0667)

Library fixes only. `regex.group_all` replaces the `find_all` + `strip_prefix`
pattern in `regex_codes`; `gz_check` uses the existing `str.byte_len()`.
Tokbench mean, five tokenizers, ten programs:

| Program | Before | After | Change |
| `regex_codes.ail` | 127.0 | 106.2 | -20.8 |
| `gz_check.ail` | 110.6 | 108.6 | -2.0 |
| other eight `.ail` and all ten `.py` | as above | unchanged | 0 |

`regex_codes` against its Python twin (94.2): +35% before, +13% after. The
|---|---|---|---|
| `regex_codes.ail` | 127.0 | 115.2 | -11.8 |
`regex_codes` against its Python twin (94.2): +35% before, +22% after. The
`task_sum` gap is a syntax question and stays open. The `frames` gap is
mostly `wire[at].to_int()` and is not changed here.

## Day 2: the 25 twins rerun with tonight's additions (lane 192)

The six rewritten twins use `Map.top`, `List[T].from_json`, regex methods,
recursive `fs.copy` and counting comprehensions (needs ADRs 0748, 0749, 0751).
AIL against Python goes from +22.7% to **+14.6%** (1,215.8 to 1,135.2 tokens
against 990.8). Nineteen twins are unchanged.

| task | Python | AIL before | AIL after | after vs Python |
|---|---|---|---|---|
| t01_wordcount | 42.4 | 57.0 | 43.6 | +3% |
| t02_batches | 32.6 | 26.8 | 26.8 | -18% |
| t03_table | 36.8 | 38.8 | 38.8 | +5% |
| t04_csv_report | 37.2 | 53.6 | 43.6 | +17% |
| t05_args | 71.0 | 95.4 | 95.4 | +34% |
| t06_log | 24.8 | 22.6 | 22.6 | -9% |
| t07_flatten | 25.2 | 18.0 | 18.0 | -29% |
| t08_whitespace | 30.2 | 38.0 | 38.0 | +26% |
| t09_group | 37.8 | 41.6 | 41.6 | +10% |
| t10_retry | 85.2 | 84.8 | 84.8 | -0% |
| t11_json_transform | 57.6 | 107.0 | 89.6 | +56% |
| t12_sort_ties | 51.4 | 91.4 | 91.4 | +78% |
| t13_walk | 40.8 | 17.4 | 17.4 | -57% |
| t14_stdin_sum | 19.8 | 39.0 | 26.4 | +33% |
| t15_dates | 44.4 | 51.6 | 51.6 | +16% |
| t16_run_cmd | 40.8 | 39.6 | 39.6 | -3% |
| t17_http_json | 32.6 | 34.2 | 34.2 | +5% |
| t18_env_default | 22.0 | 21.2 | 21.2 | -4% |
| t19_kv_config | 63.6 | 96.2 | 96.2 | +51% |
| t20_grep_count | 42.2 | 62.6 | 47.8 | +13% |
| t21_digest | 27.6 | 23.4 | 23.4 | -15% |
| t22_dedup | 18.4 | 21.2 | 21.2 | +15% |
| t23_copy_tree | 14.4 | 27.4 | 15.0 | +4% |
| t24_report | 65.8 | 79.0 | 79.0 | +20% |
| t25_sleep_loop | 26.2 | 28.0 | 28.0 | +7% |
| **total** | 990.8 | 1215.8 | 1135.2 | +14.6% |

## Where the remaining 144 tokens go, and what would close them

The 25 twins: AIL spends 1,135.2 tokens, Python 990.8, a net gap of 144.4. That
is 189.6 lost on 17 twins less 45.2 won on 8. The table ranks the losers by
tokens lost. "Saving" is measured on a sketch of the twin as it would be
written if the feature existed (`tools/tokbench/corpus/agent-r4/sketches/`);
the sketches do not run, so they are estimates of the program text only, not of
the feature's own cost (card lines, tests, ceremony elsewhere).

| # | twin | lost | root cause | candidate feature | sketch | saving |
|---|---|---|---|---|---|---|
| 1 | t12_sort_ties | 40.0 | no tuples: the data needs a `struct P` and two `sorted` passes | tuples with tuple sort keys (OQ-124) | `t12_tuple` 57.6 | **-33.8** |
| | | | | multi-key `sorted(by:, then:)` only, still a struct | `t12_keys` 89.8 | -1.6 |
| 2 | t19_kv_config | 32.6 | no split at the first `=` | `str.partition` (returns three parts, needs tuples) | `t19_tuple` 85.0 | -11.2 |
| | | | | `split(sep, limit)` (no tuples) | `t19_split2` 89.6 | -6.6 |
| 3 | t11_json_transform | 32.0 | typed JSON ceremony: `parse_json(read_text(..)?)?`, `from_json`, `to_json().to_text()` | `json.load[T](path)` and `json.print(v)` | `t11_load` 64.0 | -25.6 |
| 4 | t05_args | 24.4 | the option table spelled out call by call | `args.parse[Opts]` derived from a struct with doc-comment help | `t05_struct` 70.4 | -25.0 |
| 5 | t24_report | 13.2 | `keys().sorted()` then `sales[region]` for a pair | `items()`, needs tuples | `t24_items` 73.8 | -5.2 |
| 6 | t08_whitespace | 7.8 | method names (`trim_end`, `words`) cost more than `rstrip`, `split` | none (a rename is a tie under ADR 0076) | | 0 |
| 7 | t15_dates | 7.2 | `date.parse_iso(..)?` and `.add_ms(3 * date.DAY)` against `timedelta(days=3)` | none worth a feature | | 0 |
| 8 | t14, t04, t20 | 6.6, 6.4, 5.6 | `to_int() else 0`, `?`, `regex.compile(..)?`: AIL spells out the failure Python raises | none; the cost is the language's error model | | 0 |
| 9 | t03_table | 2.0 | rows are lists of strings, not `(str, int)` | tuples | `t03_tuple` 35.2 | -3.6 |
| 10 | the other seven | 11.8 together | none above 4 tokens | | | 0 |

### The tuple case in numbers (OQ-124)

Tuples, with `partition` and `items()`, would save **53.8 tokens** on these
twins: t12 -33.8, t19 -11.2, t24 -5.2, t03 -3.6. AIL would then spend 1,081.4
against Python's 990.8, **+9.1%** instead of +14.6%. t12 alone is 63% of that.
Without tuples, a multi-key `sorted` saves 1.6 tokens on t12, not 34: the loss
there is the struct the data must be written as, not the second sort pass.

Tuples are not the only way to the same place. Without them, `split(sep,
limit)` (-6.6), `json.load[T]` with `json.print` (-25.6) and a struct-derived
`args.parse` (-25.0) together save **57.2 tokens**, +8.8%. All of it together,
tuples and the other three, would be 1,030.8, **+4.0%**. So on this set a tuple
is worth about as much as the two ceremony features, not more, and the two
ceremony features are not a language-core change.

What the numbers do not say: the cost of tuples (card lines, a new type in the
checker, destructuring rules, `Json` and `to_json` over them, a sort-key rule)
and how often agents write tuple code outside these 25 tasks. Earlier evidence
(multi-key sorts +78% tokens, t12) is the same twin counted here.

### Order of value per feature, if all are wanted

1. `json.load[T]` / `json.print` (-25.6, one function pair, no language change).
2. `args.parse[Opts]` derived from a struct (-25.0, needs doc-comment access).
3. Tuples (-53.8 with `partition` and `items()`; a core-model decision).
4. `split(sep, limit)` (-6.6; cheaper than `partition`, and needs no tuples).

Nothing here is built; each needs its own ADR and a measurement on the
language-card cost before it lands.
