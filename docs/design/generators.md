# Generators

Status: accepted as [ADR 1073](../decisions/1073-generators-and-user-iteration.md) (built in 0.1.0; the plan below was ADR 0965). The as-built design differs: `Seq[T]` is a prelude `unique struct`, adapters are prelude generator methods, and `for` takes any type with `next(mut self) T?`. Gap 11 of the C/C++/Rust gap survey.

## Why

Lazy sequences (a tree walk, a file read line by line, an infinite counter) are written today as a function that builds a whole `List`, or as a hand-written state struct. Rust and Python users write `yield`.

## Design

- A function whose body contains `yield e` and whose return type is `Seq[T]` is a generator: `fn count(n int) Seq[int]` with `for i in 0..n` and `yield i` inside. There is no `gen`/`async` keyword and no colour: the signature is an ordinary function returning an ordinary value, and a caller needs no marker (`for x in count(3)`, `count(3).take(2)`).
- Calling it runs no body code. It returns a `Seq[T]` value holding the arguments and a state number.
- Lowering is a state machine (monomorphised per generator, own file `compiler/mir/lower_gen.ail`, ADR 0823): locals live across yields move into the `Seq` value, `yield` stores the state and returns `some(e)`, resume jumps on the state. No stack per generator, no task, no channel.
- A `Seq[T]` is a single-use value (consumed by `for`, `next()` or an adapter), enforced by the move rules (ADR 0060 family); copying it is an error naming the rule.
- `for x in seq` extends expr.loop.4: `e` may be a `Seq[T]`. Adapters (`take`, `map`, `filter`, `collect`) are prelude code on `Seq`.
- Not allowed in a generator in slice 1: `yield` inside `par`, `spawn`, or a closure; blocking calls are fine (they block the consumer, as any call does), so ADR 0095 colorless tasks are untouched.

## Dependencies

OQ-037 (associated types vs trait parameters) is open. Slice 1 makes `Seq[T]` a built-in prelude type, not a user `Iterator` trait, so it does not wait for OQ-037. A general iterator trait stays a later decision for the designer.

## Rejected alternatives

- Generator as task plus channel: a stack and a context switch per element, and cross-thread cost for single-threaded code.
- `async`/`await`-style coloured functions: rejected by ADR 0095.
- Stackful coroutines (resume/suspend as library calls): needs a stack per coroutine and hides suspension points; the state machine keeps them visible as `yield`.
- A `gen` keyword before `fn`: an extra token and a visible colour; the return type already says it.

## Slices

1. Spec rules, ADR (this change).
2. Parser and checker: `yield`, `Seq[T]` typing, single-use rule, diagnostics.
3. State-machine lowering in `lower_gen.ail`.
4. `for` over `Seq`, adapters in the prelude.
