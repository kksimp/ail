# 0387. `ail build`: static executables with the runtime compiled from AIL, checked first

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.1, native.build.3](../spec/23-native.md)
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md), [ADR 0385](0385-x86-64-code-generation.md),
  [ADR 0404](0404-native-calling-convention-and-value-layouts.md)–[0408](0408-buffered-output.md) (runtime),
  [ADR 0510](0510-licence-apache-2-with-runtime-exception.md) (Runtime Exception)

## Context

`ail build FILE` must produce a runnable file with no external linker or libc
(ADR 0010). The backend runs on the bootstrap interpreter, whose file API
writes text only until binary writes land (lane 123).

## Decision

1. **Static ELF, no sections.** `compiler/obj/elf.ail` writes the ELF header
   and two `PT_LOAD` segments: read+execute at 0x400000 holding the headers,
   code and read-only data (strings, float constants, trap texts), and a
   zero-filled read+write segment for the runtime's globals. No section
   headers, no relocations at run time: the writer resolves every symbol
   (functions, data, globals) itself. Hello world is 4.6 KB.
2. **The runtime is AIL source compiled with the program.**
   `compiler/rt/native/core.ail` (output buffer, bump allocator, lists, number
   text, traps) is lowered with the program on every build, over the
   primitives of `prim.ail` (`load64`, `store64`, `syscall`, `global`, ...),
   which only runtime code may call. Runtime arithmetic wraps and its
   indexing is unchecked: it has no trap sites. Every file there carries the
   Runtime Exception header (ADR 0510).
3. **Check first.** `ail build` runs the checker (`ail check FILE`) and stops
   on its errors; then the backend reports every construct outside the
   native subset at once (native.subset.1).
4. **Bytes through text, for now.** The backend (`ail run compiler build FILE
   RT`) prints the executable as hexadecimal; the Rust driver decodes it,
   writes the file and makes it executable. When `std.fs` writes bytes the
   backend writes the file itself and the driver shrinks to a call.
5. **Entry.** `_start` clears `rbp`, aligns the stack, calls `rt.init` and
   `main`, then `rt.exit(0)`, which flushes the output buffer.

## Options considered

- **Write a relocatable object and link with `ld`.** Rejected: ADR 0010 (no
  external backend tools), and it makes Linux builds depend on binutils.
- **Runtime as precompiled machine code checked in.** Smaller builds, but a
  second artefact to keep in step with the source; compiling ~600 lines of
  runtime costs about 0.1 s per build today. Rejected until build time
  matters.
- **Emit the runtime from the interpreter's Rust code.** Rejected: the goal
  is no Rust in the shipped toolchain (Stage 3).

## Consequences

- `ail build` works on Linux x86-64 hosts; other hosts get native.build.3
  until slice 3 (ARM64, Mach-O).
- Build time is the backend's time on the interpreter (about 0.5 s for small
  programs), dominated by front end and lowering; the bytecode tier and
  self-compilation shrink it.

## Rejected alternatives

- Writing a relocatable object and linking with `ld`: rejected by ADR 0010 (no external backend tools), and because it would make Linux builds depend on binutils.
- A precompiled, checked-in runtime as machine code: rejected until build time matters. It would be a second artefact to keep in step with the source, and compiling about 600 lines of runtime costs about 0.1 s per build.
- Emitting the runtime from the interpreter's Rust code: rejected, because the goal is no Rust in the shipped toolchain (Stage 3).
