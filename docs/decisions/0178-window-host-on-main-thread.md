# 0178. The main thread hosts the window; the interpreter keeps the grid

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 22 — Runtime architecture (bootstrap)
- **Spec rules:** [std.ui.1](../spec/17-std.md), [std.ui.5–6](../spec/17-std.md), [std.ui.8](../spec/17-std.md)

## Context

The bootstrap runs a program on a spawned thread with a 256 MB stack
(main.rs). macOS allows windows only on a process's main thread, so the
window can't live on the interpreter's thread. The deciding criterion is
**speed** (criteria 1–3 are unaffected): a program that never opens a
window must cost nothing, and drawing must not cross threads per cell.

## Options considered

### Option A — the main thread serves window requests over a channel (chosen)
- **Pros:** works on every platform; the main thread pumps the window every
  8 ms while the program computes, so the window stays responsive and keys
  are queued in order; programs without a window cost one blocked `recv`.
- **Cons:** `present` copies the grid (80x30 cells is about 40 KB) across
  threads.

### Option B — run the interpreter on the main thread
- **Pros:** no channel.
- **Cons:** the main thread's stack is fixed by the OS (8 MB), far below the
  256 MB deep recursion needs.

### Option C — open the window on the interpreter thread
- **Pros:** simplest.
- **Cons:** fails or aborts on macOS.

## Decision

`ui::run_hosted` spawns the interpreter thread and serves `open`,
`present`, `poll`, `wait` and `close` requests on the main thread until
the interpreter thread ends. The interpreter thread keeps the cell grid, so
`put`, `clear`, `cursor` and `size` are local; `present` sends a copy
without waiting for a reply, and the host redraws only the cells that
changed.

## Rationale

Only this option satisfies macOS and the deep-recursion stack together.
Keeping the grid on the interpreter's side makes the common calls free of
synchronisation.

## Consequences

- `ail run` routes through `ui::run_hosted`; `ail test` does not, so
  `ui.open` in a test fails with `no_display`, which keeps tests headless.
- A program that ends closes its window (the host drops it).

## Rejected alternatives

- **Interpreter on the main thread:** its stack is too small for deep AIL recursion.
- **Window on the interpreter thread:** macOS forbids it.
