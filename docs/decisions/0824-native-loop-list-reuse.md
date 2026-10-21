# 0824. Native: reuse lists that don't outlive a loop iteration

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.reuse.1](../spec/23-native.md)
- **Builds on:** [ADR 0391](0391-native-backend-optimisations.md) (MIR passes), [ADR 0577](0577-bounds-checks-in-counted-loops.md) (loops), [ADR 0579](0579-list-presizing.md) (sizing)

## Context

Callgrind on the natively built perfbench programs (head 1832f8f) put two
programs furthest behind C on allocation, not on code quality: fannkuch spent
115M of 274M instructions making `mut perm := []` every iteration
(`alloc_z`, `heap_pages_try`, `list_alloc`, `list_new_for`, `gc_sweep`), and
binarytrees 125M of 399M making fresh `left`/`right` lists per tree and
regrowing them (`list_grow`, `heap_pages_try`, `alloc_slow`). C reuses one
buffer in both. Mutable value semantics (own.val.1) already copies a list
whenever it is bound, assigned, stored or returned from a variable it
doesn't own, so the list made in the loop is rarely what gets kept.

## Decision

A MIR pass, `compiler/mir/reuse.ail`, runs on every non-runtime function
after bounds-check elimination:

1. **Escape analysis.** Values joined by `copy` or by a jump's arguments to a
   block's parameters are one group. A group escapes when one of its values
   is stored (`set`, `stx`, `store`), pushed as an element, returned, given to
   `icall`, `prim`, a runtime function outside a short list that keeps no
   reference (`push`, `copy`, `join`, `reserve`, `rows_min`), or given to a
   function whose parameter escapes there. Parameter summaries start as not
   escaping and are marked until nothing changes, so recursive helpers such as
   binarytrees' `make(mut left, mut right, d)` qualify.
2. **Reuse.** A `list_new` or `list_new_for` whose group doesn't escape and
   holds no loop header's parameter (nothing from an earlier iteration can
   reach it) is replaced, in the innermost loop that holds it, by emptying one
   list the loop's preheader makes: a `store` of 0 to the length word and a
   `copy`. Capacity is kept, so after the first iteration its pushes don't
   regrow; a loop guard on spare room (native.bce.6) still tests the real
   capacity, so correctness never depends on the size.

Instructions executed (callgrind, x86-64, perfbench sizes), before → after:
fannkuch 252M → 160M (−37%), binarytrees 334M → 256M (−23%) on batch head
after chain D (on the earlier head 1832f8f: 274.8M → 160.4M and 399.5M → 256.7M);
the other eight programs unchanged. Wall clock (median of 5): fannkuch 10 396 → 321 ms
(C 243), binarytrees 17 338 → 252 ms (C 135). Output matches the interpreter
on x86-64 and on aarch64 under qemu.

## Safety under the conservative collector

The collector (ADR 0564) marks from the stack, registers and globals, treats
every word of a root and of a live block's scanned range as a possible
pointer, and scans a list's elements only up to its length. So:

1. *No live object reaches the reused buffer.* The analysis proves no
   reference to the list is stored in a heap object, a global or a function
   value, or returned, so the only references are the SSA values of the
   group, which are dead once the iteration ends. Emptying the list therefore
   changes nothing any other object can observe.
2. *Stale elements are not roots.* After the length is set to 0, words
   beyond it stay in the buffer but the scan stops at the length, so they
   keep nothing alive; the elements pushed in the next iteration overwrite
   them before the length covers them. A `List[List[T]]` that is reused holds
   only copies the program made (own.val.1), so pushing into it never makes
   an inner list shared with a live object.
3. *A reused buffer is never freed early.* The preheader's list is a live
   stack value for the whole loop, so a collection during an iteration keeps
   it; there is no free to race with.
4. *Conservative misreads only lengthen lifetimes.* A stale word that looks
   like a pointer can keep a dead block until the next push overwrites it,
   as stale stack words already can (ADR 0564).

## Consequences

- A list made in a loop and read after the loop is the preheader's list; it
  stays alive for the whole loop, which only lowers peak memory.
- The pass relies on the lowering's copies (own.val.1). A future lowering
  that moves a list into a container without copying must show up as an
  escaping use (a `store`, `set`, or a call outside the list), which it does
  for every operation MIR has today.
- The `store` to a list's length word is the first MIR instruction that makes a
  list shorter. It only runs where no earlier iteration's list is live, so the
  bounds-check passes' "lists only grow" reasoning, which is per value, still
  holds; codegen that keeps a length in a register across a loop must treat a
  `store` to the list as a write.

## Rejected alternatives

- **A runtime `list_clear` call.** The runtime reaches a function only if the
  MIR calls it at lowering time (ADR 0393), and the pass runs after that; it
  would also cost a call per iteration. The `store` is one instruction.
- **Allocating lazily on the first iteration.** Saves one small allocation
  for loops that never run, but needs the list threaded through the loop as a
  block parameter, which makes it "carried" and defeats the analysis.
- **Stack allocation of non-escaping lists.** Lists grow, so the elements
  block can't live in a frame; the header alone is 24 bytes and isn't where
  the cost is.
- **Hoisting past the innermost loop.** binarytrees' lists would be made once
  instead of six times; not measurable, and the proof of no live alias is
  harder across several loops.
