# 0154. Imports come first, apply to their file, and need unique names

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 12 — Modules
- **Spec rules:** [mod.use.2–3](../spec/12-modules.md), [mod.use.6](../spec/12-modules.md)

## Context

ADR 0099 defines `use a.b.c` and qualified access, but not where a `use`
may appear, whether it applies to its file or its whole module, or how
clashes are reported.

## Options considered

### Option A — per file, at the top (chosen)
- **Pros:** every name used in a file is explained at the top of that file
  (local reasoning); moving a file between modules keeps its imports; no
  token difference.
- **Cons:** two files of one module that use the same module both import it.

### Option B — per module (an import in one file serves all files)
- **Cons:** a file's names depend on its siblings' headers; an AI must read
  the whole directory to resolve `http.get`.

### Option C — `use` anywhere at the top level
- **Cons:** imports scattered among declarations are easy to miss.

## Decision

`use` lines come before a file's other declarations (`mod.use.3`) and apply
to that file. An import name must differ from the file's other import names
and from the module's top-level names (`mod.use.6`). A bare import name is
not a value, a qualifier that names no import is an error, and `pub use`
(re-exporting) is rejected (`mod.use.2`). No measurement was needed: the
choice doesn't change what is written.

## Rationale

AI understanding decided: reading one file's header tells where every
qualified name comes from, which is the point of qualified imports (ADR 0084,
ADR 0099).

## Consequences

- Locals can't take an import's name (`expr.bind.5` already says so).
- Re-exports can be added later if packages need a facade.

## Rejected alternatives

- **B:** non-local: a file's meaning depends on other files.
- **C:** imports are harder to find, with no benefit.
