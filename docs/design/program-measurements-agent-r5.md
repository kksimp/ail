# Program measurements, agent round 5: programs that drive AIL's own tools

> **Non-normative design evidence,** 2026-10-21 (lane 224 of batch 2, night run).
> Twenty agent programs in AIL that run AIL's own tools (`ail check`, `explain`,
> `api`, `diff`, `edit`, `fix`, `fmt`, `test`, `effects`, `run --record`/`--replay`,
> Garlic, and the PDF reader) and process what they print. Each has a Python twin
> that does the same job and prints the same bytes. Earlier rounds:
> [round 4](program-measurements-agent-r4.md), [round 3](program-measurements-agent-r3.md).

## Layout

- The AIL programs are `examples/tool_*.ail`, with their expected output in `NAME.out`.
  They run in `./ail selftest` as examples; each finishes in well under a second.
- Fixtures are in [`tools/tokbench/corpus/agent-r5/data/`](../../tools/tokbench/corpus/agent-r5/data):
  broken and clean `.ail` files, a file that `fmt` rewrites, a test file with one
  wrong expectation, two module versions for `diff`, a package, five HTML pages
  (from `tools/garlic/testdata`), two PDFs (from `tools/pdf/testdata`), and the edit
  and rule lists.
- The Python twins are `tools/tokbench/corpus/programs/tool_*/tool_*.py`, next to
  the AIL copies (`tool_*.ail`, comments removed, as in round 4).

## Method

- Each pair prints the same bytes. The check was `cmp` against `NAME.out` for
  all twenty programs (run after the last change).
- Both sides run the same `ail` binary. A Python twin cannot reproduce `ail diff`
  or `ail check` byte for byte without reimplementing them, so it calls the tool
  through `subprocess` and does the processing in Python with `json` and plain
  string code. For Garlic and the PDF reader, the twin also runs the tool; the
  program measures the processing that follows.
- Output avoids anything that varies by machine: no absolute paths, and PDF and
  Garlic run with fixed widths and `--print` (no window).
- Token counts are tokbench means over the five offline tokenizers
  (`tools/tokbench/run.sh`), as in round 4.
- Reference treatment, so the comparison is fair: the card is
  `docs/language-card.md` (7044 tokens on this head) and the Python reference
  is `tools/tokbench/reference/python-reference.md` (6,813 tokens). "Both pay"
  counts each once, on top of the programs.
- Each Python twin has a short shared header: imports, the repository root, and
  only the helpers it uses (`ail()`, `parse()`, `lines_of()`). The first measurement
  used the full header on every twin, which added about 187 tokens to each twin.
  This document reports the trimmed twins. The full-header numbers were
  AIL 3,439 against Python 5,908 (−41.8%) alone, and −17.6% with both pay.

## Results: tokens

| Program | AIL | Python | AIL vs Python |
|---|---:|---:|---:|
| `api_outline` | 107.4 | 212.2 | -49% |
| `api_signatures` | 197.8 | 210.8 | -6% |
| `card_budgets` | 126.4 | 222.6 | -43% |
| `check_rules` | 206.0 | 325.0 | -37% |
| `check_summary` | 237.2 | 354.2 | -33% |
| `diff_lines` | 184.6 | 305.4 | -40% |
| `diff_summary` | 206.6 | 322.8 | -36% |
| `edit_preview` | 183.0 | 308.6 | -41% |
| `effects_table` | 221.0 | 308.8 | -28% |
| `explain_chapters` | 221.6 | 221.6 | +0% |
| `explain_rules` | 120.8 | 226.2 | -47% |
| `fix_preview` | 152.0 | 255.4 | -40% |
| `fmt_check` | 165.6 | 192.6 | -14% |
| `garlic_pages` | 205.2 | 291.0 | -29% |
| `pdf_grep` | 188.0 | 316.0 | -41% |
| `pdf_text` | 155.4 | 260.2 | -40% |
| `replay_check` | 189.8 | 291.6 | -35% |
| `test_coverage` | 109.6 | 208.8 | -48% |
| `test_summary` | 150.6 | 253.0 | -40% |
| `test_update_preview` | 161.0 | 253.2 | -36% |

Programs alone: AIL **3,490** against Python **5,340**, which is **−34.7%**.

Both pay (AIL card plus programs; Python reference plus programs): AIL 10,533
against Python 12,153, which is **−13.3%**. The programs alone save 1,850 tokens
and the both-pay saving is 1,620, because the AIL card is 231 tokens larger than
the Python reference.

Nineteen of twenty are cheaper in AIL. `explain_chapters` is a tie: 221.6 tokens
for each (the tokenizers disagree in detail, but the means match). Under about 2%
counts as a tie (ADR 0076).

## Results: time

Median of three runs per program on this machine. Most of each program's time is
spent starting `ail` and the tools it calls, so the two tiers differ only in noise
here. The bytecode tier is not what limits these programs.

