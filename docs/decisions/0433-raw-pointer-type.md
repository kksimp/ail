# 0433. `*T` is a raw pointer type with a C-layout target

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 15 — Unsafe semantics; 14 — FFI
- **Spec rules:** [unsafe.ptr.1](../spec/13-unsafe.md), [unsafe.op.2](../spec/13-unsafe.md)
- **Builds:** [ADR 0106](0106-unsafe-operations.md)

## Context

ADR 0106 lists raw pointers `*T` and says creating or copying one is safe and
dereferencing is not. C functions take and return pointers, so the type must
exist before any C string or buffer can cross the boundary.

## Decision

`*T` is a type written before another type (`*u8`, `*float`, `**u8`). `T` is `int`, `float`, `bool`, a sized
number or another pointer: types with a fixed C layout (`unsafe.ptr.1`).
A pointer is an address held as a 64-bit integer at run time. It is not a
number in the language: no arithmetic and no conversion to or from `int` in the
language (offsets are an unsafe operation of ADR 0435). It is `Eq`, not
`Ord` or `Hash`, and has no ownership, so it can be copied freely.

## Options considered

### Option A — `*T` with a layout-checked target (chosen)
- **Pros:** the spelling ADR 0106 and the brief use; the target type fixes the
  size for reads and writes; tokens: one symbol.
- **Cons:** none found; `**u8` lexes as two `*`.

### Option B — one untyped `ptr` type
- **Cons:** reads and writes would need a type at every use; typed pointers
  catch a `*float` passed where `*u8` is expected.

### Option C — pointers as `int`
- **Cons:** erases the safe/unsafe line of ADR 0106 (any `int` becomes
  dereferenceable) and makes `unsafe.op.2` meaningless.

## Consequences
- `TypeKind::Ptr`, `Ty::Ptr` and the AIL `PtrTo`/`PtrOf` carry it.
- Reads, writes and offsets arrive with ADR 0435.
