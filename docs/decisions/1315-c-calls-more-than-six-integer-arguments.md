# 1315. C calls with more than six integer-like arguments

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for it in kksimp/ail#1596 (ail-llm calls BLAS-shaped C functions)
- **Topic:** 14 — FFI
- **Spec rules:** [ffi.decl.3](../spec/14-ffi.md), [ffi.decl.1](../spec/14-ffi.md)
- **Amends:** [ADR 1074](1074-c-structs-by-value-and-callbacks.md) (the call frame now also carries stack integer arguments), [ADR 0432](0432-c-calls-through-two-register-files.md) (the six-integer limit)

## Context

`ffi.decl.3` capped an `extern c fn` at six integer-like and eight float arguments, the register files of
System V x86-64. Common C APIs pass more: `cblas_sgemm` takes twelve integer-like and two float arguments, so
ail-llm could not call it, and `cheader` dropped every such prototype. ADR 1074's struct-by-value call frame
(`rt.ffi_cs`) already places arguments per platform, stack slots included.

## Decision

There is no limit on integer-like parameters. A call with more than six of them goes through the call frame of
`ffi.struct.2`, which puts the ones past the registers on the stack as the platform says: six registers on System
V x86-64, eight on AAPCS64, four positions on Win64, and on Apple arm64 stack arguments packed by natural size.
The eight-float limit stays. `cheader` keeps prototypes with many integer arguments. `tools/ffi-abi` checks a
twelve-integer call, a fourteen-argument mix of narrow integers, floats and a pointer, and the `cblas_sgemm`
shape against C on each host.

## Rejected alternatives

- **Keep the limit and ask for a wrapper struct.** Every BLAS or system call with many arguments would need C glue,
  which AIL has no build step for.
- **A separate register-only fast path that spills by hand.** Duplicates the call frame's per-platform placement,
  which ADR 1074 already tests on every host.
- **Lift the float limit too.** No caller needs it yet; it stays until one does.
