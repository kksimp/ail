# 1115. Windows: loops the compiler chose run in order after short parallel regions

- **Status:** Accepted
- **Date:** 2026-11-14
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); found by the 0.1.0 Windows benchmark run
- **Topic:** 11 — Concurrency
- **Spec rules:** [conc.par.19](../spec/11-concurrency.md)
- **Builds on:** [ADR 0601](0601-native-par-for.md), [ADR 0946](0946-adaptive-thread-pool.md)

## Context

On Windows `spectralnorm` at 4000 took 1.115 s with four threads and 0.937 s
with one (ail-win, 2026-11-14, RC ccf77304); on Linux, on the same hardware,
934 ms falls to 308 ms. `AIL_PAR_LOG` shows the same 40 regions on both.

Windows has no futex in kernel32, and the PE writer imports from kernel32
only, so a waiter polls its word (`win_wait`): 64 `SwitchToThread` calls, then
`Sleep(1)`, which lasts until the next timer tick (about 15.6 ms). An idle
worker sees new chunks at its next tick and the main thread sees them done at
a later one, so a region costs about 28 ms whatever its work. A region whose
loop takes 23 ms in order is slower in parallel. One long region (`matmul`,
1.287 s to 0.191 s) is not hurt.

## Decision

`par_start` notes the tick count in the group and `par_wait` compares. Two
regions in a row that each end within 40 ms (`WIN_SHORT_MS`, `WIN_SHORT_RUN`)
make `par_worth` answer false for loops the compiler chose, until a region
takes longer. A written `par for` and the tensor kernels still use the pool.
Only Windows runs any of it. The loop in order is the path `AIL_THREADS=1`
takes, so no result changes.

## Rejected alternatives

- **`WaitOnAddress` and `WakeByAddress`.** The right fix: no polling. It needs
  the PE writer to import from a second DLL and every wait and wake of the
  runtime to move to it. For 0.1.1; this ADR's rule goes when it lands.
- **Spinning longer before `Sleep`.** It changes every wait in the runtime
  (locks, channels, blocked tasks) and burns a core while idle.
- **A higher work bar on Windows.** The cost model only bounds a nested
  loop's work from below (65536 for these regions), so no bar tells 23 ms
  from 2 s.
- **No compiler-chosen parallel loops on Windows.** Loses long loops, which
  do scale there.

## Consequences

A Windows program whose loops are short pays for two regions (about 56 ms),
then runs them in order. A program with short regions first and long loops
later runs the long ones in order too, unless a `par for` or tensor kernel
resets the count: slower than it could be, never much slower than one thread.
