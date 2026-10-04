# 0037. Signatures are written `fn name(param Type, ...) ReturnType`, with no colons or arrows

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax; 3 — Grammar
- **Spec rules:** none yet (grammar chapter)
- **Evidence:** [design/syntax-measurements-r1.md](../design/syntax-measurements-r1.md#addendum-indentation-signatures-declarations)

## Context

The brief proposed `fn add(a i32, b i32) i32`. Measured on one signature:
17 tokens without colons or arrows, versus 20 Rust-style (`a: i32 ... -> R`)
and 19 with colons but no arrow (cl100k; other tokenizers are proportional).

## Options considered

### Option A — `fn f(a i32, b str) R` (chosen)
- **Pros:** ~15% fewer tokens per signature. Matches the brief. Clean to read.
- **Cons:** Complex types can run together visually (e.g. `f fn(i32) i32`).
  The formatter must keep this readable.

### Option B — `fn f(a: i32, b: str) -> R`
- **Pros:** Punctuation visually separates names, types and returns.
- **Cons:** ~15% more tokens, and exactly the punctuation the brief wanted
  to avoid.

### Option C — `fn f(a: i32, b: str) R`
- **Pros:** A middle ground.
- **Cons:** ~10% more tokens, and inconsistent.

## Decision

A parameter is written as its name followed by its type, separated by a space.
The return type follows the closing parenthesis of the parameter list. There
is no colon and no arrow.

## Consequences

- Open grammar questions: grouping parameters that share a type (`a, b i32`),
  the syntax of function types, multiple return values, and named-argument
  syntax at call sites ([ai-author-wishlist.md 1.5](../design/ai-author-wishlist.md)).
- The function keyword is shown as `fn` in examples. Keyword spelling is a
  separate decision (keyword length is token-free, per the measurements).
- The type-annotation form for variables (e.g. `x i64 := 0` or similar) should
  be consistent with this decision. To be decided with the declaration
  grammar.

## Rejected alternatives

- **Rust-style:** more tokens and more punctuation.
- **Colon without arrow:** inconsistent, and still more tokens.
