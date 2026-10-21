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
  program with `unreachable: std.regex.7: ...` (or `std.regex.24`) and the
  interpreter's message text, but the first line has the `unreachable`
  header and no source position of the call.
- A `regex.Regex(pat: ...)` written by hand (std.regex.2) is not built
  natively; the struct holds the compiled program.
- `\p{L}`, `\p{N}` and the case folding use the native runtime's character
  tables and a few fixed ranges, where the Rust engine uses the standard
  library: the two can differ on rare non-ASCII characters.

## Addendum: DFAs built at compile (follow-up pull request)

Matching spends most of its time in the Pike VM's thread lists. When the
pattern has no assertion (`^`, `$`, `\b`, `\A`, `\z`) and fewer than 1,500
instructions, `compile` also builds, by exploring every state over the 128
ASCII characters (in classes of characters no instruction tells apart):

- a **forward DFA** whose states are the VM's thread lists in priority
  order, cut at `Match`, with a fresh thread appended while nothing has
  matched; running it gives where the leftmost-first match ends;
- a **reverse DFA** over the reversed program, run backwards from that end,
  whose last accepting state gives where the match starts.

Both are exactly the VM's choices, so results are unchanged; a search that
needs capture groups runs the VM only from the start the DFAs found. A
pattern needing more than 256 states in either DFA, one with an assertion,
and a search that meets a character outside ASCII use the VM as before
(still linear). The DFAs are built eagerly because a `Regex` is an immutable
value, so a cache filled by searches has nowhere to live; the cost is
bounded by the state limit and is a few thousand instructions for ordinary
patterns. The design is the standard forward-then-reverse scheme of RE2; no
code was taken from it or from `regex-automata`.

`str.chars()` on a string with one byte per character now stores the bytes in
place (`compiler/rt/native/str.ail`), which the matcher calls once per search.

## Result

`tools/native-regex` runs the 364 lines of `bootstrap/tests/regex/vectors.txt`
and random patterns through both engines: all agree (62,000+ fuzz vectors in
this change). `regex_scan` (100,000 lines, Linux x86-64, wall): `ail run`
2.46 s; native VM only 1.1-1.3 s; native with the DFAs 0.39 s; Python `re`
0.82 s; C (`regexec`) 0.26 s.

## Why not

- **Runtime code in `compiler/rt/native`:** the same work in a language of
  ints and pointers, with no structs; harder to read and to keep in step.
- **A backtracking matcher:** breaks std.regex.4 on `(a*)*b`.
- **A lazy DFA filled during searches:** the same machine, but it needs a
  mutable cache inside an immutable `Regex`; the DFAs above are built when
  the pattern compiles instead. Captures and the fallbacks keep the VM.
- **Replacing the Rust engine in the interpreter with this one:** the
  interpreter would run the matcher interpreted, an order of magnitude slower.
