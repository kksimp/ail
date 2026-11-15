# 0077. A fixed set of operators can be implemented for user types through standard traits

- **Status:** Accepted; amended by [ADR 1124](1124-arithmetic-operator-traits.md)
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 10 — Traits; 2 — Core syntax (expressions)
- **Spec rules:** [trait.op.1–3](../spec/10-traits.md)
- **Evidence:** `tools/tokbench/corpus/r3/o1_methods.ail` vs `o2_operators.ail`: 16.4 vs **10.2** mean tokens (−38%)

## Deciding criterion

**Token usage.** `price * qty + tax - discount` costs 38% fewer tokens than
`price.mul(qty).add(tax).sub(discount)`. Numeric and SIMD types (vectors,
matrices, big integers, money) depend on this. For AI understanding, the
operator set is fixed and each operator maps to one standard trait, so `a + b`
always means "the `Add` implementation for `a`'s type". No custom operator
symbols exist.

## Decision

1. Arithmetic (`+ - * / %`, unary `-`), comparison (`== != < <= > >=`) and
   indexing (`a[i]`) may be implemented for user types through
   corresponding standard-library traits (names to be fixed with the standard
   library).
2. New operator symbols can't be defined.
3. Operator implementations dispatch statically and have no implicit
   conversions. Both operands of a binary arithmetic operator have the same
   type unless the trait implementation states otherwise.

## Alternatives not chosen

- **No operator overloading:** +38% tokens for numeric code.
- **User-defined operator symbols:** worst for AI understanding and readability.
