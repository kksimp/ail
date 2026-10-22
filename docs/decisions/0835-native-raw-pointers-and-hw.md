# 0835. Native `*T` is an address in an integer register; `std.hw` lowers to plain loads, stores and fences

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Builds on:** [ADR 0820](0820-driver-level-unsafe.md), [ADR 0822](0822-std-hw-volatile-and-hw-effect.md), [ADR 0823](0823-lower-ail-feature-files.md), [ADR 0834](0834-native-sized-numbers.md)

## Context

ADR 0820 step 1 gave the interpreter and both checkers `std.hw`. Native builds
rejected it. Drivers need the native path: MMIO registers are 8, 16 or 32 bits
wide, and the examples lane wants a libc subset built on `*T`.

## Decision

1. **`Ty.Ptr(inner)`**, named `*T`, is one 64-bit address in an integer
   register. The verifier treats it as `int`.
2. **Loads and stores** are the new prims `pload8/16/32/64` (zero-extended) and
   `pstore8/16/32/64` on both back ends (x86-64 `movzx`, `mov`; ARM64
   `ldrb`, `ldrh`, `ldr w`, `ldr x` and the stores). The lowering sign-extends
   `i8 i16 i32` itself, reuses the `Sized` representation of ADR 0834, and
   moves `f64` and `bool` through the integer register. `read_volatile` and
   `write_volatile` emit the same prims. Prims the optimiser does not know are
   barriers, so none is merged, moved or dropped (`unsafe.hw.3`).
3. **Fences.** `hw.fence()` is `mfence` or `dmb ish`; `hw.fence_io()` is
   `mfence` or `dsb sy` (`unsafe.hw.4`).
4. **Traps** keep the interpreter's text, built by `rt/native/sized.ail`:
   `hw.at` of a negative address (`unsafe.hw.2`), an address below 4096 on any
   access (`unsafe.ptr.4`, a null check by comparison, no signal handler), and
   a volatile access at an address that is not a multiple of its width
   (`unsafe.hw.3`). Out-of-block and freed accesses are undefined, as in C
   (native-abi.md, Memory).
5. **Where.** `compiler/mir/lower_unsafe.ail` holds the lowering. `lower.ail`
   gains two `Ty`-driven arms (`type_of` and the method dispatch);
   `lower_os.ail` lists the `hw` module; `lower_ops.ail` gets `==` and `!=`
   on pointers.
6. **`ail build`** passes `--unsafe driver` to its check step when given.
7. **Tests.** `bootstrap/tests/native/driver/` runs address-independent
   programs against `ail run --unsafe driver`. The `mem_*` programs touch real
   memory (the program's own ELF header and an unused runtime global slot) and
   are checked against `.out` on Linux x86-64 only.

## Rejected alternatives

- **A guard page and signal handler for null.** Unreliable for an address such
  as 4096 that a driver may legitimately map; the interpreter's below-4096 rule
  is one compare.
- **Reusing `load`/`store` MIR ops.** The optimiser may merge or move them; a
  volatile access must not be.
- **Doing 16- and 32-bit access only.** Pointers to `u8` (C strings) and `int`
  need the other widths and cost the same encoder code.

## Consequences

`ail build --unsafe driver` compiles `std.hw` programs on Linux x86-64 and
macOS ARM64. The ARM64 encodings are checked against `llvm-mc`; execution is
checked on the macOS gate. Still open: `mem.alloc`/`free`/`cstr` natively
(examples lane), `@layout(c)`, port I/O, freestanding targets, `@interrupt fn`.
