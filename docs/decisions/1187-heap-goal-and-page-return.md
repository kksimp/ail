# 1187. The heap sizes itself and gives memory back

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); asked for by the designer as edge idea 1. No syntax and no new setting
- **Topic:** Native heap and collector
- **Spec rules:** [mem.gc.5](../spec/06-memory.md)
- **Amends:** [ADR 0564](0564-collection-trigger-and-sweep.md) (the collection trigger)

## Context

The collector ran by two fixed numbers: no collection before 32 MiB had been allocated, and the next after three
times the live bytes. A program that needs 8 MB sat at 48 MB, and a heap peaked near four times its live data. No
page ever went back to the system. In the hands-on trial (`opus-ail-trial.md`, 2026-11-25) memory was 4 to 6 times
the C twin's, the largest gap against C the trial found.

## Decision

The trigger is a goal the runtime adapts (`compiler/rt/native/heap_goal.ail`):

- **Goal.** The next collection comes after `live x slack` bytes, at least a floor. The slack starts at 1 and the
  floor at 4 MiB, so the heap peaks near twice the live data.
- **Guard.** Each collection is timed. While collections take more than 6% of the run time of the last few cycles
  (each cycle counts half as much as the one after it), the slack grows by a half and the floor doubles, to at most
  3 and 32 MiB: the old fixed values. Under 3% the slack falls by a quarter; under 0.75% the floor halves.
- **Page return.** After a sweep, the lowest free pages are kept for the next cycle (the goal, a quarter more and
  16 MiB, and never under three quarters of what the last collection kept). Free runs of 8 pages or more above them are
  returned: `madvise` with `MADV_FREE_REUSABLE` on macOS (and `MADV_FREE_REUSE` when a page is taken again, so the
  system's count stays right), `MADV_DONTNEED` on Linux. A byte per page (`H_PMARK`, value 2) records it.
- **Windows** keeps its pages for now: the goal applies there, the return does not.

## Options considered

- **Lower the two fixed numbers (2 times live, 4 MiB):** rejected. Programs that allocate fast with a large live
  set would pay for the extra collections with no way back.
- **A setting such as Go's `GOGC`:** rejected. "No settings" is the claim; the guard does what a user would tune.
- **Widen by how little a collection frees:** rejected for now. It saves marking while a heap only grows, but the
  first cycle after growth stops overshoots at the peak, which is the number users see.
- **Return every free page after every sweep:** rejected. The next cycle takes them again and pays a page fault
  for each.
- **Return pages from a timer thread:** rejected for now; a program that stops allocating keeps what it had at its
  last collection. Follow-up if a server shows it.
- **A moving or generational collector:** out of scope (roots are conservative, ADR 0562).

## Rationale

ADR 0076 order: nothing changes for the program or its diagnostics, and nothing for the reader. Speed and memory
decide, and the guard bounds the cost in time: a program that collects a lot ends at the old values.

## Consequences

- `heap_goal.ail` is new; `heap_init` and `heap_pages_try` (heap.ail), `gc_collect` (gc.ail) and `gc_thresh`
  (leak.ail) each gain a call. The allocation fast paths, marking and the sweep are unchanged.
- Small heaps collect more often: the first collection is at 4 MiB, not 32 MiB.
- A free page's contents may be discarded; nothing read them before (a reused page is zeroed or filled).
- Test `gc_give_back`. Job `heap-goal/ab-job.sh` in the project library measures time and peak memory against
  the main this branch merged; the results are in the pull request.
- Not done: page return on Windows, return while a program is idle.
