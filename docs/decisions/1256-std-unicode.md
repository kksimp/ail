# 1256. `std.unicode`: normalization, case folding, mark stripping, grapheme clusters

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** std.unicode.1-8 ([std-unicode](../spec/std-unicode.md))

## Context

The std gap plan (wave 3) lists Unicode normalization, case folding and grapheme clusters. `str` works on
code points (type.str.1): `"e\u{301}" == "\u{e9}"` is false, `"Straße".to_lower()` keeps `ß`, and
`"🇫🇷".len()` is 2. Programs that compare names, key maps by user text, slugify, or cut text for display
need the Unicode algorithms, and hand-written versions get them wrong. Python has `unicodedata.normalize`
and `str.casefold`, Swift compares strings by canonical equivalence and counts `Character`s, Go has
`x/text/unicode/norm` and `strings.EqualFold`.

## Decision

A module written in AIL, `std/unicode/unicode.ail`, in scope without `use`: `version`, `nfc`, `nfd`,
`nfkc`, `nfkd`, `is_nfc`, `is_nfd`, `is_nfkc`, `is_nfkd`, `casefold`, `caseless_eq`, `strip_marks`,
`graphemes`, `grapheme_count`. Every function is pure and total.

- **Data.** Unicode 16.0.0. `tools/native/gen_unicode_text.py` reads eight UCD files (fetched and
  hash-checked by `tools/native/unicode-text/run.sh`) and writes eight string constants between
  `## BEGIN GENERATED TABLES` and `## END GENERATED TABLES`: combining classes, full canonical and
  compatibility decompositions (expanded, so one lookup per character), primary composites, case foldings
  (C and F), grapheme break classes with Extended_Pictographic merged in, InCB classes, and Mn ranges.
  Numbers are base-45 digits over printable ASCII, differences from the previous entry. 52 KB of text in all.
  The module decodes them in its constants, so the work happens at compile time (ct.const.1) and the start-up
  image holds the decoded lists (ADR 1191); lookups are binary searches. Hangul syllables are handled by
  arithmetic, not tables.
- **Checked before writing.** The generator decodes its own tables and runs a Python copy of the module's
  algorithms over every line of `NormalizationTest.txt` (12 checks a line) and `GraphemeBreakTest.txt`; it
  refuses to write if one fails. All pass. `tests/cases/std_unicode_conformance.ail` (generated) runs 1,461
  normalization lines (parts 0, 3, 5 whole, every 4th of parts 2 and 4, every 32nd of part 1) and all 1,093
  grapheme lines through the AIL module, so the gate checks the AIL code against the standard's data.
- **Fast paths.** Text entirely below U+00C0 (NFD), U+00A0 (NFKD, NFKC) or U+0300 (NFC) is returned as it
  is; the generator asserts the tables allow it. ASCII skips every table in `casefold` and `graphemes`.
- **Names.** `unicode` (no local of that name in the compiler, std, examples or tests). `casefold`, not
  `fold`, which names the collection reduce. `caseless_eq` after the standard's "caseless match" (D145),
  with `eq` as in `assert_eq`. `strip_marks` says what it removes (Mn), where "remove accents" would promise
  `ø` to `o`.
- **Version.** 16.0.0, the version nearest the runtime's character tables (`compiler/rt/native/unicode.ail`,
  read off the Rust interpreter), so `c.is_alpha()` and this module agree on assigned characters.
  Moving both to a newer version is one generator run each.

## Rejected alternatives

- **Methods on `str` (`s.nfc()`, `s.graphemes()`).** Shorter, but needs `check/builtins.ail` (one lane at a
  time), native lowering and runtime tables in every target, against the plan's rule that pure text goes in
  AIL. A later PR can add method aliases that call the module.
- **`normalize(s, Form.NFC)`.** One function instead of four, but longer at each call, and a std enum through a
  module path is unmeasured (the reason ADR 1223 gave). Four names match what agents type (`nfc` is the term).
- **Python's `unicodedata` as the source.** Ties the tables to the Python on the machine (15.1.0 in the lane)
  and has no grapheme break or InCB data. The UCD files are the source; the Python copy only cross-checks.
- **Tables in the native runtime.** Faster lookups are possible with two-level arrays, but the gain is in a
  binary search over a few thousand entries, and runtime tables need lowering, a checker row and Windows
  parity. If measurement shows a need, the same generator can write a two-level table.
- **Grapheme clusters by the legacy rules, or without GB9c.** Unicode 15.1 added GB9c (Indic conjunct
  clusters) to the extended rules; leaving it out splits Devanagari conjuncts that readers see as one.
- **Copying garlic's `unicode.ail`.** It is AGPL-3.0; this module is written fresh under Apache-2.0.

## Measurements

None yet on the gate: the lane wrote the module without a compiler (builds run on the designer's machines,
ADR 0842). Its first compile is the gate, through `tests/cases/std_unicode.ail`, the conformance test and the
spec examples. Compile-time cost to measure there: decoding 52 KB of tables in the constants.
