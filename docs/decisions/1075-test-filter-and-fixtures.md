# 1075. `ail test --filter`, `--list` and test fixtures

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 19 — Testing
- **Spec rules:** [test.filter.1-3](../spec/19-testing.md), [test.list.1](../spec/19-testing.md), [test.fixture.1-4](../spec/19-testing.md)

## Context

`ail test` ran every test of a file. An agent fixing one failure had to rerun the whole file, and had no way to ask
which tests exist. Shared setup (a temporary directory, a prepared value) had to be repeated in each test, with
cleanup written by hand and skipped when an assertion trapped first. Chapter 19 listed both as not yet designed.

## Decision

**Filter and list.** `--filter PATTERN` (repeatable, any match) selects tests whose name contains the pattern; `*`
is a wildcard. Names are `NAME`, `MODULE.NAME` and `doc:NAME`. A pattern that matches nothing is exit 2, because a
mistyped filter that passes silently would hide every test. `--list` prints names (or `--json` lines) without
building. `--filter` with `--update` or `--mutate` is refused, since both need the whole run.

**Fixtures.** Three explicit parts, all checked by `ail check`:
- `@fixture fn tmp() T` or `!T`, optionally `@fixture("drop")` naming `fn drop(v T)`.
- `@test @needs("tmp") fn t(tmp T)`: each fixture is named by an attribute and received by a parameter of the same
  name and type. Other parameters stay generated inputs.
- Each run makes the fixtures in `@needs` order and tears them down in reverse order, after pass, fail, trap or
  error. A failed setup fails the test with the fixture's error at its position, and the line `context: setup of fixture F` under it.

The runner emits a wrapper `ail_fx_NAME` per test that sets up each fixture, `defer`s its teardown right after, and
calls the test. It reuses `defer`, so teardown order and trap behaviour are the language's own.

## Rejected alternatives

- **Match fixtures to parameters by name alone.** A mistyped parameter would silently become a randomly generated
  property input. The `@needs` attribute makes the intent explicit and checkable.
- **A fixture that yields (setup, yield, teardown in one function).** AIL has no generators or closures to build it
  on, and it makes the teardown's place in a trap unclear.
- **Teardown that can fail.** A failing `defer` has no defined result; an infallible teardown keeps the report to one
  cause per test.
- **Regular expressions for `--filter`.** More tokens and more ways to be wrong than substring plus `*`.
- **Fixtures with parameters (fixtures using fixtures), shared per file.** Deferred: per-test freshness is the
  isolation `test.run.2` promises, and dependency ordering can wait for a need.
