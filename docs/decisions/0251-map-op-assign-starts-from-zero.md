# 0251. `m[k] += v` on a missing key starts from zero

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Collections; token efficiency
- **Spec rules:** type.map.3 (amended), type.map.7

## Context

Counting is the commonest map update in agent code. AIL wrote
`m[k] = (m.get(k) else 0) + 1`, often with a `key :=` line so the key isn't
built twice; Python writes `counter[k] += 1` with `Counter`. ADR 0131 made
`op=` on a missing key trap.

## Options considered

### Option A — `op=` on a missing key starts from zero (chosen)
- **Pros:** −27.6 tokens (−1.6% of the corpus, −4.9% of `logscan`); no new
  name; the form models already write (Go's `m[k]++`, Python's
  `Counter`/`defaultdict`).
- **Cons:** a misspelt key no longer traps on `+=`.

### Option B — `m.count(k)` / `m.add(k, n)`
- **Pros:** explicit at the call.
- **Cons:** a new method per operation for the same tokens; `m[k] += 1` is
  what a model writes first, and it would keep trapping.

### Option C — a `Counter` type
- **Cons:** a new type for one operation; reading a missing key still differs from Python.

## Decision

When `m[k]` is the whole target of an arithmetic `op=` and `k` is missing,
the operation starts from the value type's zero and inserts `k`. Reading
`m[k]` and every longer place (`m[a][b] += 1` with `m[a]` missing) still trap.

## Rationale

Tokens (−1.6%) are under the 2% threshold, so **AI understanding** decided:
the update reads as written, needs no helper name, and matches what models
produce from Go and Python. The lost trap only ever caught a typo in a key
that the program was about to create anyway.

## Consequences

- Reads still trap, so a misspelt key is caught where it's used.

## Rejected alternatives

- `m.count(k)`: same tokens, a new name, and the natural form keeps trapping.
- `Counter`: a new type for one operation.
