# 0162. Optional prelude modules are read only when named

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Standard library; performance
- **Spec rules:** [json.prelude.1](../spec/std-json.md)

## Context

The bootstrap lexes, parses and checks the prelude before every program. The
`std.json` module is about 280 lines, and `examples/json.ail` declares its
own `Json`, `Member` and `ParseError`, which would collide with prelude types
(`mod.unit.1`).

## Options considered

### Option A — read the module only when the program names it and declares none of its names (chosen)
- **Pros:** programs that don't use JSON pay nothing (**speed**); existing programs keep their own `Json`.
- **Cons:** a pre-pass over the program's tokens; the rule is a little magic.

### Option B — always read it
- **Pros:** simplest.
- **Cons:** every run pays to parse and check it, and `examples/json.ail` breaks.

### Option C — let user declarations shadow prelude items
- **Pros:** general.
- **Cons:** needs shadowing semantics throughout the checker, for types, methods and impls. A large change to shared code.

## Decision

`bootstrap/src/prelude.ail` has a core section and optional sections, each
starting at a `## @module NAME` line. `parse_with_prelude` includes the `json`
section when the program's tokens contain one of its public names and no
`fn`, `struct` or `enum` declares one.

## Rationale

**Speed** and compatibility decided; tokens and understanding were ties.
Internal helpers are named `JsonParser` and `json_quote` so they cannot clash
with user code that doesn't use JSON.

## Consequences

- `hello.ail` runs in about the same time as before (see the PR body).
- Lane 4's real modules replace this mechanism.
- A program that both declares its own `Json` and wants the standard one gets only its own.

## Rejected alternatives

- **Always read:** breaks `examples/json.ail`, costs every run.
- **Shadowing:** too invasive for tonight's ten parallel PRs.
