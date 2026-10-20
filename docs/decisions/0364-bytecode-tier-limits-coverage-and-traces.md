# 0364. Run limits, coverage and call and effect traces run on the bytecode tier

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), within the designer's ADR 0359
- **Topic:** interpreter performance; tracing; run limits
- **Spec rules:** none changed; the tier keeps run.limit.1–3, test.cover.1 and trace.run.1 as the walker has them

## Context

Through slice 3 ([ADR 0363](0363-bytecode-tier-methods-and-data.md)) the tier
was off for a whole run whenever `--timeout`, `--max-memory`, `--max-steps`,
`--coverage`, `--trace` or `--watch` was given. So the runs where an agent is
investigating a program got none of the speed-up. Lane 111's slice 4 asks for
traps, limits and traces on the tier, with the same rule and position as the
walker, and for `--trace` either on the tier or as a per-call fallback,
measured.

The walker does all of this in three places:
- `hit` runs before every statement and every turn of a `while`. It counts a
  step, checks the watchdog's time and memory flag, and notes the line for
  coverage.
- `run_body` traces calls and returns, and `std_call` traces effects.
- `--watch` hooks each `let`, assignment and mutating method.

## Options considered

### Run limits and coverage

#### Option A — a `Hit` instruction where the walker calls `hit` (chosen)
When limits or coverage are on, the compiler puts `Hit(pos)` before each
statement it compiles and at the start of each `while` turn. It calls the
walker's own `hit`, so the rule, message and position are the walker's. A
statement handed to the walker as a whole takes its step again in the walker's
`stmt`, so `Stmt` gives the step back first (`unhit`).

- **Pros:** the same step count, trap position and snapshot as the walker,
  with no new limit code; nothing at all when no limit is on, since the
  instruction is only emitted then.
- **Cons:** one instruction per statement while limits are on.

#### Option B — count steps in the interpreter loop
- **Cons:** an instruction isn't a statement, so the counts differ from the
  walker's, and `--max-steps` would trap somewhere else (run.limit.3 promises
  the same place every time).

#### Option C — keep the tier off under limits (as before)
- **Cons:** `ail test --coverage`, and any run under `--timeout` or
  `--max-steps`, would never use the tier.

### Traces

#### Option A — leave call and effect traces where they are (chosen)
Call and return events come from `run_body`, which the tier shares, and effect
events come from the standard calls, which the tier hands to the walker. So
`--trace`, `--trace=calls`, `--trace=effects` and `--trace=all` print the same
events on the tier with no change.
`--watch` stays a whole-run fallback to the walker: it hooks every binding
and assignment, which the tier compiles.

#### Option B — fall back to the walker per traced call
- **Cons:** nothing to fall back from, since the events come from shared
  code; it would only lose speed.

## Decision

- With run limits or coverage on, the tier emits `Hit` as above.
- With `--trace` in any mode, the tier runs. With `--watch`, it doesn't.
- `bootstrap/tests/bytecode.rs` ([ADR 0362](0362-differential-harness-for-the-bytecode-tier.md))
  compares more. Every program without tasks or threads also runs with
  `--max-steps 300`, which stops most of them part way with a trap and its
  snapshot, and with `--trace=all`. Each `.test` file also runs with
  `--coverage`. A program's runs are made in turn, because some write files at
  fixed paths. Programs with tasks are left out of those runs: their step
  counts and trace order vary from run to run on either tier.

## Rationale

ADR 0359 is the designer's call for speed, and ADR 0076 ranks AI
understanding above it. Limits, traces and coverage are how an agent
investigates a program. They now keep their exact output and get the tier's
speed. Seeding a bug in `unhit` fails 4 runs of the harness.

## Consequences

- Linux x86-64, callgrind instructions, walker then tier
  ([interpreter-performance.md](../design/interpreter-performance.md)):
  - `loops --max-steps 1000000000`: 4,785M and 2,595M (1.84x).
  - `loops --trace=calls`: 3,962M and 1,883M (2.10x).
  - `interp`: 1.35x with `--max-steps` and 1.34x with `--trace=calls`.
  - Before this slice, these runs used the walker alone.
- `Hit` adds 38% to `loops` on the tier (the walker's `hit` adds 21% to the
  walker), because `loops` is almost all short statements. It costs nothing
  when no limit is set.
- The both-tier test takes about 10 s instead of 4 s.
- Still on the walker: `--watch`, `--deterministic`, `--schedule`, checked
  contracts, unique values and `defer`.

## Rejected alternatives

- Counting instructions as steps: the walker's counts and positions would
  change (run.limit.3).
- Keeping the tier off under limits: the runs an agent makes to investigate a
  program would stay slow.
- A per-call trace fallback: the events come from code both tiers share.
