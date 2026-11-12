# 1031. Variadic C calls: `extern c fn f(a, ...)` with the platform's convention

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 14 — FFI
- **Spec rules:** [ffi.var.1-4](../spec/14-ffi.md)
- **Builds on:** [ADR 0432](0432-c-calls-through-two-register-files.md), [ADR 0980](0980-windows-c-calls.md), [ADR 0926](0926-c-header-import-subset.md)
- **Found by:** the `std.term` work (ADR 1012): `ioctl`, `fcntl` and `open` are variadic

## Context

`ffi.decl.3` and the header reader skip variadic prototypes. `ioctl(TIOCGWINSZ)`,
`fcntl`, `open` with a mode, `snprintf` and `printf` are out of reach. The four
targets differ: System V needs `al` set, Win64 mirrors floats into integer
registers, Linux arm64 is plain AAPCS64, and Apple arm64 puts every variadic
argument on the stack.

## Decision

1. `...` ends the parameter list of an `extern c fn` after at least one named
   parameter (`ffi.var.1`). The parser records it as the synthesized attribute
   `variadic c`, so the AST is unchanged.
2. A call checks the named arguments as usual, then each further argument
   keeps its own static type, which must have a C layout (`ffi.var.2-3`).
   `f32` is refused (C promotes it to `double`; the author converts) and so is
   `str` (use `mem.cstr`). Narrow integers and `bool` are already carried
   extended to 64 bits (`ffi.map.1`), which is what C's default argument promotion
   needs. The six-integer and eight-float limit of `ffi.decl.3` applies to the
   whole call.
3. The call is lowered at the call site (`Lower.lower_variadic`, `mir/lower_ffi.ail`) as
   an `icall` named `cv<N>` (`N` named arguments), after the usual lazy binding
   of the symbol. That function is the single entry for any variadic
   prototype, so the C header import (ADR 0926, the header-breadth lane) only has to emit
   `extern c fn name(params, ...)` instead of skipping the prototype.
4. Code generators (`ffi.var.4`): x86-64 System V sets `al` to the number of
   float arguments; Win64 copies each of the first four float arguments into
   the integer register of its position; Linux arm64 is unchanged; Apple arm64
   stores the variadic arguments in 8-byte stack slots after the named ones,
   floats as `double`.

## Rejected alternatives

- A fixed-arity wrapper per call shape written by the author: every `ioctl` request type would need one.
- `printf`-style format-string inference: guessing types from a string the
  checker may not see; per-argument static types are exact.
- Annotating each argument (`i32: 3`): the static type is already known, and the
  syntax collides with named arguments.
- Supporting `va_list` (receiving variadics in AIL): needs callbacks and a
  varargs ABI for exports; not in this slice.

## Revival (2026-11-11)

PR #1117 was closed on one missing line: `variadic c` was absent from the native
subset's attribute allow-list in `compiler/mir/lower.ail`. Added. The C header
import now emits `...` for a variadic prototype that has a named parameter
(`compiler/cheader`). Tests added: eight variadic floats, a call with no variadic
arguments, and an `f32` argument refused (`ffi.var.3`). `open(path, flags, mode)` has
no portable case, since the C runtime on Windows names it `_open`; it is covered by
the `std.term` use on POSIX hosts instead.
