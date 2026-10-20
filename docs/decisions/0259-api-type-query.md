# 0259. `ail api --type` gives the type of the expression at a position, and the callee for a call

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.type.1–2 ([semantic-api.md](../design/semantic-api.md))

## Context

Hover needs the type of what is under the cursor, and for a call the callee's contract. Criterion: **AI understanding**, then tokens.

## Options considered

### Option A — record each expression's type as the checker computes it, keyed by start position (chosen)
- **Pros:** the checker's own answer, including inferred and generic instantiations; one wrapper around `expr`.
- **Cons:** several expressions share a start position; the innermost is chosen. Recording is off for normal runs.

### Option B — answer only for declared names
- **Pros:** trivial.
- **Cons:** no types of calls, fields or locals.

## Decision

`ail api --type FILE LINE COL` finds the name or number token at the position and prints `{api,file,line,col,text,start_col,end_col,type,symbol,callee}`. `type` is the innermost expression starting at the token. `symbol` is the id the checker resolved (or `null`). `callee` is the outline item of a function or method (signature, params, returns, effects, `requires`, `ensures`, doc) or `null`. Exit 1 with a JSON error on stderr when nothing is there.

## Rationale

A is the checker's truth; the wrapper costs one branch per expression when not recording (measured, see the PR).

## Consequences

- Hover shows `type` and, for calls, the contract.
- A qualified module use (`geo.origin`) is one token reported at `geo`.
- Operators and parentheses have no answer; ask on a name.

## Rejected alternatives

- Option B: too little for agents.
