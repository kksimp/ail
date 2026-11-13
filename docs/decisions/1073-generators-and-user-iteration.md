# 1073. Generators, `Seq[T]` and `for` over any type with `next`

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer put generators in 0.1.0 (2026-11-12, "everything Fable recommends", AI understanding first)
- **Topic:** 05 — Expressions; 03 — Types
- **Spec rules:** [expr.yield.1-4, expr.iter.1-3](../spec/05-expressions.md#generators-and-iteration); amends [expr.loop.4](../spec/05-expressions.md), [lex.kw.1](../spec/02-lexical.md)
- **Supersedes:** [ADR 0965](0965-generators.md)
- **Builds on:** ADR 1067 (function values are closure objects), [ADR 0095](0095-colorless-tasks.md)

## Context

`for` took only lists, sets, ranges, `bytes` and channels. A lazy sequence (a
tree walk, an endless counter, a pipeline that stops early) had to be a
function that builds a whole `List`, or a hand-written state struct with no
way to loop over it. ADR 0965 planned generators for 0.2; the pre-release
review moved them into 0.1.0 together with closures, which their adapters need.

## Decision

1. **Iteration protocol** (expr.iter.1). `for x in e` also takes a value whose
   type has `fn next(mut self) T?`; `x` is `T` and the loop calls `next()`
   until it gives `none`. As for a list, the loop works on a copy of `e`
   (expr.loop.4: it iterates the value `e` had when it started); a unique `e`
   is moved into the loop. Comprehension clauses take the same values. A
   `next` with another shape is an error naming the expected signature.
2. **`Seq[T]`** (expr.iter.2) is a prelude type, not a built-in:
   `unique struct Seq[T]` with one field `step fn() T?`, and
   `fn Seq.next(mut self) T?` calls `step`. So `Seq` meets the protocol of
   item 1, `Seq(step: f)` makes a sequence from any function, and its
   uniqueness (own.uniq.1) gives single use for free: a second loop over the
   same `Seq` is a use-after-move error, not a silent empty loop.
3. **Generators** (expr.yield.1-4). A function whose body contains a `yield`
   statement and whose result is `Seq[T]` is a generator. Calling it runs none
   of its body and returns a `Seq[T]`; each `next()` runs the body from where
   it stopped to the next `yield e` (`e` of type `T`, yielded as a copy). A
   bare `return` or the body's end ends the sequence; `return e` is an error.
   A generator keeps its own copy of each argument (or takes a `move` argument
   over), so it can't take `mut` parameters or `mut self`; it can't `defer`,
   have contracts, or `yield` inside `par for`, `with` or a function literal.
   No keyword marks a generator and callers need no marker: the signature
   says `Seq[T]`, and `yield` in the body says how it is made.
4. **Adapters** (expr.iter.3), prelude generator methods on `Seq`:
   `Seq.of(xs)`, `map`, `filter`, `take_while`, `take`, `skip`, `enumerate`,
   `zip`, `chain` (lazy; each takes its sequence over), `collect` and `count`
   (read to the end).
5. **Native lowering.** The body is lowered as an ordinary function in which
   `yield v` is an instruction; a pass (`compiler/mir/genframe.ail`) turns it
   into the resume function `gen.NAME(frame) T?`. The frame is the `Seq`'s
   closure object (kind 13, ADR 1067): word 0 the resume function, word 1 the
   state, then one slot per value read after a `yield` where its definition no
   longer dominates the read, stored where defined and loaded where read. The
   constructor `NAME(args)` allocates the frame, stores the arguments and
   returns the `Seq`. No stack per generator, no task, no channel; colourless
   tasks (ADR 0095) are untouched.
6. `yield` becomes a keyword (lex.kw.1). It was already rejected as a foreign
   statement word, so no valid program changes meaning.

## Rejected alternatives

- **An `Iterator` trait** (`impl Iterator for T`): needs associated types or
  trait parameters (OQ-037, open) to name `T`. The `next` method is the same
  contract with no new trait machinery, and the error names the signature.
  It stays open to add the trait later as a name for the same method.
- **`Seq[T]` as a built-in type with a copyable handle** (like `Channel`):
  copies would share a position, so reading a sequence twice would silently
  give nothing the second time; uniqueness makes that a compile-time error.
- **Single use by a dedicated rule** (ADR 0965 item 3): the ownership rules
  already say it; a prelude `unique struct` reuses them and their diagnostics.
- **Task plus channel, coloured `async`, stackful coroutines, a `gen`
  keyword:** as in ADR 0965 (a stack and a context switch per element, a
  colour, hidden suspension points, an extra token).
- **Spilling every local to the frame:** simpler, but every read of every
  variable becomes a load. Spilling only what a resume point needs keeps the
  straight-line code between `yield`s in registers.
- **`for` advancing a non-unique value in place:** contradicts expr.loop.4
  (the loop iterates the value as it was) and value semantics (ADR 0060).

## Consequences

- Lazy pipelines read as their eager forms: `count(10).filter(|x| x > 3).take(2)`.
- A `Seq` abandoned before its end drops its frame with the collector; nothing
  in the body runs again. `defer` is rejected for that reason.
- A trap in a generator's body shows the frame `gen.NAME` in the trace.
- Later: an `Iterator` name for the protocol after OQ-037; `yield` from a
  nested function literal; `Seq` methods on lists (`xs.seq()`), if measured
  code asks for them.
