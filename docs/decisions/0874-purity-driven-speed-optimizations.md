# 0874. Purity-driven speed optimizations: memoization, pure-call CSE and hoisting, merging for speed

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, option 1, "only do the things that'll increase speed")
- **Topic:** 23 — Implementation (native optimizer)

## Context

The designer asked whether the idea behind zip compression (one copy of repeated bytes)
could help the optimizer. AIL's effect system proves which functions are pure, which C
compilers mostly can't, so repeated work can be removed safely. The designer chose speed
only: binary size isn't a goal now.

## Decision

After Rust removal, the native optimizer (compiler/mir) gains three passes, all driven by
effect-proven purity:

1. **Automatic memoization** of pure calls repeated with the same arguments, only where a
   cost model says it pays (expensive callee, cheap-to-hash arguments, repeats observed in
   the call graph or by profile), with a bounded cache. A program without such calls pays
   nothing.
2. **Pure-call CSE, GVN and loop hoisting:** a pure call with the same arguments is computed
   once in a function, and a pure call whose arguments don't change in a loop moves out of
   the loop.
3. **Merging** duplicate pure functions and identical generic instantiations, only where it
   speeds builds or improves instruction-cache use. It never runs as a size-only pass.

Every pass keeps the program's observable behaviour, traps included, and is measured with
`tools/perfbench` before it lands.

## Rejected alternatives

- **Outlining repeated instruction runs for size:** adds a jump per use and costs speed; size
  isn't a goal now.
- **Memoize every pure call:** caches cost memory and hashing time, and slow down cheap calls.
