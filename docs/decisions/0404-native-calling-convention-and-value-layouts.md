# 0404. Native values: C-compatible calling convention, fixed layouts, runtime written in AIL on a machine model

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)**
- **Topic:** Native runtime and ABI (lane 122)
- **Spec rules:** [err.trap.3](../spec/08-errors.md#traps), [type.num.7](../spec/04-types.md), [type.coll.4](../spec/04-types.md), [type.str.12](../spec/04-types.md)
- **Design note:** [native-abi.md](../design/native-abi.md)

## Context

Which calling convention and value layouts native executables use, and how the runtime is written before raw pointers exist.

## Options considered

A: own convention. B: System V and AAPCS64 (chosen). C: stack-only convention.

## Decision

B. Native AIL functions use System V AMD64 and AAPCS64, so `extern` calls (ffi.call.1) need no thunk and a debugger and profiler understand the frames. `int` is i64, `float` f64, `bool` and `char` are i64 in registers, `str` points to an immutable object {bytes, chars, UTF-8 data}, a list points to {len, cap, data pointer}. `chars` is stored so `s.len()` stays O(1) (ADR 0208). The runtime is AIL in `compiler/rt/rt.ail`, written against a byte-addressed machine model (`Mem`) with `load8/store8/load64/store64`, so it runs and is tested under the interpreter now; when lane 126's raw pointers land those helpers become `unsafe` operations and nothing above them changes.

## Rejected alternatives and criterion

Criterion: speed (equal-footing with C) and AI understanding (one convention to know). Rejected: an own convention (needs thunks at every `extern` and tool support from scratch); stack-only (slower for no gain).

## Consequences

The ABI note and `compiler/rt/rt.ail` are the source of record; lane 120's backend emits code against them. Revisit when the collector, frame metadata or `unsafe` pointers land.
