# 0032. Keep the Rust interpreter permanently as an independent oracle

- **Status:** Accepted (amended by [0359](0359-bytecode-tier-in-the-bootstrap.md): a bytecode tier may sit beside the tree walker, which stays the oracle)
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 28 — Testing strategy; 23 — Compiler architecture
- **Spec rules:** none
- **Resolves:** OQ-022

## Context

Under staged self-hosting ([ADR 0022](0022-staged-self-hosting.md)), the
compiler eventually becomes all AIL. The interpreter, written in Rust
([ADR 0023](0023-rust-bootstrap.md)), is the differential-testing oracle
required by [ADR 0019](0019-miscompilation-defence.md).

## Options considered

### Option A — Keep the interpreter in Rust permanently (chosen)
- **Pros:** Two independent implementations in different languages catch
  bugs that shared code would hide (N-version testing).
- **Cons:** A permanent *development-time* Rust dependency. Two
  implementations to keep conformant.

### Option B — Port it to AIL
- **Pros:** All AIL.
- **Cons:** Shares language and tooling with the compiler, so a common bug
  can hide from both.

### Option C — Retire it after self-hosting
- **Pros:** Least maintenance.
- **Cons:** Removes the main miscompilation defence.

## Decision

The Rust interpreter is a **permanent reference implementation** used for
differential testing. It is never shipped to users.

## Consequences

- The interpreter must track the spec as it evolves. Conformance tests run
  against both implementations.
- When the interpreter and the compiler disagree, the **spec** decides which
  one is wrong. The interpreter is not automatically right.
- Rust stays a development-time dependency of the project (not of AIL's
  products). ADR 0023's consequences are updated accordingly.

## Rejected alternatives

- **Port to AIL:** loses implementation independence.
- **Retire:** loses the oracle that ADR 0019 depends on.
