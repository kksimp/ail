# 0907. Native `List[u8]` holds one byte per element

- **Status:** Accepted
- **Date:** 2026-10-25
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Amends:** [ADR 0391](0391-native-backend-optimisations.md)

## Context

A native list element takes 8 bytes, except in a `List[bool]` (ADR 0391). The
compiler holds its output images (`codegen.Image.code` and `.data`, the
assembler buffers and the object writers' output) as `List[int]` with one code
byte per element. The self-build census (`docs/design/compact-inst.md`) shows
these lists growing from 42 MB to 199 MB of list data during code generation and
linking, the largest single allocations of the build. User programs that hold
bytes in a `List[u8]` pay the same 8x.

## Decision

A `List[u8]` stores one byte per element, as a `List[bool]` does
(`Ty.elem_bytes()` is 1 for `bool` and `u8`, 8 otherwise). The language
semantics do not change: elements are `u8` values, `bytes.from_list` and
`for b in bytes` work as before, and a list's capacity is not observable.

- Layout, push, get, set, copy, slice, extend, insert, remove, reverse, `contains`,
  `index_of`, equality, `take`, `windows`, `chunks`, `flatten`, `shuffle` and the
  code generators' fast paths take the element size from the type.
- Operations that read 8-byte words (`sort`, `sorted`, `sum`, `min`, `max`,
  `<` and `cmp`) widen the list into a temporary list of words first
  (`rt.list_widen`), and `sort` writes the result back (`rt.list_narrow_into`).
  These are not hot for bytes, and no sized-element variants of the runtime
  functions are needed.
- `Set[u8]` and `Map[u8, ...]` build their key lists with the byte variants
  (`rt.map_keys_bool` and `rt.map_values_bool` store the low byte), and a set
  built from a list of bytes widens the list first.
- A list of lists of bytes has no ordering kind, so `<`, `sort` and `==` on
  `List[List[u8]]` are reported as not yet in the native subset; `flatten` and
  copies work.
- `rt.list_pushb` takes the byte as an `int` (0 or 1 for a `bool`).
- Lists no longer have a minimum capacity of 4: a list has room for exactly the
  requested number of elements (at least 1), and the first regrow goes to 4
  (then 16, then doubles); spec rule native.bce.3 says so.

## Rejected alternatives

- **A separate byte-buffer type for the compiler's images:** adds a type and its
  methods to the language for one use, and user programs would keep the 8x.
- **A `bytes` builder (`bytes` is immutable):** every append copies; the
  assembler appends a byte at a time.
- **Size-aware runtime variants of sort, sum, min, max and cmp:** more runtime
  code for operations that are rare on bytes; widening reuses the tested paths.
- **Collector pacing to hide the cost:** +6.6% wall for -19.7% RSS in the
  self-build (measured 2026-10-25), against a representation fix that removes
  the memory.
