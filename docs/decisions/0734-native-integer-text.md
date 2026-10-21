# 0734. Native integer text: digits from the end, one division each

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** type.str.1 (output unchanged)
- **Builds on:** [ADR 0408](0408-buffered-output.md), [ADR 0623](0623-branch-and-codegen-peepholes.md)

## Context

The native runtime printed an `int` by counting digits with a loop, then
for each digit dividing by a power of ten kept in a variable (`idiv`, about
40 cycles) and calling `out_byte`. `print_many` (300,000 lines of
`"row {i}: value {j} sum {s}"`) spent 55% of its 420M instructions in
`rt.out_int` and another 20% in `rt.out_byte` for its digits.

## Decision

`write_int(at, n)` in `rt/native/core.ail` writes the decimal text of `n`
from the end of a 24-byte window at `at`, one division by 10 per digit
(a multiply since ADR 0623), working on negative values so the smallest
`int` needs no negation, then moves the digits down to `at` with three
8-byte copies. The caller guarantees 48 bytes at `at` (the copies read up
to 24 bytes past the window; the bytes after the number are scratch).
`out_int` flushes first when fewer than 48 bytes remain in the output
buffer; the string builder's `put_int` (interpolation) grows until 48 fit.

## Measurements (Linux x86-64)

`print_many`: 420M to 250M instructions and 0.077 s to 0.026 s (gcc -O2
with `printf`: 0.046 s; CPython 0.63 s). Output identical to `ail run` on
x86-64 and aarch64 (qemu); `int_text` covers every digit count, both
signs and the extremes across the buffer limit.

## Rejected alternatives

- **Counting digits first, then filling in place:** needs a second loop per
  call (11 instructions a digit); the window and move is cheaper.
- **Two digits per step with a 200-byte table:** needs static data in the
  runtime; the gain over one division a digit is under 2% of the program.
- **A per-byte `out_byte` loop kept for the sign only:** the sign is one
  store in the window.
