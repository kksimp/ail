# 0260. `ail api --refs` lists a symbol's definition and uses across user modules, and a local's uses in its scope

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.refs.1–3 ([semantic-api.md](../design/semantic-api.md))

## Context

Rename and go-to-definition need every reference to the symbol under the cursor. `--symbols` already resolves uses file-wide (ADR 0151) but only in the root file and keyed by id, not by cursor. Criterion: **AI understanding**.

## Options considered

### Option A — reuse the reference pass, whole program, select by cursor (chosen)
- **Pros:** one resolver for `--symbols` and `--refs`; imported modules are already in the program with qualified names.
- **Cons:** locals are unresolved names, matched by name within the local's scope.

### Option B — text search
- **Pros:** easy.
- **Cons:** wrong inside strings and for shadowing-like names; no cross-module ids.

## Decision

`ail api --refs FILE LINE COL` prints `{api,file,line,col,symbol,refs}`; each ref is `{file,line,col,def}`, sorted by file order, line and column, with the definition (`def:true`) included. `symbol` is a declaration id or `local:NAME`. Files are the paths of the modules' source files.

## Rationale

A needs a dozen lines on top of ADR 0151 and answers across modules. Locals cannot be shadowed (expr.bind.4), so matching by name within the scope is exact.

## Consequences

- Rename is a list of edits.
- A use of a module item is reported at the start of its qualified name (`geo.origin` at `geo`).

## Rejected alternatives

- Option B: unreliable.
