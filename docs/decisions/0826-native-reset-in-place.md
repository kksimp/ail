# 0826. Native: reset a list in place

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.reuse.2](../spec/23-native.md)
- **Builds on:** [ADR 0824](0824-native-loop-list-reuse.md) (escape analysis, list reuse)

## Context

ADR 0824 leaves out a list that is carried from one iteration to the next.
The common case is a buffer that is reset: fasta's `line = []` after each
printed line made a new 4-element list that regrew to 64 every 60 bases (about
10M of its 70M instructions), and line and token accumulators in agent code
are shaped the same way.

## Decision

`compiler/mir/reuse.ail` also handles a maker in a loop whose group is
carried. The new list reuses an old member of its group when:

1. the group doesn't escape (ADR 0824's analysis);
2. every other member is a copy, a maker, or a parameter of a block that
   isn't the entry (a function parameter is the caller's list);
3. a member is defined before the maker on every path (dominance); and
4. no member but the new list is read after the maker, by a liveness
   analysis restricted to the group's values.

The maker becomes a store of 0 to the old list's length and a `copy` of it.
Capacity is kept, so a buffer that reached its working size never regrows.

fasta, instructions before → after: 70.2M → 59.3M (−15%); the other nine
perfbench programs are unchanged.

## Consequences

- The old list's elements stay in the buffer beyond the length; the
  collector scans a list only up to its length (ADR 0824's argument).
- Capacity kept means the capacity BCE guards test is the old one: a guard
  that wanted more room falls back to the checked loop, as for any list.

## Rejected alternatives

- **Last-use copy elision** (the originally planned ADR 0826). In the
  perfbench programs the copies are 0.9M of 116M instructions, and it is the
  riskiest change for value semantics; revisit when a program shows the cost.
- **Only the pattern `x = []` at the AST level.** It would need lower.ail
  and would miss resets that reach a block parameter through another path.
- **Clearing at the loop head.** The old list may still be read after the
  point the program replaced it (`b := a; a = []`).
