# 0175. Speed up the interpreter with local fast paths and a fast hasher

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Interpreter (Stage 2), [ADR 0032](0032-keep-rust-interpreter-as-oracle.md)
- **Spec rules:** none (no behaviour changes; output is byte-identical)

## Context

The interpreter is the permanent oracle, so it must stay obviously correct.
Profiling (callgrind; [interpreter-performance.md](../design/interpreter-performance.md))
found overhead in a 1.6 KB `eval` frame, SipHash on lookup tables keyed by
names and positions, and per-call allocations.

## Options considered

### Option A — local fast paths and a cheaper hasher (chosen)
- **Pros:** about 20% fewer instructions on `interp.ail` and 24% on a tight
  loop; each change is a few lines; behaviour is delegated to the existing
  code on every other path.
- **Cons:** a few duplicated arms (int arithmetic exists in `eval` and `binop`).

### Option B — rewrite to bytecode
- **Pros:** the largest possible win.
- **Cons:** a second semantic model for the oracle; Stage 3 work.

## Decision

Keep `eval` as a small function for names, int/bool literals and binary
operators and delegate everything else to `eval_general`; use a fast
non-cryptographic hasher (`fasthash.rs`) for the interpreter's name and
position tables; box traps inside `Exit`; skip the per-call work that only
`mut` parameters need; assign to a plain local without building a place.

## Rationale

ADR 0076 criterion 4 (speed) decides, with criterion 2 (understanding) as the
constraint: every fast path falls back to the general path, and overflow still
traps through `binop` (expr.arith.1).

## Consequences

- `tools/perfbench/run.sh` is the one-command measurement.
- Hash iteration order is not relied on anywhere (maps keyed this way are only
  looked up), so output is unchanged.

## Rejected alternatives

- **Bytecode or a new evaluator:** a second semantic model for the oracle.
- **Keep SipHash:** its DoS resistance is irrelevant for keys taken from the
  program being run, and it cost about 10% of a typical run.
