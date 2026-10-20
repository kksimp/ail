# 0435. Pointer reads, writes and offsets are methods; blocks are bounds-checked

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 15 — Unsafe semantics; 14 — FFI
- **Spec rules:** [unsafe.ptr.2–6](../spec/13-unsafe.md), [unsafe.mem.3](../spec/13-unsafe.md)
- **Builds:** [ADR 0106](0106-unsafe-operations.md), [ADR 0433](0433-raw-pointer-type.md)

## Context

ADR 0106 makes dereferencing, pointer arithmetic, manual allocation and
reinterpretation unsafe operations. The native runtime (lane 122) is written
against a word-addressed memory model and waits for real loads and stores
(`docs/design/native-abi.md` §8). C functions that fill buffers or return
pointers into a string need reads, writes and allocation too.

## Decision

1. **Methods on the pointer.** `p.read()`, `p.write(v)`, `p.offset(n)` and
   `p.is_null()` (`unsafe.ptr.2`); `offset` counts elements, not bytes. All but
   `is_null` are unsafe operations. A method the pointer does not have is
   `unsafe.ptr.3`.
2. **Typed allocation from an example value.** `mem.alloc(n, example)` returns
   `*T` for the type of `example`, with every element set to it, so there is
   no uninitialised memory and no type-argument syntax. `mem.cast(p, example)`
   reinterprets a pointer the same way, `mem.null()` is the null `*u8`,
   `mem.free(p)` takes any pointer (`unsafe.mem.3`).
3. **Checked where the interpreter knows the block.** It keeps the blocks
   `std.mem` allocated and freed. Access through null (`unsafe.ptr.4`), past a
   block's end including exactly at it (`unsafe.ptr.5`) or into a freed block
   (`unsafe.ptr.6`) is a trap with a rule ID instead of a crash. Memory from C
   is unchecked.
4. **Layout.** `int`, `float` and pointers 8 bytes, `bool` 1, sized numbers their
   width, little-endian, no alignment requirement (unaligned loads and stores).

## Options considered

### Option A — methods and an example-typed `alloc` (chosen)
- **Pros:** reads like the brief (`ptr.read()`, `write()`, `offset()`);
  no generic-call syntax; tokens: `p.offset(i).read()` against `load(p, i)`
  costs the same, and the method form keeps the type attached to the pointer;
  a trap instead of a segmentation fault for the common bugs.
- **Cons:** `alloc(n, 0.0)` makes the type implicit in a value; `mem.cast` is
  a second pointer-reinterpreting spelling to learn; use after free is missed
  once `malloc` reuses the address.

### Option B — free functions `mem.load(p, i)`, `mem.store(p, i, v)`
- **Cons:** the pointee type is not on the call, so the checker needs a
  generic function type the language does not have.

### Option C — `p[i]` indexing and `p[i] = v`
- **Cons:** looks safe (no `unsafe` marker per line unless the whole block is
  read as one), and reuses a bounds-checked syntax for an unchecked operation;
  rejected as in ADR 0102's reasoning for marking every unsafe operation.

### Option D — unchecked everything
- **Cons:** a segmentation fault tells an AI agent nothing. The block table
  costs one ordered-map lookup per access, on an operation that is already
  outside the fast paths.

## Consequences
- `StdCall::Ptr` carries the pointee type from the checker to the interpreter.
- The native runtime (`compiler/rt/`) can replace its word-addressed helpers
  with `read`/`write`/`offset` (`native-abi.md` §8). Compiled code has no block
  table, so `unsafe.ptr.4–6` are interpreter diagnostics; the native backend
  traps on a null pointer only (`native-abi.md`).
