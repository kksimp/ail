# 0700. Native `io.exit` and `time.sleep_ms` range traps

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** std.io.4, std.time.2, native.build.1

## Context

`std_io_exit_range` and `std_time_negative` built natively but did not trap:
`exit(256)` ended the process with a wrapped status and `sleep_ms(-1)`
returned at once. The interpreter traps with the rule and position.

## Decision

`os_call` in `lower_os.ail` emits a range check before the runtime call:
`exit` needs `0..=255`, `sleep_ms` needs `>= 0`. A failing check is a `trap`
instruction of runtime kind 9 (exit) or 10 (sleep) carrying the value, the rule
ID and the call position; `rt.trap` prints the interpreter's messages. The
checks cost two compares and a never-taken branch per call, and these calls
are not hot.

## Rejected

- Checking inside `rt.io_exit`: the runtime has no source position, so the
  trap line would differ from `ail run`.
