# 0014. Managed references may use a compressed representation; raw pointers stay 64-bit

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 5 — Memory model; 6 — GC design; 24 — ABI
- **Spec rules:** [plat.ptr.1 (refined), plat.ptr.3](../spec/01-platforms.md)
- **Refines:** [ADR 0007](0007-64-bit-little-endian-only.md). It does not supersede it.
- **Research:** [design/performance-beyond-c.md §3.2](../design/performance-beyond-c.md#32-compressed-managed-references)
- **Resolves:** OQ-016

## Context

Pointer-dense data structures (trees, graphs, ASTs, linked records) spend much
of their memory and cache on 64-bit pointers. Java's compressed ordinary
object pointers and V8's pointer compression show that 32-bit references into
a bounded heap region cut this cost substantially. As originally written,
spec rule `plat.ptr.1` ("pointers are 64 bits") would have forbidden it.

## Options considered

### Option A — Managed reference representation is implementation-defined (chosen)
- **Pros:** Pointer-heavy data can shrink by up to half, improving cache use.
  Leaves room for other representation tricks later (tagging).
- **Cons:** A shift-and-add to decode. Each compressed region is capped
  (e.g. 32 GB with 8-byte alignment). FFI must convert references to raw
  pointers.

### Option B — Always 64-bit
- **Pros:** Simplest, no heap cap, trivial FFI.
- **Cons:** Wastes cache in exactly the workloads where C and GC'd languages
  are slowest.

## Decision

Raw pointers and addresses remain exactly 64 bits (`plat.ptr.1`). The
in-memory representation of **managed references** (references to
GC-managed or compiler-managed objects) is **implementation-defined**
(`plat.ptr.3`). Programs cannot observe it except through `unsafe` code and
the FFI, which see raw 64-bit pointers.

## Rationale

Language semantics stay simple (a reference is a reference), while the
implementation gets the freedom to beat C on pointer-heavy workloads.

## Consequences

- The GC design must decide when to compress and how to handle heaps beyond
  the compressed range (multiple compressed regions, or a fallback to full
  pointers).
- Converting a reference to a raw pointer (in unsafe code or FFI) is an
  explicit operation, and may also require pinning.
- `sizeof`-style queries on types containing managed references give
  implementation-specific results. They are always reported, never guessed.

## Rejected alternatives

- **Always 64-bit references:** gives up a proven cache-efficiency win.
