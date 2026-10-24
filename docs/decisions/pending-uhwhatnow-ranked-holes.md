# NNNN. `uhwhatnow?`: how the hole that asks ranks what fits

- **Status:** Accepted (number pending from the integrator; rename this file then)
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 20 — AI semantic API

## Context

ADR 0868 item 2 (designer) makes `uhwhatnow?` a typed hole whose warning ranks the
in-scope values and calls that fit. It leaves open what counts as a candidate, the
ranking order, the output form, and how the warning relates to `--budget` (item 6).

## Decision

- **Token.** `uhwhatnow?` is one keyword token (`lex.kw.3`). `uhwhatnow` alone is
  reserved: as an expression it gets a fix that adds the `?`; as a name (`uhwhatnow :=`,
  `uhwhatnow(`) it gets a plain reserved-name error, since adding `?` there would not help.
- **Candidates and rank** (`expr.hole.3`), best first:
  1. locals, innermost first;
  2. fields of `self`, written bare;
  3. fields of other locals (`c.total`), skipping hidden fields;
  4. calls of the local's methods and of the module's own non-generic free functions
     (not `main` or tests). Each argument is the local named like the parameter, else the
     innermost local of its type, else `todo`. A call costs 2 more per `todo` argument,
     1 more if it can fail (written `?` when the enclosing function returns a `Result`,
     else `else todo`), and 3 more when it calls the function being checked.
  A value fits when its type is the expected type, or the expected type is `T?` and the
  value is a `T`. Ties break by name.
- **Output.** One warning: "uhwhatnow?: expected `T`; best fit: `X`", one note per
  candidate ("1. `X`: why"), at most eight, then "and N more"; one fix per shown
  candidate, so `ail fix` can apply the best one. With no expected type or no fit the
  warning lists the locals as `todo` does (`expr.hole.1`).
- **Budget.** The ranked list is the expanded form. When `--budget` lands (item 6), a
  tight budget keeps the first line and the best fix; `todo` stays one line at any budget.
- **Native.** The hole lowers as `todo` does; the trap names `uhwhatnow?`.

## Consequences

The ranking lives in `compiler/check/ask.ail`; the parser records each `uhwhatnow?`
position on `ast.Program.asks` (optional, so older JSON dumps still read). The Rust
bootstrap does not know the token; spec examples are tagged `skip=bootstrap`.

## Rejected alternatives

- **Type-directed search over std and other modules:** far more candidates, most noise,
  and a slower check; an agent asks `ail api` for those.
- **Generic functions as candidates:** they need type-argument inference against the hole
  type; left for later rather than ranking guesses.
- **A separate hole AST node:** `Expr.Hole` plus a position list keeps every pass that
  handles holes unchanged.
- **Ranking by name similarity:** the hole has no name to compare with; nearness of the
  binding is the better signal of intent.
