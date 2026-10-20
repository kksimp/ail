# 08. Errors

Chapter key: `err`

> **Status:** Drafted (partial). Specified: the error model, error-type
> declarations and spelling, unions, results with no success value,
> propagation, handling, `else` (including statement `else`), error context,
> the default error form, and trap behaviour. Not yet designed:
> matching on error unions, safe member access (OQ-029), and converting an
> inferred set to a public error type (OQ-030).

## Model

**err.model.1** — AIL has no exceptions. A recoverable failure is
represented as a value returned from a function. *(ADR 0045)*

**err.model.2** — Integer overflow (`expr.arith.1`), out-of-bounds access,
failed assertions and explicitly unreachable code **trap**. They are not
recoverable error values. The prelude function `unreachable(message str)`
traps with `message`, and a call to it diverges (`expr.block.3`). A call
`unreachable()` without a message is a compile-time error whose help says to
add one. The names `panic`, `assert`, `fatalError` and `loop` get helps
(`expr.name.2`) that point to `unreachable(message)` and `while true`.
*(ADR 0045, ADR 0035, ADR 0139)*

```ail skip=context
fn tag(k Kind) str
    match k
        A => "a"
        B => "b"
        _ => unreachable("tag: new Kind not handled")
```

## Error types

**err.decl.1** — `error struct Name` and `error enum Name` declare error
types. Apart from that, they are ordinary structs and enums. *(ADR 0120)*

**err.decl.2** — Only error types can be errors: the `E` of `T ! E`, the
members of an error union (`err.type.5`), and the members of an inferred
error set. *(ADR 0120)*

**err.decl.3** — In `T ! E`, `T` MUST NOT be an error type. Where a value of
type `T ! E` is expected, a value of type `T` is the success value, and a
value of an error type that is in `E` is the error. *(ADR 0120)*

```ail fragment
error enum ParseError
    Empty
    BadNumber(text str)

fn parse_qty(s str) int ! ParseError
    if s == ""
        return Empty
    n := s.to_int() else return BadNumber(text: s)
    n
```

**err.type.1** — `T ! E` is the type of a value that is either a success value
of type `T` or an error of type `E`. *(ADR 0046)*

**err.type.2** — In a function's return type, `!T` denotes `T ! S`, where `S`
is the error set **inferred** from the function body: the union of all error
types the body can propagate or return. This holds when `T` isn't a declared
error type; for `!X` with a declared error type `X`, see `err.type.7`.
*(ADR 0046, ADR 0139)*

**err.type.3** — A function that is part of its package's public API MUST
state its error type explicitly, using `T ! E` or `! E` (`err.type.7`).
Inferred error sets (`err.type.2`, and bare `!`) are permitted only for
functions that are not part of the package's public API. *(ADR 0046,
ADR 0139)*

**err.type.4** — An inferred error set is a closed union. Matching on it MUST
be checked for exhaustiveness. *(ADR 0046)*

```ail skip=context
fn read_config(path str) !Config
    text := read_file(path)?
    parse(text)?
```

**err.type.5** — An explicit union of error types is written with `|`:
`T ! A | B`. The order of the members doesn't matter. *(ADR 0122)*

**err.type.6** — An inferred error set is the smallest set that includes
every error its function's body can propagate or return. For mutually
recursive functions, the sets are computed together, and recursion adds no
error by itself. *(ADR 0122)*

**err.type.7** — A return type may have no success value. `! E` or
`! A | B`, with nothing before the `!`, means "no success value; can fail with
`E`" (or with `A` or `B`). A bare `!` means no success value and an inferred error set
(`err.type.2`). Both forms are valid for every function, not only `main`
(`mod.main.1`). A type name directly after `!` is read by name resolution: if
`X` is a declared error type, `!X` is `! X`; otherwise it is `X ! S` with `S`
inferred. A type parameter is never an error type here. No valid program
changes meaning, since `err.decl.3` already rejects an error type as a
success type. The canonical spacing, which the formatter writes, is `! E` for
an explicit error type and `!T` for an inferred set. The body of such a
function is a statement block. It succeeds when it completes normally or
reaches a bare `return`, and it fails on `return e` (with `e` an error) or on
`?`. *(ADR 0139, ADR 0046)*

