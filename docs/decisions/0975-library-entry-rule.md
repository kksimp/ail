# 0975. Library entry: lazy runtime init, caller stack as GC root, serialised first slice

- **Status:** Accepted (first slice)
- **Date:** 2026-11-07
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); escalation criterion 3 applies to the serialised first slice, see below
- **Approval:** the serialised first slice was approved by the project coordinator on 2026-11-07, on the designer's delegation of technical calls; no designer card. The parallel second slice needs its own ADR.
- **Topic:** 14 — FFI
- **Spec rules:** `ffi.export.4-5` (to be written with the code)
- **Builds on:** [ADR 0950](0950-c-abi-exports.md), [ADR 0943](0943-per-thread-allocation-caches.md)
- **Design:** [systems-abi.md](../design/systems-abi.md)

## Context

A `pub export c fn` is entered by C on a stack and thread the runtime did not
create. The runtime assumes it owns the process: `rt_start` builds the globals
block, `STACK_TOP` is the collector's stack-scan root, and allocation caches
are chosen by `sp()` inside the `STACKS` region (ADR 0943).

## Decision

First slice (serialised):

1. Each export gets a generated entry stub (`mir/lower_export.ail`, runtime
   side `rt/native/libentry.ail`, prefix `libentry_`).
2. The stub takes one library lock, initialises the runtime on the first call
   exactly as `rt_start` does but on the caller's stack, sets `STACK_TOP` to the
   caller's `sp` when it is the outermost entry (restoring the old value on
   return), runs the function, and releases the lock.
3. A callback from C back into AIL (ADR 0951) nests under the same lock and
   keeps the outer `STACK_TOP`. A trap aborts the process; nothing unwinds into C.

This serialises all calls into the library. Callers on many threads get no
parallelism inside AIL code from the library. That costs runtime performance
the user cannot opt out of, so it is **a first slice only** and is flagged to the
designer.

Second slice (parallel entry, the target):

4. A foreign thread registers once as a mutator: it takes a thread slot (the
   per-thread allocation cache of ADR 0943, found by a thread-local pointer
   instead of `sp()` in `STACKS`), publishes its stack bounds, and then runs
   without the library lock. The lock then covers only runtime init and thread
   registration.
5. Stop-the-world GC scans every registered foreign stack from its published
   `sp` to its recorded base; a thread parked in C code (outside any export) has
   no AIL frames and is not stopped. A thread inside an export reaches a safepoint
   as workers do today. This needs a measured safepoint story for tight loops
   without calls, so it is its own ADR when started.

## Rejected alternatives

- Per-call serialisation as the final design: contradicts "use every core".
- Running AIL code on a runtime-owned stack and switching stacks per call: hides
  the caller's frames from the collector but costs a stack switch per call and
  breaks deep recursion limits.
- A conservative scan of the whole foreign stack: unbounded and wrong across
  guard pages.
- One runtime instance per calling thread: duplicates the heap and cannot share
  AIL values between threads.
