# 1226. The toolchain's own compiler widens its heap goal to five times live

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); scope (compiler only) agreed with the designer's standing call that more memory for speed is fine. No syntax and no setting
- **Topic:** Native heap and collector; compile time
- **Spec rules:** [mem.gc.5](../spec/06-memory.md)
- **Amends:** [ADR 1187](1187-heap-goal-and-page-return.md) (the widest slack)

## Context

`ail build --cold` of `tools/ailtest/ailtest.ail` (3,960 lines) took 4.89 s on ail-linux in 0.1.0 and 6.27 s in 0.2.0.
The dominator fix (#1486) brought it to 5.69 s. A profile of that build puts about 58% of its time in
the collector and allocation, and ADR 1187's guard already sits at its caps for it: the build collects for far more
than 6% of its run time. A sweep of the caps on ail-linux (`coldbuild-heap-1.txt`, `coldbuild-heap-2.txt` in the project
library; best of 5 ms, peak RSS) gave:

| widest slack | ailtest cold | ailtest RSS | compiler self-build | its RSS | gc_heap | lzr RSS |
|---|---|---|---|---|---|---|
| 3x (ADR 1187) | 5,693 ms | 389 MB | 62.1 s | 2.9 GB | 930 MB | 577 MB |
| 4x | 5,201 ms | 470 MB | 55.4 s | 2.9 GB | 1,160 MB, 3% slower | 696 MB |
| 5x | 4,910 ms | 550 MB | 52.8 s | 3.9 GB | 1,275 MB, 9% slower | 696 MB |

Raising the floor alone (64 to 256 MiB) did less: 4,985 ms at 256 MiB.

## Decision

The compiler alone may widen to five times its live bytes. Lowering (`compiler/mir/lower_heapwide.ail`, one dispatch
line in `lower.ail`) starts `main` with `rt.hg_wide()` when the program is `compiler/main.ail`. That call sets heap
state word 470, and `hg_goal` then caps the slack at 20 quarters instead of `HG_SLACK_MAX` (12). The guard still decides
when to widen. Every other program is lowered and runs as before.

The shipped CLI is built by the published seed, so the change takes effect there from the first seed cut after it
lands. Stage 2 and later builds of the compiler have it at once.

## Options considered

- **Five times for every program:** rejected. GC-heavy programs pay for it, with gc_heap 9% slower and 37% more
  memory, and lzr 20% more memory. ADR 1187 chose that trade the other way for programs.
- **Four times for every program:** rejected for the same reason at a smaller scale (gc_heap 3% slower, 25% more
  memory), and it misses the target: 5.2 s is 6% over 0.1.0, against 0.4% at five times for the compiler.
- **An environment variable set by the `ail` wrappers:** rejected. It is a hidden GOGC-style setting (rejected by
  ADR 1187), and programs started from the CLI would inherit it.
- **Detecting the compiler at run time (its executable path):** rejected as fragile.
- **A std function for the program to call:** rejected. It is a public setting under another name.

## Rationale

ADR 0076 order: nothing changes for a program or a diagnostic, and the special case has a single, named home.
Speed decides. The edit loop's cold build returns to 0.1.0's time, and user programs pay nothing.

## Consequences

- Memory: the compiler building itself peaks at 3.9 GB instead of 2.9 GB, and a cold build of a 4,000-line program at
  550 MB instead of 389 MB. The Mac has 8 GB with a 6 GB cap for AIL work (one heavy job at a time), so a self-build
  keeps about 2 GB of headroom under the cap. Two self-builds side by side would not fit; the gate runs them one at a time.
- A program whose entry file is named `compiler/main.ail` outside the toolchain also gets the wider goal. It changes
  only when collections happen, never what the program does.
- Files: `rt/native/heap_goal.ail` (`hg_wide`, `HG_WIDEST` 470, `HG_SLACK_WIDE` 20), `mir/lower_heapwide.ail` (new), one
  line in `mir/lower.ail`, a sentence in spec mem.gc.5.
