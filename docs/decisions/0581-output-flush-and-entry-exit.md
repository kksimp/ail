# 0581. Output flushing at exit, traps, input and terminals; standard error through the same buffer

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)**
- **Topic:** Native runtime and ABI (lane 177)
- **Spec rules:** [std.io.3](../spec/17-std.md), [std.io.4](../spec/17-std.md), [native.build.1](../spec/23-native.md), [err.trap.3](../spec/08-errors.md#traps)
- **Design note:** [native-abi.md](../design/native-abi.md) §3 and §6

## Context

Compiled programs enter through `rt_start` (ELF `_start`, Mach-O `_main`),
not libc's startup, so nothing runs `fflush` for them (the risk lane 126
raised in native-abi.md). ADR 0408 made `print` buffered. The driver also
writes diagnostics to standard error and exits with a status, and an
interactive program must show its lines as `ail run` does.

## Options considered

### Option A — flush at defined points, and per line on a terminal (chosen)
- **Pros:** one buffer, no extra code path for `eprint`; a pipe or file gets
  64 KiB writes; a terminal gets lines when they are complete.
- **Cons:** a crash by signal loses buffered output (as C's stdio does).

### Option B — unbuffered stderr beside a buffered stdout (ADR 0408)
- **Pros:** simple.
- **Cons:** `io.eprint` after `print` could reach the terminal first; std.io.3
  requires pending standard output to come out first.

## Decision

A. Standard output is flushed: when the 64 KiB buffer fills; at `rt.exit`,
which the entry calls after `main` returns on all three targets and `io.exit`
calls with its status; at every trap, before the trap text; before a read of
standard input; before an `io.eprint`; and, when standard output is a
terminal (one `ioctl`, asked on the first newline and remembered), at each
newline. `io.eprint(x)` flushes standard output, lowers `print`'s pieces into
the same buffer with a global (`OUT_FD`) naming standard error as the target
of every flush, then flushes to descriptor 2 and restores it. A write is
retried after a partial write and after `EINTR`.

## Rationale

Criterion: AI understanding (one buffer, one rule per flush point), then
speed (a pipe sees few syscalls). It keeps the interpreter's observable
behaviour: standard output before standard error, lines on a terminal.

## Consequences

- `io.eprint` output of any size interleaves correctly with `print`.
- A program killed by a signal loses up to 64 KiB of piped output.
- Reading standard input with output pending never leaves a prompt unseen.

## Rejected alternatives

B: breaks std.io.3's ordering. Flushing every line always: a syscall per line
dominates print-heavy loops (ADR 0408).
