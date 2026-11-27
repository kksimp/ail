# 1296. `std.unicode` layout: display width, line breaking and word segmentation

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** std.unicode.9-12 ([std-unicode](../spec/std-unicode.md)); amends std.textwrap.1, 2 and 12
  ([std-textwrap](../spec/std-textwrap.md), [ADR 1223](1223-std-textwrap.md))
- **Builds on:** the `std.unicode` ADR of the same wave (ADR 1256), whose generator and
  grapheme clusters this extends.

## Context

`std.textwrap` (ADR 1223) measures text in terminal columns with a hand-curated table of wide and zero-width
ranges, and counts a flag, an emoji `ZWJ` sequence or `❤️` as the sum of its characters. Nothing in std finds the
places where text may break across lines or splits it into words by the Unicode rules; programs that wrap or
count words write `s.split(" ")` and get Chinese, hyphenated words, `can't` and `3.14` wrong. Python has
`wcwidth` and `regex \b`, Go `x/text/width` and `golang.org/x/text/unicode/...`, Swift `NSString.enumerateSubstrings`
and `String.count` by `Character`, Rust `unicode-width`, `unicode-linebreak` and `unicode-segmentation`.

## Decision

Four additions to `std.unicode` (no new module, so no new registration and one generator): `char_width`,
`east_asian_width`, `width`, `line_segments`, `word_segments`, `words`. `std.textwrap` stops carrying its own
tables and calls `unicode.char_width` and `unicode.width`.

- **Width (std.unicode.9, 10).** East Asian Width W and F are 2. Zero: controls, Mn, Me, Cf (not the soft
  hyphen), Zl, Zp, and the Hangul medial vowels and final consonants. Everything else is 1, ambiguous
  included, as terminals that are not configured for CJK draw it. `width` counts by grapheme cluster: the
  widest character, 2 for a flag, 2 for a character with the Emoji property before U+FE0F, 1 for an Indic
  conjunct. Escape sequences are not handled here (`std.textwrap.width` strips them first).
- **Line breaking (std.unicode.11).** UAX #14, rules LB2 to LB31 of Unicode 16.0, default behaviour, returned as
  the pieces after which a line may break (`List[str]`, like `graphemes`), not as an index list: pieces
  compose with `join`, `len` and the textwrap loop with no index arithmetic.
- **Word segmentation (std.unicode.12).** UAX #29 word boundaries WB3 to WB999. `words` keeps the segments that
  hold a letter or digit, which is what callers who ask for "words" want; `word_segments` keeps everything.
  No dictionary: Han is cut per character, Thai by class.
- **Data.** `tools/native/gen_unicode_text.py` gains five tables (East Asian Width, zero width, Emoji,
  Line_Break with rule LB1 resolved and Pi/Pf quotation marks and unassigned pictographs split out, Word_Break
  with Extended_Pictographic merged in) and the class constants, in the same generated block: about 12 KB of
  text on top of the 52 KB. `run.sh` pins five more UCD files by SHA-256. Defaults for unassigned code points
  (ID in the CJK blocks and the emoji planes, PR in Currency Symbols, W in the CJK blocks for width) are applied
  by the generator, so the module has no default logic.
- **Checked before writing.** The generator runs a Python copy of the algorithms over every line of
  `LineBreakTest.txt` (16,672) and `WordBreakTest.txt` (1,826), with the existing normalization and grapheme
  files, and refuses to write on a mismatch; all pass. The AIL text was then run through a throwaway
  line-by-line transpiler to Python against the same four files (not committed): all pass, so the
  transcription from the Python copy is correct, though that does not exercise the AIL checker or backend.
  `tests/cases/std_unicode_segments.ail` (generated, 5,558 line break lines and all 1,826 word lines) and
  `tests/cases/std_unicode_layout.ail` (hand-written API cases) run through the real module at the gate.
- **textwrap.** `char_width`, `width`, `truncate`, `wrap` (a word wider than the limit) and the table
  functions use `unicode.char_width` and clusters, never cutting inside one. `wrap` still breaks only at
  whitespace; wrapping by `line_segments` changes its contract (runs of whitespace become one space) and is a
  follow-up.

## Rejected alternatives

- **A separate module (`std.textlayout`, `std.uniseg`).** Needs registration in five compiler files and a
  second copy of the grapheme and EAW lookups; the data and algorithms share the tables of `std.unicode`.
- **Width as the sum of `char_width`.** Simpler, and what `wcwidth` does, but counts a flag as 2, a ZWJ family
  as 6 and a letter with accents correctly while over-counting Indic conjuncts, and disagrees with terminals
  that segment by cluster. Clusters are what `std.unicode.8` already computes.
- **Treating ambiguous width as 2.** Right in a CJK locale, wrong everywhere else, and there is no locale in
  `std`. A caller that wants it can test `east_asian_width(c) == "A"`.
- **Break and word indexes (`List[int]`).** Cheap to produce, but every caller then slices; pieces are what the
  callers wrote by hand. Indexes can be added if a measurement shows the allocation matters.
- **`words` as the only function, or splitting on `\b`.** Loses the punctuation and spaces that a tokenizer,
  a highlighter and a diff need; both functions cost one loop.
- **Tailoring (LB, WB) and dictionaries.** Not in the algorithm's default and not testable against the
  standard's files; a later change can add a `Tailoring` argument.
- **Copying garlic's or another project's tables.** AGPL or licence-incompatible; the tables here are written
  from the UCD files under Apache-2.0.

## Measurements

None yet on the gate (builds run on the designer's machines, ADR 0842; the lane has no compiler). Cost to
measure: compile-time decoding of 72 KB of tables, `width` and `line_segments` on a 1 MB mixed text.
