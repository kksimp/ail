# 1212. `std.textwrap`: display width, wrapping, indentation, aligned tables, natural order

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** std.textwrap.1-12 ([std-textwrap](../spec/std-textwrap.md))

## Context

The std gap plan (lane 7) found programs that wrap, indent, truncate and align text and that sort file
names by number, each hand-writing the same loops and getting terminal width wrong for CJK text and
colour codes. Python has `textwrap`, `str.ljust` and `natsort`; AIL has `pad_left`, which counts characters.

## Decision

A module written in AIL, `std/textwrap/textwrap.ail`, in scope without `use`: `char_width`, `width`,
`strip_ansi`, `truncate`, `pad_to_width`, `wrap`, `fill`, `dedent`, `indent`, `table`, `table_grid`,
`natural_key`, `natural_cmp`. Width counts display columns and treats ANSI escape sequences as 0.
Alignment is a character (`'l'`, `'r'`, `'c'`) and a string for tables (`"lr"`), a missing or unknown one
meaning left, so no call fails on it. `natural_key` returns a `str` whose byte order is the natural order, so
it works with `sorted(by:)` and `min(by:)` as they are: each digit run becomes a marker, a length character
and the digits without leading zeros. Names: `textwrap` (0 locals of that name in the corpus, examples and
ail-train, against 526 for `text`).

The module is pure AIL: no lowering, runtime or checker change, so it builds natively on every target.
Registered in `names.ail`, `load.ail`, `stduse.ail`, `lower_mod.ail` and `lower_err.ail`.

The width table is hand-curated (East Asian wide and fullwidth blocks, emoji presentation, combining and
zero-width marks), not generated: `compiler/rt/native/unicode.ail` holds case and class tables only, and
no Unicode data file was available in the lane. A generator under `tools/native/` is a follow-up.

## Rejected alternatives

- **An `Align` enum.** Type-safe, but `textwrap.Align.Right` is longer at every call site than `'r'` and the
  checker's behaviour for a std enum written through a module path was not measured. A mistaken letter
  aligns left, which shows at once.
- **`natural_cmp` only.** `sorted(by:)` takes a key, not a comparison; a comparison alone sends agents to
  write a sort.
- **Case-blind natural order by default.** Hides a choice; `natural_key(s.to_lower())` is one call.
- **Python's whitespace rules for `wrap` (no paragraphs).** Joins paragraphs into one block, which agents
  then repair; a blank line stays a paragraph break.
- **Grapheme clusters for width.** Needs the grapheme break tables; a per-character estimate is documented
  in std.textwrap.12.
- **Copying garlic's width code.** It is AGPL-3.0; this module is written fresh under Apache-2.0.

## Measurements

None yet: tokbench needs a hand-written twin for each helper. The module was written without a compiler
in the lane (builds run on the designer's machines, ADR 0842); its tests are `tests/cases/std_textwrap.ail`
and the spec examples, first run at the gate.
