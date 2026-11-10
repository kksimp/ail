# 1055. Prove contracts from interval facts and drop the proved checks

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer put slice E12 in 0.1.0 on 2026-11-10
- **Topic:** 20 — Contracts; 22 — Tooling
- **Spec rules:** [contract.prove.1-3](../spec/20-contracts.md), [prove.cmd.1-2](../spec/22-tools.md)
- **Builds on:** [ADR 0211](0211-contract-runtime-checks.md), [ADR 0612](0612-native-contracts.md), [ADR 0885](0885-contracts-checked-by-fuzzing.md), [ADR 0941](0941-data-range-loop-guards.md)
- **Resolves:** the static-proof half of [OQ-060](../open-questions.md) (b)

## Context

Every `requires` is evaluated at every call and every `ensures` at every return
(contract.run.1). Many are plainly true where they are checked: `get(xs, i)` with
`requires i < xs.len()` inside `for i in 0..xs.len()`, or `ensures result >= 0` on a
function that returns `n % 10` of a non-negative `n`. An agent reading `ail verify --diff`
gets "the checker accepted it" as *proved*, which says nothing about contracts. The
optimiser already computes value ranges for loops (ADR 0580, ADR 0941), but over MIR, after
the contract checks are lowered, and only to remove overflow checks. ADR 0009 rules out an
external solver; chapter 20 already allows a compiler to drop a check it proves.

## Options considered

### Option A: interval and difference facts over the AST, in the checker (chosen)
A pass in `compiler/check/prove.ail` walks each function's body with a set of facts: an
interval per immutable `int` name, `len()` of a built-in collection, and bounds `a - b <= c`
between two such terms. A contract is proved when its condition evaluates to true under
those facts; the lowering skips the check of a proved contract.
- **Pros:** no new dependency; linear in the program; the facts are the ones an agent
  writes (loop bounds, `if` guards, the caller's own `requires`); the same pass gives the
  lowering, `ail prove` and later `ail verify --diff` one answer.
- **Cons:** proves only linear, one-step facts; nothing about `mut` names, fields or calls
  in conditions.

### Option B: reuse the MIR ranges of `compiler/mir/ranges.ail`
- **Pros:** one range engine.
- **Cons:** it bounds registers inside counted loops only, after lowering, so a contract
  check is already a branch; tools that don't lower (`ail prove`, `ail verify`) can't see
  it; it has no relations between two values, which `i < xs.len()` needs.

### Option C: a small SMT-style solver written in AIL
- **Pros:** proves non-linear and disjunctive conditions.
- **Cons:** large (L+ on top of L), slow on big files, and its answer is hard for an agent
  to predict; ADR 0009 prefers the simple custom tool when it covers the cases measured.

### Option D: no proof, only fuzzing (ADR 0885)
- **Pros:** nothing new.
- **Cons:** the status quo: a sample, not a proof, and every check stays in the build.

## Decision

1. **What is proved** (contract.prove.1). A `requires` of a top-level function at a call
   `f(args)` written by name in the same file, and an `ensures` of a top-level,
   non-generic function that cannot fail, at every return of its body. A proved check is
   not evaluated at run time. Under `ail run --trace` every check stays, so the trace's
   contract events don't change.
2. **The facts** (contract.prove.2): integer literals; the function's own `requires`
   (checked or proved at each of its calls, contract.run.1); immutable `x := e`; the
   condition of an `if` or `elif` in its block, the negation of earlier ones in later
   branches and `else`, and the negation after an `if` whose block always leaves (`return`,
   `break`, `continue`); `for i in a..b` and `a..=b` with an immutable `i`; `len()` of a
   `List`, `Map`, `Set` or `str` parameter or binding is in `0..=2^57`; the `ensures`
   of a top-level function called in `x := f(args)`; and `+`, `-`, `*`, `/` and `%` by a
   positive constant, `-`, `and`, `or` and `not`. A `mut` name, a field, an element, a
   method call other than `len()`, and a call inside a condition carry no facts.
3. **Soundness of arithmetic.** In the program, an operation that overflows traps, so a
   fact past it is true when it is reached. In a condition, an arithmetic term counts only
   when its bounds are finite, so a proved condition can't overflow and dropping it removes
   no trap of its own.
4. **No solver** (contract.prove.3). The result depends only on the file; anything not
   proved stays checked at run time and by `ail test --contracts` (ADR 0885).
5. **`ail prove FILE|DIR [--json]`** (prove.cmd.1-2) lists each contract site as `proved`
   or `checked`, with a total, so the count in the compiler's own source is one command.

## Rationale

Criterion 2 (AI understanding): *proved* gets a meaning an agent can act on, and a proof is
reproducible from facts the agent can read in the function. Criterion 4 (speed): each
proved `requires` removes a branch and a call of the condition at a hot call site, at no
cost to unproved code. Token cost is nil: no syntax, no new semantics (chapter 20 already
says a proved check may go).

## Consequences

- Programs with proved contracts run fewer checks; their behaviour is unchanged.
- `ail verify --diff` can list proved contracts under *proved* (lane H, ADR 0909).
- Still not proved: methods and trait contracts, calls across modules, `old(e)`, struct
  invariants, loop invariants (OQ-060 (a)). Each can extend the same pass.

## Rejected alternatives

- **MIR ranges (B):** no relations, invisible to tools that don't lower.
- **A solver (C):** cost and unpredictability against ADR 0009, for cases not yet measured.
- **Fuzzing only (D):** a sample is not a proof and keeps every check.
- **Dropping unproved checks in an optimised build:** OQ-060 (b) recommends keeping them;
  `--no-contracts` stays the explicit way to turn checks off.
