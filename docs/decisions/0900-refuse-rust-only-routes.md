# 0900. `./ail` refuses routes only the Rust bootstrap could run

- Status: Accepted
- Date: 2026-10-24
- Decided by: AI (delegated, ADR 0076)

## Context

Step D of the Rust removal empties the Rust block of `./ail`. Every route is now
native except two that need the Rust bootstrap by their nature:

- `ail test --native [PATH...]` (native.test.1, ADR 0679) builds each `@test`
  function natively and compares its verdict with the interpreter's. With no
  interpreter there is nothing to compare with.
- `ail selftest --test NAME` runs one cargo test of `bootstrap/tests`. Those tests
  are ported to the gate suites (`ail selftest SUITE`) or deleted with the crate.

The other routes the inventory listed are native: `effects --parallel` and
`check --errors` are ported (conc.par.6, err.paths.1), `check --json`,
`check --budget`, `selftest --fuzz N [SEED]` and the gate suites were already
native, and `doc` and `budget` are not commands of the Rust CLI (`budget` is the
`--budget` flag of `check` and `run`).

## Decision

`./ail` refuses both routes with exit 2 and a message that names this ADR:

- `ail test --native` says `ail test` runs every test natively. native.test.1 and
  native.test.4 are retired; native.test.2, 3 and 5 stay as the description of the
  native runner. `ail test`'s `ok`, `FAIL` and `skip` lines are its verdicts.
- `ail selftest --test NAME` says to name a gate suite or run `ail test FILE`.

`selftest --fuzz` stays: it runs the AIL fuzzers (`tools/fuzz/suite.ail`,
`modules.ail`), which need no Rust.

`AIL_RUST=1` still reaches the Rust binary until step D4 deletes it.

## Alternatives rejected

- **Port the comparison to a native-versus-native runner.** There is a single
  engine, so the two sides would be the same build.
- **Keep the routes working through `AIL_RUST=1` after D5.** The crate is deleted
  at D5; a route that cannot run must say so.
- **Silently fall back to `ail test`.** The designer's rule is no silent fallback
  (2026-10-23).
