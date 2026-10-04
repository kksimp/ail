# Research: matching or beating C on systems and allocation-heavy code

> **Non-normative research note,** 2026-10-04. It surveys ideas, their
> constraints, and prior art. Nothing here is decided. Each idea that the
> designer wants pursued becomes an open question and then an ADR. Paper
> references are given so findings can be checked.

## 0. First, what does "beat C" mean?

There are three different bars, and the claims must say which one they mean:

| Bar | Meaning | Realism |
|---|---|---|
| **Typical C** | Code as most C programmers write it: `malloc`/`free` per object, blocking IO, null-terminated strings, linked structures | **Beatable**. Most of the wins below apply. |
| **C + best libraries** | C using mimalloc/jemalloc, arenas, io_uring, SIMD libraries | **Matchable**. Beating it needs the compiler-only ideas (§2). |
| **Expert hand-tuned C/assembly** | Custom allocators, hand-laid-out data, intrinsics everywhere | **Parity at best** in general. It can be beaten only where AIL knows facts that C *cannot express*. |

The strategic point: **AIL beats C by knowing more than a C compiler can.**
A C compiler can't move, reorder, compress or free memory on its own,
because C's semantics forbid it. AIL's semantics can allow it.

## 1. Why allocation-heavy C is slower than people think

- General-purpose `malloc`/`free` costs roughly tens of nanoseconds per
  operation, and much more under thread contention. A bump-pointer allocation
  costs a handful of instructions.
- `malloc` never moves objects, so the heap fragments and locality decays
  over a program's lifetime. A compacting collector restores locality.
- `free` touches each object individually. Freeing a whole region is O(1).
- Pointers are always 64 bits, which wastes cache in pointer-dense structures
  (trees, graphs, ASTs).

**The fundamental constraint (GC vs. explicit memory).** Hertz & Berger
(OOPSLA 2005, *Quantifying the Performance of Garbage Collection vs. Explicit
Memory Management*) found that a good generational GC **matched** explicit
memory management with about **5x** the heap, was **~17% slower at 3x**, and
**~70% slower at 2x**. In other words, a traditional tracing GC buys
throughput with memory. The ideas below are mostly ways around that trade-off:
make most objects never reach the GC, so the GC heap stays small.

## 2. Ideas for allocation-heavy code

### 2.1 Static allocation-strategy inference (the core idea)
At every allocation site, the compiler picks the cheapest safe strategy in
this order:

```
eliminate (scalar replacement)  >  stack  >  in-place reuse  >  inferred region/arena  >  task-local heap  >  shared GC heap
```

The programmer writes ordinary code. The compiler's allocation report (brief
§20) shows what it chose and *why* an object fell back to the GC heap. The AI
API can answer "why does this allocate?".

- **Prior art:** escape analysis (Java HotSpot, Go). Partial escape analysis
  (Stadler, Würthinger, Mössenböck, CGO 2014) removes allocations on the paths
  where an object doesn't escape.
- **What's novel here:** the *full ladder*, chosen statically, reported
  per site, and with language semantics designed to make each rung provable.

### 2.2 Region inference with GC fallback
The compiler infers that a group of allocations share a lifetime (for example
everything allocated while handling one request) and places them in a region
freed in one step.
- **Prior art:** Tofte & Talpin region inference (ML Kit). Hallenberg, Elsman
  & Tofte (PLDI 2002) combined regions with GC to fix region inference's
  space leaks.
- **Constraint:** pure region inference can leak badly in unpredictable ways.
  That's why the GC fallback and the report are mandatory.
- **Why it can beat C:** C programmers rarely write arenas, and AIL would get
  them automatically.

### 2.3 Garbage-free reference counting with in-place reuse
When the compiler can prove a value is uniquely referenced at its last use,
it frees the value immediately, or **reuses its memory directly** for a
new value of the same size. That gives zero allocator calls for
functional-style updates.
- **Prior art:** Perceus (Reinking, Xie, de Moura, Leijen, PLDI 2021) in
  Koka and Lean 4. ASAP memory management (Proust, 2017). Lobster's
  compile-time reference-count elision.
- **Constraint:** needs precise knowledge of uniqueness and mutability.
  Cycles still need a tracing backstop.
- **Possibly novel:** combining compile-time reuse with a tracing GC backstop
  instead of runtime reference counting.

### 2.4 Task-local heaps proven by the type system
If the type system proves that an object is never shared across tasks, it
lives in a heap private to its task:
- allocation needs no synchronisation;
- the heap is collected independently, with no global pause;
- **no write barriers** are needed for it;
- the whole heap is freed in O(1) when the task ends.
- **Prior art:** Erlang per-process heaps. Pony's per-actor heaps (ORCA).
  Multicore OCaml's earlier design.
- **Constraint:** sharing must be explicit in the type system (this ties into
  the data-race-safety design). Moving an object to another task means a copy
  or a promotion step.

### 2.5 Barrier elision through immutability
A generational GC needs write barriers when an old object gets a pointer to a
young one. Immutable objects can't be mutated, so they need no barriers.
If AIL defaults to immutability, the compiler can drop most barriers.
- **Constraint:** the benefit depends on how much code is immutable, which
  is a language-design choice (to be raised under the type-system topic).

