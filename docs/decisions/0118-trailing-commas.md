# 0118. A trailing comma is allowed in every comma-separated list

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax
- **Spec rules:** [gram.list.1](../spec/03-grammar.md)

## Deciding criterion

Token usage **ties**: the comma is optional, and the canonical form uses it
only on multi-line lists, where it costs one token per list. **AI
understanding** decides. With a trailing comma, every element line of a
multi-line list looks the same, so adding, removing or reordering an element
edits only its own line. Forgetting the comma when appending is a common AI
edit error ([ai-author-wishlist.md 3.1](../design/ai-author-wishlist.md)).

## Decision

1. In argument lists, parameter lists, list literals and struct literals, a
   comma may follow the last element.
2. The canonical formatter writes a trailing comma when the list spans several
   lines, and none when it fits on one line.

```ail fragment
struct Student
    name str
    score int

students := [
    Student(name: "Ada", score: 95),
    Student(name: "Linus", score: 82),
]
```

## Rejected alternatives

- **No trailing commas** (C function calls, JSON): appending an element
  changes two lines, and a missed comma is a syntax error.
