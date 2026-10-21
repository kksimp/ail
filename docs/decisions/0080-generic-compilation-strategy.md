# 0080. The compiler chooses, per instance, whether generic code is specialised or shared

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 9 — Generics; 23 — Compiler; 25 — Optimization
- **Spec rules:** [gen.impl.1](../spec/09-generics.md)

## Deciding criterion

Token usage, AI understanding and readability **tie**, because the
strategy can't be seen in source code. **Speed decides**: specialising hot
generic code (monomorphisation) is how AIL beats C's `void*` callbacks
([performance-beyond-c.md](../design/performance-beyond-c.md)), while sharing cold
instances limits code size and compile time.

## Decision

Generic code has the same semantics however it's compiled. The compiler may
specialise any instance or share code between instances, guided by
whole-program analysis and profiles. Its choices appear in the performance
report.

## Consequences

- Size, compile-time and speed effects are reported per generic instance.
- Escalation note: this doesn't limit performance, because users can't
  observe the strategy and every instance can be specialised.

## Rejected alternatives

- Always specialise every generic instance (full monomorphisation): loses on code size and compile time for cold instances, which the decision lets the compiler share.
- Always share generic code (type-erased, `void*`-style): loses on speed, since specialising hot generic code is how AIL beats C's `void*` callbacks.
