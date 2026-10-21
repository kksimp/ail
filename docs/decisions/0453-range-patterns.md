# 0453. Range patterns: `lo..hi` and `lo..=hi` over integer and character literals

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 3 — Expressions and control flow
- **Spec rules:** [expr.match.12](../spec/05-expressions.md)
- **Builds on:** [ADR 0138](0138-literal-and-or-patterns.md), [ADR 0452](0452-match-guards.md)

## Context

Grading, character classes and bucketing match a run of values. Today that
is a chain of `and`-joined comparisons or one literal arm per value.

## Options considered

### Option A — `lo..hi` exclusive and `lo..=hi` inclusive, as `for` ranges (chosen)
- **Pros:** the same two spellings as `expr.loop.3`, so nothing new to learn
  and the choice between them is the same one agents already make. Tokens,
  cl100k / claude-legacy / mistral-spm-v1: a three-bucket `grade` is
  44 / 40 / 52 against 61 / 52 / 72 for the comparison chain (-28%); a
  two-class `kind` over characters is 41 / 41 / 45 against 47 / 48 / 53
  (-13%).
- **Cons:** `..` adds a token after `|` alternatives; none measured.

### Option B — inclusive only, `lo..hi`
- **Cons:** disagrees with `for`, where `lo..hi` excludes `hi`. A reader would
  guess wrong in one place or the other.

### Option C — `lo...hi` or `lo to hi`
- **Cons:** a third spelling for the same idea.

## Decision

Option A, with literal ends only: both `int` (with an optional `-`) or both
`char`, matching the scrutinee. An empty range (`5..2`, `3..3`) is an error.
A range covers some values, never all, so a `match` using ranges needs a `_`
arm (`expr.match.2`), and ranges mark nothing as seen for `expr.match.5`.
Ranges may be alternatives of an or-pattern. Diagnostics cite
`expr.match.12`. The syntax tree gains `Pattern.Range`.

## Consequences

- Open-ended ranges (`10..`) and non-literal ends (constants) aren't designed.
- Overlap between ranges isn't reported; the first matching arm wins.

## Rejected alternatives

- Inclusive-only `lo..hi` (option B): rejected because it disagrees with `for`, where `lo..hi` excludes `hi`.
- `lo...hi` or `lo to hi` (option C): rejected as a third spelling for the same idea.
