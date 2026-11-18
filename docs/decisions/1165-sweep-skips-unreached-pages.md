# 1165. The sweep frees pages no pointer reached without reading them

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the open slowdowns against C, runtime speed only, no syntax change
- **Topic:** Native heap and collector
- **Spec rules:** none (no observable behaviour)
- **Builds on:** [ADR 0562](0562-native-heap-size-classes-and-header.md) (heap), [ADR 0564](0564-collection-trigger-and-sweep.md) (sweep)

## Context

`gc_sweep` read and rewrote the header of every block of every small page,
then gave a page back when it found no mark in it. A program that makes
short-lived strings or lists has almost no live pages, so nearly all of
that work was on pages about to be freed, and each header read missed the
cache. It was 17% of `splitline` and `fields` and 9% of `readline`
(`textgap-linux-29a37145.txt`: 71% of the sweep on one load).

## Options considered

### Option A — one byte per page, set when marking finds a pointer into it (chosen)

`H_PMARK` (heap state word 31) is a byte table after the page table. Every
mark goes through `gc_find`, which sets the page's byte. The sweep gives a
small page whose byte is 0 back to the page pool without reading it, and
clears the byte of the others before sweeping them as before.

- **Pros:** a dead page costs one byte load and one page-table store; its
  next blocks come from the bump pointer, in address order, instead of a
  free list threaded through cold memory; a page that holds a live block is
  swept exactly as before.
- **Cons:** one store per pointer the marker follows (into a table that
  stays in the first-level cache); one more byte of reservation per page.

### Option B — a mark count in the page table entry

- **Cons:** the entry's bits are all in use, and the parallel marker would
  need an atomic update; a byte store of the same value needs none.

### Option C — a moving or generational collector

- **Cons:** roots are conservative (ADR 0562), so objects cannot move.

## Decision

Option A. A page freed this way keeps its old words, as a freed large span
already does; `gc_find` treats blocks not yet handed out from a page being
filled as no object, so stale words are never read as headers.

## Rationale

ADR 0076 order: nothing changes for the program or its diagnostics, so
speed decides. A is the only option that fits a conservative collector and
the parallel marker without a lock.

## Consequences

- Test `gc_dead_pages`.
- ail-linux x86-64 (2026-11-18, `textgap-linux-0edef0f0.txt`, against main
  832e86a1, with this lane's other runtime changes): `fields` 2.070 s to
  1.119 s (C twin 1.113 s), `readline` 0.354 s to 0.187 s (C twin 0.209 s),
  `splitline` on one thread 0.580 s to 0.401 s (C twin 0.338 s). `gc_sweep`
  no longer appears in their profiles.
- Windows commits the page table, so its commit grows by one byte per
  heap page (about 1 MB for the 16 GiB region).

## Rejected alternatives

- **Option B:** no free bits and an atomic update per mark.
- **Option C:** conservative roots rule out moving objects.
