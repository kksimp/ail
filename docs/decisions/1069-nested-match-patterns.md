# 1069. Nested patterns in variant fields, checked as a pattern matrix

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer put every finding of the pre-release language review into 0.1.0 on 2026-11-12, AI understanding first
- **Topic:** 3 — Expressions and control flow
- **Spec rules:** [expr.match.13](../spec/05-expressions.md); amends expr.match.4 and expr.match.6
- **Builds on:** [ADR 0123](0123-variants-with-data.md), [ADR 0138](0138-literal-and-or-patterns.md), [ADR 0183](0183-bare-variant-patterns.md), [ADR 0453](0453-range-patterns.md), [ADR 0791](0791-tuples.md)

## Context

A variant pattern took one name or `_` per field (expr.match.6). Matching
`Add(Num(a), Num(b))` needed a `match` inside a `match`, with a `_` arm at each
level that the outer arm then has to repeat. Interpreters, constant folders
and tree rewrites, which agents write often, nest two or three levels, and the
compiler's own source nests `match` for the same reason. Tuple patterns
(type.tuple.6) already nest, so the grammar had the shape; variant fields
didn't.

## Options considered

### Option A — any pattern in a field; exact exhaustiveness for a `match` that nests (chosen)
- A field holds a name, `_`, or any pattern of the field's type: a variant
  pattern (nested to any depth), a literal, a range, a tuple pattern or an
  or-pattern.
- A `match` on an enum whose arms nest is checked by usefulness over a
  pattern matrix (Maranget, "Warnings for pattern matching", 2007). Every
  alternative must match a value the earlier unguarded arms leave
  (expr.match.5); the values no arm takes are named as patterns,
  `Add(Num(_), Neg(_))`, at most eight, and the fix adds one `todo` arm each.
- **Pros:** one arm per case, as the reader thinks of it; missing nested
  cases are named rather than only "add `_`", which is what an agent needs to
  finish the match in one round.
- **Cons:** a second coverage path in the checker. A `match` with no nested
  arm keeps the per-variant check, so no existing diagnostic changes.

### Option B — nested patterns that need a `_` arm, as tuples do
- **Cons:** the checker can't say which nested case is missing, and an
  agent's `_` hides cases a later variant adds. Fails AI understanding.

### Option C — no nesting; guards instead (`Add(a, b) if a is Num`)
- **Cons:** needs a variant test expression, and the fields still need a
  second `match` to bind.

## Decision

Option A. Rules:

1. **Bare names bind.** In a field, a plain name followed by `,` or `)` binds
   the field and `_` ignores it, as in tuple patterns. A name that is a
   variant of the field's enum is an error (expr.match.13) with a fix to the
   qualified spelling, so `Cons(_, Nil)` can't silently bind a variable called
   `Nil`. This applies to flat patterns too: it was a silent binding before.
   The rule is local: a reader never needs the field's type to know whether
   a name binds. A fieldless variant inside a field is written `Items.Nil`.
2. **Or-patterns nest** and still bind no names (expr.match.8).
3. **Guards** cover nothing (expr.match.11): a guarded arm is checked for
   redundancy but doesn't join the matrix.
4. **Literals and ranges** are constructors that never complete their type,
   so a field of type `int`, `str` or `char` matched by literals needs a name
   or `_` somewhere below. Ranges are never compared with each other or with
   literals for redundancy (as in ADR 0453).
5. **Scope of the exact check:** a `match` whose scrutinee is an enum and
   some arm nests. A tuple scrutinee keeps type.tuple.6 (it needs a catch-all
   arm), even with nested elements; making tuple matches exact is a separate
   change.
6. **Native:** a nested pattern tests the variant index, then each field in
   order (`compiler/mir/lower_nested.ail`); the first failing field goes to
   the next arm. The match tree (ADR 0890) is not built for a `match` that
   nests. No interpreter code (ADR 0865).
7. The compiler's own source does not use nested patterns in 0.1.0, because
   the pinned seed builds it.

The syntax tree gains `Pattern.Nested(qualifier, name, fields, pos)`, made
only when a field holds more than a name or `_`; flat `Pattern.Variant` is
unchanged.

## Consequences

- Witness lists stop at eight cases, with ", and maybe more".
- A witness writes fieldless variants qualified (`Expr.Nil`) even at the top
  level, so every suggested arm parses wherever it is pasted.
- Not designed: binding a whole sub-value (`n @ Num(_)`), or-patterns that
  bind names, patterns on optionals and on struct fields (a `match` takes
  neither yet).

## Rejected alternatives

- Nested patterns with a required `_` arm (option B): rejected because the
  missing nested case can't be named, which costs AI understanding.
- Guards with a variant test (option C): rejected because the fields still
  need a second `match`, so it saves nothing.
- Bare fieldless variants resolved by the field's type (`Cons(_, Nil)` as a
  match): rejected because whether a name binds would depend on a type the
  reader may not see; the tuple rule (type.tuple.6) already chose the local
  reading.
