# 0434. `std.mem` hands out C strings from C's allocator and traps on bad frees

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 14 — FFI; 15 — Unsafe semantics
- **Spec rules:** [std.mem.1](../spec/17-std.md), [unsafe.mem.1–2](../spec/13-unsafe.md)

## Context

An AIL `str` is not NUL-terminated, so `strlen` and every other C string
function need a copy. A program must also free what C allocated and may need C
to free what it allocated.

## Decision

`std.mem` (an unsafe operation each, `unsafe.op.1`) has `cstr(s) *u8`,
`from_cstr(p) str` and `free(p)`. Blocks come from the C `malloc` so ownership
can cross the boundary. The interpreter remembers the blocks it allocated and
the ones it freed: `from_cstr` stops at the end of a known block and traps if
it has no NUL; `free` of null or of an already-freed block traps
(`unsafe.mem.1–2`) instead of corrupting the heap. A pointer from C is
trusted: `from_cstr` scans to its NUL and `free` calls `free`.

## Options considered

### Option A — C's allocator plus a checked block table (chosen)
- **Pros:** interoperates with C; double frees and unterminated reads of our
  own blocks are caught without a custom allocator.
- **Cons:** pointers from C are unchecked, as unsafe code must be.

### Option B — a Rust allocation per string
- **Cons:** C code can't free it and AIL can't free C's blocks.

### Option C — pass `str` to `extern` functions directly
- **Cons:** hides a copy and the NUL requirement from a signature that claims
  `*u8`; the brief asks for explicit helpers.

## Consequences
- Blocks that are not freed leak, as in C; a scoped helper (`defer`-based) can
  wrap `cstr` in a later change.
- Typed allocation (`alloc`) joins this module in ADR 0435.
