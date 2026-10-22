# 0846. Native `proc.start` with fd-backed channels, and `std.proc` on macOS

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; allocated by the integrator)
- **Topic:** 23 — native backend
- **Spec rules:** std.proc.1-7, native.subset.1
- **Builds on:** [ADR 0770](0770-native-proc-run.md), [ADR 0534](0534-macos-os-layer-through-libsystem.md), the native `Channel[str]` of lane 178

## Context

`proc.start` returns a `Child` whose `stdin`, `out` and `err` are
`Channel[str]` (std.proc.4-5). Garlic and `std.app` need it natively, and
`proc.run` still trapped on macOS (ADR 0770).

## Decision

- **Streams are channels fed by a poll, with no thread per stream.** A
  channel record has a `source` word (lane 178's `chan.ail`). For a child it
  points to one state object (`proc.ail`): the pid, the three descriptors,
  a line buffer per output stream, the time limit and the exit status. A
  receiver that finds an empty open channel calls `chan_pump`, which polls
  both output pipes, splits what arrives into lines (`\n` or `\r\n`, a last
  line with no end counts, `os_lossy` for invalid UTF-8) and queues them on
  `out` and `err` with `chan_push`. A stream's end closes its channel.
  Every pump fills both streams, so a child is never blocked by a stream
  nobody reads, as std.proc.5 requires, as long as someone receives or
  waits. Pumps poll in slices of at most 100 ms, so a time limit and a child
  that exited while a grandchild holds its pipe are noticed.
- **`stdin.send` writes at once.** It writes to a nonblocking pipe; when the
  pipe is full it polls for room *and* drains the output, so a child that
  writes as it reads cannot deadlock the sender. A gone child is not an
  error (`SIGPIPE` stays ignored once a child is started). `stdin.close`
  closes the descriptor.
- **`wait` and `kill`** reap with `wait4`, drain what is left so every line
  written before the exit stays readable, then close the pipes. A limit
  kills with `SIGKILL`; `wait` then fails with kind `timed_out`. `kill`
  gives 137. A child has `PR_SET_PDEATHSIG` (Linux), so none outlives the
  program (std.proc.7).
- **Lowering:** `proc.start`, `proc.wait` and `proc.kill` are `os_lookup`
  entries; `Child` is the registered struct `proc.Child` (pid and three
  `Channel[str]` slots), like `proc.Output`. A program that writes
  `proc.start` links the task runtime (the channel runtime needs it).
  Nothing is added to `lower.ail`.
- **macOS:** the same code. `fork`, `pipe` and `signal` come from libSystem
  (`import_call`, ADR 0534): the raw `fork` and `pipe` system calls return
  two registers, which the `syscall` primitive does not expose. The rest are
  BSD system calls: `wait4` 7, `kill` 37, `dup2` 90, `poll` 230, `fcntl`
  92, `access` 33. Close-on-exec comes from `fcntl` after `pipe`. This
  replaces the trap of ADR 0770 and the `posix_spawn` plan: after `fork`
  the child makes only system calls, so libSystem's fork handlers are not
  needed.

## Why not

- **A reader task per stream:** a pool thread would block in `read`, the
  task count would depend on the pool size, and `select` over the streams
  would need a second wake-up path.
- **Reading directly from the descriptor in `recv`:** reading `out` would
  not drain `err`, and a child writing a lot to `err` would hang.
- **`posix_spawn` on macOS:** it needs the pipe-status trick replaced by a
  file-action list built in AIL, and gives no benefit over `fork` here.

## Evidence

`bootstrap/tests/native/run/proc_start.ail` and `proc_start_flow.ail` match
`ail run` on x86-64 and aarch64 under qemu: sort through `stdin`, both output
streams, `wait` status, `kill` (137), a time limit, 20000 lines written
before any is read, and `task.select` over `out` and `err`. The macOS
binaries build with `--target aarch64-macos`; the run is on the Mac gate.
