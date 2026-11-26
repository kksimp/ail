# 0048. There is no null; optional values are written `T?` (shorthand for `Option[T]`)

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 4 — Type system; 8 — Error handling
- **Spec rules:** [type.opt.1–3](../spec/04-types.md)
- **Evidence:** cl100k tokens for a signature's return type: `Option[User]` 10, `User?` 8
- **Amended by:** [ADR 1182](1182-tested-optional-reads-as-its-value.md) (a tested optional reads as its value)

## Context

Brief §6 requires optional values. The brief's diagnostic example already
writes `User?`. Null references are a well-known source of runtime failures.

## Options considered

### Option A — `T?` shorthand for `Option[T]`, no null (chosen)
- **Pros:** ~20% fewer tokens, matches the brief, and readable.
- **Cons:** `?` also means "propagate" after an expression. They are told
  apart by position (after a type vs. after an expression).

### Option B — `Option[T]` only
- **Pros:** No special syntax.
- **Cons:** More tokens on a very common type.

### Option C — Nullable references
- **Pros:** Familiar.
- **Cons:** Null errors at runtime. Contradicts safety by default.

## Decision

1. AIL has no null value.
2. `T?` denotes an optional `T` and is exactly equivalent to `Option[T]`.
3. The canonical formatter writes `T?`.
4. Applying postfix `?` to an optional expression inside a function that
   returns an optional propagates absence (ADR 0045 rule 3).

## Consequences

- Absence-handling forms (`else` fallbacks, safe member access like `?.`,
  matching) are part of OQ-029. The forms in example previews are
  illustrative.
- Whether `?` on an optional inside a function returning `T ! E` is allowed,
  and what error it produces, is part of OQ-029.
- Representation is free to use niche optimisations (e.g. an optional
  reference costs no extra space). That's an implementation detail.

## Rejected alternatives

- **`Option[T]` only:** more tokens for the same meaning.
- **Null:** unsafe.
