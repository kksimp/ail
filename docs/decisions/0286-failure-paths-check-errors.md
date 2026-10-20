# 0286. `ail check --errors`: every failure path of a function as JSON

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI-first tooling
- **Spec rules:** `err.paths.1`

## Context

An agent changing a function needs to know what can fail inside it, and where each failure goes. `ail api` shows signatures only, and reading bodies costs tokens.

## Options considered

### Option A — a `--errors` view of `ail check` (chosen)
- **Pros:** the checker already knows each `?`, `else` and returned error with its types; no new analysis; the data is exact, including inferred sets.
- **Cons:** one more flag.

### Option B — put paths into `ail api`
- **Pros:** one command.
- **Cons:** the outline gets large for the common case that doesn't need paths.

## Decision

`ail check --errors FILE` prints one JSON line: for each function that can fail or touches a failure, its `fails_with` list and its paths in source order. A path has `line`, `col`, `kind` (`propagate`, `handle`, `return`, `absent`), `errors`, and where they apply `how`, `call` and `context`. The checker records paths on its final pass (`check/errpaths.rs`); `api/errors.rs` prints them. Traps aren't failure paths.

## Rationale

Token usage: the view is the answer to "what can fail here" without reading bodies, and an agent asks once per function it changes. Speed: recording is a push per `?` or `else`, below measurement noise.

## Rejected alternatives

- Paths inside `ail api` (Option B), as above.
- Listing traps (overflow, bounds): every arithmetic and index operation is one, so the list would drown the failures.
