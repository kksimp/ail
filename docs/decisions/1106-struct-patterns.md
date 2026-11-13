# 1106. Struct patterns and fields matched by name

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer put every item of the AI-first review into 0.1.0 on 2026-11-12 (package S15)
- **Topic:** 3 — Expressions and control flow
- **Spec rules:** [expr.match.14](../spec/05-expressions.md); amends expr.match.4
- **Builds on:** [ADR 1069](1069-nested-match-patterns.md), [ADR 0058](0058-struct-literal-syntax.md), [ADR 0142](0142-positional-constructor-arguments.md)

## Context

A `match` couldn't take a struct value, so `if p.x == 0 and p.y == 0` chains
stood in for patterns, and a struct inside a variant field (`Word(text, at)`)
could only be bound whole. Variant patterns were positional only, while
variants are built with field names (`Shape.Rect(w: 2, h: 3)`): an agent that
writes the label it used to build the value got a parse error.

## Options considered

### Option A — `Name(f: p, g)`, the literal's own spelling (chosen)
- A pattern is written as the value is built: `Point(x: 0, y)`. `f: p`
  matches field `f`; a bare `g` binds field `g` to the name `g`, the
  shorthand struct literals already have; unnamed fields match anything.
- The same form names variant fields: `Rect(w: 0)`, `Op(c: '+' | '*')`.
- **Pros:** nothing new to learn; a pattern reads as "a value like this";
  fields can be added to a struct without breaking a pattern that doesn't
  need them, and the pattern says which fields it looks at.
- **Cons:** a field left out is silently "any": accepted, since a pattern
  that names its fields can't be misread by position.

### Option B — positional struct patterns, `Point(0, y)`
- **Cons:** the meaning depends on declaration order the reader can't see;
  reordering fields silently changes every pattern. Fails AI understanding.

### Option C — require every field, or a `..` for the rest
- **Cons:** more tokens and an extra rule for no gain in clarity: the
  named fields already say what is tested.

## Decision

Option A.

1. On a struct value, `Point` and `Point(x, y)` (bare field names, each
   binding the field of that name) match every value; `Point(x: p, ...)`
   matches by name. A struct pattern never matches by position:
   `Point(0, y)` is an error (expr.match.14).
2. Each name must be a field (with did-you-mean), at most once. Mixing named
   items with anything other than bare field names (`Point(x: 1, _)`) is a
   parse error: leave the field out instead.
3. On an enum value the same form names a variant's fields. Positional
   variant patterns (expr.match.6) stay.
4. Coverage: a `match` on a struct, or on an enum with an arm that names
   fields, is checked by the pattern matrix of ADR 1069. A struct is one
   constructor whose arguments are its fields, so `Point(x, y)` or `_`
   completes it and missing cases are named. Inside a tuple pattern a struct
   element counts as matching every value only when it binds or ignores all
   fields (type.tuple.6 still asks for a catch-all otherwise).
5. Native: the fields are read by slot (`compiler/mir/lower_nested.ail`),
   after the variant index for an enum. The match tree is not built.

## Consequences

- The syntax tree gains `Pattern.StructPat(qualifier, name, labels, fields,
  pos)`. Bare `Point(x, y)` stays a `Pattern.Variant`; the checker reads it as
  field names when the value is a struct.
- A witness for a struct prints only the fields that matter, or `_` when
  none do.
- Not designed: matching through `ref T`, and struct patterns in `if` or
  `:=` (destructuring), which is a separate question.

## Rejected alternatives

- Positional struct patterns (option B): rejected because their meaning
  depends on field order, which costs AI understanding.
- Required fields or `..` (option C): rejected because it adds tokens and a
  rule while the named fields already say what is tested.
