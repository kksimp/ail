# 0462. Every pure-subset program is built and diffed against both interpreter tiers

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 28 — Testing strategy
- **Spec rules:** none (a tool); harness rule ID `fuzz.mod.6`

## Context

The native backend ([lane 120](../design/finish-plan.md)) is written in AIL
and is the only thing that can beat C. A wrong native binary is worse than a
slow one, and the interpreter is the permanent oracle ([ADR 0032](0032-keep-rust-interpreter-as-oracle.md)).

## Options considered

### Option A — differential in the existing selftest and fuzzers (chosen)
- **Pros:** one mechanism; every case, example and fuzzed program already runs under both tiers, so native is a third column.
- **Cons:** selftest slows by the build time; mitigated by building only programs the backend accepts.

### Option B — a separate native test suite
- **Cons:** two lists to keep in step; programs the suite forgets are untested.

## Decision

- **`fuzz.nat.1`** `fuzz_native_differential` in `bootstrap/tests/fuzz_modules.rs`
  generates pure-subset programs (`fuzz/pure.rs`: integers, floats, lists of
  integers, `if`, `for`, `while`, functions; arithmetic may overflow or divide
  by zero on purpose). Each runs on the walker, on the bytecode tier and as
  `ail build FILE -o OUT` output, and must print the same stdout, exit the same,
  and trap with the same first stderr line (ADR 0405). With a `qemu-aarch64` emulator
  installed (`apt install qemu-user`), the `--target aarch64-linux` build runs under
  it and is compared too. Programs the backend
  declines are counted, and the test fails if fewer than half build, so the
  generator and the subset cannot drift apart silently.
- **`fuzz.nat.2`** `fuzz_native_strings` does the same for programs over `str` and `char`
  (`fuzz/pure.rs::strings`: slicing, case, padding, replace, split, interpolation with
  width and fill, character predicates). Programs the backend declines are reported,
  not failed, until the feature has merged on every branch. `fuzz_native_maps`, `fuzz_native_options`, `fuzz_native_enums` and `fuzz_native_funcs` cover maps and sets, enums and `match` (ADR 0602-0603), optionals
  and results, and function values with key sorts.
- **`fuzz.mod.6`** A module-fuzz template that carries `# fuzz-native: yes` is
  built and compared too. None do yet: module code uses strings, maps and
  optionals, which are outside the subset (ADR 0390).
- The smoke run builds 6 programs (about 8 s); `AIL_FUZZ_N=N` builds N/5, and each
  build costs about 1.3 s because the backend runs on the interpreter.
- Lane 120 compares `bootstrap/tests/cases` and `examples` in `cases.rs`; this
  test covers what no one wrote by hand. The pure generator grows with the subset.

## Rejected alternatives

- A separate native test suite (option B): rejected because it means two lists to keep in step, and programs the suite forgets go untested.