| Program | Bytecode tier (s) | Tree-walker (s) | Python twin (s) |
|---|---:|---:|---:|
| `api_outline` | 0.032 | 0.031 | 0.054 |
| `api_signatures` | 0.056 | 0.050 | 0.073 |
| `card_budgets` | 0.070 | 0.076 | 0.108 |
| `check_rules` | 0.129 | 0.127 | 0.138 |
| `check_summary` | 0.122 | 0.117 | 0.142 |
| `diff_lines` | 0.065 | 0.064 | 0.086 |
| `diff_summary` | 0.063 | 0.060 | 0.091 |
| `edit_preview` | 0.031 | 0.032 | 0.060 |
| `effects_table` | 0.032 | 0.028 | 0.050 |
| `explain_chapters` | 0.097 | 0.105 | 0.053 |
| `explain_rules` | 0.128 | 0.152 | 0.161 |
| `fix_preview` | 0.045 | 0.047 | 0.076 |
| `fmt_check` | 0.115 | 0.121 | 0.144 |
| `garlic_pages` | 0.644 | 0.670 | 0.585 |
| `pdf_grep` | 0.296 | 0.327 | 0.331 |
| `pdf_text` | 0.310 | 0.351 | 0.339 |
| `replay_check` | 0.039 | 0.044 | 0.066 |
| `test_coverage` | 0.028 | 0.029 | 0.052 |
| `test_summary` | 0.028 | 0.028 | 0.047 |
| `test_update_preview` | 0.027 | 0.028 | 0.048 |

## The three biggest gaps

Ranked by how little AIL saves over Python, with the programs that show each one.
These are library or tool proposals. None needs a syntax change. Each needs a
measurement before it becomes an open question.

1. **Counting a key in a map** (`explain_chapters` 0%, `check_rules` −37%,
   `effects_table` −28%). Every tally is three lines: read the count with a default,
   add one, write it back. Python's `collections.Counter` and `most_common()` make it
   one line. Proposal: `counts.bump(key)` (creates the entry at 0 and adds 1) and
   `counts.top(n)` for the sorted pairs. Measure with `explain_chapters` first.

2. **Reading a field with a default from parsed JSON** (`api_signatures` −6%,
   `check_summary` −33%, `diff_summary` −36%). Each field is written as
   `d.str("kind") else ""` or `d.bool("summary") else false`, and the same
   `parse_json(line) else continue` opens each loop. Python's `.get(k) or ""` is
   shorter. Proposal: `j.text(k)` (`""` when missing), `j.flag(k)` (`false` when
   missing) and `parse_json_lines(text)` that skips lines that are not JSON.
   Measure with `api_signatures`.

3. **Combining a tool's output and reading its status** (`fmt_check` −14%,
   `garlic_pages` −29%, `test_coverage` −48%). A string cannot be joined with `+`,
   so `"{r.out}{r.err}"` is written every time a tool reports on stderr
   (`check --json`, `fix`, `test --coverage`). Proposal: `r.text` (stdout then stderr)
   and `r.ok` (status is zero). Measure with `fix_preview` and `test_coverage`.

## Caveats

- Twenty programs, one author, one run. The twins are idiomatic Python, not the
  shortest possible.
- Most of each program's cost is in the tool it calls, which is the same in both
  languages. The comparison is of the processing around the call.
- `explain_chapters.out` counts the spec rules, so it changes whenever a rule is
  added. When that happens the program's expected output must be regenerated in the
  same change. This is the cost of testing against the spec itself.
- `replay_check` writes `replay.rec` into the data directory and removes it.
- `garlic_pages` and `pdf_*` need the Garlic and PDF tools to run from the
  repository root.
- Time measures a single machine. Token counts are tokbench means and are not per model.


## Library additions (lane 234): before and after

The three proposals above are now in the language (ADRs 0732 and 0733):
`m.bump(k)` and `m.top(n)`, `j.text(key, default)`, `j.flag(key, default)`,
`parse_json_lines(text)`, and `r.ok()` and `r.text()` on a `proc.Output`. Each
program that uses a pattern they replace was rewritten with the helper, and its
expected output was checked: identical, except `check_rules`, whose ties now keep
insertion order (its twin was changed to match). Counts are tokbench, mean of five
tokenizers, the committed version against the rewritten one.

| Program | AIL before | AIL after | change |
|---|---:|---:|---:|
| `poller (r4)` | 234.0 | 234.4 | +0% |
| `runner (r4)` | 290.0 | 287.0 | -1% |
| `supervise (r4)` | 265.6 | 264.4 | -0% |
| `tasks (r4)` | 596.0 | 597.0 | +0% |
| `api_signatures` | 197.8 | 193.8 | -2% |
| `check_rules` | 206.0 | 188.4 | -9% |
| `check_summary` | 237.2 | 236.2 | -0% |
| `diff_lines` | 184.6 | 182.2 | -1% |
| `diff_summary` | 206.6 | 206.2 | -0% |
| `edit_preview` | 183.0 | 182.6 | -0% |
| `effects_table` | 221.0 | 205.8 | -7% |
| `explain_chapters` | 221.6 | 181.0 | -18% |
| `explain_rules` | 120.8 | 119.0 | -1% |
| `fix_preview` | 152.0 | 142.8 | -6% |
| `fmt_check` | 165.6 | 163.2 | -1% |
| `test_coverage` | 109.6 | 105.0 | -4% |

Sixteen programs changed: twelve round-5 tools and four round-4 examples. Their AIL
total is 3,591.4 before and 3,489.0 after (−2.9%); the twelve round-5 tools account
for −99.6 tokens. The largest saving is `explain_chapters`: 221.6 → 181.0, which
moves it from a tie with Python to 18% below it. `check_rules` drops by 9%,
`effects_table` by 7% and `fix_preview` by 6%.
The round-4 examples `poller` and `tasks` are a little larger: `j.text(k, default)`
is about as long as `j.str(k) else default`, so those two gain nothing.

Twenty programs, with the rewritten round-5 tools: AIL 3,390.0 against Python 5,340
(−36.5%, was −34.7%). Both pay, the language card (now 7,128 tokens, was 7,044, under
the 8,000 budget) counted once: AIL 10,518 against Python 12,153 (−13.5%, was −13.3%).
The new card lines cost about 85 tokens; the twelve program changes save about 100.
