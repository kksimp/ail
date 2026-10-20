# 0275. `ail run --parallel`, `AIL_THREADS` and `AIL_PAR_LOG`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 — Concurrency; 20 — AI semantic API
- **Spec rules:** [conc.par.8–9](../spec/11-concurrency.md)
- **Relates to:** [ADR 0207](0207-deterministic-mode.md) (`--deterministic`)

## Context

Lane 39 proves many loops independent without `par`. Running all of them on
the pool by default would copy captured values for loops too small to gain;
the interpreter has no cost model (ADR 0238's is for the native runtime). An
agent also needs to see whether the pool was used.

## Decision

- `par for` uses the pool by default. `ail run --parallel` adds every loop the analysis proves independent.
- `AIL_THREADS=N` sets the pool size (1 turns it off); `--deterministic` turns it off.
- `AIL_PAR_LOG=1` prints one line per pooled loop or task on standard error: `par: line 23: loop: 200 iterations on 4 threads`, `par: line 22: task \`work\` on its own thread`.

## Rationale

Criterion 4 (speed): `par` marks the loops worth the copy. Criterion 2: the
log lets an agent verify parallel execution without timing it.

## Consequences

- The native runtime will parallelise proven loops by default with its cost model (ADR 0238); `--parallel` is the interpreter's stand-in.

## Rejected alternatives

- **Pool every proven loop by default:** small loops get slower.
- **A `--threads` flag:** an environment variable also reaches `ail test` and child runs without changing their command lines.
