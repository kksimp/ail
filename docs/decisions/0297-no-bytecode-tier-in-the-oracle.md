# 0297. Keep the interpreter a tree walker; the compiled form belongs to Stage 3

- **Status:** Superseded by [0359](0359-bytecode-tier-in-the-bootstrap.md)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** interpreter performance
- **Spec rules:** none (no behaviour changes)

## Context

Round two ([ADR 0219](0219-interpreter-speed-round-two.md)) brought `interp`
and `loops` to parity with CPython. Lane 61 asks for 1.5x faster than CPython
on `interp`, `loops`, `json` and `sheet`. `json` (0.19x of CPython's time)
and `sheet` (0.26x) already meet it. `interp` (about 1.1x) and `loops`
(about 1.0x) would need a further 1.7x to 1.5x gain. The profile of `interp`
is flat (see [interpreter-performance.md](../design/interpreter-performance.md)):
the largest self cost is 9%, and each remaining local change is worth 0.5% to 3%.

[ADR 0032](0032-keep-rust-interpreter-as-oracle.md) keeps this interpreter
as the permanent oracle of the language, so it has to stay obviously correct,
and every new path has to keep the ownership, `ref`, task, pool, trace and limit
semantics of ADRs 0190-0193, 0265-0268, 0272-0275 and 0288.

## Options considered

### Option A — keep the tree walker, take only measured local wins (chosen)
- **Pros:** the oracle stays small and readable; one evaluation model.
- **Cons:** `interp` and `loops` stay near CPython, short of 1.5x.

### Option B — add a lazily built bytecode tier for hot functions
- **Pros:** the only route to a large gain without changing the language.
- **Cons:** a second evaluator with its own copy of every rule, inside the oracle
  that exists to be the single reference; its differential testing and the
  trace, task, pool and unique semantics are days of work, not two hours.

### Option C — resolve locals to slots in the checker
- **Pros:** removes the name scan.
- **Cons:** about 5% of a run ([ADR 0176](0176-keep-name-based-locals.md)); hidden
  hoisted locals and `for mut` make slots differ between visits.

## Decision

Keep the tree walker. Take the literal-line mask, which skips two table probes
per int literal. Build a compiled tier for the Stage 3 backend, checked against
this interpreter, and not inside it.

## Rationale

ADR 0076 ranks speed last. The gap to the 1.5x target needs a second evaluator,
which would cost the oracle its main property, and that is a change to ADR 0032
that belongs to the designer.

## Consequences

- `interp` and `loops` stay at about CPython speed on the bootstrap interpreter.
- Programs that matter for speed should run on the Stage 3 backend, which is
  where a compiled form belongs.
- If the designer wants a bytecode tier in the bootstrap anyway, it needs its
  own lane and a decision that amends ADR 0032.

## Rejected alternatives

- Fat LTO with one codegen unit: 12% faster collatz, 8% slower `interp`; no net win.
- A 24-byte `Value` and `Rc<[Value]>` fields: touch every builtin for about 2% of `interp`.
