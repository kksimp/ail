# 0967. Sanitizers: `ail sanitize`

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076), from the designer's goal that AIL replace C, C++ and Rust (2026-11-05); number allocated by the integrator
- **Builds on:** [ADR 0322](0322-random-schedule-testing.md) (`--schedule=random:SEED`), ADRs 0641-0644 (native trap stacks), `unsafe.ptr.4-6` and `unsafe.mem.2` (existing traps)
- **Topic:** 11 — Concurrency, 13 — Unsafe

## Context

C and Rust developers reach for AddressSanitizer, ThreadSanitizer and Miri. AIL differs: safe code is data-race-free (`conc.race.1`), integer overflow and bounds traps are always on, and `std.mem` already traps null, out-of-bounds, use-after-free and double free (`unsafe.ptr.4-6`, `unsafe.mem.2`) with a native trap stack. What is left is (1) races in `unsafe` or FFI code and missing joins, which show as results that depend on the schedule, (2) leaked `std.mem` blocks, and (3) a precise detector for unsynchronised raw-pointer access.

## Decision

Three stages; only stage 1 is built now.

1. **Schedule sanitizer (built).** `ail sanitize FILE` runs the program (or `--tests`: its tests) on the fixed schedule and under random schedules with seeds 1..N, and reports each run whose exit status or output differs, as `san.sched.1` with the replay command. Output is JSON lines under `--json` (`api` 1, stable rule id). No runtime code, no codegen change, zero cost for programs that don't use it (ADR 0868).
2. **Leak report (designed).** `AIL_SANITIZE=leak`: the native `std.mem` shim keeps a table of live blocks (address, size, allocation trap stack from ADR 0641) and, at exit, reports each as `san.leak.1` with its stack. Only blocks `std.mem` allocated are tracked; blocks C allocated are not.
3. **Unsafe race detector (designed).** `ail run --sanitize=race`: a native build that instruments raw-pointer `read`/`write` with a shadow word per 8 bytes and a vector clock per task, advanced on `spawn`, `wait`, `Mutex` and `Atomic` edges. Two accesses with no happens-before edge and at least one write is `san.race.1`, with both stacks. Builds differ from normal builds, so the cost is paid only when asked. Needs the debug lane's source positions (ADR 0947) for the stacks.

Rule keys: `san.sched.*` now; `san.leak.*` and `san.race.*` reserved.

## Consequences

Stage 1 can miss a race a given seed set doesn't expose; more seeds raise the odds, and a finding always replays. It cannot name the racing accesses; stage 3 does. Output that legitimately varies by schedule (arrival order from workers) is reported too; the message says what differed so the author can judge.

## Rejected alternatives

- **Port ThreadSanitizer or AddressSanitizer:** an external backend (ADR 0010, 0009), and the memory checks already trap.
- **Detector first (stage 3):** the precise tool, but a codegen change on both architectures (ADR 0008) that needs the gate to build; stage 1 gives agents a usable signal now.
- **Run under `--deterministic` only:** hides exactly the schedule dependence being looked for.
