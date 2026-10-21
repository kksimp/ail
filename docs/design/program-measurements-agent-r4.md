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
