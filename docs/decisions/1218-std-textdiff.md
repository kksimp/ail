# 1218. `std.textdiff`: line diffs, unified patches and edit distance

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); part of the designer's std library push (2026-11-26)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.textdiff.1 to std.textdiff.10](../spec/std-textdiff.md)

## Context

The corpus tasks `diff_lines` and `sparse_text_diff` and the ail-train task `line_diff` show agents hand-writing a
line diff; `ail-edit` carries its own. Agents also reach for `difflib` (`unified_diff`, `ratio`) and for Levenshtein
distance, and no std module has either.

## Decision

- A new module written in AIL, `std.textdiff` (`std/textdiff/textdiff.ail`), pure, no capability.
- `lines(a, b) List[Edit]`: Myers' O(ND) shortest script. `Edit` is a struct with `kind` (`Keep`, `Add`, `Remove`),
  `text`, 1-based `a` and `b` line numbers (0 where the side has no line) and `nl`. Within a run of changes the removes
  come first, so the output reads as `diff` writes it.
- `unified(a, b, name_a, name_b, context) str` writes `diff -u` text without timestamps; the expected output of the
  tests was recorded from GNU diff. `apply(text, patch) str ! PatchError` reads it back (and GNU output), strictly: no
  offset search and no fuzz. `PatchError` carries a `kind` enum (`Malformed`, `Mismatch`), `why` and the hunk number.
- A line is compared with its line break, so a last line with no `\n` differs from the same text with one, as in `diff`.
- `stat(a, b) (int, int)` is (added, removed). `distance` is Levenshtein by characters; `similarity` is one minus the
  distance over the longer length.
- The module is named `textdiff`: `diff` is a local name in 45 programs of the corpus, examples and ail-train, and a
  local hides a module (std.use.4).
- No syntax change; no new built-in.

## Rejected alternatives

- **`std.diff`.** Measured clash with 45 local names.
- **Enum edits (`Keep(text)`, `Add(text)`, `Remove(text)`).** A match on three variants is longer than reading `e.kind`
  and `e.text`, and the line numbers would repeat in every variant.
- **Fuzzy and offset patch application (like `patch`).** A patch that applies in the wrong place silently is the worst
  failure; an agent can regenerate a patch. Strict now, a `fuzz` option later if a task needs it.
- **Linear-space Myers (middle snake).** More code and more risk for a case (a very large number of changes) that this
  module does not target; the trace costs room proportional to D squared, stated in std.textdiff.10.
- **A patience or histogram diff.** Shortest is the one property a test can state; `diff -u` itself defaults to it.
- **Byte distance.** Strings are characters in AIL (`s.len()`).

## Consequences

Each public function has a test in `tests/cases/std_textdiff_*.ail`. The module registers in the five lists of the
std-gap plan (names, load, stduse, lower_mod, lower_err). `ail ask std "diff"` finds it through `std-synonyms.txt`.
