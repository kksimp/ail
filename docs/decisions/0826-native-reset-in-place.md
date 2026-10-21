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

fasta, instructions before → after: 44.3M → 36.1M (−19%) on the batch head after chain K (70.2M → 59.3M earlier); the other nine
perfbench programs are unchanged.

## Why the old list is dead and nothing else holds either list

Let G be the group of values that may name the same list (copies, jump
arguments to block parameters). Three facts together make the reset
unobservable:

1. *Nothing outside G names either list.* G does not escape (ADR 0824): no
   member is stored in a heap object, passed to a function value, a
   primitive or a runtime function that might keep it, or passed to a
   function whose parameter escapes, and none is returned. A list read
   from a container or a field is a different value, not a member: binding
   or assigning from a place copies (own.val.1), so the maker's value never
   aliases a container's list. Aliasing through `ref T`, function values and
   tasks is excluded by construction: the instructions that would create it
   (`store`, `set`, `icall`, `prim`, `spawn`) make the group escape, and the
   native subset doesn't compile `ref` or tasks yet, so there is nothing to
   alias through today.
2. *The old list is dead at the maker.* No member but the new list is live
   after it (liveness over the group's values, around back edges), so no
   later instruction reads the old one by any name.
3. *The old list is the program's own.* Each other member is a copy, a maker
   or a non-entry block parameter. A function parameter is the caller's list,
   and a value from `get`, `load` or a call result isn't a maker, so either
   makes the group ineligible.

GC safety is ADR 0824's argument unchanged: the collector scans a list only up
to its length, so words left beyond the new length of 0 are not roots and are
overwritten before the length covers them; the reset list was already live
(it is a live stack value until the maker), so no collection can free it.

Tests: `reuse_reset.ail` (reset after use, kept by copy, read after the
reset point, caller's parameter, passed to a reading function) and
`reuse_reset_aliases.ail` (struct field, function value, list of lists,
struct assignment in the loop) match the interpreter on x86-64 and aarch64.

## Consequences

- The old list's elements stay in the buffer beyond the length; the
  collector scans a list only up to its length (ADR 0824's argument).
- Capacity kept means the capacity BCE guards test is the old one: a guard
  that wanted more room falls back to the checked loop, as for any list.

## Rejected alternatives

- **Last-use copy elision** (the originally planned ADR 0826). It measured
  about 0.8% of matmul (0.9M of 116M instructions) and nothing elsewhere in
  perfbench, and it is the riskiest change for value semantics, so it was
  dropped; revisit when a program shows the cost.
- **Only the pattern `x = []` at the AST level.** It would need lower.ail
  and would miss resets that reach a block parameter through another path.
- **Clearing at the loop head.** The old list may still be read after the
  point the program replaced it (`b := a; a = []`).
