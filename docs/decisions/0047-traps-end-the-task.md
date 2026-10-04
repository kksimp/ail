# 0047. A trap ends the current task; its parent observes it; a trap in the main task ends the program

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 8 — Error handling; 11 — Concurrency; 22 — Runtime
- **Spec rules:** [err.trap.1–3](../spec/08-errors.md)

## Context

ADR 0035 and ADR 0045 make bugs trap. What a trap *does* affects fault
tolerance (servers), predictability, and runtime design.

## Options considered

### Option A — End the task; the parent observes it (chosen)
- **Pros:** A bug in one request doesn't take down a whole server. Fits
  task-local heaps (the failed task's heap is freed in O(1), per ADR 0012).
  Data-race freedom (ADR 0025) limits the shared state that could be left
  inconsistent.
- **Cons:** Locks held by the failed task need "poisoning". This is a weaker
  guarantee than stopping everything.

### Option B — Abort the whole program
- **Pros:** The simplest and most predictable.
- **Cons:** One bug kills an entire server.

### Option C — Catchable anywhere
- **Pros:** Flexible.
- **Cons:** Exceptions by another name. Contradicts ADR 0045.

## Decision

1. A trap stops the current task. Its deferred cleanup runs.
2. The task that spawned or awaits it receives a **trapped** outcome carrying
   the full diagnostic (kind, location, stack trace, spec rule ID where
   applicable).
3. A trap in the program's main task (or one that nothing observes up to the
   main task) ends the program with the diagnostic and a non-zero exit status.
4. A trap can't be intercepted *within* the task that trapped.

## Consequences

- Concurrency design inputs: how a trapped outcome is represented in the
  spawn/await API. Synchronisation primitives held by a trapped task must
  become **poisoned**, so later users see an error instead of inconsistent
  state. Supervisor patterns build on this.
- Shared mutable state behind locks is the main risk area. Poisoning is the
  mitigation.
- A process-wide "abort on any trap" mode may be offered as a build or
  runtime option for programs that prefer it. To be asked when the runtime
  is designed.
- The glossary definition of "trap" is updated.

## Rejected alternatives

- **Abort always:** poor fault tolerance for servers.
- **Catchable anywhere:** reintroduces hidden control flow.
