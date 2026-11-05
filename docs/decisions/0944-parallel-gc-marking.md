# 0944. Native: parallel marking by the parked threads on large heaps

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none (no observable behaviour changes)
- **Builds on:** [ADR 0562](0562-native-heap-size-classes-and-header.md), [ADR 0564](0564-collection-trigger-and-sweep.md) (collector), [ADR 0599](0599-native-threads-and-task-pool.md) (stopping the world), [ADR 0943](0943-per-thread-allocation-caches.md) (per-thread caches)

## Context

The collector is stop-the-world mark and sweep on one thread. While it marks,
every other thread is parked and idle. Programs that reach the pool (Clove,
Garlic, Husk) hold heaps of hundreds of MB, so the pause is long and the
cores sit unused.

## Decision

1. After the collector has marked the roots (its stack, the parked threads'
   published stacks and registers, the pool block, the globals), and when the
   process has workers and the heap spans 32 MB or more, the parked threads
   help drain the mark stack (`gc_par.ail`). Otherwise marking stays on the
   collector, as before.
2. The collector sets `GC_REQ` to 2 and wakes the threads parked on it and
   the idle workers (the pool's `seq` word). A thread leaving a wait while
   `GC_REQ` is 2 joins; it is already parked and scanned, so it runs only
   marking code.
3. Each marker takes a share of the shared stack (a quarter, 64 to 16384
   entries) under a spin lock into its own stack (32768 entries, in its
   allocation cache block), scans from it, and gives the top half back when
   its own stack is full or holds 256 entries while the shared one is empty.
   Marking is done when the shared stack is empty and no marker holds taken
   work; the count of busy markers is raised under the lock with the take.
4. Two markers may mark one object at once. Both set the same bit and both
   queue it, which only repeats work; a mark bit is never cleared during
   marking, so nothing is missed.
5. The collector sets `GC_REQ` back to 1, waits until every helper has left,
   then sweeps alone. `AIL_GC_VERIFY` and deterministic scheduling keep
   marking on one thread.

## Data races (ADR 0025)

The shared stack is read and written under the spin lock; its count is also
stored atomically for the racy "is it empty" test, which only decides when to
share. The busy count, the done flag and the joined count are atomic. Header
words are written only by markers during the phase, and only to set the mark
bit, so concurrent writes store the same value. The mutator is stopped, so no
safe code runs during the phase.

## Measured

Pending: collection pause and total time on ail-linux and the Mac through the
runner, with the pool active and with `AIL_PARALLEL=0`, on a heap of a few
hundred MB (Clove max mode, `tests/native/run/gc_par_cache.ail` scaled up).

## Rejected alternatives

- **Dedicated GC threads.** More threads than cores while the world is
  stopped would only compete with the parked ones; the parked workers are
  the cores.
- **Concurrent marking** (mutators run while marking). Needs write barriers
  in compiled code on both ISAs and changes the cost of every store; the
  pause shrink from parallel marking needs neither.
- **Parallel sweep in the same change.** The sweep rebuilds the shared free
  lists in address order; splitting it by page range needs per-range lists
  joined at the end. A follow-up once marking is measured.
- **Atomic test-and-set of the mark bit.** Avoids the repeated scan of an
  object marked twice, at a locked instruction per object marked; the race
  is rare and only costs work.
- **Marking in parallel on every heap.** Waking the helpers costs tens of
  microseconds; below 32 MB marking takes about as long.

## Consequences

The marking pause on a large heap falls by up to the number of threads in the
pool. Programs that never start the pool are unchanged.
