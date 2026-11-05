# 0943. Native: per-thread allocation caches once the pool exists

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none (no observable behaviour changes)
- **Builds on:** [ADR 0562](0562-native-heap-size-classes-and-header.md) (heap), [ADR 0599](0599-native-threads-and-task-pool.md) (tasks and the allocation lock), [ADR 0911](0911-default-automatic-parallel-loops.md) (parallel loops by default)

## Context

`pool()` set the fast-path limit `H_FASTLIM` to 0 the first time a task,
`par for`, proven loop or fork-join ran, and the collector kept it at 0. From
then on every allocation in the process took `ALLOC_LOCK`, on every thread,
including the main thread after the parallel region ended. With ADR 0911
making proven loops parallel by default, most non-trivial programs reach the
pool, and paid a lock (two atomic read-modify-writes, contended between
workers) per allocation for the rest of the run.

## Decision

1. Each thread slot (main thread 0, worker `k` at `k + 1`) has an allocation
   cache of 1536 bytes, laid out like the heap state block's per-class words
   (free-list heads, bump page and end, dirty flag).
   Heap state word 280 points at the caches, made by `pool` before the second
   thread starts.
2. With threads `H_FASTLIM` stays 0, so the fast path's limit test fails
   and falls to a second branch that serves the running thread's cache with
   no lock and no counting (its blocks were counted when it was filled). The
   slot comes from the stack pointer (a worker's stack lies in the `STACKS`
   region; on Windows from the thread id, as `worker_index`). One thread runs
   exactly the instructions it ran before.
3. A class with nothing ready goes to `alloc_slow`, which takes the lock and
   refills: about 4 KiB of the shared free list, else the shared page being
   filled, else a new page. The bytes count toward the next collection as
   they move. Large objects still take the shared, locked path.
4. A collection (the world stopped, so no thread is inside the fast path,
   which has no safe point) first empties every cache: the headers of blocks
   not yet handed out are cleared, so the conservative scan never takes them
   for objects, and the sweep chains them with the other free blocks.
5. `AIL_GC_VERIFY` keeps every allocation on the counted, shared path, as
   before.

## Measured

d0a5d418 against main 78955b4c, min of 7, `tools/perfbench/alloc/run.py`:

| | Mac (arm64, 8 cores) | ail-linux (x86-64) |
|---|---|---|
| objs_par (allocation on all threads, then main) | 2.394 s → 0.385 s | 0.47x |
| gc_heap (300 MB live, workers churn; with ADR 0944) | 6.176 s → 1.089 s | 0.15x |
| bt_par, binarytrees, fasta | neutral | neutral |
| BIGRACE, classic ten | 1.00 (collatz 1.05, regex_scan 1.02: noise) | matmul-1400 1.07 (same instructions), regex_scan +2.4% instructions |
| stage-2 self-build | 12.03 s → 12.14 s | |

The same gains hold with `AIL_PARALLEL=0` (written `par for` still starts the
pool). The regex_scan instructions came from a `MULTI` test that the first
version put on the one-thread fast path; the cache branch now sits behind the
limit test, which one thread passes, so that path is the old one again.

## Rejected alternatives

- **Keep the lock, make it cheaper** (a spin or a ticket lock). Still a
  shared cache line written per allocation; contention grows with cores.
- **Thread-local storage through the OS** (`fs`/`tpidr_el0`, `TlsGetValue`).
  Needs per-OS setup in thread start and a different instruction sequence per
  target; the stack-pointer test needs neither, since workers' stacks are one
  region already.
- **Per-thread heaps** (each thread owns pages for good). Needs ownership in
  the page table, cross-thread frees, and a collector that sweeps per owner;
  a large change for the same fast path.
- **Turn the fast path back on for the main thread only.** Leaves every
  worker on the lock, where most allocation of a parallel loop happens.
- **Bigger refills (a whole page every time).** Takes free blocks out of the
  shared lists faster than they are used and strands memory in idle threads'
  caches; 4 KiB keeps the lock rare without that.

## Consequences

After the first parallel loop allocation costs what it did before it. Memory
held in caches between collections is at most one page and 4 KiB of list per
class per thread. Parallel marking ([ADR 0944](0944-parallel-gc-marking.md))
keeps each marker's stack in the same cache block.
