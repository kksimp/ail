# 0741. A `ref` is not passed to a spawned task (own.ref.2)

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 7 — Ownership
- **Spec rules:** own.ref.2, conc.share.2
- **Builds on:** [ADR 0063](0063-use-site-ref.md), [ADR 0100](0100-task-communication.md)

## Context

Lane 238 checked each spec rule that the lane 223 sweep marked `skip=bootstrap`.
Six of the eight were already implemented and only lacked a runnable example.
`own.ref.2` was not: `spawn f(c)` with `c: ref T` was accepted and ran, so two
tasks could reach one cell.

## Decision

The checker rejects, at the argument, a `spawn` of a declared function or
associated function whose parameter type is or holds a `ref`: a `ref T`, a list,
set, map, optional or result of one, or a struct or enum with such a field
(depth-limited, so `List[ref Node]` inside `Node` ends the search). The message
points to `Channel` and `Mutex`. Values wrapped in `Mutex` or `Atomic` are
not `ref`, so they pass. The test is on the declared parameter types, so it
needs no second check of the arguments.

## Rejected alternatives

- **Check the argument expressions' types.** Rechecking an argument after the
  call would repeat its move and borrow effects; the signature has the same
  information.
- **Reject only a bare `ref T`.** A `List[ref Node]` reaches the same cell.
- **A run-time check on cell access from two tasks.** Costs every cell access;
  the compile-time rule is complete for `spawn` and free for programs that obey it.

## Not done

`err.type.4` (needs OQ-029, matching on an error set) and `err.ctx.3` (the report
layout has no program-visible form) stay `skip=bootstrap`. `ail fix` adds no fix when
the unhandled call is followed by a trailing `# comment` or is an argument of
another call (`err.fix.1` gap; reported on the PR).
