# 0819. Native `std.regex` is an AIL module compiled with the program

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; approved by the integrator on 2026-10-21)
- **Topic:** 23 — native backend
- **Spec rules:** std.regex.1-25, native.subset.1
- **Builds on:** [ADR 0438](0438-regex-engine-is-a-pike-vm.md), [ADR 0772](0772-native-std-modules-in-ail.md)

## Context

`regex` was the first blocker of several example programs and of the
`regex_scan` perfbench program: `ail build` stopped at `use std.regex`. The
interpreter's engine is Rust (`bootstrap/src/regex`), which no native build
can link (ADR 0010, `ffi.native.1`).

## Decision

- The engine is written once in ordinary AIL at
  `bootstrap/src/std/regex/regex.ail`: the pattern parser, the program
  compiler and the Pike VM of the Rust engine, ported function by function.
  `ail build` compiles it as a module of the program (ADR 0772; `std.regex`
  joins `STD_AIL`). The interpreter keeps the Rust engine and does not read
  the file.
- `regex` needs no `use` (std.use.1), so `ail build` adds the import of
  `std.regex` when a file mentions `regex.` and has no `use` ending in
  `regex`.
- Matching is linear (std.regex.4) as in the Rust engine. Three additions
  speed it up without changing results: the ASCII characters a match can
  start with are precomputed, so a scan skips positions that cannot start
  one; every class instruction has a 128-entry table for ASCII; and when no
  capture group is wanted and the program has no assertion, the empty
  transitions after each instruction are precomputed in priority order, so a
  step does not walk the program.
- The build cache key includes the text of `bootstrap/src/std`, because a
  module that only `ail build` reads is not in the executable.
- Calls of a module function that can fail (`regex.compile(..)?`, `else err`)
  and the typed error binding of ADR 0806 now work through a module name
  (`lower_err.ail`).

## Differences from the interpreter, all listed

- `regex.captures` and the `Captures` type are not built yet: a struct
  field of type `List[str?]` isn't in the native subset. The other nine
  functions and the methods of std.regex.25 are.
- An unknown group in a replacement template or in `group_all` stops the
  program with an index trap instead of the std.regex.7 / std.regex.24
  message (native traps carry no custom message yet).
- A `regex.Regex(pat: ...)` written by hand (std.regex.2) is not built
  natively; the struct holds the compiled program.
- `\p{L}`, `\p{N}` and the case folding use the native runtime's character
  tables and a few fixed ranges, where the Rust engine uses the standard
  library: the two can differ on rare non-ASCII characters.

## Result

`tools/native-regex` runs the 364 lines of `bootstrap/tests/regex/vectors.txt`
and random patterns through both engines: all agree (62,000+ fuzz vectors in
this change). `regex_scan` (100,000 lines, Linux x86-64): see the pull
request for the table.

## Why not

- **Runtime code in `compiler/rt/native`:** the same work in a language of
  ints and pointers, with no structs; harder to read and to keep in step.
- **A backtracking matcher:** breaks std.regex.4 on `(a*)*b`.
- **A lazy DFA now:** faster, but captures and lazy repetition still need the
  Pike VM; it stays a later optimisation.
- **Replacing the Rust engine in the interpreter with this one:** the
  interpreter would run the matcher interpreted, an order of magnitude slower.
