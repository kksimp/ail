# 0702. Native `bytes`: literals, indexing, slices, `==`, `len`, `text`, `hex`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** lex.bytes.1, type.bytes.1-4, type.bytes.7, type.bytes.8, type.bytes.11, native.build.1

## Context

`examples/frames.ail` was outside the native subset because of its `b"..."`
literal. `bytes` has no native representation, and the lexer already gives a
literal as hex digits.

## Decision

1. **Layout:** a bytes object is a string object (native-abi.md §2): +0 length
   in bytes, +8 the same number, +16 the data. The allocator, `len` and the
   collector need nothing new. It is typed `Ty.Bytes`.
2. **Literal:** `rt.bytes_from_hex` decodes the literal's hex digits into a new
   object where the literal stands. This adds no instruction to the MIR or
   either code generator. A literal in a hot loop pays an allocation and a
   decode; a data-section object is the follow-up if a benchmark needs it.
3. **Index and slice:** the lowering checks the bounds and traps with kind 11
   (`type.bytes.3`) or 12 (`type.bytes.4`), printing the interpreter's
   messages (`... for bytes of length N`), then calls `rt.bytes_at` or
   `rt.bytes_slice`.
4. **The byte:** `b[i]` has the new type `Ty.Byte`, held in a register as an
   `int`. Only `.to_int()` accepts it, so `u8` arithmetic, which traps past
   255, cannot be miscompiled; it stays outside the subset.
5. **Methods and operators:** `len` (the length word), `text` (UTF-8 with each
   invalid sequence as U+FFFD, the same replacement lengths as the
   interpreter), `hex`, `==` and `!=`. Everything else on `bytes` reports
   "not in the native subset yet".

The code is `compiler/mir/lower_bytes.ail` and `compiler/rt/native/bytes.ail`;
`lower.ail` has five hook lines.

## Rejected

- **A literal in the data section from the start:** it needs a new MIR
  instruction and encoders for x86-64 and ARM64 for no measured gain.
- **`Byte` as `Int`:** it would let `b[i] + 255` compile as an `int` sum
  instead of a `u8` overflow trap.
