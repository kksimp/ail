# 0305. Struct invariants, checked at construction and after `mut self` methods

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** [contract.inv.1–3, lex.kw.1](../spec/20-contracts.md)

## Context

Some facts hold for every value of a struct ("`lo <= hi`", "`balance >= 0`").
Repeating them as `ensures` on each method is noisy and misses construction
([OQ-060](../open-questions.md) item 3). The language has mutable value
semantics with `ref T` for shared mutation ([ADRs 0060–0063](0060-mutable-value-semantics.md)),
so a struct is only changed through its own `mut self` methods, through a
`mut` parameter, or through a `ref`.

## Options considered

### Option A — `invariant COND` lines in the struct, checked at construction and after each `mut self` method (chosen)
- **Pros:** the fact is stated once, next to the fields; the two check points
  are the places a value of the struct is created or changed by code that
  owns its meaning; `invariant` as a word is the term agents already know.
- **Cons:** `invariant` becomes a keyword (lex.kw.1); a field assigned from
  outside the type's methods isn't checked until the next check point.

### Option B — check after every write to a field
- **Cons:** a method must pass through broken states (moving money between two
  fields); every write would need a check, a real cost with checks on.

### Option C — invariants as an `impl Invariant` trait method
- **Cons:** more tokens, and the checker couldn't tell that it is a contract
  (purity, `ail api`).

## Decision

Option A. After a value is built with the struct's literal form
(`contract.inv.2`) and after a `mut self` method returns, value or failure
(`contract.inv.3`), every invariant holds. Conditions are pure `bool`s over the
fields. A trap names the struct, the condition, the method or construction,
and the value. `--no-contracts` skips them. Not checked: values built by
other means (generators in property tests, `Json` decoding), and writes
through a `mut` parameter in a free function until the value next passes a
check point. A struct inside another is checked when it is built or changed
itself.

## Rationale

Criterion 2 (AI understanding): the invariant is stated where the data is, so
an agent editing a method sees what it must keep true. Criterion 4 (speed): a
program without invariants pays one boolean test per construction and per
`mut self` return.

## Consequences

- `invariant` is reserved. A program that used it as a name must rename it
  (none in the corpus or the examples).
- Loop invariants and static proof remain open (OQ-060).

## Rejected alternatives

Option B (too many checks, forbids intermediate states), Option C (more
tokens, no tool support).
