# 14. FFI and C interoperability

Chapter key: `ffi`

> **Status:** Drafted (partial). Specified: how C declarations enter a program,
> and the safety of calls. Not yet specified: the type mapping, callbacks,
> exporting AIL functions to C, and header search configuration.

## Declarations

**ffi.decl.1** — `use c "header.h" as name` makes the C declarations of
`header.h` available under `name`. The header is parsed by the
implementation's own C header parser. *(ADR 0103)*

**ffi.decl.2** — `extern c fn name(params) Result` declares one C function.
*(ADR 0103)*

**ffi.decl.3** — Types used in C declarations MUST have C-compatible fixed
layouts. *(ADR 0103, ADR 0013)*

## Calls

**ffi.call.1** — Calling a C function is an unsafe operation
(`unsafe.op.1`). *(ADR 0104)*

## Linking (Linux)

**ffi.link.1** — On Linux, a program that declares or imports no C functions
MUST NOT depend on libc. A program that does is linked against the system
libc. *(ADR 0105, ADR 0020)*
