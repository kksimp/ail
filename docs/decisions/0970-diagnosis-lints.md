# 0970. Diagnosis lints: `ail check` warns on four bug shapes no tool flagged

- **Status:** Accepted
- **Date:** 2026-11-06
- **Decided by:** AI (delegated, ADR 0076), from the designer's go-ahead to build into `ail` what would help an AI diagnose the diagnosis benchmark's bugs (2026-11-06); number confirmed by the integrator
- **Builds on:** [ADR 0379](0379-allow-attribute.md) (`@allow`), [ADR 0967](0967-sanitizers.md) (`ail sanitize`)
- **Topic:** 8 — Errors, 11 — Concurrency, 7 — Ownership
- **Spec rules:** err.else.7, conc.share.8, expr.loop.8, own.defer.4, san.sched.3

## Context

The diagnosis benchmark (`/mnt/project-files/diagbench/`) planted five bugs in one program written in AIL, C, Python and Rust. A blind diagnoser found all five from source in every language, but no tool pointed at four of them: a parse failure swallowed by `else 0`, a slot taken and given back only on the success path, a floor-divided batch count that drops the last batch, and a get-then-set on a mutex that loses updates. `ail check` and `ail fmt --check` passed. Plain `ail run` printed the same wrong total three times, which hid the race. Only `ail sanitize` said something, and it did not say which value raced.

## Decision

Four warnings from `ail check`, in `compiler/check/lint.ail`. Each reads the syntax of one function body, reports at the line that carries the defect, carries a `help` that names the fix (in `--json` too) and is allowed per function with `@allow("RULE")`:

- **err.else.7**: `x.to_int() else d` or `x.to_float() else d` with a non-negative default, in a function that also rejects a failed parse of the same kind (`else return`, `else continue` or a block). The inconsistency is the signal: it is cheap to state and almost never intended. A note points at the parse that rejects.
- **conc.share.8**: `m.set(e)` on a `Mutex` where `e` uses a value read from `m` with `get()`, directly or through a local. The help gives `with m as mut x` (conc.share.6).
- **expr.loop.8**: `for b in 0..xs.len() / N` with `N` a name and `b * N` in the body. The help gives the ceiling and says to clamp the last batch.
- **own.defer.4**: `p -= e` (or `+=`) directly in the body, undone by the opposite adjustment later in the body, with an exit (`return`, `?`, `else return`) between that does not undo it. The help gives `defer p += e`. AIL has `defer` (own.defer.1); the bug exists because nothing nudged the author towards it.

`ail sanitize` (san.sched.3): when schedules differ it also runs `ail check --json` and prints a note for each `conc.share.8` warning, so the schedule-dependent result comes with the likely racing mutex and line.

All four are warnings, so no program that checks today stops checking. Where they fired in this repository (`std/regex` `P.group`, `examples/interp.ail`, an error path that aborts the parse or the program anyway) the function carries `@allow("own.defer.4")`.

## Consequences

The lints are syntactic and local: they miss a leak through a helper, a mutex hidden in a struct, and a loop bound computed elsewhere. They are tuned to have few false positives (a negative sentinel such as `else -1` is not flagged; a literal divisor is not flagged) at the cost of missing some real bugs. `@allow` is the escape hatch and names the rule, so the intent is visible to the next reader. A GC leak report stays with `ail leak` (ADR 0969); own.defer.4 covers the counter-style leak that no collector sees.

## Rejected alternatives

- **Warn on every `else <default>` after a parse:** 100+ sites in this repository use it on purpose (ports, counts, flags); the warning would be ignored. Requiring a sibling that rejects keeps it to a contradiction.
- **A type-aware data-flow pass for the mutex pattern:** needs the checker's types at every use and gains little: `get` then `set` on the same name is the pattern in practice.
- **Make `ail run` print the nondeterminism:** a plain run is deterministic by design (conc.sched.3); `ail sanitize` is where schedules are explored.
- **Auto-fixes for these warnings:** each fix changes meaning (which error to return, where the critical section ends, how the last batch is clamped), so the help names the fix and leaves the choice.
- **A general resource-pairing lint (`open`/`close`, `acquire`/`release` by name):** names are not a contract; the counter form needs none.
