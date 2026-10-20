# 0206. Add `std.random` with a process-wide seedable generator (*delegated*)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Standard library; effects
- **Spec rules:** `std.random.1–4`, `eff.infer.4`

## Context

Programs need randomness (shuffles, sampling, tests), and the plan (§3.3)
needs it to be a capability so `--deny random` and `--deterministic` can refuse
it. The module must be small and typed like the other `std` modules.

## Options considered

### Option A — `random.int(lo, hi)`, `float()`, `seed(s)`, `shuffle(mut xs)` over one generator (chosen)
- **Pros:** four functions cover dice, probabilities, seeding and shuffles;
  `int` takes both bounds inclusive, as a person writing `1, 6` expects, so a
  call needs no `+ 1`; one hidden generator means `--seed` and `seed` fix
  every later draw.
- **Cons:** one global generator, so tasks (not yet implemented) will need a
  per-task rule; revisit with ADR 0095–0096 when tasks land.

### Option B — a `Rng` value passed explicitly
- **Pros:** no global state; trivially deterministic per value.
- **Cons:** an extra argument on every call chain (more tokens), and the
  capability becomes invisible to inference.

### Option C — `random.range(lo, hi)` with an exclusive upper bound
- **Pros:** matches `lo..hi`.
- **Cons:** the common dice case costs `+ 1`, a classic off-by-one.

## Decision

`use std.random` gives `int(lo, hi)` (both inclusive; traps if `lo > hi`,
`std.random.2`), `float()` in `[0, 1)`, `seed(s)` and `shuffle(mut xs)`
(Fisher-Yates, any list). All four give the capability `random`. The
generator is SplitMix64; unseeded it starts from the clock, and `seed`
(or `ail run --seed S`) makes it repeat exactly. The stream is specified
only as "a function of the seed", so a later release may change the algorithm.

## Rationale

AI understanding decided: the fewest arguments and no off-by-one. Tokens:
`random.int(1, 6)` is the shortest unambiguous dice call; Option B adds a
parameter to every function that draws. Speed is equal.

## Consequences

- `--deny random` works; `ail effects` shows `random`.
- Seeded runs are reproducible on all platforms (the generator is integer-only
  except `float`, which divides by 2^53 exactly).
- Per-task generators are future work.

## Rejected alternatives

- Option B: costs tokens on every chain and hides the capability.
- Option C: off-by-one trap for the commonest use.
