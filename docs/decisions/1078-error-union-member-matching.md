# 1078. Matching on which member of an error union failed

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** Designer (Kaleb, 2026-11-12: every recommendation of the
  language review goes into 0.1.0; "never compromise on AI understanding").
  Details: AI (delegated, ADR 0076).
- **Topic:** Errors
- **Spec rules:** err.match.1, err.match.2, err.match.3; amends err.else.3
  and makes err.type.4 checkable
- **Resolves:** the union part of [OQ-029](../open-questions.md)

## Context

`else err` on a call that can fail with `! A | B` (written or inferred) bound
`err` to a union whose only use was its display text. Code that needed to know
which error it got could not ask: `match err` was `bootstrap.unsupported`, and
fields of the member were unreachable. err.type.4 required exhaustive matching
on an inferred set but nothing implemented it. The language review of
2026-11-12 asked for member matching in 0.1.0.

## Options considered

### Option A — member type patterns with narrowing (chosen)
```ail excerpt
n := load(path) else err
    match err
        IoError => print("io: {err.kind}")      # `err` is an IoError here
        ParseError.Empty => print("empty")
        ParseError.BadDigit(c) => print("bad {c}")
    0
```
- **Pros:** reads like every other `match`; the arm's type is where the name
  is used (local reasoning); variants of an error enum reuse enum patterns;
  exhaustiveness per member, with a fix that adds the missing arms.
- **Cons:** a type name is a new kind of pattern, valid only here.

### Option B — a binding form `IoError as e =>`
- **Pros:** a new name, so `err` keeps one type.
- **Cons:** two names for one value; more tokens per arm; `as` exists nowhere
  else in patterns.

### Option C — `match` with success and error arms (`Ok(v) =>`, `Err(e) =>`)
- **Pros:** one construct for both outcomes.
- **Cons:** duplicates `else err` and `if x := e`; still needs a way to tell
  members apart.

### Option D — test functions (`err.is(IoError)`, `err.as(IoError)`)
- **Pros:** no grammar change.
- **Cons:** no exhaustiveness; `as` returns an optional to unwrap again.

## Decision

`match name`, where `else name` bound an error of two or more types, takes as
patterns member types (`T`, `m.T`), variants of a member error enum written
with its name (`T.Variant(...)`), `_`, and alternatives (err.match.1). In an
arm of member `T` the name has type `T`; a member type pattern takes no
fields; the scrutinee is the bound name itself (err.match.2). Every member is
covered by its type, all its variants, or `_`; a missing member is an error
whose fix adds `T => todo` arms (err.match.3).

## Rationale

AI understanding (ADR 1077): the narrowed name keeps one name per value and
the type is visible at the arm; the checker lists the members and adds the
missing arms. Human readability: the arm says which error it handles in the
words of the signature. Tokens: one name per arm, fewer than B.

## Consequences

- Each failure records its type next to its value: the runtime's error slot
  holds a tag (`rt.err_tag`, a hash of the type's key; global slot 67, and the
  pool range 13312-13832 for workers). The lowering tags every `return E(...)`
  and `Absent`; the runtime tags `fs.IoError`, `proc.ProcError`,
  `net.NetError`, `compress.CompressError` and `Trapped`.
- `else err` of a union binds `err` to the display text as before, plus two
  hidden locals (the value and the tag) that the `match` reads
  (`compiler/mir/lower_errmatch.ail`).
- A union value can't be copied to another name and matched; the `match` must
  be on the bound name (err.match.2).
- `_` on a union warns as `expr.match.3` does on an enum.

## Rejected alternatives

- B: two names for one value and more tokens.
- C: duplicates existing forms and doesn't tell members apart.
- D: no exhaustiveness checking.
