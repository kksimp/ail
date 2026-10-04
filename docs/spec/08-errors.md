# 08. Errors

Chapter key: `err`

> **Status:** Drafted (partial). Specified: the error model, error-type
> spelling, propagation, `else` handling, error context, and trap behaviour.
> Not yet designed: matching on errors, safe member access (OQ-029), and
> error-type declarations (OQ-030).

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

**err.prop.2** — Applying postfix `?` to an expression of optional type `T?`
is a compile-time error unless the enclosing function's return type is an
optional type. *(ADR 0051)*

## Handling with `else`

**err.else.1** — For `e` of type `T?` or `T ! E`, the declaration
`x := e else d` binds `x` to the success value of `e`. If `e` is absent or
failed, it binds `x` to the value of `d` instead. `d` is evaluated only in
that case. *(ADR 0049)*

**err.else.2** — The `else` part may instead be a statement, or an indented
block, that definitely leaves the enclosing block (by `return`, `break`,
`continue` or a trap). It is a compile-time error if an `else` block can
complete normally without producing a value of type `T`. *(ADR 0049)*

**err.else.3** — For `e` of type `T ! E`, `else name` followed by an indented
block binds the error value to `name` within that block. *(ADR 0049)*

**err.else.4** — After a declaration with an `else` part, the declared name
holds a success value on every path that reaches the following statement.
*(ADR 0049)*

```ail
port := env("PORT") else 8080
user := find(id) else return NotFound
data := read(path) else err
    log.warn("read failed", err)
    return Defaults
```

## Context and traces

**err.ctx.1** — Each propagation by `?` MUST record its source location in
the propagated error's return trace. *(ADR 0050)*

**err.ctx.2** — `e? m`, where `m` is a string expression, behaves as `e?` and
additionally attaches `m` to the propagated error as context. *(ADR 0050)*

**err.ctx.3** — Error reports MUST present the error, every attached context
message, and the return trace. *(ADR 0050)*

```ail
cfg := parse(text)? "parsing config file"
```

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
