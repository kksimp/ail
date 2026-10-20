# 0314. Standard modules written in AIL, starting with `std.ui.widgets`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.ui.19](../spec/17-std.md)
- **Extends:** [ADR 0157](0157-standard-os-modules.md), [ADR 0097](0097-directory-modules.md)

## Context

Widgets (lists, fields, dialogs) are pure logic over `std.ui`. Writing
them in Rust would add bootstrap code the real toolchain throws away;
writing them in AIL keeps them (ADR 0022). Until now every standard module
was built into the bootstrap, and only its types were AIL source
(`std/types.ail`). Deciding criteria: **AI understanding** (a standard
module behaves like any other module) and keeping the bootstrap small.

## Options considered

### Option A — embed AIL source in the toolchain and load it as a module (chosen)
`use std.ui.widgets` makes the loader read `bootstrap/src/std/ui/widgets.ail`
(compiled into the binary) as the module `std.ui.widgets`. Everything else
is the ordinary module machinery: qualified items, `pub`, diagnostics
pointing at `<std>/ui/widgets.ail`.
- **Pros:** no new semantics; the module is tested like user code; the
  same path serves later AIL-written standard modules.
- **Cons:** the one standard module with a submodule path.

### Option B — widgets in Rust behind `std.ui`
- **Cons:** throwaway code; every widget change is a bootstrap change.

### Option C — a package under `tools/`
- **Cons:** programs would need a path dependency to get standard widgets.

## Decision

Option A. Standard modules written in AIL are listed in
`stdlib::AIL_MODULES` (path, file name, source).

## Rejected alternatives

- **Rust widgets:** throwaway.
- **A package:** not standard.
