# 0361. The bytecode tier: register code over the tree walker's environment

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), within the designer's ADR 0359
- **Topic:** interpreter performance; 23 — Compiler architecture
- **Spec rules:** none (the tier is unobservable; it changes no output, exit code or trap)

## Context

[ADR 0359](0359-bytecode-tier-in-the-bootstrap.md) adds a bytecode tier
beside the tree walker, which stays the oracle
([ADR 0032](0032-keep-rust-interpreter-as-oracle.md)). Lane 111's first
slice covers locals, arithmetic, comparisons, `if`, `while`, `for` over
ranges and lists, calls of plain functions, and list and map indexing, and
asks for stack-based and register-based code to be measured on `loops`
before choosing. Everything outside the subset must still run, on the walker.

The walker's state is a name-keyed environment, `Vec<(&str, Value)>`, where
`let` pushes and each block truncates back to its mark. Its traps, trap
snapshots (trace.snap.1), task scopes (conc.task.4) and `mut` arguments all
read that environment.

## Options considered

### Code shape (measured)

A standalone prototype ran the collatz section of `loops` (30,000 starts,
2.86 million loop turns) with the interpreter's 32-byte `Value` and locals in
an environment like the walker's. Instructions from callgrind:

| Form | Instructions | vs the walker |
|---|---|---|
| Tree walker (`ail run`, this program) | 2,765M | 1.00x |
| Stack code (push, pop, operate) | 2,707M | 1.02x |
| Register code, operands as an enum | 1,431M | 1.93x |
| Register code, operands as tagged `u32` | 1,062M | 2.60x |
| Same, with int-literal immediates | 900M | 3.07x |

Stack code moves every operand through a `Vec<Value>` push and pop, which
costs as much as the walker's recursion saves. Register code with tagged
operands and immediates was chosen.

### Option A — register code over the walker's environment (chosen)
- **Pros:** every binding sits at the slot the walker gives it, under its
  name, so any expression or statement outside the subset is handed to the
  walker mid-function and sees exactly the walker's state; the subset can grow
  one construct at a time; traps and snapshots come out identical without
  separate code.
- **Cons:** locals keep their names in the environment (16 bytes a slot) and
  `let` still pushes; temporaries need a second array.

### Option B — a separate frame of slots, synced to the walker on hand-off
- **Pros:** a tighter frame.
- **Cons:** a hand-off in a loop (a method call, say) copies every live local
  out and back on each turn; snapshots need their own code.

### Option C — compile only functions wholly in the subset
- **Pros:** no hand-offs.
- **Cons:** one unsupported construct (a method call, an interpolation) keeps a
  whole hot function on the walker; `loops`'s `main` and most of `interp`
  would never run on the tier.

## Decision

- A function's body is compiled on its first call into register code, run
  in place of the walker's `block` inside `run_body`, so depth limits,
  contracts, return traces and snapshots stay the walker's code.
- Operands are a local slot (relative to the first parameter), a register
  or a constant, tagged in a `u32`. Int operations, comparisons, conditional
  jumps, loops, calls and indexing are instructions; any other expression or
  statement is a hand-off to `eval` or `stmt`. A `break`, `continue` or
  `return` coming out of a hand-off ends the enclosing blocks as the walker's
  would, from a table built at compile time.
- A block that binds nothing emits no end-of-block code.
- The tier is off for a program that uses something it doesn't carry yet:
  `--trace`, `--watch`, run limits and coverage, unique types, `defer`, and
  contracts (while they are checked); and for `--deterministic` and
  `--schedule`. A function with `spawn`, a task, channel or mutex
  constructor, `with`, or a `par for` stays on the walker. `--no-bytecode`
  turns the tier off. `AIL_BYTECODE_STATS=1` prints how many calls ran on it.

## Rationale

Speed is the fourth criterion of ADR 0076, but ADR 0359 is the designer's
call for it, and the first criteria are kept: the tier adds no syntax or
diagnostics, and keeping the walker's environment makes it hard to be wrong.
The register form is the only measured option near ADR 0359's 2–3x.

## Consequences

- Slice 1 results (Linux x86-64, [interpreter-performance.md](../design/interpreter-performance.md)):
  `loops` 1.79x fewer instructions and 0.62x of CPython's time (was 1.04x);
  its collatz section 2.55x; `interp`, `json` and `sheet` gain little until
  method calls, structs, `match` and interpolation compile (slice 3).
- Every example, test case, corpus program and package prints the same bytes
  with and without the tier; slice 2 makes that check part of `./ail selftest`.
- Two evaluators must stay in step; the hand-off keeps the second one small.

## Rejected alternatives

- Stack code: no faster than the walker on `loops` (measured above).
- Option B: a hand-off in a loop would copy the frame twice per turn.
- Option C: the hot functions of the benchmarks aren't wholly in any small subset.
