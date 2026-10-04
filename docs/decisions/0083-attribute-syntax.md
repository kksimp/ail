# 0083. Attributes are written `@name` or `@name(args)` before a declaration, on the same line

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax
- **Spec rules:** [gram.attr.1–2](../spec/03-grammar.md)
- **Resolves:** OQ-027
- **Evidence:** `corpus/r3/`: own-line `@inline` 19.2, same-line `@inline fn` 18.2, keyword modifier `inline fn` 17.2

## Deciding criterion

**Token usage ties**: a one-token difference, on rare constructs.
**AI understanding decides**: one uniform `@` mechanism, visibly distinct
from semantic keywords (like `unique`, `mut`, `move`), is easier to recognise
than a growing set of modifier keywords. Same-line placement is 1 token
cheaper than own-line.

## Decision

1. An attribute is `@name` or `@name(arguments)`.
2. Attributes precede the declaration they apply to, on the same line. The
   formatter moves them onto their own preceding line only when the line
   would exceed the line-length limit.

```
@inline fn hash(x u64) u64
    x * 31

@layout(c) struct Header
    magic u32
```

## Consequences

- Attributes carry hints and metadata (layout, inlining, tests, deprecation,
  suppressing warnings, FFI names). Features that change ownership or
  semantics stay keywords (`unique`, `mut`, `move`).
- The set of standard attributes is defined per feature.
