# 0156. The bootstrap gives every item its qualified name before checking

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 12 — Modules
- **Spec rules:** [mod.unit.1–2](../spec/12-modules.md), [mod.use.1–2](../spec/12-modules.md)

## Context

The bootstrap checker and interpreter work on one program with one global
namespace. Modules need per-module names without rewriting either.

## Options considered

### Option A — rename before checking (chosen)
All files are parsed as one program, each file's lines numbered from its own
multiple of 2^20 so a position names its file. Items of module `a.b` are
renamed `a.b.Item`, and every reference (bare inside the module, `b.Item`
through an import) is rewritten to that name. Privacy of items is checked
during the rewrite; fields and methods in the checker, from positions.
- **Pros:** no change to type checking or evaluation; small, local edits.
- **Cons:** diagnostics and text forms show the module path (`geo.Point`),
  not the importer's alias.

### Option B — a module-aware checker and interpreter
- **Cons:** restructures the two largest files while nine other lanes edit
  them, for no difference in behaviour.

## Decision

Option A. A type's text form and diagnostics use its qualified name,
`geo.Point(x: 1, y: 2)`; root-module items keep their bare names.

## Rationale

Speed of delivery and merge safety, with AI understanding favouring the
qualified form in output: `geo.Point` is unambiguous, while `Point` could be
any module's.

## Consequences

- Runtime and checking speed are unchanged: renaming is one pass at load.
- Programs are limited to 1023 files (positions below the prelude's line).
- The future compiler is free to implement modules natively; only the
  qualified text form is observable.

## Rejected alternatives

- **B:** large, conflict-prone restructuring for identical behaviour.
- **Showing the importer's alias in output:** the same type would print
  differently in different files.
