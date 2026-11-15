# 1119. `never`: a result type for functions that never return

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** designer (Kaleb approved the direction on 2026-11-15 with the E13 follow-up plan; details by the AI under [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system; 8 — Error handling
- **Spec rules:** [type.never.1-3](../spec/04-types.md), [expr.block.3](../spec/05-expressions.md) (amended), [err.else.1](../spec/08-errors.md), [err.decl.2](../spec/08-errors.md)
- **Amends:** [ADR 0139](0139-unit-results-statement-else-and-divergence.md) (its rejected alternative "a nameable `Never` type" is adopted for function results only)

## Context

In the E13 agent benchmark, 13 of 288 AIL trials wrote a command-line
program with a helper that prints a usage line and calls `io.exit`, then used
it as a fallback:

```ail excerpt
fn usage()
    io.eprint("usage: count N")
    io.exit(2)

fn main()
    n := env.args()[0].to_int() else usage()    # err.else.1: no value
```

`io.exit` and `unreachable` already diverge (ADR 0139), but a function that
ends in one returns nothing as far as its callers know, so every call site
needs an `else` block with the call and a `return`. The error said only that
the fallback had no value, and agents spent a compile round or more on it.
ADR 0139 rejected a nameable divergence type because none of its three
programs needed a user-written diverging function; E13 shows agents write one
by habit (`sys.exit`, `process::exit`, `-> !`, `NoReturn`).

## Options considered

### Option A — a result type `never`, written only on declared functions (chosen)
- **Pros:** the signature says what the function does, so a reader and an
  agent know from one line that `usage()` ends the program; the checker
  verifies the body (it must not reach its end); the name is the one
  TypeScript uses and reads as English; no parser change (it is a type name).
- **Cons:** one more built-in type name; limited to result position, so a
  function value or field can't be `never`.

### Option B — infer divergence from the body
- **Pros:** nothing to write.
- **Cons:** a call's meaning would depend on a body the reader can't see
  (against local reasoning, ADR 0076 criterion 1); a change inside `usage`
  silently breaks or fixes callers in other files; the checker would need
  whole-program order for mutually recursive helpers.

### Option C — an attribute, `@noreturn fn usage()`
- **Pros:** no new type.
- **Cons:** attributes don't change types anywhere else in AIL; the result
  type still says "no value", so the signature contradicts the attribute.

### Option D — a full bottom type (`never` in any type position)
- **Pros:** general (`fn() never` values, `List[never]`).
- **Cons:** coercion and inference rules for a type no program in the corpus
  needs; hard to reverse. Can be added later on top of A.

## Decision

`never` is a built-in type name written only as the result type of a declared
function or method (`type.never.1`). A call of such a function diverges
(`type.never.2`, `expr.block.3`), so it fits where any value is expected and
leaves its block. Its body must diverge at its end, and `return` in it is an
error (`type.never.3`). `io.exit` and `unreachable` behave as `never` calls.
Anywhere else `never` is an error that says where it may be written.

Native code lowers a `never` function with no result and ends the block after
each direct call with `unreachable`, as after `io.exit`
(`compiler/mir/lower_never.ail`).

Two diagnostics change with it:
- `err.else.1`: a fallback that calls a function returning no value says so,
  and its help offers both fixes: declare the function `never`, or write an
  `else` block that calls it and then `return`s. `err.else.2` gets the same
  help when an `else` block ends in such a call.
- `err.decl.2`: `T ! str` printed six errors (the declaration, then each
  `return` and call). It now prints one, with a help that shows a two-line
  `error enum` declaration; the function's result counts as unknown after it.

## Rationale

AI understanding first: the signature carries the fact, the checker enforces
it, and the common agent program compiles as written once `never` is added.
Readability: `fn usage() never` reads as a sentence. Speed: no cost; the
lowering already had the block-ending path for `io.exit`. Tokens: one word
per diverging helper against a three-line `else` block at each call site.

## Consequences

- `n := s.to_int() else usage()` compiles when `usage` is declared `never`.
- Naming a `never` function as a value (`f := usage`, `type.fn.2`) checks, but
  native code doesn't end the block after a call through the value; a call of
  it where a value is needed is reported by the lowering. Follow-up if agents
  meet it.
- `never` can't be a parameter, field, local, type argument, function-type
  result or the success part of `T ! E`.
- The compiler's own source can use `never` only after a seed carries it.
- The language card gets a line in the S17 card sync.

## Rejected alternatives

- **Inferring divergence from the body:** a call's type would depend on code
  the reader doesn't see, and edits inside a helper would change its callers.
- **An attribute (`@noreturn`):** the result type would still say "no value";
  attributes change no types elsewhere.
- **A full bottom type usable in every position:** rules for coercion and
  inference with no program that needs them; harder to reverse than adding it
  later.
- **Keeping ADR 0139's position (no nameable type):** costs agents a compile
  round in 13 of 288 E13 trials and a three-line block at every call site.
