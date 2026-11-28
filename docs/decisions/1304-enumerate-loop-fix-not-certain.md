# 1304. The `.enumerate()` loop fix of `type.tuple.5` is not certain

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), after the 2026-11-27 state review found a certain fix that produced a wrong program
- **Topic:** 22 — Tools
- **Spec rules:** [fix.auto.1](../spec/22-tools.md), [type.tuple.5](../spec/04-types.md)
- **Amends:** [ADR 1130](1130-certain-fixes-applied-by-check.md) (which fixes `ail check` applies)

## Context

`for a, b in xs` over a list of non-tuples is a `type.tuple.5` error. Its fix appends `.enumerate()`, and it was
certain, so plain `ail check` applied it. In the review an agent wrote `for k, v in top` where `top` was
`counts.top(3)`, a `List[str]` of keys, meaning key and count. The fix rewrote the loop to `top.enumerate()`,
`check` reported `ok`, and the program printed list positions where the counts should be. The error alone would have
said that `top` is a list of keys. A certain fix must have one meaning (`fix.auto.1`); this one picks between two
(index and element, or the pairs the author thought the list held).

## Decision

- The `.enumerate()` fix stays, with its help, and is not certain. `ail check`, `build`, `run` and `test` no longer
  apply it; `ail fix` and editors still offer and apply it on request.
- `fix.auto.1` names it among the fixes that guess.

## Consequences

- An agent that meant index and element now takes one more cycle; one that meant something else gets the error
  instead of a quiet wrong result. The review ranks the second as the hazard to avoid.
- The other certain fixes are to be audited the same way (state review, move 4).

## Rejected alternatives

- **Keep it certain when the loop body uses the first name as an `int`.** Still a guess about intent, and the
  body of the review's loop printed both names, which fits either meaning.
- **Remove the fix.** It is right most of the time; offering it costs nothing when it is not applied unasked.
