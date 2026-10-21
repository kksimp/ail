# 0822. `std.hw`: `at`, volatile methods and fences, and the effect `hw`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved [ADR 0820](0820-driver-level-unsafe.md)
- **Topic:** 15 — Unsafe semantics
- **Spec rules:** [unsafe.hw.2–5](../spec/13-unsafe.md), [eff.infer.1](../spec/18-effects.md), [eff.pure.1](../spec/18-effects.md)
- **Builds:** [ADR 0820](0820-driver-level-unsafe.md), [ADR 0435](0435-pointer-operations-and-typed-alloc.md)

## Context

ADR 0820 names the contents of `std.hw` but leaves the spelling and the checks
to implementation. Port I/O, CPU intrinsics and `@layout(c)` follow in later
changes; this one covers fixed addresses, volatile access and barriers.

## Options considered

### Option A — `hw.at(addr, example)`, `p.read_volatile()`, `p.write_volatile(v)`, `hw.fence()`, `hw.fence_io()` (chosen)
- **Pros:** the typed-example form of ADR 0435 (`mem.alloc(n, example)`) is
  reused, so there is no type-argument syntax; the volatile methods sit beside
  `read` and `write`, so a register access reads like any other pointer access.
- **Cons:** `read_volatile` is longer than `vread` (a tie by tokbench: a
  handful of tokens in rare code).

### Option B — `p.vread()` and `p.vwrite(v)`
- **Cons:** a cryptic name for the most safety-relevant distinction.

### Option C — a separate pointer type `*volatile T`
- **Cons:** a second pointer type through the checker, the interpreter and
  the self-hosted checker, for one property of an access.

## Decision

1. `hw.at(addr, example)` is a `*T` at `addr`; a negative address traps
   (`unsafe.hw.2`).
2. `read_volatile` and `write_volatile` are `PTR_METHODS`; they need the
   address to be a multiple of the access width (`unsafe.hw.3`) and keep the
   null and block checks of ADR 0435.
3. `hw.fence()` and `hw.fence_io()` are a sequentially consistent fence in the
   interpreter (`unsafe.hw.4`).
4. The effect `hw` comes from every `std.hw` call and every volatile access,
   propagates like `unsafe`, makes `@pure` fail and orders parallel loops
   (`unsafe.hw.5`).

## Consequences

- The bytecode tier does not run these (a function with an `unsafe` block runs
  on the tree walker, ADR 0430).
- Native lowering is step 2 of ADR 0820; the interpreter's checks are
  diagnostics there, and compiled code traps on null only.

## Rejected alternatives

- `vread` and `vwrite`: rejected for readability at equal token cost.
- A `*volatile T` type: rejected because it spreads one property of an access over the type system.
