# 1067. Function literals that copy their locals, failing function values, and list methods that take functions

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer asked on 2026-11-12 for every language-review recommendation to go in the 0.1 RC, by the judgment of the AI, never compromising AI understanding)
- **Topic:** 4 — Type system
- **Spec rules:** [type.fn.2-8](../spec/04-types.md), [type.coll.21-26](../spec/04-types.md), [expr.key.1](../spec/05-expressions.md), [native.fnval.1](../spec/23-native.md)
- **Builds on:** [ADR 0455](0455-function-types-and-values.md), [ADR 0456](0456-calling-function-values.md), [ADR 0250](0250-key-functions-and-take.md)
- **Amends:** [ADR 0455](0455-function-types-and-values.md) (failing functions are values), [ADR 0456](0456-calling-function-values.md) (anonymous functions exist)

## Context

Function values (ADR 0455) could only name a declared function. A callback
that needs a local (`xs.filter(|x| x > limit)`) had to become a loop or a
top-level function with an extra parameter, and a function that fails could
not be a value at all. Agents write `filter`, `map`, `any` and lambdas by
reflex; the language review found every such program rejected, with the fix
far from the error. Key functions (`by: |x| e`, ADR 0250) already gave `|x|`
its meaning in argument position.

## Options considered

### Syntax

#### Option A — `|a, b| e`, one expression on the line (chosen)
- **Pros:** the spelling of key functions, so `by: |x| e` and `|x| e` read
  the same; short; agents produce it from Rust and Ruby habits; the body is one
  expression, so a literal never hides control flow.
- **Cons:** `|` also spells bit-or (ADR 1070, the bit-operators change). Settled by position: bit-or is
  infix only, so a `|` where an operand is expected opens a literal, and a
  stray one gets a diagnostic naming both readings.

#### Option B — a keyword form, `fn(x) x + 1`
- **Cons:** clashes with the type `fn(int) int` in the reader's eye; more
  tokens on every callback; a second spelling beside `by: |x|`.

#### Option C — multi-line block bodies
- **Cons:** a literal becomes a second kind of function body, with
  `return` meaning two things. A longer body is a declared function, which
  the literal can call.

### Captures

#### Option A — copy the named locals when the literal is made, read-only (chosen)
- **Pros:** matches value semantics (ADR 0060): a literal is a value like a
  struct of its copies; no lifetimes, no aliasing, no surprise when the local
  changes later; safe to send to a task. Shared state is explicit: capture a
  `ref` cell.
- **Cons:** a counter needs `ref`.

#### Option B — capture by reference
- **Cons:** second-class borrows (ADR 0062) can't escape, so a returned
  closure would need lifetimes or hidden boxing; aliasing breaks local
  reasoning.

#### Option C — writable copies
- **Cons:** a write that changes nothing the caller can see is a silent bug;
  read-only copies make it an error with the fix (`ref`) in the help.

### Errors in literals

#### Option A — `fn(A) R ! E` types; `?` leaves the literal (chosen)
- **Pros:** the error model works unchanged inside callbacks; a parse
  callback can be passed to a function that handles its errors; a function
  that can't fail fits where a failing one is expected.
- **Cons:** `?` in a literal doesn't leave the enclosing function, which an
  agent may expect; the type of the literal says so, and the call must be
  handled.

#### Option B — literals can't fail
- **Cons:** forces every fallible callback back into a loop.

### List methods

#### Option A — `any`, `all`, `find`, `filter`, `map`, `group_by`, taking a function that can't fail (chosen)
- **Pros:** the names agents reach for; each is one loop the native code
  inlines when given a literal.
- **Cons:** a fallible step needs a `for` loop; the error says so. Methods
  that pass errors on can come later without breaking these.

#### Option B — comprehensions only
- **Cons:** comprehensions stay, but `xs.any(...)` is what agents write; a
  rejected `any` costs a round trip.

### Representation

A function value is a pointer to a collected object: word 0 is the code
address, the rest are the copies. A call loads word 0 and passes the object
first. A named function gets a one-word object and a thunk. Rejected: a pair
of words (code, environment), which doubles every `fn` slot in structs and
lists and complicates `List[fn(...)]`; and passing environments only to
literals, which makes every call site test the kind of value.

## Decision

Option A in each part. Arguments are checked in order, so a literal's
parameter types come from the arguments before it; type arguments of a generic
function settle from a literal's result. `return`, `break` and `continue`
can't leave a literal; effects in its body count for the function where it is
written (type.fn.7).

## Consequences

- `|x| e` with no `by:` is now a value everywhere; `by: |x| e` keeps the key
  function rules.
- A failing named function can be a value (amends ADR 0455).
- The compiler's own source doesn't use literals until a seed with them is
  published.
- The generators lane (ADR 1073) builds `Seq[T]` on the same object layout.
