# 0726. A memory cap for native programs, checked when the heap takes a chunk

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** run.limit.2, run.limit.5, run.limit.6
- **Builds on:** [ADR 0288](0288-run-limits-timeout-memory-steps.md), [ADR 0407](0407-bump-allocator-no-header.md)

## Context

`ail run --max-memory MB` protects an agent from a program that grows
without bound (run.limit.2), but a binary from `ail build` had no cap and
could exhaust the machine.

## Decision

`rt.alloc` calls `limits_chunk` only when it maps a new chunk. The runtime
sums the bytes of earlier chunks that were used and traps, with the
interpreter's message, if that plus the request exceeds the cap
(`rt/native/limits.ail`, slots 49-51). Traps print the trap line and exit 1.
The check costs nothing on the allocation fast path. A `list.push` that grows
a list reaches it through `alloc`. The cap counts heap bytes in use, not
resident memory, so it is a little below what `ail run` measures for the same
program. When lane 171's collector replaces the bump allocator, the check
moves to its chunk request.

## Rejected alternatives

- **`setrlimit(RLIMIT_AS)`.** The kernel kills the program on a failed `mmap`
  with no message, and address space is not resident memory.
- **Read `/proc/self/status` as the interpreter does.** Linux only, and a
  read per check.
