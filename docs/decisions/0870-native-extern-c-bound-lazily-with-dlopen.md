# 0870. A native `extern c fn` outside the libc subset binds lazily with `dlopen` and `dlsym`

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076), as the integrator specified in item 279
- **Topic:** 14 — FFI; 23 — Compiler architecture
- **Spec rules:** [ffi.native.1](../spec/14-ffi.md), [ffi.load.1](../spec/14-ffi.md), [ffi.load.2](../spec/14-ffi.md), [ffi.load.3](../spec/14-ffi.md)
- **Builds on:** [ADR 0436](0436-native-c-calls-by-dynamic-imports.md), [ADR 0844](0844-native-libc-subset.md), [ADR 0431](0431-extern-c-fn-and-dynamic-loading.md)

## Context

ADR 0844 let `ail build` accept `extern c fn` only for a fixed libc subset the runtime provides, so `ail build` and `ail run` could differ: the interpreter calls any C function, and two spec programs (`ffi_missing_library`, `ffi_missing_symbol`) stayed `ffi.native.1`. With the interpreter going, nothing may be dropped.

## Decision

1. **Same rules as the interpreter.** An `extern c fn` that is not in the subset is bound on its first call: `dlopen` of its library (or the running program's own symbols without one) with the candidate names of `ffi.load.1`, then `dlsym`, cached in a cell. A library that can't be opened traps with `ffi.load.1`, a missing symbol with `ffi.load.2`, and Windows with `ffi.load.3`, as in `ail run`.
2. **The call** is an indirect call of the bound address with the platform's C convention (the native convention already is it). The result is converted as `ffi.map.2` says; `xmm8`-`xmm13`, which AIL functions keep, are saved around the call on x86-64.
3. **Zero cost when unused.** The libc subset keeps its runtime functions and the executable stays static. Only a program that calls a bound function reaches `compiler/rt/native/ffi_load.ail`, and only then does the executable import `dlopen` and `dlsym`: a dynamic ELF with `PT_INTERP`, `DT_NEEDED libc.so.6`, `BIND_NOW` and two weak `GLOB_DAT` slots after the globals on Linux (x86-64 and ARM64), and the existing libSystem bind table on macOS (`_dlopen`, `_dlsym`). The entry stays `rt_start`.
4. **Limits.** At most 6 integer and 8 float arguments (the checker's limit), 4096 bound functions per program, and the first call must be from the main thread (`dlopen` needs the C library's thread state, which AIL's worker threads don't set up). Linux needs glibc 2.34 or later, where `dlopen` lives in `libc.so.6`.

## Rejected alternatives

- **Build-time errors for non-libc symbols** change behaviour, and break programs that load a library only sometimes.
- **Linking every named library at build time** needs the library on the build machine and doesn't fit cross builds.
- **A hand-written dynamic loader** in the runtime: far more code than asking the system's loader, and it would still need the C library.
- **A `DT_NEEDED` per declared library** makes a missing library a load failure of the whole program instead of a trap at the call.
- **Binding eagerly at start** breaks `ffi.load.1` (the library is loaded on the first call, not at start).

## Consequences

`ffi_missing_library`, `ffi_missing_symbol` and `ffi_dynamic` run natively with the interpreter's output and exit status; `examples/ffi_libc.ail` and `examples/ffi_buffers.ail` stay static. `ffi.native.1` now describes the subset only. A C function's own buffered output isn't flushed at exit (`rt_exit` ends with a system call, not `exit`).
