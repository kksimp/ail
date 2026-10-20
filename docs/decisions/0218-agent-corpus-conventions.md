# 0218. Conventions for the agent-tool corpus programs

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 28 Testing strategy (benchmarks)
- **Spec rules:** none (tooling)

## Context

The programs in [ADR 0217](0217-agent-corpus-standing-benchmark.md) touch the
file system and processes. They run from `cargo test` (cwd `bootstrap/`),
`ail run` (cwd anywhere), `tools/perfbench/run.sh` and `ail test`. Their
outputs must not change when other lanes add examples or fixtures.

## Decision

1. **Find data from the program's own path.** Fixtures live in
   `examples/data/` and are found through `env.program()`, never the working
   directory. The Python twin finds them from `__file__`.
2. **Leave no files.** A program that writes (`tasks.ail`) writes into
   `examples/data/`, reads the file back, and removes it. `@test` functions
   touch no files or processes, so tests can run in parallel. Because they
   find their fixtures from their own path, the formatter test checks these
   programs' layout is already canonical instead of running a relocated copy.
3. **Deterministic output that survives growth.** `build.ail` walks
   `examples/`, so it prints only findings plus an error and warning total,
   never a file count, and it exits 1 on errors via `io.exit`.
4. **Twins match exactly.** Same output byte for byte (checked by hand and
   in review), the same helper split, idiomatic Python (`Counter`,
   `json`, `subprocess`) rather than a line-by-line translation.
5. **Tokens are counted without the header comment**, like the rest of the
   corpus. The corpus copy of an example is the example minus that comment.
6. **Tests per program:** exactly two `@test` functions in AIL, and two
   `test_` functions in the twin, so both count the same testing code.

## Rationale

ADR 0076 criterion 2 (AI understanding): local, unambiguous programs with
fixed inputs. Rules 1 to 3 make the `.out` files stable under the parallel
overnight work; rule 4 keeps the comparison fair.

## Rejected alternatives

- **Fixtures relative to the working directory:** breaks under `cargo test`.
- **A temp directory for `tasks.ail`'s output:** `std.fs` has no portable
  temp directory (Windows).
- **File counts in `build.ail`'s report:** every new example would change
  `build.out`.
