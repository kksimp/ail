# 0914. Run independent calls in a straight-line block as a fork-join, when each is worth it

- **Status:** Accepted (stage (a) built; stages (b) and (c) follow)
- **Date:** 2026-10-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); direction from the designer 2026-10-25; the plan approved by the integrator 2026-10-26 under ADR 0076 on these conditions: results and traps equal the sequential run; `AIL_PARALLEL=0` and `--deterministic` run the plain sequential clone; the stage-2 size, the self-build time and a fib/tree bench are reported against main
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

7. **Stage (b) as built: a live-task cap, not a depth clone.** The calls after
   the first become `spawn`-style tasks only when `rt.fork_ok` says fewer than
   twice as many tasks as workers are alive; otherwise the run executes in
   order. This replaces the hidden depth counter and second clone of point 3:
   it needs no extra function copy and no hidden parameter, and the cap bounds
   task creation the same way. Measured on fib(35): checking for room in every
   recursive call cost 147 ms against 50 ms sequential, so a self-recursive
   function also gets a `.seq` copy that calls itself and never checks. Only the
   shallow spine forks; in-order, rerun and task calls use the copy (fib(35):
   46 ms on 1 core, 19 ms on 3, 50 ms sequential). The task graph is not fixed from run to run,
   but results and traps are (point 4). Arguments may be a name, a literal or
   `name +- k`, whose overflow is checked first (a failed check runs in order),
   so evaluating them early cannot change which trap fires. Results of type
   `int`, `float` or `bool` only; anything else runs in order.

## Rejected alternatives

- **Fork every pure call:** task cost (microseconds) dwarfs most calls.
- **A `fork` keyword only:** that is `spawn` plus `wait`, which exists.
- **Speculative execution of impure calls:** results would not be
  bit-identical on traps and effects.
- **A static depth cut with a cloned function (the first plan):** doubles the
  code of every recursive candidate and adds a hidden parameter. Replaced by
  the live-task cap of point 7 because the cap is simpler and bounds tasks as well.

## Consequences

- Needs a purity/effect fact per function that the lowering can read (today
  the checker has it; the native lowering does not). Stage (a) adds it.
- Recursion depth counters add a hidden parameter, so recursive candidates
  get a second clone of the function. Cost: code size; revisit if large.
- Programs with deep recursion and traps keep their sequential trap.
