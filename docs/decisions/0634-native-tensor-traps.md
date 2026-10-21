# 0634. Tensor failures in native code: a failure record and trap kind 8

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.tensor.2](../spec/23-native.md), std.tensor.2-4, std.tensor.7, std.tensor.11-12
- **Builds on:** [ADR 0405](0405-native-traps.md) (trap messages), [ADR 0633](0633-native-tensor-kernels.md)

## Context

A trap prints `PATH:LINE:COL: trap[RULE]: MESSAGE`, the source line, and a
caret (native.build.1). The tensor messages contain shapes (`[2, 3]`,
`2x3`), which the six message kinds of ADR 0405 cannot build.

## Decision

A runtime tensor function that finds a bad shape or axis stores a failure
record (a code and up to three numbers and two shapes) in global slot 48 and
returns a harmless tensor. The lowered code follows every such call with
`rt.tensor_failed`, and on failure jumps to a MIR `trap` of kind 8 at the
call's site (rule `std.tensor.N`); kind 8 prints `tensor_trap_text`, which
rebuilds the interpreter's message from the record.

## Alternatives rejected

- **A string built at the failure** and passed to `trap`: needs an
  allocation and the formatter inside the failing path; the record is two
  stores.
- **Checking shapes in the lowered code.** Duplicates the rules in MIR for
  each operation; the runtime already has the shapes.

## Consequences

`rt.trap` grows one `elif`; every program now links `tensor_trap_text`
(about 1 KB, dead-stripped otherwise only if `trap` is). Slot 48 is taken.
