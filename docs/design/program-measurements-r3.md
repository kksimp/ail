# Program measurements, round 3: closing the gap to Python

> **Non-normative design evidence,** 2026-10-20. Where AIL still costs more
> tokens than Python on whole programs, why, and the two changes adopted
> (ADRs 0182–0183) and the ones measured and kept (ADR 0184).

## Method

The corpus is the eight programs in `tools/tokbench/corpus/programs/`, each
with a Python twin that prints the same output: the seven from rounds 1–2 and
`tool`, the agent-style tool of ADR 0157, whose Python twin was added in this
round. The AIL files are the examples with comments, blank lines and `@test`
functions removed (`tools/tokbench/corpus_from_examples.py`); `inventory` has
no example and is unchanged. Counts are the mean of the five offline
tokenizers (`tools/tokbench/run.sh`).

Two scripts, both measurement only (ADR 0070):
- `tools/tokbench/attribute.py` deletes one construct class at a time from
  every program and reports the difference, for AIL and for Python.
- A per-function comparison (the same splitter, run ad hoc) put each AIL
  function next to its Python twin, which is how the comprehension gap was
  found: it isn't a construct class that can be deleted.

The "before" column re-measures round 2's examples through the same pipeline,
so it differs slightly from round 2's hand-made copies (`interp` 5,426.0
against 5,410.4, from `ail fmt` wrapping three long lines).

## Results

| Program | Python | AIL before | AIL/Py before | AIL after | AIL/Py after |
|---|---|---|---|---|---|
| commands | 677.2 | 523.8 | 0.773 | 522.4 | 0.771 |
| gradebook | 270.0 | 212.8 | 0.788 | 212.8 | 0.788 |
| interp | 5,565.2 | 5,426.0 | 0.975 | 5,399.8 | 0.970 |
| inventory | 325.8 | 284.8 | 0.874 | 284.8 | 0.874 |
| json | 3,233.2 | 3,412.2 | **1.055** | 3,312.2 | **1.024** |
| shapes | 569.2 | 444.2 | 0.780 | 444.2 | 0.780 |
| sheet | 4,674.0 | 4,774.0 | **1.021** | 4,654.4 | 0.996 |
| tool | 328.4 | 318.0 | 0.968 | 318.0 | 0.968 |
| **Total** | **15,643.0** | **15,395.8** | **0.984** | **15,148.6** | **0.968** |

AIL is now 3.2% below Python over the corpus (was 1.6%), and seven of the
eight programs are at or below their Python twins. `json` stays 2.4% above,
about the ADR 0076 tie threshold (see below).

Where the 247 tokens came from:

| Change | Tokens | Where |
|---|---|---|
| Comprehensions ([ADR 0182](../decisions/0182-comprehensions.md)) | −118 | `sheet` −75, `json` −29, `interp` −14 |
| Bare variant patterns ([ADR 0183](../decisions/0183-bare-variant-patterns.md)) | −85 | `sheet` −41, `json` −31, `interp` −13 |
| `json` rewritten with a map comprehension in place of `Stats.new`, and `Parser(...)` in place of `Parser.new` | −40 | `json` |

Interpreter speed (`tools/perfbench/run.sh 20`, two alternating runs against
the base branch, Linux x86-64): `interp` 0.197–0.201 s against 0.200–0.201 s
minimum, `json` 0.010–0.011 s against 0.010–0.011 s, `sheet` 0.020–0.021 s
against 0.022 s; unchanged within noise.

## Attribution

Tokens spent on each construct class, deleted one at a time (after the
changes; share of each language's total):

| AIL construct | Tokens | Share | | Python construct | Tokens | Share |
|---|---|---|---|---|---|---|
| Indentation | 1,284.4 | 8.5% | | Indentation | 1,231.0 | 7.9% |
| Newlines | 845.8 | 5.6% | | Newlines | 695.0 | 4.4% |
| Signature return types | 380.6 | 2.5% | | `self.` prefixes | 604.2 | 3.9% |
| Struct and variant field types | 257.2 | 1.7% | | `return` | 245.4 | 1.6% |
| `self` / `mut self` parameters | 232.8 | 1.5% | | `self` parameters | 176.2 | 1.1% |
| `fn` | 179.2 | 1.2% | | Block colons | 139.8 | 0.9% |
| Match arrows `=>` | 175.0 | 1.2% | | `def` | 133.0 | 0.9% |
| Method owner `T.` in `fn T.m` | 174.0 | 1.1% | | `import` lines | 128.6 | 0.8% |
| `return` | 155.6 | 1.0% | | `raise` | 60.0 | 0.4% |
| Signature parameter types | 135.2 | 0.9% | | | | |
| Named call arguments | 122.4 | 0.8% | | | | |
| `?` | 86.4 | 0.6% | | | | |
| `mut` (not on declarations) | 82.2 | 0.5% | | | | |
| `!` result markers | 58.2 | 0.4% | | | | |
| `mut` on declarations | 52.0 | 0.3% | | | | |
| `else` fallbacks | 44.0 | 0.3% | | | | |
| `use` lines | 13.6 | 0.1% | | | | |
| `:=` (against `=`) | 0.0 | 0.0% | | | | |

The top three AIL costs that Python doesn't pay are signature return types,
field types and the method header (`fn T.` plus `self`). AIL's implicit
fields and methods (ADR 0141) already save more than the method header costs:
Python's `self.` prefixes alone are 3.9%. `=>` is one token, as `:` or any
other separator would be, so it can't be cut. `:=` and `=` are each one token
on every tokenizer.

The per-function comparison found the costs that aren't a construct class:
- **Building a collection in a loop** (`mut out := []`, `for`, `push`)
  against Python's comprehensions: about 200 tokens in `sheet`. Fixed by
  ADR 0182.
- **Patterns that list every field as `_`** to stay exhaustive without a
  warned catch-all: 28 patterns. Fixed by ADR 0183.
- **Declared data shapes** in `json` (`enum Json`, `Member`, `Step`, and
  `Json.get`/`Json.at` where Python uses `isinstance` on native dicts and
  lists), about 190 tokens. This is the remaining `json` gap and the cost of
  static types; it isn't a target.
- **Error messages** written as `impl ToStr` matches (about 57 tokens in
  `sheet`, 12 in `json`): recorded as OQ-072.

## Candidates measured and kept

Applied by script to all eight programs after ADRs 0182–0183 (base 15,148.6);
decided in [ADR 0184](../decisions/0184-round-3-syntax-measurements.md):

| Change | Tokens | Change |
|---|---|---|
| `=` instead of `:=` | 15,148.6 | 0.00% |
| No `fn` keyword | 14,969.4 | −1.18% |
| Implicit read-only `self` | 15,069.8 | −0.52% |
| Implicit `self` and `mut self` (upper bound) | 14,915.8 | −1.54% |
| Return-type inference, keeping `!` | 14,872.8 | −1.82% |
| 2-space indentation | 15,128.0 | −0.14% |
| Tab indentation | 15,751.8 | +3.98% |
| One-line `use std.env, fs, proc` | 15,143.4 | −0.03% |
| No `use std.x` lines | 15,135.0 | −0.09% (−4.3% of `tool`) |

All are ties under ADR 0076, and each loses on AI understanding (explicit
signatures and receivers, `fn` as a search anchor), so none was adopted. The
`use` lines matter only for small tools and are left to the user-modules work
as OQ-071.
