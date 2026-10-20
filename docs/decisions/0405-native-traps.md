# 0405. Native traps print the interpreter's first line and exit 1

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)**
- **Topic:** Native runtime and ABI (lane 122)
- **Spec rules:** [err.trap.3](../spec/08-errors.md#traps), [type.num.7](../spec/04-types.md), [type.coll.4](../spec/04-types.md), [type.str.12](../spec/04-types.md)
- **Design note:** [native-abi.md](../design/native-abi.md)

## Context

What a trap in a native program does (err.trap.3).

## Options considered

A: only exit 1. B: the first line `FILE:L:C: trap[RULE]: MESSAGE` and exit 1 (chosen). C: full parity with the source excerpt and stack trace.

## Decision

B. The compiler emits a trap record per check site {file, line, column, rule, message}; `rt_trap` flushes stdout, writes the line to stderr and exits 1. The differential harness compares exit status, rule ID and that line. The excerpt and stack trace need frame metadata; that is a later ABI addition once the backend has debug info (ADR 0016).

## Rejected alternatives and criterion

Criterion: AI understanding (the rule ID is what an agent acts on) over parity of decoration, which costs code size in every executable now. Rejected: A loses the rule ID; C needs metadata that does not exist yet.

## Consequences

The ABI note and `compiler/rt/rt.ail` are the source of record; lane 120's backend emits code against them. Revisit when the collector, frame metadata or `unsafe` pointers land.
