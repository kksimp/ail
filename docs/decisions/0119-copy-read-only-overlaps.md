# 0119. A read-only argument that overlaps a `mut` argument is passed as a copy

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** Designer
- **Topic:** 7 — Ownership
- **Spec rules:** [own.excl.1–4](../spec/07-ownership.md), [type.coll.7](../spec/04-types.md)
- **Partly supersedes:** [ADR 0064](0064-exclusivity.md) (overlap between a `mut` argument and a read-only one)

## Context

Under ADR 0064, any argument overlapping a `mut` argument of the same call was
an error. Implementing it showed that this rejects natural code that has one
obvious meaning:

```ail excerpt
xs.push(xs[0])          # the receiver is `mut`, and xs[0] is part of xs
total.add(total.x)
```

The designer doesn't want compile errors for code like this.

## Options considered

- **Copy read-only overlaps (chosen):** a read-only argument that overlaps a
  `mut` one is passed as a copy made before the call. Two overlapping `mut`
  arguments stay an error. Swift does the same.
- **Copy everything:** overlapping `mut` arguments also get copies, written
  back left to right. There are no overlap errors, but `inc_both(mut x, mut x)`
  silently loses one update.
- **Keep ADR 0064 unchanged:** strictest, but rejects `xs.push(xs[0])`.

## Decision

1. Exclusivity applies only between `mut` arguments, including a `mut self`
   receiver. Two of them must not overlap: a compile-time error when certain,
   and a run-time trap when it depends on index values (as in ADR 0064).
2. A read-only argument or receiver that overlaps a `mut` argument of the same
   call is passed as a copy of its value, made before the call. Each such
   copy site is listed in the performance report.
3. The standard library adds `xs.swap(i, j)`, so swapping list elements never
   needs two `mut` arguments. It does nothing when `i == j`.

## Why ADR 0064's objections no longer decide

ADR 0064 rejected implicit copies as hidden and bug-masking.
- **Hidden cost:** copies happen only where arguments actually overlap. With
  copy-on-write storage they're usually cheap, and every site is reported, as
  `own.copy.1` already requires for copies.
- **Masking bugs:** reading a value while changing another place has one
  obvious meaning under value semantics (`own.val.1` already says a read-only
  argument is an independent value). The genuine bug, two `mut` handles to
  the same place, remains an error.

## Consequences

- The optimizer can still assume that a `mut` argument aliases nothing else
  the callee can see, because overlapping read-only arguments are copies.
- `f(mut x, mut x)` remains an error, and `swap(mut a[i], mut a[j])` still
  traps when `i == j`. The fix is `a.swap(i, j)`.

## Rejected alternatives

- **Copy everything:** unpredictable results when `mut` arguments overlap.
- **Keep ADR 0064 unchanged:** rejects code with one obvious meaning.
