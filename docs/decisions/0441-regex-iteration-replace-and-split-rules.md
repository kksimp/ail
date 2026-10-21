# 0441. `find_all`, `replace` and `split` rules for empty matches

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.regex.6–8](../spec/std-regex.md)
- **Criterion that decided:** AI understanding: one rule for empty matches that produces no surprising extra pieces.

## Context

An empty match (`a*` on `b`) makes iteration ambiguous. Python 3.7+ reports
an empty match right after a non-empty one, giving `['', 'aaa', '', '']` for
`a*` on `baaac`; RE2-family engines (Go, Rust) drop the abutting empty match.

## Options considered

### Option A — drop an empty match that begins where the previous match ended (chosen)
- **Pros:** no doubled boundaries, `replace(a*, "-")` on `baaac` is `-b-c-`; the same rule as RE2-family engines, so agents' expectations from Go or Rust hold.
- **Cons:** differs from Python for patterns that can match empty.

### Option B — Python's rule
- **Cons:** `-b--c-` for the same replacement.

## Decision

Option A for `find_all` and so for `replace` and `split`. `split` follows Go's
`Split`: no leading piece for a match that is empty at the start, and no
trailing empty piece for an empty match at the end, so the empty pattern
splits `abc` into `a`, `b`, `c`. The replacement template is `$name`, `${name}`,
`$1`, `${1}`, `$0`, `$$` (the spelling of the Rust and Go libraries, shortest
for agents); an unknown group traps (`std.regex.7`) instead of silently
giving an empty string, because a typo there is a bug, not data.

## Consequences

- The vectors in `bootstrap/tests/regex/vectors.txt` fix these rules, and
  property tests (`std_regex_props`) check non-overlap and the identity
  `replace(re, t, "$0") == t`.

## Rejected alternatives

- Python's rule for empty matches (option B): rejected because `replace(a*, "-")` on `baaac` gives `-b--c-`, with doubled boundaries.
