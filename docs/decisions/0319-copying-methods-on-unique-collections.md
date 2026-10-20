# 0319. Methods that copy elements are errors on unique collections; task handles stay out of lists

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 7 — Ownership; 11 — Concurrency
- **Spec rules:** [own.uniq.10](../spec/07-ownership.md#unique-values-in-lists-and-maps), conc.task.5

## Context

With ADR 0318, several built-in methods would duplicate a unique element:
`first`, `last`, `min`, `max` and a map's `get` return a copy; `sorted`,
`take` and `values` build a new collection of copies. Task handles
(`Task[T]`) are must-wait handles checked by conc.task.5, not `unique` types.

## Options considered

### Copying methods: reject with a help (chosen)
Each error names the borrowing or moving form: `xs[0]`, `xs.pop()`, `m[k]`,
`m.remove(k)`, a comprehension over the fields needed.
- **Pros:** one rule; no silent duplication; the fix is in the message.
- **Cons:** some familiar methods are unavailable on these lists.

### Make them borrow (return a borrowed element)
- **Cons:** the bootstrap has no borrowed return values (borrows are
  second-class, ADR 0062).

### Make them move (`first` takes the element out)
- **Cons:** the same name would read or remove depending on `T`.

### Task handles in lists
Chosen: keep conc.task.5 (a handle stays in its block); a `TaskGroup`
already holds any number of tasks and waits them all. Rejected for now:
`List[Task[T]]` with "dropping unwaited handles is an error", which needs
the wait check to follow handles through list operations.

## Decision

On a list or map of a unique type, `first`, `last`, `min`, `max`,
`sorted`, `take`, `get` and `values` are compile-time errors (own.uniq.10);
task handles still can't be stored in a list.

## Rationale

AI understanding decided (criterion 2): each method keeps one meaning, and
an error with the fix beats a run-time double release.

## Consequences

- `check/ownership.rs` (`unique_builtin`) lists which built-in methods
  move in, move out, borrow or copy; a new built-in method must be added
  there (an unlisted one is rejected, which is the safe default).

## Rejected alternatives

- **Borrowed returns:** not in the language.
- **Moving `first`/`get`:** two meanings per name.
- **`List[Task[T]]` now:** needs flow tracking through collections.
