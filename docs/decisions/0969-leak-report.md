# 0969. The leak report: `ail leak`

- **Status:** Accepted
- **Date:** 2026-11-06
- **Decided by:** AI (delegated, ADR 0076), from the designer's go-ahead for the leak report staged in ADR 0967 (2026-11-05); number requested from the integrator
- **Builds on:** [ADR 0967](0967-sanitizers.md) (stage 2), [ADR 0562](0562-native-heap-size-classes-and-header.md) and [ADR 0564](0564-collection-trigger-and-sweep.md) (the collector and `AIL_GC_STATS`)
- **Topic:** 13 — Unsafe, 11 — Runtime
- **Spec rules:** san.leak.1-4

## Context

AIL is garbage-collected (a feature), so a block cannot be lost the way `malloc` memory is. A program leaks by keeping what it no longer needs. ADR 0967 staged a leak report as a table of live `std.mem` blocks. That covers only manual blocks. Most of a program's memory is on the GC heap, where the question is what keeps it alive: growth over time, which kinds grew, and which roots hold them.

## Decision

- `AIL_LEAK=1|json|all|all,json` turns the report on in any native executable. `ail leak [--json] [--all] FILE [ARGS...]` builds FILE and runs it with the variable set (`ail`, shell form; `ail.ps1` has the same command).
- With it on, the collector marks the roots one at a time through counting versions of its scan and drain (`compiler/rt/native/leak.ail`) and records bytes and objects per kind and per root. The counting versions mirror `gc_scan_in` and `gc_drain`; the normal path is untouched except for one word test.
- Findings: `san.leak.1` live memory grew (first against latest collection, with a monotone check over the last 64); `san.leak.2` the top roots that hold it, with kind and first/latest bytes. `--all` lists roots without growth. Text on standard error, or JSON lines with a summary.
- Collections run at least every 4 MiB allocated, or every half of the live bytes, and single-threaded, so there are enough samples. A program with fewer than four collections gets one more at exit, which shows what globals keep.
- It reports reachable memory. `std.mem` blocks stay with the existing traps and, later, a block table (ADR 0967's original stage, still open).

## Consequences

The report is exact about what is reachable but names a root by slot number, not by source name, until the debug lane's positions (ADR 0947) give names. A conservative stack word can retain an object no variable holds, and the report cannot tell. Growth is a heuristic: a program that legitimately loads a big file looks the same as a cache that never shrinks; the message says "grew", and the roots say who holds it. Programs that keep many tasks parked count their stacks under "the other threads".

## Rejected alternatives

- **Allocation-site tracking (a stack per allocation):** the usual leak-tool answer, but it taxes every allocation and needs source positions the native backend doesn't carry yet. Roots plus kind answer "what holds it" at no cost when off.
- **A heap snapshot to diff offline:** larger output than an agent can use, and the diff has no root information.
- **Only a final collection at exit:** misses a leak held by a local until the program ends, since locals are gone by then.
- **A table of `std.mem` blocks first (ADR 0967 stage 2 as written):** covers manual memory only; kept as a later addition.
