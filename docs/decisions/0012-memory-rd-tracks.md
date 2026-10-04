# 0012. Pursue four memory-management R&D tracks to beat C on allocation-heavy code

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 5 — Memory model; 6 — GC design; 7 — Ownership/lifetimes; 22 — Runtime
- **Spec rules:** none yet
- **Research:** [design/performance-beyond-c.md §2](../design/performance-beyond-c.md#2-ideas-for-allocation-heavy-code)
- **Resolves:** OQ-018

## Context

A traditional tracing GC trades memory for throughput. Hertz & Berger
(OOPSLA 2005) found it needed about 5x the heap to match explicit memory
management. To match or beat C on allocation-heavy code, AIL has to keep most
objects **out of** the GC heap entirely, using language semantics that let the
compiler prove cheaper strategies are safe.

## Options considered

All four tracks were offered as independent choices, and all four were selected.

### Track 1 — Allocation ladder with mandatory per-site reports (chosen)
For each allocation site, the compiler picks the cheapest safe strategy:
`eliminate > stack > in-place reuse > inferred arena > task-local heap > shared GC heap`.
- **Pros:** The highest-leverage idea. Keeps the GC heap small. "Why does
  this allocate?" can be answered by humans and AI.
- **Cons:** A large compiler-analysis effort. The type system must be
  designed so that each rung is provable.

### Track 2 — Task-local heaps (chosen)
- **Pros:** Lock-free allocation. Each heap is collected independently.
  No write barriers. O(1) teardown when a task ends.
- **Cons:** Sharing must be explicit in types. Moving an object to another
  task requires a copy or promotion.

### Track 3 — Compile-time in-place reuse (chosen)
Perceus-style uniqueness reasoning frees or reuses memory at a value's last
use. A tracing GC acts as backstop.
- **Pros:** Zero allocator calls for update-heavy code.
- **Cons:** Needs uniqueness and mutability information in the language.
  Cycles still need the GC.

### Track 4 — Inferred arenas (regions) with GC fallback (chosen)
- **Pros:** Arena-speed allocation and O(1) bulk free automatically.
- **Cons:** Pure region inference leaks unpredictably (the ML Kit
  experience), so the GC fallback and reports are mandatory.

## Decision

AIL pursues all four tracks as one design. The **allocation ladder** is the
umbrella, and tracks 2–4 are rungs of it. Every allocation decision is reported
(brief §20) and queryable through the AI semantic API (brief §12).

## Rationale

Together these attack every cost of allocation-heavy code: allocation speed,
deallocation cost, GC memory headroom, barriers and pauses. This is how a
GC'd language can plausibly match or beat typical C, and close in on C with
good libraries.

## Consequences

- The type system, ownership/lifetime and concurrency designs **must** supply
  the facts these tracks need: escape, uniqueness, mutability, and task
  confinement. These requirements are inputs to those topics.
- The shared GC must interoperate with every rung (references between rungs,
  promotion between them).
- A new constraint for later design: the ladder must stay **predictable**.
  Small source edits shouldn't silently move a hot allocation down to the GC
  heap. The report and diagnostics, e.g. an opt-in "this must not allocate on
  the GC heap" assertion, should guard against that. To be raised under the
  memory model topic.
- Benchmarks must cover allocation-heavy workloads against typical C, C with
  mimalloc or jemalloc, and C with hand-written arenas.

## Rejected alternatives

None. All proposed tracks were accepted. A **traditional GC-only design**
(the implicit baseline) is rejected because of its memory/throughput trade-off.
