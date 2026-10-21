# 0601. Native `par for`: the body becomes a chunk function

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 11 — Concurrency
- **Spec rules:** [conc.par.1–11](../spec/11-concurrency.md), conc.atomic.6, [native.task.1](../spec/23-native.md)
- **Builds on:** [ADR 0236](0236-automatic-parallel-loops.md), [ADR 0238](0238-parallel-runtime-model.md), [ADR 0599](0599-native-threads-and-task-pool.md)

## Decision

1. **The loop is outlined.** A `par for` over `lo..hi` (not inclusive) or a
   list variable of numbers, bools or strings, whose body the scan below
   accepts, is lowered twice: once as the sequential loop, once as a function
   `task.K(rec)` that runs iterations `lo..hi` of the same body over values it
   loads from a shared environment record. The body is lowered by the ordinary
   code, so a trap names the interpreter's rule, line and operands.
2. **What a body may do** (conc.par.8, checked on the syntax tree): write
   list elements through any index path; read anything; update an `int`
   variable declared outside by one `x += e` outside nested loops, with no other
   use (at most three of them); call free functions; update atomics. No
   `return`, `break` of the loop, `?`, `spawn`, `with`, `defer`, `unsafe`,
   `mut` argument or method that changes a captured list (`push`...). A body that
   declares a name an outer variable also has, uses a task handle, or has any
   other construct runs in order. That is the loop's meaning (conc.par.5), so
   the answer is the same, only slower.
3. **Lists are shared, scalars are copied.** The environment holds each
   captured value; a list is a pointer, so element writes reach the caller's
   list. The checker's assertion (conc.par.5) is that iterations write
   distinct elements.
4. **Chunks.** A loop with fewer than twice as many iterations as workers runs
   in order (conc.par.8). Otherwise it splits into four chunks per worker, queued
   in order; the loop finishes before the next statement (conc.par.7).
5. **Accumulators are exact.** Each chunk keeps its sum, and the lowest and
   highest running sum, starting from 0 (two compares per `+=`). The caller
   combines the chunks in order and checks that every running total stays in
   `int`: `total >= MIN - lowest` and `total <= MAX - highest`. A sum that would
   leave `int` anywhere, even when no chunk's own sum does, sends the loop to
   the in-order path, which traps where the sequential loop does (conc.par.4, conc.par.11).
6. **A trap in a chunk runs the loop again in order** (conc.par.11): the body
   has no effect but its element writes, which the second run repeats, so the
   result and the trap are the sequential ones. **A body that updates an atomic
   is not run again** (conc.atomic.6): iterations that ran can't be undone, so
   the first trapping chunk's trap is reported as it is.
7. **Hidden names:** `__plo`, `__phi`, `__plist`, `__lo`, `__hi`, `__k`, `NAME#min`,
   `NAME#max` can't clash with source names.

## Rejected alternatives

- **Capturing only what the body names, found by name resolution in the
  checker:** the lowering already has the variables in scope; scanning the
  body for names is shorter and, being conservative, safe.
- **Per-iteration addends in a list, summed in order afterwards:** exact, but
  8 bytes per iteration (800 MB for 10^8).
- **Letting a chunk's sum wrap and checking only the total:** misses the
  [MAX, 1, -1] case that the interpreter traps on (tested in `par_overflow`).
- **Running a nested `par for` on the pool from inside a chunk:** the workers
  are busy; the inner loop runs in order.

## Consequences

- collatz over 3,000,000 numbers: 1.19 s in order, 0.33 s with `par for` on
  four cores (3.6x), same output.
- A loop whose body is rejected by the scan loses nothing but speed.
