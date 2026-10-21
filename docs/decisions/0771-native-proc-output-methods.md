# 0771. Native `Output.ok()` and `Output.text()`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — native backend
- **Spec rules:** std.proc.9, std.proc.10, native.subset.1
- **Builds on:** [ADR 0770](0770-native-proc-run.md)

## Context

With `proc.run` native, the next first error in the programs that call it
is `calling proc.Output.ok` and `proc.Output.text` (`tool_fmt_check`,
`tool_fix_preview` and others). `Output` is a standard-library struct whose
methods the prelude defines for the interpreter; the lowerer has no body
for them.

## Decision

`Lower.struct_call` lowers the two calls inline for the `proc.Output`
struct that ADR 0770 registers: `ok()` is `status == 0` (one comparison),
`text()` is `rt.str_cat(out, err)`. Any argument, or another method name,
is the usual `native.subset.1` report. The change is in `lower_struct.ail`
only, with no edit to `lower.ail` and no new runtime function.

## Why not

- **Compile the prelude's methods:** the prelude is interpreter source with
  features outside the subset, and two one-line bodies don't justify a
  second code path for std structs.
- **Runtime functions `proc_ok` and `proc_text`:** a call for a comparison
  costs more than the comparison.

## Evidence

`bootstrap/tests/native/run/proc_output_methods.ail` matches `ail run` on
x86-64 and on aarch64 under qemu.
