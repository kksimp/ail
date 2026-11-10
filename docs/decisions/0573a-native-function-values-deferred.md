# 0573a. Function values in native code wait for the language

- **Status:** Superseded by [0573](0573-native-function-values.md) (this earlier draft shared the number 0573; renamed 0573a, no links point here)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [type.fn.1-3](../spec/04-types.md) (lane 130, pending)
- **Builds on:** ADR 0455 (lane 130, PR #149, pending), [ADR 0571](0571-key-functions-inline.md)

## Context

Function types and declared functions as values (`fn(A) R`, `f := double`,
`f(x)`) are in lane 130's PR #149, not yet in the batch branch. Closures
and lambdas as values don't exist in the language (ADR 0455 defers them).
The compiler's own source uses neither.

## Decision

Not implemented in this slice, so nothing depends on it for self-compile.
When #149 merges, the design is: a `fn` value is a pointer to a closure
object (kind 6, header word at pointer − 8) whose first word is the code
address; a declared function gets one static object per function. A call
through a value is a new MIR op `call_indirect` (callee, arguments) that
both generators emit as a call through a register with the argument
registers of native-abi.md §1. Captures, when the language gets them,
follow the code word. The op needs lane 120's agreement before it is added.

## Rejected alternatives

- **Writing the op now against a language feature that is unmerged.** Its
  type rules (`type.fn.*`) may still change in review.
- **Monomorphising callbacks away.** Works for parameters passed
  known functions but not for values stored in structs or lists.
