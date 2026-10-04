# 08. Errors

Chapter key: `err`

> **Status:** Drafted (partial). The error model, error-type spelling,
> propagation, and trap behaviour are specified. Handling forms (matching,
> fallbacks, context) and error-type declarations are not yet designed
> (OQ-029, OQ-030).

## Model

**err.model.1** — AIL has no exceptions. A recoverable failure is
represented as a value returned from a function. *(ADR 0045)*

**err.model.2** — Integer overflow (`expr.arith.1`), out-of-bounds access,
failed assertions and explicitly unreachable code **trap**. They are not
recoverable error values. *(ADR 0045, ADR 0035)*

## Error types

**err.type.1** — `T ! E` is the type of a value that is either a success value
of type `T` or an error of type `E`. *(ADR 0046)*

**err.type.2** — In a function's return type, `!T` denotes `T ! S`, where `S`
is the error set **inferred** from the function body: the union of all error
types the body can propagate or return. *(ADR 0046)*

**err.type.3** — A function that is part of its package's public API MUST
state its error type explicitly, using `T ! E`. Inferred error sets
(`err.type.2`) are permitted only for functions that are not part of the
package's public API. *(ADR 0046)*

**err.type.4** — An inferred error set is a closed union. Matching on it MUST
be checked for exhaustiveness. *(ADR 0046)*

```ail
fn read_config(path str) !Config
    text := read_file(path)?
    parse(text)?
```

> **Note:** syntax for explicit error unions, which types may be errors, and
> conversion between error sets are open (OQ-030).

## Propagation

**err.prop.1** — For an expression `e` of type `T ! E`, `e?` evaluates to the
success value if there is one. Otherwise it returns the error from the
enclosing function, whose error type must be able to hold `E` (or, for an
inferred set, then includes `E`). For an expression of type `T?` inside a
function returning an optional type, `e?` evaluates to the contained value,
or returns absence. *(ADR 0045, ADR 0048)*

## Traps

**err.trap.1** — A trap stops the task in which it occurs. Deferred cleanup in
that task runs. The trap cannot be intercepted within that task. *(ADR 0047)*

**err.trap.2** — The task that spawned or awaits a trapped task MUST be able
to observe a *trapped* outcome. That outcome includes the trap kind, the
source location, a stack trace and, where applicable, the spec rule ID.
*(ADR 0047)*

**err.trap.3** — A trap in the program's main task, or one that nothing
observes up to the main task, terminates the program with the diagnostic and
a non-zero exit status. *(ADR 0047)*
