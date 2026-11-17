# 1143. Split a loop into a parallel head and an ordered tail; the chunk scan takes what the judge proves

- **Status:** Accepted
- **Date:** 2026-11-17
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); asked for by the designer ("work on all changes the lanes found that would improve, without changing our language syntax", 2026-11-17) after the round-2 optimizer review
- **Topic:** 11 — Concurrency
- **Spec rules:** [conc.par.20](../spec/11-concurrency.md) (new); amends [conc.par.8](../spec/11-concurrency.md) and [conc.par.13](../spec/11-concurrency.md)
- **Builds on:** [ADR 0911](0911-default-automatic-parallel-loops.md), [ADR 0938](0938-auto-parallel-push-builder-loops.md), [ADR 0987](0987-auto-parallel-inner-loops.md)

## Context

The independence judge (conc.par.2-4) takes a loop whole or not at all. A
loop that computes something expensive and pure per iteration and then folds
it into an outer variable, prints it, or keeps the best so far runs on one
core: raytrace's pixel loop (`cks = (cks * 31 + ...) % m` after `trace`),
collatz (`if s > best_steps` after `steps(i)`). The judge's own help asks the
user to restructure such a loop by hand. Hand-split twins of both
(`perf-review/round2/parfold_split.ail`, `collatz_split.ail`) show the shape
the compiler can produce itself.

Separately, the scan that decides whether a proven loop can be lowered as
chunks (`compiler/ast/parscan.ail`) refused field reads, `match`, `if x :=`,
`else`, tuple bindings, tuples, map and set literals and comprehensions, even
when the judge had proven the loop independent, and a loop the compiler chose
said nothing. Most were refused because the scan had no rule for them, not
because a chunk can't run them: a chunk lowers the body as the loop in order
does, over values it captures and does not share.

## Decision

1. **Split loops (conc.par.20).** For a `for` loop the judge keeps in order,
   over `lo..hi` or a list variable, the checker
   (`compiler/check/parsplit.ail`) takes the longest run of leading `x := e` /
   `a, b := e` statements (no `else` that leaves) as the *head*, when the head
   alone is independent (the same judge), is heavy (a loop, or a call of a free
   function with a loop or recursion, as conc.par.15 counts it), the tail
   (the rest) uses between one and four of its names, the tail writes nothing
   the head reads (assignment targets, receivers of any method, `mut`
   arguments, by root) nor the list iterated, the tail does not leave the loop
   (`break`, `return`, `?`), the function is not a `mut self` method, and,
   when the program has any `ref`, the tail calls nothing. It marks the loop
   for the lowering with the head's length.

   The lowering (`compiler/mir/lower_split.ail`) runs the loop in blocks of
   4096 iterations. In each block the head runs on the pool as a builder loop
   (conc.par.16) that pushes each kept value to a list of its own; then the
   tail runs in order with each value taken from its list. A block too small
   to repay the pool (conc.par.14) or run with one worker runs the body as
   written, with no lists; one where a chunk traps runs in order with the
   pushes (whose lowering also finds the lists' element types). The head has no effect and writes nothing that outlives the
   iteration, so running it early or elsewhere can't be seen; the tail runs
   in iteration order; a trap in the head sends the block back to run in
   order, so the tail's output before the trap and the trap are the
   sequential ones (conc.par.1, conc.par.11).

2. **The chunk scan takes what the judge proves (amends conc.par.8 and
   conc.par.13).** Field reads (not `self.f`), `match` and its pattern
   bindings, `if x :=`, `x := e else ...` and `e else ...` whose leaving
   part is `continue`, `a, b := e`, tuple, map and set literals, `none`,
   char and bytes literals, and comprehensions (counted as nested loops) are
   taken. A write to a field of a variable, a method call on a field path and
   a `mut` argument that is a field path count as writing the whole variable,
   which a chunk may do only to its own locals. Still refused: `?`, which
   leaves the function; `self.f` (a chunk has no `self`); calls through a path;
   function literals other than an inline key function.

## Rejected alternatives

- **Splitting inner loops of a loop that runs in order.** Measured on
  ail-linux (4 cores, 2026-11-17): raytrace at 4000 went from 1.70 s to
  2.96 s, one pool dispatch per row of cheap pixels, and the split marks
  linked the task runtime, whose allocation path is slower for this
  allocation-heavy program even on one thread (4.81 s). Only outermost loops
  split.
- **Splitting a loop whose head is light.** At 4f28ab8 raytrace's sphere loop
  in `trace` (`t := hit(...)` over four spheres) split: each call of `trace`
  paid a block's setup, and the program linked the task runtime; raytrace ran
  3x slower with no region on the pool. The head must now be heavy, as a
  fork-join call must be (conc.par.15).

- **Run the head for the whole loop first, then the tail.** Unbounded memory
  for long loops, and a trap late in the head would be reported before
  output the tail should have printed. Blocks bound the kept values to 4096
  per name and keep traps in place.
- **Fall back to the head alone in order after a trap.** The tail's output for
  the iterations before the trap would come after the trap: not sequential.
- **More reduction kinds in the judge instead** (`if s > best`, `max`, `*=`).
  Exact too, and cheaper, but it covers only folds; a split also covers
  "compute, then print", the most common loop an agent writes. It can still
  come later for loops with no expensive head.
- **Split any statement prefix, not only bindings.** A prefix that assigns or
  calls for an effect is not independent by construction; bindings are the
  shape every observed case has. The general form waits for a measured case.
- **Report split loops in `ail effects --parallel`.** Left for a later change:
  the verdict words are part of conc.par.6's output format.

## Consequences

A loop whose expensive part is a pure call and whose cheap part is ordered
uses every core with no change to the source; its results, output order and
traps are the sequential ones. The body is lowered four times (the head as a chunk,
the block after a trap, the block as written, and the tail), so code size grows
for split loops. Each
block allocates up to four lists and copies struct values into them. More
loops proven by the judge now reach the pool, and a written `par for` with
those constructs no longer warns under conc.par.13.
