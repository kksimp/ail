# 0860. The machinery of `ail fix` is ported to AIL (`compiler/fix/`), with the same fixes and output as the Rust tool

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** AI (delegated, ADR 0076; allocated by the integrator)
- **Topic:** 23 — Rust removal (RR-5)
- **Spec rules:** fix.apply.1-4, fix.edit.1-6
- **Builds on:** [ADR 0203](0203-ail-fix-command.md), [ADR 0335](0335-explain-and-fix-coverage.md)

## Context

`ail fix` applies the machine-applicable fixes that diagnostics carry, checks
again, and repeats (ADR 0203). Its Rust code has three parts: the fix and edit
types with anchors that name a position by a rule (ADR 0335), the resolution of
those anchors against the source lines, and the driver. The checker attaches the
fixes at about 85 sites (B4 ports those to the AIL checker).

## Decision

1. **`compiler/fix/` holds the types, the resolution and the driver.** `Edit`
   and `Fix` have the Rust constructors (`replace`, `insert`, `call`,
   `operands`, ...). The anchor is a string (`"line_end"`, `"call"`, ...), the
   shape the AIL checker uses. An edit whose anchor doesn't match its line is
   dropped with its fix, never guessed.
2. **The driver takes each round's check from the caller.** `begin`, `step`,
   `canonicalize` and `finish` need neither the module loader nor function
   values (not native yet), so the same code runs before RR-1's loader lands.
3. **Proof:** `tools/fix/parity.py` compares the settled edits with the Rust
   ones over 1154 fixes (the repository's files and 745 generated mistake
   programs covering every anchor); `tools/fix/run-parity.py` compares the
   driver with `ail fix` using the Rust checker (744 runs, plain, `--dry-run`
   and `--json`). Both found no difference.
4. **Routing `ail fix` to this code** waits for the loader and the AIL
   checker's fixes.

## Rejected alternatives

- **A `command` that takes the check as a function value.** Not native yet.
- **Settling anchors in the checker.** The checker proposes edits without the
  source text (ADR 0335); resolution belongs with the application.
