# 0182. List, set and map comprehensions

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Expressions; token efficiency
- **Spec rules:** [expr.comp.1–4](../spec/05-expressions.md#comprehensions)

## Context

Round 3 of the token measurements ([program-measurements-r3.md](../design/program-measurements-r3.md))
compared each function of the seven corpus programs with its Python twin. The
largest gaps that a language change can close were loops that build a
collection: AIL wrote

```ail excerpt
mut parts := []
for item in items
    parts.push(compact(item))
```

where Python writes `[compact(item) for item in items]`. In `sheet`,
`eval_order`, `table` and `formulas` alone cost 200 tokens more than Python's
list, set and dict comprehensions. Closures, and so `map`/`filter` methods,
are not designed ([ADR 0121](0121-for-mut.md)).

## Options considered

### Option A — Python-shaped comprehensions (chosen)
`[e for x in xs if c]`, `{e for ...}`, `{k: v for ...}`, with any number of
`for` clauses, each with an optional `if`.
- **Pros:** −0.9% tokens on the corpus (−118 tokens; `sheet` −1.6%, `json`
  −0.9%), and another −40 in `json` where a map comprehension replaces a
  constructor function; no closures needed; the form models know best from Python;
  replaces a `mut` accumulator with one immutable value.
- **Cons:** a second way to build a list; the element comes before the loop
  that binds its names.

### Option B — loop-first comprehensions, `[for x in xs: e]`
- **Pros:** names are bound before they are used, left to right.
- **Cons:** the same tokens; a `:` that AIL uses nowhere else for blocks; no
  model has seen it.

### Option C — `map`/`filter` methods with closures
- **Pros:** composes; familiar from Rust and TypeScript.
- **Cons:** needs the closure design (captures, `mut` captures, effects),
  which is far larger; `xs.map(|x| x * x)` costs more tokens than the
  comprehension.

### Option D — keep loops
- **Pros:** one way to build a collection.
- **Cons:** costs the tokens above, in the programs where AIL trails Python.

## Decision

Adopt Option A, with `continue` skipping an element and `break` ending the
comprehension, so `[t.to_int() else continue for t in texts]` keeps the
values that parse.

## Rationale

On the 2% threshold of ADR 0076 the saving across the corpus (−0.9%) is a tie
on tokens, but it is concentrated where AIL loses to Python (`sheet` went from
+2.1% to −0.4% with this and [ADR 0183](0183-bare-variant-patterns.md)). **AI
understanding** then decides: the comprehension is a single immutable
expression with no `mut` accumulator, its shape is one models already read
and write fluently, and it removes a class of mistakes (pushing into the
wrong list, forgetting to return it). `continue` and `break` keep their loop
meaning, since a comprehension is a loop.

## Consequences

- The bootstrap parses, checks and runs comprehensions (`Expr::Comp`); `ail
  fmt` splits a long one before each `for` with no trailing comma (fmt.comp).
- A comprehension isn't a constant expression.
- Not included: `for mut` clauses (a comprehension builds a new collection),
  generator expressions, and destructuring loop variables.

## Rejected alternatives

- **Loop-first syntax:** no token gain, unfamiliar.
- **Closures with `map`/`filter`:** a much larger design, and more tokens.
- **Keep loops:** leaves AIL behind Python on the programs that build collections.
