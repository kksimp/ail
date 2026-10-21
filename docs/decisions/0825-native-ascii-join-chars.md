# 0825. Native: an ASCII path for joining characters

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** type.coll.9 (behaviour unchanged)
- **Builds on:** [ADR 0391](0391-native-backend-optimisations.md)

## Context

Callgrind of fasta (70M instructions against C's 18M) put 27M, 38%, in
`rt.list_join_chars` and `rt.char_put`: one width test, one call and a
chain of width branches per character, to build 60-character lines of `a`,
`c`, `g` and `t`. mandelbrot's rows are the same shape.

## Decision

`list_join_chars` (compiler/rt/native/core.ail) first ORs all the
characters. When the result is below 128 every character is one byte, so the
result length is the count, with no width pass. With an empty separator it
then stores each character as one byte and returns. Otherwise the old
path runs, unchanged. Output is the same bytes either way (type.coll.9).

Instructions executed, before → after: fasta 70M → 49M (−30%),
mandelbrot 39M → 34M (−13%); the other eight perfbench programs are unchanged.

## Consequences

- Strings with non-ASCII characters pay one extra pass over the list
  (an OR per character); they were already slower per character.
- The ASCII path with a separator still goes through `char_put`; joining
  with a separator is rare for characters.

## Rejected alternatives

- **Inlining `char_put`'s ASCII case into the loop only.** Keeps the width
  pass, so the saving is about half.
- **A code generator intrinsic.** The change is runtime-only and needs no
  second implementation per ISA.
