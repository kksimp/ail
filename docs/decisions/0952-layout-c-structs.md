# 0952. `@layout(c)` structs: raw memory behind `*P`, not GC objects

- **Status:** Deferred (past 0.1.0: design only, no spec rules or code yet)
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 14 — FFI
- **Spec rules:** `ffi.layout.1-4` (to be written with the code)
- **Builds on:** [ADR 0835](0835-native-raw-pointers-and-hw.md), [ADR 0950](0950-c-abi-exports.md), [ADR 0392](0392-native-structs-and-methods.md)
- **Design:** [systems-abi.md](../design/systems-abi.md)

## Context

An AIL struct is a GC object: a header, then one 8-byte slot per field
(native-abi.md, ADR 0392). C structs have packed fields of mixed width, fixed
padding and no header, and C code keeps pointers to them. The two cannot be the
same representation. The kernel and driver work (struct tables, MMIO blocks,
C library structs) needs the C one.

## Decision

1. `@layout(c)` (also `@layout(c, packed)` and `@layout(c, align N)`) before a
   `struct` declares a **C struct**. Fields are declaration order with C
   padding; `packed` has none; `align N` raises the struct's alignment (a power
   of two). Only C-layout field types are allowed (`ffi.decl.3` types, other
   C structs, and fixed arrays `[N]T` when those exist), and the struct has no
   type parameters, no invariants, and is not an error type.
2. A C struct is **not a GC object and has no value semantics of its own**. It
   exists in raw memory (`mem.alloc`, an arena, a C library, an MMIO region) and
   is reached through `*P`. Under `unsafe`: `p.x` reads and `p.x = v` writes the
   field with a typed load or store at a constant offset (the `pload`/`pstore`
   prims of ADR 0835, volatile when the package is `unsafe driver` and the field
   is marked `volatile`). `P.size`, `P.align` and `P.offset_of(field)` are
   compile-time constants.
3. **By value** across `extern c fn` and `pub export c fn`: allowed for C structs
   up to 16 bytes, classified as the platform ABI says (System V INTEGER/SSE
   eightbytes; AAPCS64 composite in x0-x1 or HFA in v0-v3). Larger ones are
   passed by hidden pointer as the platform specifies. Variadics stay out.
4. A C struct is never stored in an AIL `List`, `Map` or normal struct field
   (those would copy it as a GC slot); only `*P` is. The checker says so with
   `ail explain ffi.layout.3`.
5. Checks the compiler adds: overlapping or misaligned fields never occur by
   construction; a `packed` struct's fields are read with unaligned-safe loads
   (a byte-wise fallback on targets that need it).

## Rejected alternatives

- Reusing GC struct objects with a "C-compatible" flag: the header and 8-byte
  slots cannot match C, and a moving or scanning collector would see raw bytes.
- Value-type C structs in locals with copy semantics: needs a second struct
  value representation in MIR (SROA over byte offsets); deferred until measured.
- `#[repr(C)]`-style attribute on every struct: AIL structs stay one representation
  unless the author opts in.