```ail skip=context
fn save(path str, text str) ! IoError
    file := open_file(path)?
    file.write(text)?

fn sync_all() !
    save("a.txt", "x")?
    save("b.txt", "y")?
```

> **Note:** converting an inferred set to a public API's error type is open
> (OQ-030).

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

**err.prop.3** — If the enclosing function states its error type, every error
type `e` can fail with MUST be a member of it. Otherwise `e?` is a
compile-time error that names the missing types. *(ADR 0122)*

**err.use.1** — A value of type `T ! E` MUST be handled with `?` or `else`,
or returned from a function whose result can hold it. Discarding it, or
using it where a `T` is expected, is a compile-time error. *(ADR 0122)*

**err.use.2** — An `if` whose condition binds a value of type `T ! E`
(`expr.if.5`) MUST have an `else`. *(ADR 0125)*

## Handling with `else`

**err.else.1** — For `e` of type `T?` or `T ! E`, the declaration
`x := e else d` binds `x` to the success value of `e`. If `e` is absent or
failed, it binds `x` to the value of `d` instead. `d` is evaluated only in
that case. *(ADR 0049)*

**err.else.2** — The `else` part may instead be a statement, or an indented
block, that definitely leaves the enclosing block (by `return`, `break`,
`continue` or a trap). It is a compile-time error if an `else` block can
complete normally without producing a value of type `T`. This is for a
declaration with a success value; `err.else.5` covers results with none.
*(ADR 0049, ADR 0139)*

**err.else.3** — For `e` of type `T ! E`, `else name` followed by an indented
block binds the error value to `name` within that block. The same holds for
`if ... else name` (`expr.if.6`) and for statement `else` (`err.else.5`). The
type of `name` is the error part's type: when the error type is an inferred or
explicit set with one member, that member's type (so field access works); with
two or more members, field access waits for OQ-029. *(ADR 0049, ADR 0139)*

**err.else.4** — After a declaration with an `else` part, the declared name
holds a success value on every path that reaches the following statement.
*(ADR 0049)*

**err.else.5** — When `e` has a type that is a result with no success value
(`!` or `! E`, `err.type.7`), the expression statement `e else BLOCK`,
`e else name BLOCK` (which binds the error as in `err.else.3`) or
`e else STATEMENT` (as in `save() else return`) runs its `else` part only on
failure. The block MAY complete normally: `err.else.2`'s requirement to
produce a `T` doesn't apply, and execution continues after the statement. A
value fallback (`save() else 0`) is a compile-time error. On a `T?`, or on a
`T ! E` with a success value, statement `else` is a compile-time error whose
help says to bind the value: `x := ... else ...`. *(ADR 0139)*

```ail skip=context
port := env("PORT") else 8080
user := find(id) else return NotFound
data := read(path) else err
    log.warn("read failed", err)
    return Defaults
```

```ail skip=context
save(path, text) else err
    print("save failed: {err}")       # completes normally; execution continues
save(path, text) else return          # in a function with no success value, a bare return succeeds
```

## Context and traces


**err.else.6** — `e else d` is also an expression: the value of `e` if it is
present or succeeded, otherwise `d`, which MUST have `e`'s success type and
is evaluated only when needed. `e else return x`, `else break` and
`else continue` are expressions whose fallback leaves. `else` binds more
loosely than every other operator and associates to the right
(`expr.prec.1`). An `else` followed by an indented block is the binding or
statement form (`err.else.1–5`); at the start of a statement,
`e else return ...` is the statement form. *(ADR 0144)*

```ail skip=context
fn smallest(nums List[int]) int
    nums.min() else 0
count := (m.get(k) else 0) + 1
```
**err.ctx.1** — Each propagation by `?` MUST record its source location in
the propagated error's return trace. *(ADR 0050)*

**err.ctx.2** — `e? m`, where `m` is a string expression, behaves as `e?` and
additionally attaches `m` to the propagated error as context. *(ADR 0050)*

**err.ctx.3** — Error reports MUST present the error, every attached context
message, and the return trace. *(ADR 0050)*

**err.text.1** — An error report shows the error value's text form
(`trait.std.3`). By default it is the type name, then `.Variant` for an enum,
then the fields as `(name: value, ...)` when there are any, with strings
quoted: `ParseError.BadNumber(text: "x1")` (`trait.auto.2`). *(ADR 0122,
ADR 0126)*

```ail skip=context
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
