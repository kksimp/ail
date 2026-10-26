# 0914. Run independent calls in a straight-line block as a fork-join, when each is worth it

- **Status:** Proposed
- **Date:** 2026-10-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); direction from the designer 2026-10-25
- **Topic:** 11 — Concurrency; 25 — Optimization strategy
- **Spec rules:** conc.par.1 (result and trap identical to sequential), conc.par.15 (new, below)
- **Builds on:** [ADR 0163](0163-inferred-effect-sets.md), [ADR 0236](0236-automatic-parallel-loops.md), [ADR 0238](0238-parallel-runtime-model.md), [ADR 0599](0599-native-threads-and-task-pool.md), [ADR 0601](0601-native-par-for.md), [ADR 0911](0911-default-automatic-parallel-loops.md)

## Context

ADR 0911 runs proven loops on the pool. The other common shape is straight-line
code: `a := f(x)`, `b := g(y)`, then `h(a, b)`, where `f` and `g` are pure and
expensive (a recursive `fib(n-1) + fib(n-2)`, two halves of a sort).
Mutable value semantics and effect sets make independence provable from the
signatures: the calls share no `mut` argument and neither has `io`, `unsafe`,
`spawn`, a capability or an atomic effect.

## Decision

1. **Candidates.** Two or more adjacent `name := call(...)` statements (or
   operands of one expression) in a function body, where every call is to a
   free function or method with the inferred effect set empty (no `io`, no
   `unsafe`, no `spawn`, no atomics, no capability), no argument is `mut`,
   and no later statement of the group reads a name an earlier one defines.
   Calls through closures or trait objects are not candidates.
2. **Lowering.** Calls 2..n become pooled tasks (ADR 0599); call 1 runs on the
   calling thread, which then waits for the rest (work-stealing helps while it
   waits). Results arrive in the order of the source.
3. **Cost model (conc.par.15).** A call is worth forking only when its callee
   is *heavy*: it contains a loop, or is recursive, or calls a heavy function.
   A recursive call forks only above a depth cut set by a depth argument the
   lowering adds (a hidden counter, forked while below `log2(workers)+2`), so
   `fib` does not create 10^9 tasks. Leaf functions without loops stay inline.
4. **Traps are the sequential ones (conc.par.1).** If any forked call traps,
   the group is run again in source order on the calling thread, as ADR 0601
   does for a chunk, so the first trap in source order is the one reported.
   Pure callees make the rerun unobservable.
5. **Switch.** Same as ADR 0911: `AIL_PARALLEL=0` and deterministic runs fork
   nothing.
6. **Stages.** (a) analysis and `ail effects --parallel` rows for candidate
   groups; (b) lowering behind the hidden depth counter; (c) benchmark on
   `fib` and merge sort; ship (b) only if (c) beats sequential on 4 cores.

## Rejected alternatives

- **Fork every pure call:** task cost (microseconds) dwarfs most calls.
- **A `fork` keyword only:** that is `spawn` plus `wait`, which exists.
- **Speculative execution of impure calls:** results would not be
  bit-identical on traps and effects.
- **Dynamic cutoff by queue length only:** non-deterministic task graphs
  make profiles and tests harder to read; a static depth cut is stable.

## Consequences

- Needs a purity/effect fact per function that the lowering can read (today
  the checker has it; the native lowering does not). Stage (a) adds it.
- Recursion depth counters add a hidden parameter, so recursive candidates
  get a second clone of the function. Cost: code size; revisit if large.
- Programs with deep recursion and traps keep their sequential trap.
