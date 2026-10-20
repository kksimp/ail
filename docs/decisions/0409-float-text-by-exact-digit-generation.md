# 0409. Float text by exact digit generation, written from the textbook

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)**
- **Topic:** Native runtime and ABI (lane 122)
- **Spec rules:** [err.trap.3](../spec/08-errors.md#traps), [type.num.7](../spec/04-types.md), [type.coll.4](../spec/04-types.md), [type.str.12](../spec/04-types.md)
- **Design note:** [native-abi.md](../design/native-abi.md)

## Context

How the runtime prints a `float` byte-identically to the interpreter (type.num.7).

## Options considered

A: Ryu or Grisu with tables (port). B: exact big-integer free-format digit generation (chosen). C: fixed precision with trimming.

## Decision

B. The runtime generates the shortest digits that read back, closest when several tie, with big integers in base 10^9 (Steele and White; Burger and Dybvig's exact variant), then lays them out per type.num.7. It needs only `int` and `float` arithmetic, and it is checked against the interpreter on 2,400 random floats across the whole exponent range plus the edge cases (subnormals, powers of two, 2^53+1, 1e22, -0.0, inf, nan). It is slower than Ryu; a table-driven version replaces it when the backend can embed 128-bit tables, with the same tests. Nothing is ported: no third-party code is in the runtime, which keeps it clear for the Runtime Exception.

## Rejected alternatives and criterion

Criterion: AI understanding (small, local, provably exact) over speed (formatting is not a hot path in the benchmark set). Rejected: A (tables, and licence provenance to audit); C (not round-trip shortest).

## Consequences

The ABI note and `compiler/rt/rt.ail` are the source of record; lane 120's backend emits code against them. Revisit when the collector, frame metadata or `unsafe` pointers land.
