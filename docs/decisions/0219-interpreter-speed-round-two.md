# 0219. Speed up the interpreter again with int fast paths, pooled allocations and in-place updates

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Interpreter (Stage 2), [ADR 0032](0032-keep-rust-interpreter-as-oracle.md)
- **Spec rules:** none (no behaviour changes; output is byte-identical)

## Context

[ADR 0175](0175-interpreter-fast-paths.md) cut instruction counts by about 20%,
and the designer's bar is "beat Python". The plan branch still ran `loops.ail`
2.85 times and `interp.ail` 1.95 times slower than CPython
([interpreter-performance.md](../design/interpreter-performance.md)). A
callgrind profile showed a flat distribution with three visible clusters: a
call per leaf operand and one `Value` built and dropped per int operation;
roughly 1.46 million allocations in `interp.ail` (environments, argument and
operand vectors, place paths, struct field lists, unit variants); and
per-call hashing of names and positions.

## Options considered

### Option A — more local fast paths and fewer allocations (chosen)
- **Pros:** every change is local and falls back to the existing general code;
  `interp.ail` runs 45% fewer instructions and `loops.ail` about 55% fewer; the
  oracle keeps one evaluation model.
- **Cons:** more special cases in `interp.rs`; each must keep the same traps and
  evaluation order as the general path, so each has a test.

### Option B — bytecode or closure compilation
- **Pros:** the largest possible win.
- **Cons:** a second semantic model for the oracle (ADR 0032); Stage 3 work.

### Option C — slot-resolved locals
- **Pros:** removes the name scan.
- **Cons:** counters on `interp.ail` showed 1.7 million lookups scanning 2.7
  entries on average, about 5% of the run: under the threshold
  [ADR 0176](0176-keep-name-based-locals.md) set (10%).

## Decision

Keep the tree-walking evaluator and add these paths, each falling back to the
general code when its conditions are not met:

- **Int operands.** Int locals and literals, and one arithmetic operator over
  two of them, are read without cloning or building a `Value`; comparisons in
  `if` and `while` conditions produce a `bool` directly; `n op= leaf` and
  `n = leaf-expression` overwrite an int local in place. `+ - *` still trap on
  overflow and `/ %` on a zero divisor through `binop` (expr.arith.1,
  expr.arith.3).
- **Calls.** Environments are pooled and operands move straight into the
  callee's. A call with no `mut` operand skips places and the overlap check.
  A call whose `mut` operands are plain locals moves each out of its slot and
  back, checking own.excl.3 on the slots. Read-only arguments and method
  receivers are evaluated by value: reading a place and evaluating the same
  expression agree.
- **Fewer allocations.** `return` leaves its value in the interpreter, so
  `Flow<Value>` is 32 bytes instead of 48; unit variants share one empty field
  list; string literals share their text; builtin argument lists come from a
  pool; place roots are found without building a path.
- **Fewer calls and clones.** Field and index reads borrow the local; `Value`
  clones inline; field and index expressions have their own small frames, so
  `eval` stays small.
- **Built-in equality, ordering and text** skip trait dispatch when the program
  implements none of `Eq`, `Ord` or `ToStr` and gives none a default body.
- **Map keys** hash with the fast hasher (ADR 0175), integers in one step.

## Rationale

ADR 0076 criterion 4 (speed) decides, with criterion 2 (understanding) as the
constraint: each path is a few lines beside the code it shortcuts. The
instruction counts and wall times are in
[interpreter-performance.md](../design/interpreter-performance.md).

## Consequences

- `loops.ail` and `interp.ail` are at or near CPython's wall time; `json.ail`
  and `sheet.ail` are four to five times faster.
- The shortcuts have to be kept in step with the general paths. The new cases
  `fast_paths*.ail` exercise each one and the traps it keeps; the differential
  check against the previous build (every example and case, byte-identical
  stdout, stderr and exit code) is in the PR.
- Map iteration order never depended on the hasher (it is insertion order,
  type.map.1), and keys come from the program being run, so the hasher's lack
  of DoS resistance matters no more than it did in ADR 0175.

## Rejected alternatives

- **Inline two steps of a place path in `Place`:** the larger `Place` made
  `interp.ail` 1.5% slower overall (the allocation it saved was outweighed by
  moving the value around).
- **`lto = "fat"` and `codegen-units = 1`:** no instruction savings on the
  tight loops, and the release build took 60 s instead of 25 s.
- **Marking the cold arms of `stmt` `#[inline(never)]`, or handling assignment
  in `block`:** both increased instruction counts on `loops.ail`.
- **Reading int operands two operators deep:** no gain; the extra inlining
  cancelled the saved calls.
- **A custom global allocator:** allocation fell from 1.46 to 0.37 million calls
  without one, and a hand-written allocator is a correctness risk for under 5%.
