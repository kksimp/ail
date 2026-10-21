# 0781. A task that blocks on a child's stream while output arrives is not deadlocked

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 11 — Concurrency
- **Spec rules:** conc.task.7, std.proc.5
- **Builds on:** [ADR 0085](0085-interim-local-testing-gate.md)

## Context

`tier_matches_the_walker` failed intermittently with a `conc.task.7` deadlock in `std_proc_stream.ail`
while the child was alive. `recv`, `task.select` and a `for` over a child's stream look at the channel,
register as a waiter, then call `block`. `block` pumps the children's output once more. Output (and
end-of-stream) that arrived between the caller's look and that pump drained the task's waiter entry
without waking it, since it was running. With every stream then closed, `wait_external` found nothing
to wait for and `block` reported the deadlock while the data sat in the channel.

## Decision

`Sched::pump` returns the tasks it took off waiter lists. `block` returns to its caller at once when
`me` is among them; every caller already loops and looks again.

Regression tests: `interp::tasks::tests::output_that_arrives_before_blocking_is_not_a_deadlock` builds the
window exactly (it failed with the deadlock message before the change), and `cases/std_proc_select_burst`
runs 60 rounds of a child that answers, then writes to both streams and exits.

## Rejected alternatives

- **Count a live child as a pending producer in `wait_external`.** The child's liveness isn't what was
  wrong: the data was already in the channel, so this is a lost wake-up. Counting children would leave it
  lost and could hide a real deadlock behind a child that never writes.
- **Only retry the select in the interpreter.** `recv` and the stream `for` loop share `block`.
