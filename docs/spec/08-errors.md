# 08. Errors

Chapter key: `err`

> **Status:** Drafted (partial). Specified: the error model, error-type
> declarations and spelling, unions, results with no success value,
> propagation, handling, `else` (including statement `else`), error context,
> the default error form, matching on error unions, and trap behaviour. Not
> yet designed: converting an inferred set to a public error type (OQ-030).

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

**err.msg.1** — An error type may carry a message, a string literal written
after `error struct Name` or after a variant of an `error enum`. The message
is the type's display text: it defines `impl ToStr` for the type, with the
struct's or variant's fields in scope by name (for an enum, the variant's
fields; a field the message doesn't name is skipped). An `error struct` with a
message may have no fields. A message on a type that isn't an error type is a
compile-time error. *(ADR 0278)*

**err.msg.2** — Either every variant of an `error enum` has a message or none
does, and a generic error type has none; otherwise it's a compile-time error.
A type with messages can't also have an `impl ToStr` (`trait.impl.6`).
*(ADR 0278)*

```ail
error enum LoadError
    MissingEquals "missing '='"
    BadName(text str) "bad cell name '{text}'"

error struct NotFound "nothing at {path}"
    path str

fn main()
    print(LoadError.BadName(text: "Z9"))   # bad cell name 'Z9'
```

**err.decl.2** — Only error types can be errors: the `E` of `T ! E`, the
members of an error union (`err.type.5`), and the members of an inferred
error set. Another type there is reported once, with a help that shows an
`error enum` declaration; the function's `return`s and calls are then not
checked against the result it failed to declare. The one plain type allowed
is `str` alone (`T ! str`, `err.str.1`). *(ADR 0120, ADR 1119, ADR 1129)*

**err.str.1** — `T ! str` declares that the function fails with text. In it,
`return TEXT` of any `str` value (a literal, an interpolation, a variable, a
call) is the failure, and a `return` of any other value is the success value.
The error has no error type: an `else |e|` handler binds `e` as a `str`, and
it prints as that text. The success type `T` is required: `! str` with
nothing before the `!` still means the success type `str` with inferred
errors (`err.type.2`). `str` is the only plain type allowed this way;
`T ! int` and `T ! Plain` are still `err.decl.2`, and `str` in a union
(`! str | E`) is too. Only `return` makes a `str` the failure: the last
expression of the body is the result, as always. *(ADR 1129)*

**err.str.2** — When the success type of `T ! str` is `str` itself, a `str`
returned could be either, so `str ! str` is a compile-time error: declare
another result type, or an error type for the failure. *(ADR 1129)*

**err.str.3** — A function declared `T ! str` passes on any error with `?`, or
by returning a call that fails: the error becomes its text. `?` in another
function of a `T ! str` call needs that function to declare `str` or to infer
its errors. *(ADR 1129)*

```ail rules=err.str.1,err.str.3
fn parse(s str) int ! str
    n := s.to_int() else return "not a number: {s}"
    if n < 0
        return "negative"
    n

fn double(s str) int ! str
    n := parse(s)?
    n * 2

fn load(path str) int ! str
    text := fs.read_text(path)?          # an fs.IoError, passed on as its text
    parse(text.strip())?

fn show(s str)
    n := double(s) else |e|
        print("failed: {e}")
        return
    print(n)

fn main()
    show("21")                # 42
    show("-3")                # failed: negative
```

```ail error=err.str.2 rules=err.str.2
fn f(n int) str ! str
    if n < 0
        return "negative"
    return "ok"

fn main()
    print(f(1) else "bad")
```

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

```ail error=err.decl.3
error enum Oops
    Bad

fn f() Oops ! Oops
    Oops.Bad
```

```ail fragment
error enum Oops
    Bad

fn f() int ! Oops
    return Oops.Bad
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

```ail fixture=packages/pub_inferred_errors rules=err.type.3
```

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
optional type, its errors are inferred (`err.prop.5`), or the `?` is inside
the value of an `else` (`err.prop.4`). *(ADR 0051, ADR 0252, ADR 0279)*

**err.prop.3** — If the enclosing function states its error type, every error
type `e` can fail with MUST be a member of it. Otherwise `e?` is a
compile-time error that names the missing types. *(ADR 0122)*

**err.prop.4** — A `?` on an optional inside the value `e` of `e else ...`
(`err.else.1`, `err.else.6`) goes to the nearest such `else` instead of
leaving the function: when it finds `none`, the rest of `e` is skipped and the
`else` part runs as if `e` were absent. If `e` contains such a `?` and its type
`T` isn't optional or a result, `e` has type `T?` for the `else`. A result
`?` inside `e` still leaves the function (`err.prop.1`). When `e` is a result
and also contains such a `?`, `else err` is a compile-time error: an absent
value has no error to bind. *(ADR 0252)*

```ail
fn label(j Json) str
    j.get("user")?.str("name") else "anonymous"

fn main()
    print(label(Json.Null))   # anonymous
```

**err.prop.5** — In a function whose error set is inferred (`!T` or `!`,
`err.type.2`), and that doesn't return an optional, `e?` on an optional `e`
evaluates to the contained value, or fails with the prelude's
`error struct Absent` (display text `a value is absent`), which then joins the
inferred set. `e? m` attaches the context `m` (`err.ctx.2`). A function that
states its errors must handle absence with `else`. *(ADR 0279)*

```ail
fn port(text str) !int
    text.to_int()? "port {text}"

fn main()
    print(port("8080") else 0)   # 8080
    print(port("http") else -1)  # -1
```

A `?` that has nowhere to go is rejected with the signature change that would
make it legal, as a fix (`err.fix.1`):

```ail error=err.prop.1
error enum Oops
    Bad

fn risky() int ! Oops
    return Oops.Bad

fn main()
    n := risky()?
    print(n)
```

```ail fragment norun
error enum Oops
    Bad

fn risky() int ! Oops
    return Oops.Bad

fn main() ! Oops
    n := risky()?
    print(n)
```

```ail error=err.prop.2
fn first(xs List[int]) int
    xs.max()?
```

```ail fragment
fn first(xs List[int]) int
    xs.max() else 0
```

```ail error=err.prop.3
error enum A
    X
error enum B
    Y

fn a() int ! A
    return A.X

fn b() int ! B
    return B.Y

fn both() int ! A
    return a()? + b()?
```

```ail fragment
error enum A
    X
error enum B
    Y

fn a() int ! A
    return A.X

fn b() int ! B
    return B.Y

fn both() int ! A | B
    return a()? + b()?
```

**err.use.1** — A value of type `T ! E` MUST be handled with `?` or `else`,
or returned from a function whose result can hold it. Discarding it, or
using it where a `T` is expected, is a compile-time error. *(ADR 0122)*

```ail error=err.use.1
error enum Oops
    Bad

fn risky() int ! Oops
    return Oops.Bad

fn main()
    n := risky()
    print(n)
```

```ail fragment
error enum Oops
    Bad

fn risky() int ! Oops
    return Oops.Bad

n := risky() else 0
print(n)
```

**err.use.2** — An `if` whose condition binds a value of type `T ! E`
(`expr.if.5`) MUST have an `else`. *(ADR 0125)*

```ail error=err.use.2
error enum Oops
    Bad

fn risky() int ! Oops
    return Oops.Bad

fn main()
    if n := risky()
        print(n)
```

```ail fragment
error enum Oops
    Bad

fn risky() int ! Oops
    return Oops.Bad

if n := risky()
    print(n)
else err
    print("failed: {err}")
```

**err.ret.1** — `return e`, with `e` of an error type, is a compile-time error
in a function whose return type can't fail. The help names the return type
that would allow it. *(ADR 0285)*

**err.ret.2** — `return e`, with `e` of type `T?`, is a compile-time error in
a function that returns `T ! E`: absence has no error to report. *(ADR 0285)*

**err.ret.3** — `return e`, with `e` of an error type that isn't in the
function's stated error type, is a compile-time error. The help gives the
return type with it added. *(ADR 0285)*

```ail error=err.ret.1
error enum Oops
    Bad

fn check(n int) int
    if n < 0
        return Oops.Bad
    n
```

```ail fragment
error enum Oops
    Bad

fn check(n int) int ! Oops
    if n < 0
        return Oops.Bad
    n
```

```ail error=err.ret.2
error enum Oops
    Bad

fn first(xs List[int]) int ! Oops
    return xs.max()
```

```ail fragment
error enum Oops
    Bad

fn first(xs List[int]) int ! Oops
    m := xs.max() else return Oops.Bad
    m
```

```ail error=err.ret.3
error enum A
    X
error enum B
    Y

fn f(n int) int ! A
    if n < 0
        return B.Y
    n
```

```ail fragment
error enum A
    X
error enum B
    Y

fn f(n int) int ! A | B
    if n < 0
        return B.Y
    n
```

## Handling with `else`

**err.else.1** — For `e` of type `T?` or `T ! E`, the declaration
`x := e else d` binds `x` to the success value of `e`. If `e` is absent or
failed, it binds `x` to the value of `d` instead. `d` is evaluated only in
that case. `d` may be a call of a `never` function (`type.never.2`), such as a
`usage()` that ends in `io.exit`; a call that returns no value is an error
whose help says to declare the function `never` or to write an `else` block
that leaves. *(ADR 0049, ADR 1119)*

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
two or more members, a union of them, which `match name` narrows to one member
(`err.match.1-3`). *(ADR 0049, ADR 0139, ADR 1078)*

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

```ail error=err.else.3
fn main()
    n := [1, 2].max() else err
        0
    print(n)
```

```ail fragment
n := [1, 2].max() else
    0
print(n)
```

```ail error=err.else.4
error enum Oops
    Bad

fn risky() int ! Oops
    return Oops.Bad

fn main()
    risky() else err
        print("failed: {err}")
```

```ail fragment
error enum Oops
    Bad

fn risky() int ! Oops
    return Oops.Bad

n := risky() else err
    print("failed: {err}")
    0
print(n)
```

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

## Matching on an error union

**err.match.1** — When `else name` binds an error of two or more types,
`match name` takes as patterns the member types (`T`, or `m.T` for a type of
module `m`), the variants of a member error enum written with its name
(`T.Variant` and `T.Variant(a, b)`), `_`, and alternatives of these
(`A | B`). Any other pattern, or a type that isn't a member, is a
compile-time error whose help lists the members. *(ADR 1078)*

**err.match.2** — In an arm whose pattern is a member type `T`, or a variant
of member `T`, the matched name has type `T`, so its fields and variants can
be used. A member type pattern takes no fields. The scrutinee MUST be the name
`else` bound; any other expression of a union type is a compile-time error.
*(ADR 1078)*

**err.match.3** — Every member MUST be covered: by an arm of its type, by
arms for all of its variants, or by `_`. A missing member is a compile-time
error whose fix adds `T => todo` for each one; an arm after its member is
covered is `expr.match.5`, and `_` warns as `expr.match.3`. A failure records
its type, so the arm taken is the one for the error's actual type.
*(ADR 1078)*

```ail rules=err.match.1,err.match.2,err.match.3
error struct Missing
    path str

error enum Parse
    Empty "empty input"
    BadDigit(c char) "not a digit"

fn load(text str) int ! Missing | Parse
    if text == ""
        return Parse.Empty
    if text == "?"
        return Missing(path: "conf.txt")
    for c in text.chars()
        if c < '0' or c > '9'
            return Parse.BadDigit(c)
    text.to_int() else 0

fn describe(text str) str
    n := load(text) else err
        msg := match err
            Missing => "no file {err.path}"
            Parse.BadDigit(c) => "bad digit '{c}'"
            Parse.Empty => "nothing to read"
        return msg
    "read {n}"

fn main()
    print(describe("42"))    # read 42
    print(describe("?"))     # no file conf.txt
    print(describe("4x"))    # bad digit 'x'
    print(describe(""))      # nothing to read
```

```ail error=err.match.3
error struct A
    why str

error struct B
    why str

fn f(n int) int ! A | B
    if n == 0
        return A(why: "zero")
    if n < 0
        return B(why: "neg")
    n

fn main()
    n := f(1) else err
        match err
            A => print("a: {err.why}")
        0
    print(n)
```

```ail error=err.match.1
error struct A
    why str

error struct B
    why str

error struct C
    why str

fn f(n int) int ! A | B
    if n == 0
        return A(why: "zero")
    if n < 0
        return B(why: "neg")
    n

fn main()
    n := f(1) else err
        match err
            A => print("a")
            C => print("c")
            _ => print("other")
        0
    print(n)
```

## Context and traces


**err.else.6** — `e else d` is also an expression: the value of `e` if it is
present or succeeded, otherwise `d`, which MUST have `e`'s success type and
is evaluated only when needed. `e else return x`, `else break` and
`else continue` are expressions whose fallback leaves. `else` binds more
loosely than every other operator and associates to the right
(`expr.prec.1`). An `else` followed by an indented block is the binding or
statement form (`err.else.1–5`); at the start of a statement,
`e else return ...` is the statement form, except as a block's value
(`err.else.7`). A `for` header's iterable may be
`e else d`: `for x in j.arr("items") else []`. *(ADR 0144, ADR 0252)*

```ail skip=context
fn smallest(nums List[int]) int
    nums.min() else 0
count := (m.get(k) else 0) + 1
```

**err.else.7** — When `e else return ...`, `e else break` or `e else
continue` is the last statement of a block whose value is used (a function
body, an `if` branch or a `match` arm that gives a value), it is the
expression of `err.else.6`: its value is `e`'s success value. So a
function may end with `m.get(key) else return Missing(key: key)`. Anywhere
else the statement form stays, and a value it would give must be bound
(`err.else.4`). *(ADR 1095)*

```ail rules=err.else.7
error enum CfgError
    Missing(key str) "missing key {key}"

fn need(m Map[str, str], key str) str ! CfgError
    m.get(key) else return Missing(key: key)

fn main()
    print(need({"port": "80"}, "port") else "none")    # 80
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

```ail norun rules=err.trap.1
fn main()
    xs := [1]
    print(xs[3])
```

**err.trap.2** — The task that spawned or awaits a trapped task MUST be able
to observe a *trapped* outcome. That outcome includes the trap kind, the
source location, a stack trace and, where applicable, the spec rule ID.
*(ADR 0047)*

**err.trap.3** — A trap in the program's main task, or one that nothing
observes up to the main task, terminates the program with the diagnostic and
a non-zero exit status. *(ADR 0047)*

```ail norun rules=err.trap.3
fn main()
    xs := [1]
    print(xs[3])
```

## Tooling

**err.fix.1** — A diagnostic about error handling carries a machine-applicable
fix (`fix.edit.1`) where one is mechanical: `?` for an unhandled failure the
function can pass on (`err.use.1`); the signature change `! E` for a function
that can't yet fail (`err.use.1` with `?`, `err.prop.1`, `err.ret.1`); and
removing the name of an `else` on an optional (`err.else.3`). A fix never
changes an explicit error type. *(ADR 0285)*

**err.api.1** — `ail api` gives every function with a failing return type a
`fails_with` list: the names of the error types it can fail with, sorted and
without duplicates. For an inferred set (`err.type.2`) it is the inferred set,
and `ail api --text` spells it out as `fails with A | B`. *(ADR 0287)*

```ail tool=api,FILE expect="fails_with":["Bad"] rules=err.api.1
error enum Bad
    Oops "oops"

fn parse(n int) int ! Bad
    if n < 0
        return Oops
    n

fn main() !
    print(parse(1)?)
```

**err.paths.1** — `ail check --errors FILE` prints, as JSON, every function in
the file that can fail or touches a failure, with its `fails_with` list and its
failure paths in source order. A path has `line`, `col`, `kind` and `errors`,
and where they apply `how`, `call` and `context`. `kind` is `propagate` (`?`
on a result), `handle` (`else` on a result), `return` (of an error value, or
of a result), or `absent` (`?` or `else` on an optional, which has no error).
`how` is `default`, `leave` or `block`. A trap isn't a failure path
(`err.trap.1`). *(ADR 0286)*

## Runnable examples for the rules above

Each rule ID is cited by a checked example (ADR 0498). `error=` blocks must
report the named rule. Blocks that trap, or that describe tool output the
fence can't check, are `norun`; blocks the compiler doesn't implement yet are
`skip=bootstrap` (the fence word keeps its old name).

```ail rules=err.model.1
error struct Bad
    why str

fn f(n int) int ! Bad       # a failure is a returned value, not an exception
    if n == 0
        return Bad(why: "zero")
    n

fn main()
    print(f(1) else 0)      # 1
```

```ail norun rules=err.model.2,err.trap.1,err.trap.3
fn main()
    xs := [1]
    print(xs[5])            # a trap: the program stops with a diagnostic, not a value
```

```ail rules=err.decl.1
error struct Bad
    why str                 # an error struct is an ordinary struct

fn main()
    b := Bad(why: "x")
    print(b.why)            # x
```

```ail error=err.decl.2 rules=err.decl.2
error struct Bad
    why str

fn f(n int) int ! int       # `int` is not an error type; only error types may be errors
    n
```

```ail rules=err.msg.1
error enum Bad
    Empty "no items"        # every variant has a message: the display text
    Full "too many"

fn main()
    print(Bad.Empty)        # no items
```

```ail error=err.msg.2 rules=err.msg.2
error enum Bad
    Empty "no items"
    Full                    # some variants have a message and some don't
```

```ail rules=err.type.1
error struct Bad
    why str

fn parse(s str) int ! Bad   # a value that is an int or a Bad
    if s == ""
        return Bad(why: "empty")
    s.len()

fn main()
    print(parse("abc") else 0)  # 3
```

```ail rules=err.type.2,err.type.6
fn half(n int) int?
    if n % 2 == 0
        return n / 2
    none

fn f(n int) !int        # the error set is inferred from the body: here, none that is named
    x := half(n)?
    x

fn main()
    print(f(4) else 0)      # 2
    print(f(3) else -1)     # -1
```

```ail rules=err.type.3
error struct Bad
    why str

fn parse(s str) int ! Bad   # a public function states its error type explicitly
    if s == ""
        return Bad(why: "empty")
    s.len()

fn main()
    print(parse("ab") else 0)   # 2
```

```ail rules=err.type.4
error struct A
    why str

error struct B
    why str

fn f(n int) !
    if n == 0
        return A(why: "zero")
    if n < 0
        return B(why: "neg")
    print(n)

fn main()
    f(0) else err              # the inferred set is A | B
        match err              # every member must be covered (err.match.3)
            A => print("a: {err.why}")
            B => print("b: {err.why}")
```

```ail rules=err.type.5
error struct A
    why str

error struct B
    why str

fn f(n int) int ! A | B     # the members of an explicit union, in any order
    if n == 0
        return A(why: "zero")
    if n < 0
        return B(why: "neg")
    n

fn main()
    print(f(3) else 0)      # 3
```

```ail rules=err.type.7
error struct E
    why str

fn f(n int) ! E            # no success value: it can only succeed or fail
    if n == 0
        return E(why: "zero")

fn main()
    f(1) else err
        print("failed")
    print("done")           # done
```

```ail rules=err.prop.4
error struct Bad
    why str

fn half(n int) int?
    if n % 2 == 0
        return n / 2
    none

fn f(n int) int
    x := half(n) else 9     # the else of `x := e else d` takes the absent case
    x + 1

fn main()
    print(f(4))             # 3
    print(f(3))             # 10
```

```ail rules=err.prop.5
fn half(n int) int?
    if n % 2 == 0
        return n / 2
    none

fn f(n int) !int           # an inferred error set: `?` on an optional is allowed here
    x := half(n)?
    x

fn main()
    print(f(3) else -1)    # -1
```

```ail rules=err.else.1
error struct Bad
    why str

fn parse(s str) int ! Bad
    if s == ""
        return Bad(why: "empty")
    s.len()

fn main()
    x := parse("abc") else 0   # x is the success value, or 0 if it failed
    print(x)                   # 3
```

```ail rules=err.else.2
error struct Bad
    why str

fn parse(s str) int ! Bad
    if s == ""
        return Bad(why: "empty")
    s.len()

fn main()
    x := parse("") else err
        print("bad {err.why}") # the block leaves the function, so x is defined after it
        return
    print(x)
```

```ail rules=err.else.5
error struct E
    why str

fn f(n int) ! E
    if n == 0
        return E(why: "zero")

fn main()
    f(0) else err
        print("e {err.why}")   # e zero
    print("done")              # done
```

```ail rules=err.else.6
error struct Bad
    why str

fn parse(s str) int ! Bad
    if s == ""
        return Bad(why: "empty")
    s.len()

fn main()
    print(parse("") else -1)   # -1: the default has the success type
```

```ail rules=err.ctx.1,err.ctx.2
error struct Bad
    why str

fn parse(s str) int ! Bad
    if s == ""
        return Bad(why: "empty")
    s.len()

fn load(s str) int ! Bad
    n := parse(s)? "while loading {s}"   # the context message is attached to the error
    n

fn main()
    x := load("") else err
        print(err.why)      # empty
        return
    print(x)
```

```ail tool=run,FILE status=1 expect=context: rules=err.ctx.3
error struct E
    why str

fn read(n int) int ! E
    if n == 0
        return E(why: "empty")
    n

fn main() ! E
    x := read(0)? "reading the count"
    print(x)
```

```ail rules=err.text.1
error enum Kind
    Empty
    Full(n int)

fn main()
    k := Kind.Full(n: 2)
    print("{k}")           # Kind.Full(n: 2): the type name, then the variant
```

```ail rules=err.trap.2
fn boom(n int) int
    unreachable("boom {n}")

fn main()
    t := spawn boom(1)
    r := t.wait() else err
        print("observed a trap")     # observed a trap
        return
    print(r)
```

```ail tool=fix,--dry-run,FILE expect=fail rules=err.fix.1
error struct Bad
    why str

fn load(n int) int ! Bad
    if n == 0
        return Bad(why: "zero")
    n

fn main()
    # The failure isn't handled: `ail fix` makes `main` fail and adds `?`.
    x := load(1)
    print(x)
```

```ail norun rules=err.api.1,err.paths.1
fn main()
    # `ail api` lists each failing function's fails_with; `ail check --errors` prints the paths.
    print("tools")
```
