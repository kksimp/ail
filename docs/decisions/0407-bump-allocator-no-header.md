# 0407. Bump allocator over mmap, no object header, collector later

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)**
- **Topic:** Native runtime and ABI (lane 122)
- **Spec rules:** [err.trap.3](../spec/08-errors.md#traps), [type.num.7](../spec/04-types.md), [type.coll.4](../spec/04-types.md), [type.str.12](../spec/04-types.md)
- **Design note:** [native-abi.md](../design/native-abi.md)

## Context

How native memory is allocated before the GC.

## Options considered

A: bump allocator, never frees (chosen). B: free-list malloc. C: a collector now.

## Decision

A. `rt_alloc(n)` returns zeroed 16-byte-aligned memory from `mmap`ed chunks (1 MiB, then doubling or the request). Objects have no header yet; the collector revision adds an 8-byte header word and the allocator's alignment already leaves room. Programs that allocate in a loop use memory until exit; that is the interim for the pure subset.

## Rejected alternatives and criterion

Criterion: speed (an allocation is an add and a compare) and token cost of the runtime. Rejected: B (nothing frees explicitly in AIL); C (needs type maps from the backend first, ADR 0012).

## Consequences

The ABI note and `compiler/rt/rt.ail` are the source of record; lane 120's backend emits code against them. Revisit when the collector, frame metadata or `unsafe` pointers land.
