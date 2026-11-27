# 1268. `std.textdiff` gains a word diff and a patience line diff

- **Status:** Accepted (number to be allocated by the integrator)
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Standard library, text diffs
- **Spec rules:** [std.textdiff.11-12](../spec/std-textdiff.md)
- **Amends:** [ADR 1218](1218-std-textdiff.md) (line diff, `diff -u`, apply, distance already shipped)

## Context

The std-gap brief asked for a line and word diff with a patience option, `diff -u` output and strict patch apply.
`std.textdiff` (ADR 1218) already has the Myers line diff, `unified`, `apply`, `stat`, `distance` and `similarity`.
Missing: a word diff and a patience diff.

## Decision

Add `words(a, b)` and `patience(a, b)`, both giving `List[Edit]`. `words` diffs maximal runs of non-space and of
whitespace characters (Myers), so both texts can be rebuilt. `patience` anchors on lines unique to both sides (longest
ordered run, patience sorting), recurses between anchors and falls back to Myers when none exist. Pure AIL, no runtime
or checker change.

## Rejected alternatives

- A `patience` flag or mode argument on `lines`: a bool an agent must remember, and `lines` keeps one meaning.
- Word diff by splitting on whitespace and dropping it: the texts could not be rebuilt.
- Histogram diff: not asked for.
- A `unified` variant that uses patience: left for a later request; `unified` stays shortest.
