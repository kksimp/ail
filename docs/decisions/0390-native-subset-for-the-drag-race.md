# 0390. The native subset grows to the drag-race programs

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), std.env.1, err.else.1, err.else.2, type.str.3, type.str.5, type.str.11, type.coll.9, own.val.1
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (the subset), [ADR 0387](0387-static-executables-and-ail-build.md) (`ail build`)

## Context

Lane 120's last slice races native AIL against `clang -O2`, `gcc -O2` and
Python on the perfbench programs (ADR 0460). Those programs read their size
with `env.args()` and `args[0].to_int() else return default`, define
constants such as `SOLAR_MASS := 4.0 * PI * PI`, print `{x:.9}`, keep
`List[List[int]]` matrices, and build lines with `s.chars()` and
`xs.join("")`. None of that was in the subset.

## Decision

1. **Arguments.** The entry passes argc and argv to `rt.init` (Linux: from
   the stack; macOS: dyld's x0 and x1). `env.args()` makes string objects
   of argv[1..].
2. **`s.to_int() else ...`** is the one `else` binding in the subset. The
   runtime parses as the interpreter does (an optional `-`, digits, fits in
   `int`) and sets a flag; the lowering branches on it to the fallback value
   or the leaving statement. Other `else` bindings stay outside the subset.
3. **Constants** may be constant expressions: literals, other constants,
   and unary and binary operators on them. They are inlined at each use, so
   the code generator sees what it would for the written-out expression.
   Calls stay out: a constant must not run code.
4. **`{x:.N}`** on a float prints through `rt.out_float_fixed`, which rounds
   the float's exact binary value half to even with arbitrary-precision
   integers, as the interpreter's `format!("{x:.N}")` does, so output is
   byte-identical. Width, fill, alignment and hex stay outside.
5. **Lists of `str` and of lists.** Elements are pointers either way. A list
   copied by value (own.val.1) is copied as deep as its type nests
   (`rt.list_copy_deep`), and a list pushed or put in a literal from a
   place is copied first. Strings are immutable and shared.
6. **Strings:** `s.len()`, `s.chars()` (one string per UTF-8 character) and
   `xs.join(sep)`, each one runtime function.
7. **C calls.** An `extern c fn` declaration or a `mem.*` call is reported
   under `ffi.native.1` (ADR 0436), not native.subset.1, so the message says
   it is the linker that is missing.

## Consequences

- Eleven perfbench programs build natively and print the interpreter's
  bytes on Linux x86-64 and ARM64. `loops` (maps, interpolation into a
  value), `wordfreq` (maps) and the tensor programs (comprehensions,
  `std.tensor`) stay out of the subset for now.
- `out_float_fixed` costs big-number arithmetic per print; the benchmarks
  print a handful of such values, so it doesn't show.

## Rejected alternatives

- **Rewriting the benchmarks to avoid these constructs.** The race would then
  measure programs nobody writes; the interpreter, C and Python columns run
  the same source.
- **Constant folding into literals at lowering.** Inlining the expression is
  simpler, keeps one code path for arithmetic checks, and costs nothing at
  run time once the backend folds constants.
- **`{x:.N}` by scaling and rounding in floating point.** Wrong in the last
  digit for values such as 2.675, and the race compares bytes (drag.out.1).
- **General `else` bindings.** They need optionals and error values in the
  MIR; the parse-with-fallback shape covers every perfbench program now.
