# 0621. Bytecode tier: bounds checks stay; no safe way to remove them

- **Status:** Superseded by [ADR 0873](0873-rust-only-features-after-rust-removal.md)
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — Interpreter
- **Builds on:** [ADR 0619](0619-bytecode-calls-loops-and-fusion.md), [ADR 0620](0620-bytecode-element-method-calls.md)

## Context

After ADR 0620 the tier is about 0.95-0.97x of CPython (geometric mean over the
compute programs). Programs still above 1.0x are binarytrees (1.5x), sieve
(1.1-1.3x) and matmul (1.1-1.3x). Callgrind on binarytrees (depth 10, 368 M
instructions) puts 60% of the instructions in the `exec` loop itself: slice
indexing 11%, `mem::replace` 8%, `Vec` handling 5%, pointer arithmetic 5%,
`run_code` 5%, `run_call` 3%. There are no hand-offs left in these programs.
Sieve (55 M bytecode instructions, about 9 ns each) has the same shape.

## Decision

Keep the per-operand bounds checks. The loader validates every register and
constant index, but the compiler cannot see that proof across the loader:
`regs[i]` and `consts[i]` stay checked unless the access is `unsafe`
(`get_unchecked`), which the designer has not approved (decision card open).
Safe forms were measured on a prototype (not kept): registers as a fixed
`[Value; 64]` per frame (functions needing more stay on the walker) with every
register index masked to 63, so the compiler drops the register checks.
Callgrind instruction counts, before to after: binarytrees (depth 10)
368.5 M to 359.8 M (-2.4%), sieve (300,000) 319.4 M to 313.6 M (-1.8%), matmul
(60) 133.6 M to 128.3 M (-4.0%). Local and constant accesses stay checked, and
the 64-register cap and 8 KB frames are costs, so it is below the 3% bar on
two of three programs and not worth landing. Replacing `mem::replace` with an
enum swap moves the same stores and was not built.

The measured ceiling for unchecked access is about 12% fewer instructions
(lane 190, step 1). Beyond that the remaining cost is the design: operands are
decoded per instruction, locals live in a `Vec<(&str, Value)>`, and each call
recurses through `run_code` and `exec`.

## Rejected alternatives

- **Audited `unsafe` unchecked access.** About 12% fewer instructions, but
  needs the designer's approval; the card stays open.
- **A frame with locals in the register file and non-recursive calls.** The
  real fix for calls (binarytrees), but a redesign of the tier's environment
  that the walker shares; a lane of its own, not an end-of-night change.
- **Per-program fusion (a sieve-inner-loop instruction).** Wins one benchmark
  and nothing else.

## Hand-off survey (2026-10-21, for day 3)

Every program in `examples/` and `bootstrap/tests/cases/` (390 programs that ran)
under `AIL_BYTECODE_STATS=2`: 54.3 M tier instructions and 29,706 hand-offs, so
hand-offs are about 0.05% of the work and not where the time goes. Ranked by count:

| Hand-offs | Construct |
|---:|---|
| 17,692 | assignment to a field or element (16,620 are `examples/interp.ail`: `env.scopes[k][name] = eval(value)?`, a map-in-list place with a fallible value) |
| 5,676 | method call `.call` |
| 2,329 | method call `.from_code` |
| 940 | slice |
| 661 | method call `.len` (on non-local receivers) |
| 484 | `let ... else` with a block |
| 259 | comprehension; 259 `call`; 154 `if ... else err` |

No fix is worth a PR: the top entry is one program's deep place assignment
with `?` in the value (a tree-walker style the tier would need a place-path
instruction for), and the rest are under 1% of the programs' instructions.
