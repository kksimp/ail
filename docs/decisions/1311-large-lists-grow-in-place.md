# 1311. Large lists grow in place; never-scanned blocks weigh less in the GC goal; small-heap widening is a trial

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), toward the memory target of the 2026-11-27 state review
- **Topic:** 06 — Memory
- **Spec rules:** [mem.gc.5](../spec/06-memory.md)
- **Amends:** [ADR 1226](1226-compiler-wider-heap-goal.md)

## Context

Peak memory ran well above the C and Go twins: `lzr` 577 MB, `gcints` 992 MB, the LRU program about 6 times C. A
growing list copied its element block to a new one each time, leaving dead copies until the next collection, and
large byte and string blocks, which the collector never scans, raised the heap goal as much as pointer-rich data.

## Decision

- A list whose element block is larger than a size class grows where it stands when the pages after it are free,
  and otherwise moves (`compiler/rt/native/heap_extend.ail`).
- Blocks the collector never scans (strings, bytes, lists of bytes or bools) count an eighth of their size toward
  the live bytes the goal is taken from.
- On a heap under 48 MiB a widening of the goal is a trial: it is undone, and not tried again for a while, when the
  run time per allocated byte rose by more than 1% over the next few collections.
- None of this changes what a program observes. Measured with `tools/perfbench/mem.py` and `memab.py`: `lzr` 577 to
  440 MB, `gcints` 992 to 736 MB, LRU 91 to 66 MB, instruction counts flat or lower. The trial widening stays only
  if the Mac timing A/B shows no regression on `gc_heap`, `gcchurn`, `binarytrees`, `objs_par` and LRU (the
  designer's rule: no speed regression).

## Rejected alternatives

- **A smaller fixed goal.** Saves memory at a run-time cost every program pays.
- **Count never-scanned blocks at zero.** They still take memory; an eighth keeps a byte-heavy program's heap bounded.
