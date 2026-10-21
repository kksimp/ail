# 0613. `unsafe` blocks and attributes in native code

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** unsafe.block.1, gram.attr.3-4, test.attr.2, [native.subset.1](../spec/23-native.md)

## Context

The coverage tracker found 9 programs whose only gap was an `unsafe` block,
and 25 `@test` and one `@deprecated` attribute that made `ail build` refuse
a function.

## Decision

- **`unsafe` block:** lowered as a plain block with its own scope. The
  unsafe operations it allows (`unsafe.op.1`) are not native yet and are
  reported where they occur. The manifest rule (`unsafe.block.3`) and the
  operation check (`unsafe.block.1`) are the front end's, before lowering.
- **`@test`:** a test function is not part of the program, so `ail build`
  neither declares nor lowers it. It may use anything, since `ail test` runs
  it on the interpreter.
- **`@deprecated`:** accepted. The warning (gram.attr.4) is the front end's,
  printed at build time as for `ail run`; the executable changes nothing.
  `@inline` and `@pure` were accepted already.

## Rejected alternatives

- **Lowering tests too and dropping them later:** their problems would
  block the build for code that never runs there.
- **A marker instruction for the block:** an `unsafe` block has no run-time
  effect to mark.
