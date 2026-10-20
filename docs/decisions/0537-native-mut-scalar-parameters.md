# 0537. Native `mut` parameters of numbers, `bool` and `str`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), [own.param.2](../spec/07-ownership.md), [own.excl.1](../spec/07-ownership.md)
- **Builds on:** [ADR 0061](0061-call-site-mut.md), [ADR 0392](0392-native-structs-and-methods.md)

## Context

A `mut` parameter of a list or struct already worked natively: the callee
gets the caller's object and changes it in place. A `mut` parameter of an
`int`, `float`, `bool` or `str` was rejected, because the value lives in a
register or spill slot of the caller, which the callee can't reach. The
native coverage survey counted 29 such problems across the examples and
cases.

## Decision

1. **Copy in, copy out.** For each such parameter the caller allocates an
   8-byte **frame cell** after its spill slots (MIR op `cell`, numbered per
   function), stores the variable in it, passes the cell's address, and
   loads the variable back after the call. The callee loads the value on
   entry into an ordinary local and stores it back before every return.
2. Because a `mut` argument is exclusive for the call (own.excl.1), nothing
   else can observe the variable while the callee runs, so copy-in/copy-out
   behaves as the interpreter's in-place update.
3. Only a local variable may be the `mut` argument for now; a field or
   element (`mut b.n`, `mut xs[i]`) is reported under native.subset.1.
4. Cells cost no allocation: they are part of the frame, and a call site
   reuses its cell each time it runs.

## Consequences

- Recursion with `mut` counters and helpers that update several numbers
  build natively. Calls without `mut` scalars are unchanged.
- Each such argument costs a store and a load around the call, and a load
  and a store in the callee; inlining removes the call but not yet the
  cell traffic.
- The ARM64 generator addresses cells with a 12-bit `add`, so a frame with
  more than about 500 spill slots stops with an internal error instead of
  miscompiling.

## Rejected alternatives

- **A heap box per call.** Simple, but allocates on every call, and the
  bump allocator never frees until the collector (lane 171) lands.
- **Return the new values with the result** (multiple returns). No memory
  traffic, but MIR has one result per call and every caller and return
  would change shape.
- **Keep the variable in memory for its whole life** (an address-taken
  local). Every use would load and store, not just the call.
