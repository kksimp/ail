# 0075. Calls that mutate their receiver are not marked at the call site

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership; 10 — Traits/methods
- **Spec rules:** [trait.method.3](../spec/10-traits.md)
- **Evidence:** [traits-generics-options.md §4](../design/traits-generics-options.md#4-mutating-a-methods-receiver-mark-the-call) (unmarked 28.8, marked 31.8: +10%)
- **Relation:** a deliberate exception to the call-site `mut` rule for arguments ([ADR 0061](0061-call-site-mut.md))

## Options considered

- **Unmarked (chosen):** ~10% fewer tokens in mutation-heavy code. Matches
  Swift and Rust. Common mutators (`push`, `insert`) are self-explanatory.
  Con: inconsistent with ADR 0061. A reader sees that the binding is
  mutable, but not which calls change it.
- **Marked (`mut tokens.push(x)`):** fully consistent, but noisy on the most
  common operations.

## Decision

Calling a method declared with `mut self` requires the receiver to be a
mutable place (a `mut` binding or a mutable part of one), but the call isn't
marked: `tokens.push(x)`. Arguments other than the receiver still follow ADR 0061.

## Consequences

- Editor hints and the AI semantic API flag calls that mutate their receiver,
  restoring visibility without tokens.
- Exclusivity (ADR 0064) still applies between the receiver and the arguments
  (e.g. `xs.append(xs)` must not alias).

## Rejected alternatives

- **Marked receivers:** +10% tokens on the most frequent mutations.
