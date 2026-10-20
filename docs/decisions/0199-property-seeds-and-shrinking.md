# 0199. Seed properties from the test name and shrink failures

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Testing, [ADR 0167](0167-test-functions-and-ail-test.md)
- **Spec rules:** `test.prop.3`, `test.prop.4`, `test.prop.5`

## Context

A property failure is only useful if it can be replayed and the input is small. Both need decisions: where the seed comes from, how inputs are simplified, and how the result is reported.

## Options considered

### Option A — name-hashed default seed, greedy shrinking (chosen)
- **Pros:** `ail test` is deterministic by default (a green run stays green, a red run stays red), the seed is always printed so `--seed S` replays, shrinking needs no stored state, and the shrunk input is printed as AIL literals.
- **Cons:** the default run explores the same 100 inputs every time; `--seed` is needed to explore others.

### Option B — a random seed each run
- **Pros:** more coverage over time.
- **Cons:** flaky by design; an AI agent that sees a pass then a fail cannot tell what changed.

### Option C — integrated shrinking per generator
- **Pros:** shrinks respect invariants.
- **Cons:** needs a generator DSL; type-driven shrinking covers every supported type.

## Decision

The default seed is a hash of the test's name; `--seed S` uses `S` for every property. Inputs grow with the case number. On a failure `ail test` shrinks greedily (ints go to 0 and halve, strings and lists empty, halve and lose elements, optionals become `none`, enums move to an earlier variant, struct fields shrink in place) within a budget of 2000 extra runs, then reports the smallest failing input as `name = literal, ...`, the cases run, and the seed. `--json` adds `cases`, `seed` and (on failure) `input` to the test's line.

## Rationale

ADR 0076 criterion 2 (AI understanding): determinism and replay make a failure something an agent can act on. Greedy type-driven shrinking is small to implement and gives the minimal inputs the brief asks for.

## Consequences

- A replay needs the same `--cases`, because the input size grows with the case index.
- Shrinking can stop at a local minimum; the budget bounds time.
- Text and JSON output of property tests carry the extra seed and cases information.

## Rejected alternatives

- Option B: flaky.
- Option C: needs a DSL.
