# 0093. Absent optionals are written `none`; present values convert to optionals implicitly

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system; 8 — Error handling
- **Spec rules:** [type.opt.4–5](../spec/04-types.md)
- **Evidence:** `tools/tokbench/corpus/r4/` implicit wrapping 22.8 vs. explicit `some(x)` 24.2 (−6%)

## Deciding criterion

**Token usage:** implicit wrapping saves a call on every "found" path.
**AI understanding** is unaffected, because the expected type `T?` is always
visible in the signature or binding.

## Decision

1. `none` is the absent value of every optional type `T?`.
2. Where a `T?` is expected, a value of type `T` converts to a present optional
   implicitly. There's no wrapping function in source code.

```
fn find(id int) User?
    if id == 0
        return none
    users[id]
```
