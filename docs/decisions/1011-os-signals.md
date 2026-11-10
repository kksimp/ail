# 1011. `os.signals`: OS signals as a channel

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.os`)
- **Spec rules:** std.os.1-4
- **Builds on:** [ADR 0188](0188-channel-api.md), [ADR 0996](0996-try-recv-and-group-fields.md)

## Context

A CLI or daemon needs a clean shutdown on Ctrl-C or `kill`: finish a write, close a socket, remove a lock file. AIL had no way to see a signal; the default action ended the program at once. C has `sigaction`, Rust has `ctrlc` and `signal-hook`, Go has `signal.Notify` on a channel. The runtime has no libc on Linux (native-abi.md §4), and a handler can interrupt the program anywhere, including inside the allocator and the collector.

## Options considered

### Option A — a handler that counts, and a channel fed on demand (chosen)
`os.signals(names) Channel[str]`. The system's handler only adds one to a counter in a table and wakes sleepers on it. The channel is a fed channel like a child's output (std.proc.5): a receiver that finds it empty asks the table.
- **Pros:** the handler allocates and locks nothing, so it is safe at any point; no thread or task is needed; works with `recv`, `try_recv`, `recv_for` and `task.select` as they are; same shape as Go's `signal.Notify`.
- **Cons:** signals of one kind that arrive before a receiver looks become one value (as the OS itself merges them); a few runtime lines per target.

### Option B — a callback, `os.on_signal(name, fn)`
- **Pros:** familiar from C.
- **Cons:** the callback would run user code at an arbitrary point, with the collector and locks in any state; the native subset has no fallible or generic function values (ADR 0996 notes this). Rejected.

### Option C — `signalfd` and `kqueue` with no handler
- **Pros:** no async context.
- **Cons:** `signalfd` needs the signal blocked in every thread, including pool threads already running; `kqueue` has no Windows twin. Rejected.

### Option D — a blocking `os.wait_signal(names) str`
- **Pros:** smallest API.
- **Cons:** cannot be combined with other channels in a `select`, so a server cannot wait for a request or a shutdown. Rejected.

## Decision

`os.signals(names List[str]) Channel[str]` (std.os.1-4), names `HUP`, `INT`, `QUIT`, `TERM`, `WINCH`; unknown names trap. Closing the channel ends it and restores a signal's default action once no open channel names it. Runtime global slot 66 holds the table (counts by signal, watchers by signal), made on first use. Slot 66 was the next free one in the allocation memory on 2026-11-09; the integrator confirms it.

Per target (`compiler/rt/native/signal.ail`):
- **Linux x86-64, ARM64:** `rt_sigaction` with `SA_RESTART`. x86-64 also gives `SA_RESTORER`: the function `sig_ret`, whose body is a new intrinsic `sigreturn()` (`lea rsp, [rbp+8]; mov eax, 15; syscall`, valid because the function is entered by the handler's `ret`). ARM64 needs none.
- **macOS:** libSystem's `signal`.
- **Windows:** `SetConsoleCtrlHandler` with the runtime function `sig_win_ctrl`. The runtime's compiled functions use the System V convention, so a new intrinsic `win_arg0()` (`mov dst, rcx`, first statement of the function) reads the event number: Ctrl-C and Ctrl-Break are `INT`; close, logoff and shutdown are `TERM`. The function ends its thread with `ExitThread(1)` instead of returning, so the registers Win64 requires preserved (`rdi`, `rsi`) are never handed back changed. Hand-written machine code is not used anywhere (intrinsics only).

## Rationale

Counting in the handler and delivering from a pump is the one design where the async part cannot break the collector. It reuses the fed-channel hooks in `chan.ail` (five small branches in `proc.ail`).

## Consequences

- A new module `os` is registered in `names.ail`, `stdlib.ail`, `effects.ail`, `lower_mod.ail` and `lower_os.ail`. It needs the `proc` capability.
- `WINCH` is delivered on Linux and macOS; `std.term` (ADR 1012) builds resize events on it.
- New intrinsics `sigreturn` and `win_arg0` in `prim.ail`, the x86-64 generator (`gen.ail`), the ARM64 generator (a trap), `lower.ail` and `alloc.ail`.
- Windows delivery of Ctrl-C is tested by hand; the native test self-signals through `sh` and so runs on Linux and macOS only.
- Native only (ADR 0865). Not recorded or replayed.

## Rejected alternatives

Options B, C and D above.