### 2.6 Modern GC algorithms for what remains
For objects that do reach the shared heap: mark-region allocation (Immix,
Blackburn & McKinley PLDI 2008) and LXR (Zhao, Blackburn & McKinley, PLDI
2022), which combines reference counting with Immix and achieved both high
throughput and low latency. AIL's GC would be custom-built (ADR 0009), but
these are the state of the art to beat.

### 2.7 Allocation-site specialisation and pretenuring
Every allocation site has known sizes and types, so the compiler can emit
specialised inline bump allocations. Profiling (PGO) can mark long-lived sites
to allocate directly in the old generation (pretenuring, as in V8's
allocation-site tracking).

## 3. Ideas for "typical systems apps"

### 3.1 Compiler-chosen data layout
C fixes struct layout to declaration order, so its compiler can't fix bad
layouts. AIL could allow the compiler to:
- reorder fields to minimise padding and group hot fields together;
- split hot and cold fields into separate memory;
- turn an array of structs into a struct of arrays when access patterns favour it;
- report every layout decision.
- **Constraint and a contradiction with the brief:** the brief asks for
  *predictable* data layouts. Resolution to propose: compiler-chosen layout by
  default, an explicit fixed-layout attribute for FFI, serialization and
  hardware use, and layout always visible in reports. **This needs a decision.**

### 3.2 Compressed managed references
Store GC-managed references as 32-bit offsets into a heap of up to 32 GB
(using 8-byte alignment). Pointer-dense data halves in size, and more of it
fits in cache.
- **Prior art:** Java compressed oops. V8 pointer compression (2020).
- **Constraint and a conflict with spec rule `plat.ptr.1`:** that rule says
  "pointers are 64 bits". Raw pointers would stay 64-bit, but the
  representation of managed references would become an implementation detail.
  **This needs a decision, and possibly a refinement of ADR 0007.**

### 3.3 Aliasing guarantees
If the type system knows when two references can't alias (mutable access is
exclusive), every loop gets what C only gets with a hand-written `restrict`.
That means more vectorisation and fewer redundant loads. Fortran and Rust
show the benefit.

### 3.4 Whole-program optimisation by default
Specialise generics, devirtualise, propagate constants across modules, and
delete unused code program-wide. C is limited by translation units unless LTO
is used.
- **Constraint:** tension with fast incremental compilation. That needs a
  build design that caches whole-program facts. **This needs a decision.**

### 3.5 Better defaults than typical C
These come from the standard library and runtime, not the compiler:
- IO through completion-based interfaces (io_uring on Linux, IOCP on
  Windows, kqueue on macOS), with batched syscalls.
- Length-prefixed strings and slices with zero-copy parsing (instead of
  `strlen` scans).
- Small-string and small-vector optimisation.
- Async tasks whose frame sizes are computed at compile time, so there's no
  per-task heap allocation.

### 3.6 Continuous PGO
Profiles are collected during tests and benchmarks and stored in the build
cache, so profile-guided optimisation is the normal case rather than an
expert step.

### 3.7 Compile-time execution
Lookup tables, parsers, and serialisers can be generated at compile time.
C's preprocessor and code generators can do this only crudely.

## 4. Constraints in one place

| Constraint | Source | Can we design around it? |
|---|---|---|
| A tracing GC trades memory for speed (~5x heap to match explicit management) | Hertz & Berger 2005 | Partly: keep most objects out of the GC heap (§2.1–2.5). |
| Write barriers and safepoint polls cost a few percent | Inherent to concurrent/generational GC | Elide them via immutability (§2.5) and task-local heaps (§2.4). Remove polls from loops that provably don't allocate. |
| A precise moving GC needs stack maps, which limit register allocation | Inherent | Design the backend's register allocator around stack maps from day one (an advantage of having no LLVM). |
| Moving objects conflicts with C holding pointers | FFI | Pinning, non-moving FFI buffers, and handles. Covered in the FFI design. |
| Region inference can leak | ML Kit experience | GC fallback plus reports. |
| Compiler-chosen layout vs. predictable layout | Brief §1 | Default plus opt-in fixed layout, always reported (§3.1). |
| Compressed references vs. "64-bit pointers" | ADR 0007 | Separate raw pointers from managed references (§3.2). |
| Whole-program optimisation vs. incremental compile speed | Inherent | Cache whole-program facts. Optimise fully in release, minimally in debug. |
| A custom backend starts behind LLVM | ADR 0010 | Accepted cost. Measure and publish the gap from day one. |
| Expert hand-written assembly can't be beaten in general | Inherent | Provide intrinsics, inline assembly in `unsafe`, and superoptimisation research later. |

## 5. Suggested R&D tracks (for the designer to approve or reject)

1. **Allocation ladder (§2.1)** with mandatory per-site reports. Highest leverage.
2. **Task-local heaps (§2.4)**, tied to the concurrency and type-system design.
3. **Compile-time reuse (§2.3)**, tied to the mutability and uniqueness design.
4. **Compiler-chosen layout (§3.1)** and **compressed references (§3.2)**.
5. **Benchmark suite** covering all three "beat C" bars (§0), from the first
   native binary.
