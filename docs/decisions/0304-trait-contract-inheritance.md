# 0304. Implementations inherit trait method contracts

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** [contract.inh.1–3, contract.api.2](../spec/20-contracts.md)

## Context

A caller holding a `Shape` knows only the trait ([trait.any.1](../spec/10-traits.md)).
If the trait states `requires k > 0`, the implementation must honour that for
every caller, and may accept more ([OQ-060](../open-questions.md) item 2,
recommended: yes, with Liskov rules). Lane 28 left trait methods without
contracts.

## Options considered

### Option A — inherit; own `requires` weaken (OR), own `ensures` add (AND) (chosen)
- **Pros:** Liskov-correct by construction, so no extra compile-time check of
  "is this weaker?" is needed; the same rule as Eiffel's `require else` and
  `ensure then`; the check runs on the implementation, so it works through the
  trait and through the concrete type with one mechanism.
- **Cons:** an implementation `requires` that is stricter than the trait's
  silently widens what is accepted instead of being rejected.

### Option B — implementations repeat the contract; the checker compares
- **Cons:** needs a prover for "weaker than"; the conditions are arbitrary
  expressions, so it would reject or accept by syntax only.

### Option C — contracts only on the concrete type
- **Cons:** a call through the trait would check nothing, so a trait's
  contract would promise nothing to its callers.

## Decision

Option A. A trait method may have contracts, bodyless or not. For a call of an
implementation, the inherited `requires` hold, or the implementation's own
all hold; the inherited `ensures` and the implementation's own all hold. An
inherited condition reads the arguments under the trait method's parameter
names. A trap says `(inherited from Trait.method)`.

## Rationale

Criterion 2 (AI understanding): `ail api` shows what an implementation
inherits, so an agent never needs to open the trait to know the contract.
Criterion 1: nothing is repeated in implementations.

## Consequences

- A trait's contract is a promise to every caller of an implementation.
- A later prover can discharge inherited conditions per implementation.
- `ail api` gains `inherits`, `inherited_requires`, `inherited_ensures`.

## Rejected alternatives

Option B (needs a prover), Option C (breaks the trait's promise).
