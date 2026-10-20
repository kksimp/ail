# 0258. `ail api --locals` lists the names in scope at a position, from facts the checker records

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.loc.1–2 ([semantic-api.md](../design/semantic-api.md))

## Context

An agent or editor asks which names are in scope at a position, with their types. The checker knows (it keeps the typed `locals` stack); `ail api` didn't expose it. Criterion: **AI understanding** (one query instead of reading and inferring), then tokens.

## Options considered

### Option A — the checker records each local when it leaves scope; scope is found by indentation (chosen)
- **Pros:** no second inference; a few lines in `check.rs`; the block structure of AIL is its indentation, so scope is exact for every block kind.
- **Cons:** the AST has no block end positions, so a scope's end comes from the source text.

### Option B — add end positions to the AST
- **Pros:** exact.
- **Cons:** touches the parser and every `Block`; ten lanes edit those files.

### Option C — re-run inference in `api`
- **Pros:** none that A lacks.
- **Cons:** a second type checker to keep in step.

## Decision

`ail api --locals FILE LINE COL` prints `{api,file,line,col,locals}`; each local is `{name,type,mut,kind,line,col}` with `kind` one of `self`, `param`, `loop`, `binding`. Locals are in scope from the line after their `:=` (or from the first statement of the block they scope) until a line indented less than that line.

## Rationale

A is cheapest to build and merges cleanly. Indentation-defined scope matches the language (lex.indent), and blank and comment lines are skipped.

## Consequences

- Editors can offer completion of locals with types.
- A multi-line initialiser's continuation lines see the name early (they are indented deeper); rare, accepted.
- Pattern names in an `if x := ...` header are in scope from the body's first statement, not the header line.

## Rejected alternatives

- Option B: needless churn in shared files for the same answer.
- Option C: duplicates inference.
