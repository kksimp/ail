# 0408. Output is buffered in the runtime; errors are not

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)**
- **Topic:** Native runtime and ABI (lane 122)
- **Spec rules:** [err.trap.3](../spec/08-errors.md#traps), [type.num.7](../spec/04-types.md), [type.coll.4](../spec/04-types.md), [type.str.12](../spec/04-types.md)
- **Design note:** [native-abi.md](../design/native-abi.md)

## Context

How `print` reaches standard output.

## Options considered

A: one `write` per print. B: a 64 KiB buffer flushed at exit, trap and when full (chosen).

## Decision

B. `print` appends to the buffer; exit, a trap and a full buffer flush it; standard error is unbuffered. Interleaving of stdout and stderr in one terminal can differ from the interpreter, which flushes per line; the differential harness compares the streams separately.

## Rejected alternatives and criterion

Criterion: speed (a syscall per line dominates print-heavy loops). Rejected: A.

## Consequences

The ABI note and `compiler/rt/rt.ail` are the source of record; lane 120's backend emits code against them. Revisit when the collector, frame metadata or `unsafe` pointers land.
