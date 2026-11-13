# 1083. `ail ask` answers honestly: methods, misses and `ail ask how`

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076), within the AI-first review the designer approved for 0.1.0
- **Topic:** 22 — Tooling
- **Spec rules:** api.find.1-7 (extends [1051](1051-core-card-and-std-lookup.md); no rule of it is reversed)

## Context

A probe of `ail ask std` on 2026-11-12 showed it answering with a stray word: "priority queue heap" gave
`audio.stream_open`, "sort list by key" gave `ui.Key`. It did not index the methods of `str`, `List`, `Map` and the
other built-in types at all, the things an agent reaches for most. A wrong answer costs a round and teaches the agent
that a missing function exists; an honest miss would have sent it to the right idiom.

## Options considered

### Option A: index methods from a checked table, a minimum score, an honest miss with a hint, and `ail ask how` (chosen)
- **Pros:** every lookup ends in a right answer or a clear "not in std" with the idiom; the how-to blocks are
  compiled by the existing docs check; the table is audited against the spec and the checker.
- **Cons:** two small tables to keep in step. `tools/check-method-table.py` fails when a rule is missing or a checker
  method has no row.

### Option B: derive method signatures from `compiler/check/builtins.ail`
- **Pros:** no second list.
- **Cons:** the checker has no words or rule IDs, and `find` would depend on the checker's internals.

### Option C: only tighten the score
- **Cons:** methods stay unfindable.

## Decision

- Methods are indexed from `docs/design/builtin-methods.txt` and printed as `List.sorted(by: |x| key) List[T]   (type.coll.11)`.
- A function or method is printed only when answered: a name hit that is not a generic word, or two words, or a lone module
  word. A miss prints `nothing in std matches WORDS` and the row of `docs/design/not-in-std.txt`, exit 1.
- `ail ask how WORDS` reads `docs/how-to.md`; the MCP tool `ail_find` takes `how`.

## Rationale

AI understanding first (ADR 1077): a wrong answer is worse than none. Tokens: one extra line on a miss, and the how-to
answer is a few lines instead of a card section.

## Consequences

Every new built-in method needs a row (the audit says so). The `Heap` row of the miss table must agree with the std Heap
lane. Not done: ranking by usage, and fuzzy spelling.
