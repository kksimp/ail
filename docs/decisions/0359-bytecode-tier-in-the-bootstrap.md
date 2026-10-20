# 0359. Add a bytecode tier to the bootstrap interpreter

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** Designer
- **Topic:** interpreter performance; 23 — Compiler architecture
- **Spec rules:** none yet (no behaviour changes; the tier must be unobservable)
- **Supersedes:** [ADR 0297](0297-no-bytecode-tier-in-the-oracle.md); amends [ADR 0032](0032-keep-rust-interpreter-as-oracle.md)

## Context

After three speed rounds the tree-walking interpreter runs `json` at 0.20x
and `sheet` at 0.28x of CPython's time, but `interp` at about 1.1x and
`loops` at about 1.0x. Its profile is flat (largest self cost 9%,
[interpreter-performance.md](../design/interpreter-performance.md)), so
only a second, compiled evaluation form can gain more than a few percent.
ADR 0297 kept the tree walker because ADR 0032 makes the interpreter the
single oracle, and a second evaluator inside it is the designer's call.
The designer was asked on 2026-10-20 and chose to start the work now.

## Options considered

### Option A — defer to the Stage 3 backend (ADR 0297)
- **Pros:** one evaluation model in the oracle.
- **Cons:** tight loops stay at CPython speed until the native backend exists.

### Option B — a bytecode tier inside the bootstrap, next batch
- **Pros:** scheduled, scoped.
- **Cons:** waits.

### Option C — start a bytecode tier now, in its own lane (chosen)
- **Pros:** the expected 2–3x on loops arrives first; the tree walker stays
  the reference and the bytecode tier is checked against it on every
  program (differential testing, ADR 0019).
- **Cons:** two evaluators to keep conformant; the tier must preserve the
  ownership, `ref`, task, pool, trace, contract and limit semantics exactly.

## Decision

Build a bytecode tier in the bootstrap interpreter, in a lane that opens a
pull request against `main`. The tree walker remains the oracle: it stays
the default for `--trace`, `--watch`, `--schedule`, `--deterministic` and
the differential harness, and every program in the test corpus must print
byte-identical output under both tiers. The tier is opt-out (`--no-bytecode`)
so a difference can always be bisected to one evaluator.

## Rationale

The designer weighs loop speed above the single-evaluator property for the
bootstrap, given that ADR 0019's differential testing already assumes two
implementations and that the tier can be switched off.

## Consequences

- ADR 0297's decision is superseded; its measurements stay valid.
- ADR 0032 is amended: the oracle is the tree walker, and the bytecode tier
  is a second implementation inside the same binary, tested against it.
- A new lane brief (`/mnt/project-files/lanes/111-bytecode-tier.md`) scopes
  the first slice: expressions, locals, calls and loops for functions with
  no tasks, `ref` cells or contracts, falling back to the tree walker
  otherwise.

- The tier's design and measurements: [ADR 0361](0361-bytecode-tier-design.md);
  method calls, data construction, `match` and `?`: [ADR 0363](0363-bytecode-tier-methods-and-data.md).

## Rejected alternatives

- Options A and B above, by the designer's choice.
