# 0763. Comprehensions compile to loops in the bytecode tier

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 13 — Standard library (the interpreter's speed)
- **Spec rules:** expr.comp.1, expr.comp.2, expr.comp.3 (behaviour unchanged)
- **Builds on:** [ADR 0359](0359-bytecode-tier-in-the-bootstrap.md) (the bytecode tier)

## Context

The tier handed every comprehension to the tree walker as one node, so the
element, the filters and the ranges ran as tree evaluation with a clone of every
item and a vector for each range. `tensor_ops_loops` builds three
65,536-element lists per round this way: 29% of its instructions were walker
`eval`, `eval_binary`, `binop` and `eval_index`. The hand-off counters hid it,
because a whole comprehension counts as one hand-off.

## Decision

A comprehension compiles to the loops of its clauses, as the `for` statement
does: a counted loop for `lo..hi` and `lo..=hi`, an iterator loop for lists,
sets and bytes. Two instructions are new: `NewAcc` builds the empty list, set or
map in a register, and `Collect` adds one element (a key and a value for a map)
to it. The filter of a clause is a conditional jump to the loop's next step.
The result moves to its destination at the end, so `xs = [x + 1 for x in xs]`
reads the old `xs` throughout.

The tier keeps the walker for a comprehension when anything in it could leave
early or needs the walker's steps: a `?`, an `if` or `match` expression, an
`else` that leaves, a `par` clause, a run limit, coverage or a trace (the
existing `has_try` test, which now looks inside nested comprehensions). Those
programs behave exactly as before.

## Measured (Linux x86-64)

Callgrind instructions, `tools/perfbench/instr.sh`:

| program | before | after |
|---|---|---|
| tensor_ops_loops | 3,628.5 M | 2,548.5 M (-29.8%) |
| tensor (list setup) | 73.4 M | 31.0 M (-57.8%) |
| tensor_loops | 4,215.6 M | 4,128.6 M (-2.1%) |

Median of 7 wall-clock runs (`tools/perfbench/wall.sh`): tensor_ops_loops
0.380 s to 0.214 s (CPython 0.211 s), tensor 0.018 s to 0.011 s. Other
programs are unchanged within 1%.

`./ail selftest` passes. `bootstrap/tests/cases/comp_loops.ail` covers nested
clauses, filters, ranges, sets, maps, structs, bytes, shadowing and
self-reference, and agrees with `--no-bytecode`. The bytecode fuzz generator
(ADR 0756) now writes comprehensions: 26,000 programs, no disagreement.

## Amendment 2026-10-21: `for a, b in xs` clauses (type.tuple.5)

Lane 255's fix sent these clauses to the walker, because the loops bound only
the clause's hidden variable. They now compile too: after `IterNext` binds the
element, `Ins::Unpack` pushes each part as the walker's `push_clause` does, so
the names, their order and the trap snapshot's locals are the same. A range
clause never unpacks (its items are integers). `comp_unpack_tier` stays as the
pin test, and `bcprogs.rs` generates unpack clauses alone, after a range
clause and in a map comprehension.

## Rejected alternatives

- **Faster walker `comp`.** It would still clone each item and build a vector for
  each range; the element is evaluated by tree walking whatever is done there.
- **Compile `if` and `match` elements too.** They need the walker's block and
  binding rules inside the loop, and the first version is cheaper to verify
  without them. A later change can lift the restriction.
