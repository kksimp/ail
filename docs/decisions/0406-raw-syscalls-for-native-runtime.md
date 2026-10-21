# 0406. Raw syscalls for the first native runtime, including on macOS

- **Status:** Accepted; its macOS part amended by [ADR 0534](0534-macos-os-layer-through-libsystem.md)
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)**
- **Topic:** Native runtime and ABI (lane 122)
- **Spec rules:** [err.trap.3](../spec/08-errors.md#traps), [type.num.7](../spec/04-types.md), [type.coll.4](../spec/04-types.md), [type.str.12](../spec/04-types.md)
- **Design note:** [native-abi.md](../design/native-abi.md)

## Context

How a libc-free executable reaches the OS.

## Options considered

A: raw syscalls on all three targets (chosen). B: import `libSystem` on macOS from the start. C: link libc everywhere.

## Decision

A. Linux x86-64, Linux ARM64 and macOS ARM64 use their syscall instructions through a four-function shim (`exit`, `write`, `mmap`, `read`); numbers are in native-abi.md §4. Apple documents `libSystem` as the stable interface, so if raw syscalls break on a macOS release the shim switches to dyld-imported `_write`, `_exit`, `_mmap` (the import table lane 126 needs for `extern`); the shim is one file per target so only it changes. Windows is stubbed (ADR 0004).

## Rejected alternatives and criterion

Criterion: speed and ADR 0009/0010 (no external toolchain or libc). Rejected: B for now (needs the Mach-O import machinery before `hello` runs); C (ADR 0017, ADR 0010).

## Consequences

The ABI note and `compiler/rt/rt.ail` are the source of record; lane 120's backend emits code against them. Revisit when the collector, frame metadata or `unsafe` pointers land.
