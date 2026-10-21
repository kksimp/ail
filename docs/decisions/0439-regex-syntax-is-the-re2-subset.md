# 0439. `std.regex` syntax is the RE2 subset; Unicode classes are a documented subset

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.regex.3](../spec/std-regex.md), [std.regex.10](../spec/std-regex.md), [std.regex.11–22](../spec/std-regex.md)
- **Criterion that decided:** token usage and AI understanding (the syntax every model already knows, with errors that say what to change).

## Context

The syntax must be one agents already write correctly, and every failure
must say which character is wrong and how to repair it.

## Options considered

### Option A — the RE2 syntax subset (chosen)
- **Pros:** the most common Perl/Python spellings work unchanged; the
  grammar is small and unambiguous; the features RE2 leaves out are
  exactly the ones with unbounded cost.
- **Cons:** a few spellings differ from Perl (`\d` is ASCII).

### Option B — a new syntax (verbose, readable)
- **Cons:** every model would write it wrong; tokenizer cost is not better
  (ADR 0011: resemblance isn't a goal, but familiarity cuts mistakes).

### Option C — full Unicode tables for `\p{...}`
- **Pros:** exact categories.
- **Cons:** a generated table of tens of thousands of ranges, or a new
  data module, for rare use. Deferred.

## Decision

- Syntax: classes, POSIX classes, `\d \w \s` (ASCII), `\p{...}` (below),
  anchors, `\b`, greedy and lazy repetition with counts up to 1000,
  alternation, capturing, non-capturing and named groups (`(?P<n>)` and
  `(?<n>)`), and the flags `i`, `s`, `m`. The flag `U` and `x` are left out
  (`bad_flag`): neither is worth its tokens.
- Unicode classes: the general categories `L Lu Ll N Nd P S Z Zs Cc` (and
  long names), `Alphabetic`, `White_Space`, `Any`, and eleven scripts. They
  are built from the standard library's character properties and fixed
  ranges. Known approximations: `L` is the Alphabetic property without
  numbers; `Lu`/`Ll` are the Uppercase/Lowercase properties; `P`, `S`, and
  `Nd` outside ASCII use fixed ranges of the common blocks. The error for
  an unknown name lists the supported ones. Exact tables are a follow-up.
- Case folding is simple (one character to one), so `(?i)ß` does not match
  `SS`.
- Each failure is a `RegexError` with a `kind`, an offset and a rule
  (`std.regex.11–22`); `ail explain` has an entry for each, written as the
  repair.

## Consequences

- A later Unicode data module replaces the approximations without changing
  the API.

## Rejected alternatives

- A new regex syntax (option B): rejected because every model would write it wrong, and familiarity cuts mistakes (ADR 0011).
- Full Unicode tables for `\p{...}` (option C): deferred. It needs a generated table of tens of thousands of ranges, or a new data module, for rare use. The chosen form uses fixed ranges with documented approximations.
- Case folding that matches `ß` against `SS`: rejected by the decision. Case folding is simple, one character to one.
