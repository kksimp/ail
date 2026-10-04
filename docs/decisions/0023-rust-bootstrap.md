# 0023. The bootstrap front end and interpreter are written in Rust

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none
- **Resolves:** [OQ-010](../open-questions.md)

## Context

Under staged self-hosting ([ADR 0022](0022-staged-self-hosting.md)), the
front end and interpreter must initially be written in an existing language.
The custom-tooling policy ([ADR 0009](0009-custom-tooling-policy.md)) governs
AIL's *product*. The bootstrap code is temporary scaffolding that gets
replaced by AIL code.

## Options considered

### Option A — Rust (chosen)
- **Pros:** Fast and memory-safe. Excellent sum types and pattern matching.
  Systems-level like AIL. Mature fuzzing and testing tools.
- **Cons:** Slow builds. Graph-shaped data needs arena or index patterns.

### Option B — Zig
- **Pros:** Closest to the "own everything" ethos.
- **Cons:** Pre-1.0 churn, less safety, fewer testing tools.

### Option C — OCaml
- **Pros:** Ideal for type checkers.
- **Cons:** Far from AIL's domain, and a smaller pool of contributors.

### Option D — C
- **Pros:** Compiles everywhere.
- **Cons:** No safety and no sum types. Slow to get right.

## Decision

The bootstrap front end (lexer, parser, type checker, early semantic API) and
the interpreter are written in Rust.

## Consequences

- Rust's own toolchain is a **development-time** dependency. AIL's shipped
  products never include Rust-compiled code once the front end has been
  ported (ADR 0022 step 3). The interpreter stays in Rust permanently as a
  test oracle ([ADR 0032](0032-keep-rust-interpreter-as-oracle.md)).
- Rust dependencies (crates) are kept to a minimum, in the spirit of ADR 0009.
  Each one needs a stated justification.
- The bootstrap code should be written to make porting to AIL easy: plain data
  structures, arena-indexed graphs, and no Rust-specific cleverness.

## Rejected alternatives

- **Zig:** instability and weaker safety for a large codebase.
- **OCaml:** weaker fit for a systems-language project.
- **C:** too slow to write correctly.
