# 0432. C calls use two register-file trampolines, with no foreign-function library

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 14 — FFI; 27 — Dependencies and tooling
- **Spec rules:** [ffi.decl.3](../spec/14-ffi.md), [ffi.call.2](../spec/14-ffi.md), [ffi.map.1–2](../spec/14-ffi.md)

## Context

Calling a function whose signature is known only at run time normally takes
libffi or hand-written assembly per signature (ADR 0009 prefers custom
tooling; every crate needs a justification, ADR 0023). The brief asks for "a
small hand-written trampoline per signature class".

## Decision

On x86-64 System V and on ARM64 (AAPCS64, and Apple's variant) integer-class
and float-class arguments use two separate register files. One function
pointer type taking six `i64` then eight `f64` calls every C function with at
most six integer-like and eight float arguments, whatever their order: integers
fill the integer registers in order, floats the float registers in order, and
registers the callee does not read are ignored. The result is in the integer or
the float register, so two function pointer types (`-> i64`, `-> f64`) are the
whole trampoline. The checker enforces the limits (`ffi.decl.3`).

Type mapping (`ffi.map.1–2`): `int` is `int64_t`, `float` is `double`, `f32` is
`float` (its bits in the low half of a float register), `bool` is a byte, the
sized integers keep their width, `*T` is a pointer. Narrow arguments are
extended by signedness (Apple's ARM64 convention requires the caller to extend);
narrow results are truncated and re-extended because C leaves the upper bits
unspecified. Not supported, refused or absent: variadic functions (they need
`%al` on x86-64 and a different stack layout on Apple ARM64), structs by value,
more arguments than registers, and callbacks.

## Options considered

### Option A — two function pointer types (chosen)
- **Pros:** about 40 lines, no dependency, correct on both gate platforms.
- **Cons:** Windows x64 assigns registers by position, so the trick does not
  apply there (`ffi.load.3`); stack arguments and structs are out of reach.

### Option B — the `libffi` crate or C library
- **Cons:** an external dependency (and a C build) for a task two function
  types cover; ADR 0009.

### Option C — generated assembly per signature
- **Cons:** an assembler in the toolchain for no gain over Option A at the
  interpreter's call rate.

## Consequences
- Windows needs its own trampoline (a stub today, `ffi.load.3`).
- Callbacks and structs by value will need real per-signature code; the native
  backend (lane 120) generates that natively (`native-abi.md`).

## Rejected alternatives

- The `libffi` crate or C library (option B): rejected because it is an external dependency, and a C build, for a task that two function types cover (ADR 0009).
- Generated assembly per signature (option C): rejected because it needs an assembler in the toolchain for no gain over option A at the interpreter's call rate.
- Windows x64 register assignment: not handled by this trick (`ffi.load.3`). Windows reports not-supported rather than guessing.
