# 0269. The fuzz harness generates typed programs, edge-value probes and token-level mutants from a seed

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 28 — Testing strategy (tool)
- **Spec rules:** none (a tool); harness rule IDs `fuzz.gen.1–4`, `fuzz.mut.1`
- **Related:** [ADR 0270](0270-fuzz-properties.md), [ADR 0271](0271-every-crash-becomes-a-case.md), [ADR 0085](0085-interim-local-testing-gate.md)

## Context

Twenty-two lane merges landed in one night, each with hand-written cases. A
generator finds the interactions nobody wrote a case for. Criterion
**AI understanding**: the harness must be small enough to read in one
sitting and fail with a seed that reproduces the program exactly.

## Options considered

### Option A — a seeded generator in Rust test code (chosen)
- **Pros:** no new dependency (ADR 0023 wants every crate justified); programs
  are a pure function of `AIL_FUZZ_SEED + i`, so any failure reproduces from
  one number; it runs under `cargo test`, so `./ail selftest` and
  `tools/verify.sh` run it unchanged on macOS.
- **Cons:** the harness drives the `ail` binary as a subprocess (about 15 ms
  per program), because the front end is in a binary crate, not a library.

### Option B — a coverage-guided fuzzer (cargo-fuzz, AFL)
- **Pros:** finds paths a grammar generator doesn't reach.
- **Cons:** an external tool and toolchain (ADR 0009 default is custom); needs
  nightly Rust or a C compiler; random bytes almost never get past the
  lexer, so the checker and interpreter would stay unexplored.

### Option C — grammar-free mutation of the existing corpus only
- **Pros:** trivial.
- **Cons:** the corpus is small and fixed; it can't make programs that
  check clean and exercise new combinations.

## Decision

`bootstrap/tests/fuzz.rs` with `fuzz/progs.rs` (generators and mutator) and
`fuzz/json.rs` (a validating JSON reader, ~100 lines).

- **`fuzz.gen.1` Typed programs.** A fixed prelude (a struct with methods, a
  payload enum, an error enum, a trait and `impl`, a generic `largest`, a
  contract function, a `?` chain) plus 0–3 helper functions and a `main` of
  4–13 statements, built from a typed expression generator over `int`,
  `float`, `bool`, `str`, `List[int]`, `List[str]`, `Map[str, int]`, `int?`,
  structs and enums. Statements: bindings, `mut` updates, `if`/`elif`/`else`,
  `for`, `while`, `match` as statement and value, `if x := opt`, `else err`,
  `spawn`/`wait`, `Channel`, `Mutex`, comprehensions, format specs,
  `break`/`continue`, early `return`. Names are fresh (no shadowing, no
  redeclaring), `while` counters can't be assigned (loops end), and the only
  calls are to earlier helpers (no recursion). Well-typed by construction:
  tonight 10,000 of 10,000 programs checked clean, and the harness fails if
  more than 2% don't.
- **`fuzz.gen.4` Probes.** 66 one-line templates that push edge values
  (`int` extremes, 0, −1, empty and non-ASCII strings, empty lists, NaN) into
  the operations most likely to trap: arithmetic, indexing, slicing,
  `repeat`, `pad_left`, sized-integer conversions, deep recursion, channels
  and mutex misuse. A trap is correct; a panic is the finding.
- **`fuzz.mut.1` Mutants.** Up to three stacked token-level edits of a
  generated program: delete, duplicate, swap, replace or insert a token from
  a pool, delete a line, shift indentation, truncate. Pieces re-concatenate to
  the source, so an unmutated edit is byte-identical.
- **Reproducing.** `AIL_FUZZ_SEED=N AIL_FUZZ_N=1 tools/fuzz/run.sh`; or
  `AIL_FUZZ_DUMP=N AIL_FUZZ_MUT=K cargo test --test fuzz dump -- --nocapture`
  prints a program and its first K mutants.
- **Minimising.** A failing input is shrunk by deleting chunks of lines while
  the *same* failure message persists (a hang is not a panic).

## Consequences

The smoke subset (200 programs, default) is what `./ail selftest` runs; the
full run is `tools/fuzz/run.sh` (10,000 by default).

## Rejected alternatives

Options B and C above; also a hand-maintained list of "interesting" programs
(it is what `tests/cases/` already is).
