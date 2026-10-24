# 0207. `ail run --deterministic` denies the nondeterministic capabilities, and single-task programs are deterministic by construction (resolves OQ-046) (*delegated*)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Effects; reproducibility
- **Spec rules:** `eff.det.1–3`, `std.random.3`, `std.random.5`

## Context

[ai-first-plan.md §3.3](../design/ai-first-plan.md) wants two runs of a
program to give byte-identical output, so an agent can compare runs and
reproduce failures. OQ-046 asked whether that is a flag only or a language
guarantee.

## Options considered

### Option A — a guarantee for single-task programs, plus a flag that enforces it (chosen)
- **Pros:** the guarantee is cheap now (maps iterate in insertion order,
  `type.map.1`; no hash order is visible); the flag turns every violation
  into a compile-time error with the call chain (`eff.cap.1`).
- **Cons:** constrains the standard library and runtime forever: any new
  source of nondeterminism must be a capability.

### Option B — a flag only
- **Pros:** no promise to keep.
- **Cons:** programs can't rely on it, and the flag would have no stated meaning.

## Decision

Specify `eff.det.1`: a program whose `main` reaches none of the capabilities
`clock`, `random`, `proc` and `net` (and no `spawn`, which the bootstrap
doesn't have yet) produces the same output on every run given the same
inputs (arguments, environment, files, standard input). `ail run
--deterministic` (also on `check`) is `--deny clock,net,proc,random`.
`--seed S` seeds `std.random` so a program that uses `random` repeats
without being denied it. `fs`, `env` and standard input stay allowed: they
are inputs, and the guarantee is "same inputs, same output".

## Rationale

AI understanding: one stated rule an agent can check by running with the
flag. The criteria tie on tokens and speed.

## Consequences

- New nondeterministic sources (task scheduling, ASLR-visible addresses,
  floating-point fast-math) must either be capabilities or be defined away.
- Task scheduling will need its own rule when `spawn` lands (ADR 0095).
- Hash values are never visible (ADR 0133), so map order can't leak.

## Native port (2026-10-24)

`ail run --deterministic` checks `main`'s effects with `ail effects` before
building (`eff.cap.1`), exports `AIL_DETERMINISTIC=1` so the task runtime
runs one task at a time (`conc.sched.3`), and `--seed S` exports `AIL_SEED`,
which `std.random` reads at its first call (`std.random.3`).

## Rejected alternatives

- Option B: no meaning for programs to rely on.
- Denying `fs`/`env` as well: would forbid reading input, making the mode useless.
