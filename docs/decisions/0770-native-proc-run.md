# 0770. Native `proc.run` and `proc.which` on Linux

- **Status:** Accepted (macOS and `start` amended by [ADR 0846](0846-native-proc-start-and-macos-proc.md))
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — native backend
- **Spec rules:** std.proc.1, std.proc.2, std.proc.3, native.subset.1
- **Builds on:** [ADR 0580](0580-os-calls-by-system-calls-with-a-status.md), [ADR 0574](0574-native-optional-layout.md)

## Context

`std.proc` is the first error in 24 of the 74 programs that don't build
natively (survey at bd982d2 with `std.path` merged). Most call
`proc.run(cmd, args)` or with `none` and a limit, and read `r.status`,
`r.out` and `r.err`.

## Decision

- **Runtime** (`compiler/rt/native/proc.ail`, no global slots): find the
  program (a name with `/` as given, else the first executable file on
  `PATH`), build `argv` in an object whose words the collector scans, flush
  standard output, `clone` with `SIGCHLD`, and in the child `dup3` three
  pipes onto 0, 1 and 2 and `execve` with the environment vector. A
  close-on-exec pipe carries the `errno` of a failed `execve`; only that is
  a `ProcError` (std.proc.1), with the kind of the same `errno` mapping as
  `fs.IoError`. The parent feeds `input` and drains both pipes in one
  `ppoll` loop (stdin non-blocking, `SIGPIPE` ignored only while it runs), so
  a full pipe can't deadlock. A `timeout_ms` bounds the loop and the wait;
  past it the child gets `SIGKILL`, is reaped, and the call fails with
  kind `timed_out` (std.proc.2). Status is the exit code, or 128 plus the
  signal. Captured text goes through `os_lossy` (invalid UTF-8 becomes
  U+FFFD). `proc.which` is the same search.
- **Lowering:** `proc.run` and `proc.which` are two `os_lookup` entries.
  `OsFn` gains `optional`, the count of trailing parameters that may be left
  off; the call passes a zero word (`none`) for each (ADR 0284). `Output` is
  a struct the lowerer registers as `proc.Output` (three slots), so `r.out`
  needs no special case; `Metadata` and `Child` can follow the same way.
- **Not in this slice:** `ProcError.kind` as a readable field (the error is
  its message natively until lane 179's error structs), `start`, `wait`,
  `run_bytes`, and macOS and Windows. On macOS `proc.run` traps at run time
  with `native.subset.1`, because the `syscall` primitive returns one
  register and macOS `fork` returns two; the answer is `posix_spawn`
  through libSystem (ADR 0534).
- A negative `timeout_ms` traps `std.proc.2` with the interpreter's message
  but without its source excerpt (the same shortcut as `time.sleep_ms`).

## Why not

- **A shell (`sh -c`):** std.proc.1 says directly and without a shell.
- **`vfork`/`posix_spawn` without a status pipe:** the failed-exec case
  would be indistinguishable from the program exiting 127.
- **Threads for draining, as the interpreter does:** `ppoll` needs no second
  task and no heap sharing.
- **A `fork` inside `proc.run` that allocates in the child:** the child only
  makes system calls, so no collector state is touched after `clone`.

## Evidence

`bootstrap/tests/native/run/proc_run.ail` matches `ail run` on x86-64 and
on aarch64 under qemu: status, stderr, input (a 300 KB input and output
through both pipes), a kill signal (137), `timed_out`, a missing program,
`which`, and the `? "label"` form.
