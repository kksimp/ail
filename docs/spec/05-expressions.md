# 05. Expressions and statements

Chapter key: `expr`

> **Status:** Drafted (partial). Specified: bindings, names, blocks and
> divergence, `if`, loops, ranges, logical and compound operators, comparisons,
> `match` (variant, literal and or-patterns), and integer arithmetic. Not yet
> designed: binding or-patterns and open-ended ranges (see the note at the end of
> the match section). Also specified: guards, range patterns, struct and nested
> patterns, labeled loops, `while name := e`, default parameters, bit operators,
> comprehensions, generators and `select` (see the rule list below).

## Bindings

**expr.bind.1** — `name := expr` declares an immutable local binding. An
immutable binding MUST NOT be reassigned. *(ADR 0038, ADR 0039)*

**expr.bind.2** — `mut name := expr` declares a mutable local binding.
*(ADR 0038, ADR 0039)*

**expr.bind.3** — `name = expr` assigns to an existing mutable binding. Using
`=` with an undeclared name, or with an immutable binding, is a compile-time
error. *(ADR 0039)*

**expr.bind.4** — Declaring a local name that is already visible from the same
or an enclosing local scope of the same function is a compile-time error. This
applies to variables, parameters, and loop and pattern bindings, but not to `_`
(it declares nothing) or the names of `expr.bind.16`. *(ADR 0039, ADR 0044, ADR 1179, ADR 1180)*

```ail skip=context
fn total(items List[Item]) i64
    mut sum := 0
    for item in items
        sum = sum + item.price
    sum
```

**expr.bind.5** — Declaring a local name equal to a visible module-level
type, trait or constant, or a prelude type, is a compile-time error. A local
may have the name of a function (module-level or prelude) or of an imported
module, standard modules included: it hides that name from its declaration to
the end of its block. Its own initializer still sees the function or module
(`qty := qty(s)`, `path := path.stem(f)`). In the block, a call of the name is
`expr.call.1`, and `m.f` where the local `m` hides a module that has an item
`f` is a compile-time error that names the module (`std.use.4` for a standard
module). *(ADR 0084; ADR 0469; ADR 1121)*

**expr.bind.6** — `p = e` and `p op= e` require `p` to be a mutable place
(`own.place.1`), and `e` MUST have the type of `p`. *(ADR 0117)*

**expr.bind.14** — A name bound by `x := e` or `mut x := e` that the function
never reads is a warning; the program still runs. The warning's fix renames the
binding `_`. A name that starts with `_`, a parameter, a loop variable and a
pattern binding are not reported, and only the functions of the program's own
main file are checked. A program that has an error gets no such warning.
*(ADR 1087)*

```ail rules=expr.bind.14
fn main()
    total := 40 + 2     # warns: `total` is bound but never used
    _ := 7              # ignored on purpose
    print("done")
```

**expr.bind.15** — `mut x := m[k]` (or `xs[i]`, or `m.get(k) else d`) binds a
copy of the entry, not the entry (`own.val.1`). When the function assigns to
that copy (`x = e`, `x.f = e`, `x[i] = e`) and never reads it, so the change
is never stored back, it is a warning; the program still runs. Reading the
copy, as in `m[k] = x`, is a read. Changing the entry in place, `m[k].f = e`,
is not reported. A program that has an error gets no such warning.
*(ADR 1134)*

```ail rules=expr.bind.15
struct Count
    n int

fn main()
    m := {"a": Count(n: 1)}
    mut c := m["a"]
    c.n += 1            # warns: `c` is a copy, changed but never stored back
    print(m["a"].n)     # 1
```

## Names

**expr.name.1** — Every name used in an expression MUST refer to a declaration
visible at that point. Otherwise it's a compile-time error. The diagnostic
SHOULD suggest the closest visible names. *(ADR 0107)*

**expr.name.2** — When an unknown function or method is the name another
mainstream language uses for something AIL spells differently, the
diagnostic MUST also give a fixed help that names the AIL form. The
implementation keeps these helps in a table. The table MUST contain at least
the entries below. A declared name is never overridden by the table.
*(ADR 0140, ADR 0107)*

| Habit | Help names |
|---|---|
| `len(x)`, `x.length`, `x.size`, `x.count` | `x.len()` (a `str` counts characters; `s.byte_len()` counts bytes) |
| `str(x)` | `"{x}"` |
| `int(s)` | `s.to_int()` |
| `float(x)` | `s.to_float()` for text, `n.to_float()` for an `int` |
| `ord(c)`, `chr(n)` | `c.code()`, `char.from_code(n)` |
| `abs(x)` | `x.abs()` |
| `sorted(xs)` | `xs.sorted()` |
| `sum(xs)` | `xs.sum()` |
| `min(a, b)`, `max(a, b)`, `min(xs)`, `max(xs)` | `a.min(b)`, `a.max(b)`, `xs.min()`, `xs.max()` |
| `"".join(xs)`, `str.from_chars(xs)` | `xs.join("")` |
| `strip`, `lstrip`, `rstrip` | `trim` |
| `startswith`, `endswith` | `starts_with`, `ends_with` |
| `removeprefix`, `removesuffix` | `strip_prefix`, `strip_suffix` |
| `rjust`, `ljust`, `zfill`, `padStart` | `pad_left`, `pad_right`, with a fill char: `s.pad_left(4, '0')` |
| `splitlines` | `lines` |
| `isdigit`, `is_numeric`; `isalpha`, `is_alphabetic`; `is_whitespace` | the `char` methods `is_digit`; `is_alpha`; `is_space` |
| `contains_key`, `has`, `hasKey` | `contains` |
| `m.get(k, d)` | `v := m.get(k) else d` |
| `items()` | iterate `m.keys()` and index with `m[k]` |
| `append`, `add` on a list | `push` |
| `panic`, `assert` | `unreachable(message)` |
| `loop` | `while true` |

## Calls

**expr.call.1** — A call MUST supply exactly one argument per parameter of the
called function, in order. *(ADR 0108)*

**expr.call.2** — Each argument's type MUST equal the corresponding parameter's
type, or satisfy its constraint for a generic parameter. No implicit conversion
applies, except optional wrapping (`type.opt.5`) and conversion to `any Trait`
(`trait.any.3`). *(ADR 0108, ADR 0129)*

**expr.call.3** — An argument of a call to a user-declared function or
method may be named, `name: value`, where `name` is a parameter. Positional
arguments come first and fill the leading parameters; the named ones follow
and fill the rest in any order. Every parameter without a default (`expr.call.4`) MUST be given exactly once,
so a missing one is `expr.call.1`, and an unknown name, a repeated name, a name
for a parameter already given by position, or a positional argument after a
named one is an error. Arguments are evaluated in parameter order, not source
order. `mut` and `move` go before the value (`f(mut v: xs)`). Built-in
functions and methods don't take named arguments. *(ADR 0454, ADR 1104)*

```ail rules=expr.call.3
fn show(tag str, n int, scale int) str
    "{tag}:{n * scale}"

fn main()
    print(show("a", scale: 3, n: 4))
```

**expr.call.4** — A parameter of a function or method may declare a default,
written `name Type = value`. Parameters with defaults come after those without.
A call may leave a defaulted parameter out, and gives it only by name:
`fetch(u, retries: 5)`; passing it by position is an error that says to name
it. The value follows the rule of a struct field's default (`type.struct.8`):
a plain constant of the parameter's type with no names or calls, evaluated at
each call. A `mut` or `move` parameter has no default, nor has a method of a
`trait` or of an `impl Trait` block; a function value's type has every
parameter, so `fn(str, int, int) Response` accepts only a full call. Adding a
defaulted parameter after the others doesn't break callers (`ail diff`).
*(ADR 1104)*

```ail rules=expr.call.4
fn fetch(url str, retries int = 3, timeout_ms int = 5000) str
    "{url} {retries} {timeout_ms}"

fn main()
    print(fetch("a"))
    print(fetch("b", timeout_ms: 10))
    print(fetch("c", timeout_ms: 1, retries: 2))
```

```ail error=expr.call.4
fn fetch(url str, retries int = 3) str
    "{url} {retries}"

fn main()
    print(fetch("a", 5))     # error: `retries` has a default, so it is passed by name
```

## Blocks

**expr.block.1** — A block is an expression. Its value is the value of its
last expression, or no value if it ends in a statement. *(ADR 0091)*

```ail rules=expr.block.1
fn main()
    v := if true then 1 else 2
    print(v)
```

**expr.block.2** — A function returns the value of its body block. `return e`
exits the function early with value `e`. *(ADR 0091)*

**expr.block.3** — A statement **diverges** when it is `return`, `break` or
`continue`; a call to `unreachable(message)` (`err.model.2`) or to a function
whose result type is `never`, such as `io.exit` (`type.never.2`); a `while true`
(the literal `true`) whose body has no `break` for that loop (`expr.loop.7`);
or an `if`/`else` or `match` all of whose branches diverge. A block whose last
statement diverges produces no value and is accepted wherever a value of any
type is expected, including as a one-line arm (`expr.match.9`) and as an
`else` part. So a function can end with `while true`. The type `never`
(`type.never.1`) is written only as a function's result. *(ADR 0117, ADR 0139,
ADR 1119)*

```ail rules=expr.block.3
fn find(n int) int
    mut i := 0
    while true
        i += 1
        if i > n
            return i

fn main()
    print(find(3))
```

## Conditionals and loops

**expr.if.1** — `if c` is followed by an indented block, then optionally any
number of `elif c` blocks and one `else` block. An `if` is an expression when
it has an `else` and every branch produces a value of the same type.
*(ADR 0091)*

**expr.loop.1** — `for x in e` runs its block once per element of `e`.
`while c` runs its block while `c` is true. *(ADR 0091)*

**expr.loop.2** — `break` exits the innermost loop. `continue` starts its
next iteration. `break name` and `continue name` name an enclosing loop
instead (`expr.loop.10`). *(ADR 0091, ADR 1107)*

**expr.loop.3** — `a..b` is the half-open range from `a` up to but excluding
`b`. `a..=b` includes `b`. A range is written only as a `for` bound, a slice
index `xs[a..b]` or a `match` pattern; anywhere else (`(0..10).step_by(3)`,
`r := 0..n`) it is an error under this rule. *(ADR 0091, ADR 1111)*

**expr.loop.4** — In `for x in e`, `e` MUST be a `List[T]`, giving `x` type
`T`, a `Set[T]`, giving `x` type `T`, or a range of `int`, giving `x` type
`int`. A set is iterated in insertion order (`type.map.1`). `x` is an
immutable binding scoped to the loop body. The loop iterates the value `e` had
when the loop started. `e` MUST NOT be a `Map`: `for k in m` is a
compile-time error, and its help says to iterate `m.keys()` (the habit is
ambiguous between keys and pairs). `e` may also be a `Channel[T]`, giving `x`
type `T`: the loop receives until the channel is closed and empty
(`conc.share.7`), or a value whose type has a `next` method (`expr.iter.1`),
a `Seq[T]` among them. *(ADR 0117, ADR 0131, ADR 0300, ADR 1073)*

**expr.loop.5** — `break` and `continue` outside a loop are compile-time
errors. *(ADR 0117)*

**expr.loop.6** — In `for mut x in e`, `e` MUST be a mutable place
(`own.place.1`) of type `List[T]`. `x` is a mutable place of type `T` that
refers to each element in turn, in order, so changes to `x` change the
element. The body's use of `e` is restricted by `own.excl.5`. `for mut` over
a `Set`, over `m.keys()` or over `m.values()` is a compile-time error: they
are not places. If `e` is a map entry `m[k]` (`own.place.1`), `k` MUST be
present, or the loop traps (`type.map.3`). *(ADR 0121, ADR 0131)*

**expr.loop.7** — A `while` whose condition is the literal `true` and whose
body contains no `break` for that loop never completes normally, so as a
statement it diverges (`expr.block.3`). A `break` inside a nested loop belongs
to that loop, unless it names an outer loop (`expr.loop.10`). AIL has no
`loop` keyword: the help for it says `while true`. *(ADR 0139, ADR 1107)*

**expr.loop.8** — `while name := e` runs its block while `e`, of type `T?` or
`T ! E`, has a value. Each iteration evaluates `e` first. When `e` is present
or succeeded, `name` is bound to its value, immutably and only within that
iteration's block, and the block runs. Otherwise the loop ends, and an error
value is dropped. `break` and `continue` apply to this loop; `continue` evaluates
`e` again. It is `while true` whose block starts with `name := e else break`
(`err.else.6`), so `name` follows `expr.bind.4` and a `while` of this form
that has a value is never a diverging statement (`expr.loop.7`). There is no
`else err` part and no `elif`. The Rust spelling `while let x = e` is a
compile-time error whose fix writes `while x := e`. If `e` is neither optional
nor a result, that is a compile-time error that names the `while` binding. *(ADR 1088)*

**expr.loop.9** — A loop may start with a label: `name: for x in e` or
`name: while c`, on one line, with no space before the `:`. A label is a name
in a namespace of its own: it never clashes with a binding, a function or a
type. It is visible in the loop's header and block and nowhere else, so two
loops one after the other may reuse a label. A label that an enclosing loop of
the same function already has is a compile-time error. A `par for` can't be
labeled. A function literal or a local function does not see the labels
around it. *(ADR 1107)*

**expr.loop.10** — `break name` leaves, and `continue name` continues, the
enclosing loop labeled `name`; both also leave every loop inside it, run the
`defer` statements of the blocks they leave (`own.defer.1`) and end the task
scopes they leave, as `break` does. The statement then diverges
(`expr.block.3`). `name` MUST label an enclosing loop of the same function,
and no `par for` may lie between the statement and that loop
(`conc.par.2`); otherwise it is a compile-time error whose fix writes the
label of the enclosing loop, or labels the outermost enclosing loop `name`
when none is labeled. A `break name` inside a nested loop is a `break` of the
labeled loop for `expr.loop.7`. `break 'name` and `for x in e as name` are not
labels. *(ADR 1107)*

**expr.if.2** — Conditions of `if`, `elif` and `while` MUST have type `bool`,
apart from binding conditions (`expr.if.5`). *(ADR 0110, ADR 0125)*

**expr.if.3** — When an `if` begins inside a statement (e.g. as a binding's
value), its blocks are indented one level deeper than that statement's line,
and its `elif` and `else` lines are at that statement's level. *(ADR 0111)*

```ail skip=context
kind := if n < 0
    "negative"
elif n == 0
    "zero"
else
    "positive"
```

**expr.if.5** — A condition of `if` or `elif` may be a binding `name := e`,
where `e` has type `T?` or `T ! E`. When `e` is present or succeeded, `name`
is bound to its value, immutably and only within that branch, and the branch
runs. Otherwise the next `elif` or `else` is tried. *(ADR 0125)*

**expr.if.6** — When an `if` has exactly one condition, and it binds a value
of type `T ! E`, its `else` may be written `else name`, which binds the error
value to `name` within the `else` block. The type of `name` is given by
`err.else.3`. *(ADR 0125, ADR 0139)*

```ail skip=context
if cfg := parse(lines)
    print("ok: {cfg.width}x{cfg.height}")
else err
    print("error: {err}")
```

**expr.if.8** — `if c then a else b` is the one-line form of an `if` with an
`else`: `a` and `b` are expressions on the same line, and its value is `a`
when `c` holds, else `b`. Any number of `elif d then e` arms may come between
`a` and the `else`: `if c then a elif d then e else b` tries `c`, then `d`, as
`expr.if.1` does. Each condition may be a binding (`expr.if.5`). `then` is a
keyword only right after a condition, so it stays usable as a name. The `else`
belongs to the `if`, not to `a` (`err.else.6`). A one-line `if` without its
final `else`, or an `elif` without `then`, is a compile-time error that names
the missing part. `else if c then ...` is a one-line `if` as the `else` value,
and means the same as `elif`. *(ADR 0280, ADR 1120)*

An `if` used as an operand is written in parentheses: `col + (if raw then 4 else 3)`.
Without them, the one-line form as an operand is `expr.if.8`. *(ADR 0470)*

```ail rules=expr.if.8
fn sign(n int) str
    if n < 0 then "negative" else "non-negative"

fn grade(n int) str
    if n >= 90 then "A" elif n >= 80 then "B" else "C"

fn main()
    print(sign(-2))
    print(grade(85))
    print("{if 3 > 2 then "big" else "small"}")
```

**expr.if.4** — An `if` or `match` whose value is used (bound, passed,
returned, or ending a block whose value is used) MUST produce a value on every
path, so such an `if` MUST have an `else`. All its branches or arms MUST have
the same type, except that branches which differ only in that one is optional
give the optional type (`type.opt.11`). *(ADR 0117, ADR 1169, ADR 1179)*

## Operators

**expr.prec.1** — Operators bind, from tightest to loosest: postfix (call,
field, method, index, `?`); unary `-` and `~`; `* / %`; `+ -`; comparisons
`== != < <= > >=`; `not`; `and`; `or`; ranges `.. ..=`; `else`
(`err.else.6`). Binary operators at the same level associate to the left,
except comparisons, ranges, and `else`, which associates to the right.
*(ADR 0112, ADR 0144)*

A bit operator (`expr.bit.3`) has no place in this order: it never mixes
with another operator without parentheses (`expr.bit.4`).

**expr.prec.2** — Two comparison operators in sequence without parentheses
(e.g. `a < b < c`) are a compile-time error. *(ADR 0112)*

**expr.op.1** — The logical operators are `and` and `or` (both
short-circuiting) and `not`. *(ADR 0092)*

**expr.op.2** — `x op= e` for `op` in `+ - * / %` means `x = x op e`. `x`
MUST be a mutable place. *(ADR 0092)*

**expr.op.3** — Every method call is written with parentheses, including
calls with no arguments. *(ADR 0092)*

**expr.cmp.1** — In `a == b`, `a != b`, `a < b`, `a <= b`, `a > b` and
`a >= b`, both operands have the same type, and the expected type of `b` is
the type of `a` (so `d == East` needs no qualification, `type.enum.3`).
`==` and `!=` need a type implementing `Eq`; the others need `Ord`
(`trait.std.1–2`). On `float` operands all six are the IEEE 754 comparisons
(`type.num.6`): NaN is unequal to everything, itself included, and every
ordering comparison with NaN is false. *(ADR 0129, ADR 0127, ADR 0130)*

**expr.cmp.3** — `==` or `!=` between an optional and a negative integer
literal is a compile-time error. The help says that `index_of` returns `none`
when the element is absent, not `-1`. *(ADR 0136)*

```ail skip=context
i := xs.index_of(x)
if i == -1        # compile-time error: index_of returns none when absent
    print("absent")
```

## Match

**expr.match.1** — A `match` expression is the keyword `match`, the
scrutinee expression, and an indented block containing one arm per line. Each
arm is written `pattern => result`. The result is an expression, or a
one-line statement on the arm's line (`expr.match.9`). A result consisting of
several statements is written as an indented block following `=>`.
*(ADR 0054, ADR 0138)*

```ail skip=context
area := match s
    Circle(r) => pi * r * r
    Rect(w, h) => w * h
    Empty => 0.0
```

**expr.match.2** — A `match` MUST be exhaustive. A non-exhaustive match is a
compile-time error, and the diagnostic MUST list the uncovered cases.
*(ADR 0056)*

**expr.match.3** — When the scrutinee has an enum type and a catch-all arm
`_` covers one or more of its named variants, the implementation MUST warn
and name those variants, unless the function allows it with
`@allow("expr.match.3")` (`gram.attr.5`). *(ADR 0056, ADR 0379)*

**expr.match.4** — A pattern is `_`; a variant of the scrutinee's enum type,
bare or qualified; a literal (`expr.match.7`); or an or-pattern
(`expr.match.8`). A variant with data is followed by a parenthesised pattern
for each of its fields (`expr.match.6`, `expr.match.13`), by patterns for
some fields by name (`expr.match.14`), or written bare to match any fields
(`expr.match.10`). A struct value is matched by a struct pattern
(`expr.match.14`). *(ADR 0117, ADR 0123, ADR 0138, ADR 0183, ADR 1069,
ADR 1106)*

**expr.match.5** — An arm whose pattern is already covered by earlier arms is
a compile-time error. Literals are compared by value (`0x41` after `65` is
redundant), and the rule applies to each alternative of an or-pattern.
*(ADR 0117, ADR 0138)*

**expr.match.6** — In a pattern `Variant(p1, ..., pn)`, there MUST be one
`pi` per field, in declaration order. Each `pi` is a new immutable binding
name, `_`, which ignores the field, or a pattern of the field's type
(`expr.match.13`). *(ADR 0123, ADR 1069)*

**expr.match.7** — A literal pattern is an `int` literal (with an optional
`-`), a `char` literal, a `str` literal without interpolation (a raw string is
allowed), or a `bool` literal. The literal MUST have the scrutinee's type, with
no conversion; when a `char` and a `str` are confused, the help offers the
other quote. A `float` literal pattern is a compile-time error, and its help
says to bind the value and test it in a guard (`x if x == 1.5 =>`). A match on `int`, `str` or `char`
MUST have a `_` arm even when every value is listed; a match on `bool` is
exhaustive with `true` and `false`. A `-` directly before an integer literal
is part of the literal for `lex.num.3`, so `-9223372036854775808` is valid in
patterns, expressions and constants. *(ADR 0138, ADR 0130)*

**expr.match.8** — `p1 | p2 | ...` is an or-pattern. It matches when any
alternative matches. No alternative may bind a name; a variant pattern whose
fields are all `_` is fine. *(ADR 0138)*

**expr.match.9** — The result of an arm may be a one-line statement on the
arm's line: `return x`, `break`, `continue`, `total += n`, `x = y` or
`print(x)`. A statement that diverges (`expr.block.3`) is accepted wherever
the match's type is expected. Any other statement gives the arm no value, so
the match can't be used as a value (`expr.if.4`). *(ADR 0138, ADR 0139)*

```ail skip=context
fn class(c char) int
    match c
        '0' | '1' => 1
        'a' | 'b' | 'c' => 2
        '\n' => return 0
        _ => 3

fn kind(t Token) str
    match t
        Number(_) | Label(_) => "atom"
        Plus => "op"
```

**expr.match.10** — A variant with data written without parentheses, bare or
qualified, matches that variant whatever its fields hold and binds no names.
`Circle | Rect => 0` is the same as `Circle(_) | Rect(_, _) => 0`.
*(ADR 0183)*

```ail rules=expr.match.10
enum Shape
    Circle(r int)
    Rect(w int, h int)
    Dot

fn has_area(s Shape) bool
    match s
        Circle | Rect => true
        Dot => false

fn main()
    print(has_area(Shape.Circle(r: 1)))
```

**expr.match.11** — An arm may carry a guard, `pattern if cond => result`.
`cond` is a `bool` expression checked after the pattern matches, with the
pattern's bindings in scope. When it is `false`, matching goes on with the
next arm. A guarded arm covers no values for exhaustiveness (`expr.match.2`)
or redundancy (`expr.match.5`), so a `match` whose arms are all guarded needs
an unguarded one. A guard on an or-pattern applies to every alternative.
*(ADR 0452)*

**expr.match.12** — A range pattern is `lo..hi` or `lo..=hi`, where `lo` and
`hi` are both `int` literals (with an optional `-`) or both `char` literals.
It matches the values from `lo` up to but not including `hi` (`..`), or up to
and including it (`..=`), as `for` ranges do (`expr.loop.3`). The range MUST
be non-empty, and its type MUST be the scrutinee's. A range covers some
values, never all, so a `match` that uses ranges needs a `_` arm, and an
arm after `_` is still unreachable (`expr.match.5`). A range may be an
alternative of an or-pattern. *(ADR 0453)*

```ail
fn grade(score int) str
    match score
        90..=100 => "A"
        _ if score >= 60 => "pass"
        0..60 => "fail"
        _ => "?"

fn main()
    print(grade(95))
    print(grade(70))
    print(grade(30))
```

**expr.match.13** — A field of a variant pattern may hold a pattern of the
field's type: a variant pattern, nested to any depth, a literal, a range, a
tuple pattern, or an or-pattern. A plain name binds the field and `_` ignores
it; a name that is a variant of the field's enum is an error, so a variant
without fields is written qualified, `Expr.Nil`. A `match` on an enum whose
arms nest is exhaustive (`expr.match.2`) only when every value is taken by an
unguarded arm, and the diagnostic names each missing case as a pattern, such
as `Add(Num(_), Neg(_))`. An arm, or an alternative of an or-pattern, that
matches no value the earlier arms leave is redundant (`expr.match.5`).
*(ADR 1069)*

```ail rules=expr.match.13
enum Expr
    Num(n int)
    Neg(e Expr)
    Add(a Expr, b Expr)
    Nil

fn simplify(e Expr) Expr
    match e
        Add(Num(a), Num(b)) => Expr.Num(n: a + b)
        Add(x, Expr.Nil) => x
        Neg(Neg(x)) => x
        Neg(Num(0)) => Expr.Num(n: 0)
        Num | Neg | Add | Nil => e

fn show(e Expr) str
    match e
        Num(n) => "{n}"
        Neg(x) => "-{show(x)}"
        Add(a, b) => "({show(a)} + {show(b)})"
        Nil => "nil"

fn main()
    print(show(simplify(Expr.Add(a: Expr.Num(n: 2), b: Expr.Num(n: 3)))))    # 5
    print(show(simplify(Expr.Neg(e: Expr.Neg(e: Expr.Num(n: 7))))))         # 7
    print(show(simplify(Expr.Add(a: Expr.Neg(e: Expr.Nil), b: Expr.Nil))))  # -nil
```

**expr.match.14** — A pattern with field names, `Name(f: p, g, ...)`, matches
the fields it names and takes any value in the others. `f: p` matches field
`f` with pattern `p`; a bare `g` binds field `g` to a new name `g`. On a
struct value `Name` is the struct's name, and `Point` or `Point(x, y)` (bare
field names) match every value. On an enum value `Name` is a variant, as in
`Circle(r: 0)`. Each name MUST be a field of `Name`, at most once, and a
struct pattern names its fields: `Point(0, y)` is an error. A `match` on a
struct value, or on an enum whose arms use field names, is checked as
nested patterns are (`expr.match.13`): it is exhaustive when every value is
taken, and the missing cases are named. *(ADR 1106)*

```ail rules=expr.match.14
struct Point
    x int
    y int

enum Shape
    Circle(r int)
    Rect(w int, h int)

fn place(p Point) str
    match p
        Point(x: 0, y: 0) => "origin"
        Point(x: 0, y) => "on the y axis at {y}"
        Point(x, y: 0) => "on the x axis at {x}"
        Point(x, y) => "at {x}, {y}"

fn kind(s Shape) str
    match s
        Circle(r: 0) | Rect(w: 0) | Rect(h: 0) => "empty"
        Circle => "round"
        Rect(w, h) => if w == h then "square" else "oblong"

fn main()
    print(place(Point(x: 0, y: 4)))                    # on the y axis at 4
    print(place(Point(x: 2, y: 3)))                    # at 2, 3
    print(kind(Shape.Rect(w: 0, h: 5)))                # empty
    print(kind(Shape.Rect(w: 2, h: 2)))                # square
```

> **Note:** not yet designed: or-patterns that bind names and open-ended ranges (`10..`). Warnings are
> suppressed with `@allow` (gram.attr.5). *(ADR 0138, ADR 0379, ADR 0452,
> ADR 0453)*

## Comprehensions

**expr.comp.1** — A comprehension builds a list, set or map from one or more
`for` clauses. `[e for x in xs]` is a `List[T]`, `{e for x in xs}` a `Set[T]`
and `{k: v for x in xs}` a `Map[K, V]`, where `T`, `K` and `V` are the types of
`e`, `k` and `v`. A set's elements and a map's keys MUST be `Hash`
(`type.map.2`). A comprehension may span lines inside its brackets.
*(ADR 0182)*

**expr.comp.2** — Each clause is `for name in iterable`, optionally followed
by `if condition`. The iterable is anything a `for` loop accepts
(`expr.loop.3`). The name is a new immutable binding (`expr.bind.16`: it may
hide a local declared outside the comprehension), visible in the condition, in later
clauses and in the element; it ends with the comprehension. `for mut` is a
compile-time error. A clause may be `par for` (`conc.par.12`). *(ADR 0182,
ADR 0302)*

**expr.comp.3** — The clauses run like nested `for` loops, outermost first.
For each combination whose conditions are all `true`, the element (and, in a
map, the value) is evaluated and added: in order for a list, and as by `add`
and `m[k] = v` for a set or map, so a later equal key replaces an earlier
one's value. The element MUST be a value, and when the context expects a
collection of a known element type, it MUST fit that type. *(ADR 0182)*

**expr.comp.4** — The element and conditions are ordinary expressions:
`?`, `return` and `else return` leave the enclosing function. `continue`
skips the current element and `break` ends the comprehension, which then
holds the elements added so far. *(ADR 0182)*

```ail rules=expr.comp.1,expr.comp.2,expr.comp.3,expr.comp.4
fn main()
    xs := [3, 1, 4, 1, 5]
    print([x * x for x in xs if x > 1])          # [9, 16, 25]
    print({x for x in xs}.len())                 # 4
    sizes := {w: w.chars().len() for w in ["a", "bcd"]}
    print(sizes["bcd"])                          # 3
    print([r * 10 + c for r in 1..3 for c in 0..2])   # [10, 11, 20, 21]
    print([t.to_int() else continue for t in ["1", "x", "3"]])  # [1, 3]
```

## Key functions

**expr.key.1** — An argument written `by: |x| e` or `by: |x T| e` is a key
function. It is allowed only as the one argument of a list's `sorted`,
`sort`, `min` and `max`, and of a set's `sorted`; anywhere else, and a `by:`
argument that isn't a literal with one parameter, is a compile-time error.
The error names what is wrong with the argument written: the method, the
receiver's type, the other arguments, the label, the number of parameters, or
the type of a value given instead. A key function isn't a value: it can't be
bound, returned, stored or passed to a user function. Without `by:`, `|x| e`
is a function literal, a value (`type.fn.4`). *(ADR 0250, ADR 1067, ADR 1122)*

**expr.key.2** — `x` is a new immutable binding of the element type
(`expr.bind.16`: it may hide a local declared outside), visible only in
`e`. A type written after it, `|x T|`, MUST be the element type. `e` is an
ordinary expression and sees the enclosing locals. *(ADR 0250, ADR 1122)*

**expr.key.3** — `e` MUST be `Ord`. It is evaluated once per element, in the
collection's order, before any comparison. *(ADR 0250)*

**expr.key.4** — `sorted(by:)` and `sort(by:)` order the elements by their
keys and are stable; `min(by:)` and `max(by:)` return the first element with
the least (greatest) key as a `T?`, or `none` if the list is empty.
*(ADR 0250)*

```ail rules=expr.key.1,expr.key.2,expr.key.3,expr.key.4
struct Person
    name str
    age int

fn main()
    people := [Person(name: "bo", age: 30), Person(name: "al", age: 25)]
    print([p.name for p in people.sorted(by: |p| p.age)])   # ["al", "bo"]
    counts := {"b": 2, "a": 2, "c": 5}
    print(counts.keys().sorted().sorted(by: |k| -counts[k]).take(2))   # ["c", "a"]
    xs := [5, 9, 5]
    print([i for i in 0..3].sorted(by: |i int| (-xs[i], i)))   # [1, 0, 2]
```

## Integer arithmetic

**expr.arith.1** — When the mathematical result of `+`, `-` (binary or
unary) or `*` on fixed-width integer operands can't be represented in the
result type, the operation MUST trap. It MUST NOT wrap silently, and it MUST
NOT have undefined behaviour. *(ADR 0035)*

**expr.arith.2** — The language MUST provide explicit wrapping, saturating
and checked forms of the arithmetic operations. A checked form returns a value
that indicates overflow instead of trapping. *(ADR 0035; syntax to be decided)*

**expr.arith.3** — Integer division or remainder by zero, and signed division
of the minimum value by `-1`, MUST trap. *(ADR 0035)*

**expr.arith.4** — An implementation MAY omit an overflow check that it proves
can never fail. It MUST report the overflow checks that remain in the
performance report. *(ADR 0035)*

**expr.arith.5** — Integer `a / b` rounds toward zero, and `a % b` has the
sign of `a`, so that `a == (a / b) * b + a % b`. `a % -1` is `0` for every
`a`, since that result is representable. *(ADR 0115)*

**expr.bit.1** — An `int` has the methods `bit_and(b)`, `bit_or(b)` and
`bit_xor(b)`, which combine the two 64-bit two's complement values bit by bit,
and `bit_not()`, which flips every bit. `shl(n)` shifts left and discards the
bits that leave the top; `shr(n)` shifts right and copies the sign bit into the
vacated positions, so `(-8).shr(1)` is `-4`. *(ADR 0465)*

**expr.bit.2** — A shift count must be in `0..=63`. A count outside that range
MUST trap. *(ADR 0465)*

```ail fragment
flags := 5.bit_or(8)        # 13
low := flags.bit_and(4)     # 4
hi := (-8).shr(1)           # -4
```

**expr.bit.3** — The operators `&`, `|`, `^`, `<<` and `>>` and the prefix
`~` on `int` are the methods `bit_and`, `bit_or`, `bit_xor`, `shl`, `shr` and
`bit_not`: `a & b` is `a.bit_and(b)`, `~a` is `a.bit_not()`, and so on, with
the same results and the same trap (`expr.bit.2`). The operands of a bit
operator are unary expressions: a name, a literal, a call, `-x`, `~x` or a
parenthesised expression. A `|` where an operand is expected opens a key
function (`expr.key.1`); after an operand it is bit-or; a function body runs to
the end of the expression, so `|x| x | m` is `|x| (x | m)`. *(ADR 1070)*

**expr.bit.4** — A bit operator never meets another operator without
parentheses. A chain repeats one of `&`, `|`, `^` (`a & b & c`); `a | b & c`,
`a << 1 << 2`, `a & b == c`, `a + b & c` and `a & b + c` are compile-time
errors whose help names the parenthesised spelling. No precedence is defined
between bit operators and the others, because the C ranking is a trap and the
ranking of other languages differs. *(ADR 1070)*

**expr.bit.5** — `x op= e` for `op` in `& | ^ << >>` means `x = x op e`, for a
plain name `x`. A place with an index or a field is written out in full. A
trailing `|` does not continue the line (`lex.stmt.2`), since `|` also closes
a key function's parameter. *(ADR 1070)*

```ail rules=expr.bit.3,expr.bit.4,expr.bit.5
fn main()
    flags := 5 | 8              # 13
    low := flags & 4            # 4
    hi := -8 >> 1               # -4
    mask := ~0 & 255            # 255
    is_odd := (flags & 1) == 1  # parentheses are required here
    mut h := 1
    h ^= h << 3
    h <<= 1
    print("{flags} {low} {hi} {mask} {is_odd} {h}")
```

```ail error=expr.bit.4
fn main()
    flags := 5
    print(flags & 1 == 1)       # error: `&` and `==` need parentheses
```

```ail error=expr.bit.4
fn main()
    print(1 | 2 & 3)            # error: `|` and `&` need parentheses
```

> **Note:** "trap" is defined in the [glossary](../glossary.md). What a trap
> does (whether it stops the task or the program, and how it is reported) is
> decided in the error-handling chapter.

## Holes

**expr.hole.1** — `todo` is a *typed hole*: an expression that stands for code
not yet written. It is accepted wherever an expression is. Its type is the type
the context expects; where no type is expected, its type is unknown and every
use of the value is accepted. Each hole MUST be reported as a warning, not an
error, whose message states the expected type (or that none is known) and lists
the local names in scope with their types, sorted by name. *(ADR 0147)*

```ail rules=expr.hole.1
fn area(w int, h int) int
    todo

fn main()
    print("started")
```

The checker warns "hole: expected `int`; in scope: `h int`, `w int`", and the
program still runs: `area` is never called.

**expr.hole.2** — Evaluating a hole MUST trap (`err.trap.1`). *(ADR 0147)*

**expr.hole.3** — `uhwhatnow?` is the hole that asks: one keyword token
(`lex.kw.1`) with the type and trap of `todo` (`expr.hole.1–2`). Its warning
MUST rank what in scope fits the expected type, best first: locals, innermost
first; then fields of `self`, written bare; then fields of other locals; then
calls of the local's methods and of the module's own non-generic functions,
each argument filled by the local named like the parameter, else the innermost
local of its type, else `todo`. A call ranks lower for each argument left as
`todo`, for a result that can fail (written `?` in a function that returns a
`Result`, else `else todo`), and when it calls the function being checked. A
value fits when its type is the expected type, or the expected type is `T?` and
the value is a `T`. The warning lists at most eight candidates, each with a fix
that writes it in place of the hole. Where no type is expected, or nothing
fits, it says so and lists the locals as `todo` does. The trap names the
`uhwhatnow?`. *(ADR 0872)*

```ail norun rules=expr.hole.3
struct Cart
    total int

fn price(c Cart, tax int) int
    uhwhatnow?

fn main()
    print(price(Cart(total: 3), 1))   # traps: reached a `uhwhatnow?` hole
```

The checker warns "uhwhatnow?: expected `int`; best fit: `tax`", with
`tax`, then `c.total`, then `price(c, tax)` listed below it.

> **Note:** a hole lets a program be written top-down: the checker says what
> each gap needs, and the program runs until it reaches one. The compiler
> doesn't infer an unknown hole type from later uses; a later lane may.

## Generators and iteration

**expr.iter.1** — In `for x in e` and in a comprehension clause, `e` MAY be a
value whose type has a method `fn next(mut self) T?`; `x` then has type `T`,
and the loop calls `next()` until it gives `none`. The loop works on a copy of
`e` (`expr.loop.4`), or, when `e` is unique, takes it over (`own.uniq.2`), so
a later use of it is an error. A `next` with another receiver, any parameter,
or a result that isn't optional is a compile-time error. *(ADR 1073)*

**expr.iter.2** — `Seq[T]` is the prelude's `unique struct Seq[T]` with one
field, `step fn() T?`; `s.next()` calls `step`, and `Seq(step: f)` makes a
sequence from any function value (`type.fn.1`). Reading a `Seq` uses it up:
it is never copied (`own.uniq.1`). *(ADR 1073)*

**expr.iter.3** — The prelude gives `Seq[T]` these functions. Lazy, each
taking its sequence over and reading only as far as its result is read:
`Seq.of(xs)` (the elements of a list), `s.map(f)`, `s.filter(keep)`,
`s.take_while(keep)`, `s.take(n)`, `s.skip(n)`, `s.enumerate()` (pairs
`(i, x)` from 0), `s.zip(move other)` (pairs, until either ends) and
`s.chain(move other)`. Reading to the end: `s.collect()` (a `List[T]`) and
`s.count(keep)`, how many elements `keep` is true for. *(ADR 1073, ADR 1099)*

**expr.iter.4** — The prelude also gives `Seq[T]` `s.any(keep)`, `s.all(keep)`,
`s.find(keep)`, `s.fold(init, f)`, `s.flat_map(f)` (a `Seq[U]`) and
`s.skip_while(keep)`, with the meaning of `type.coll.22-31`. Each uses the
sequence up. *(ADR 1099)*

**expr.yield.1** — A function whose body contains a `yield` statement is a
*generator*; its result type MUST be `Seq[T]`. Calling a generator runs none
of its body: it returns a `Seq[T]`, and each `next()` of it runs the body from
where it last stopped to its next `yield`. `yield` outside a generator,
is a compile-time error, and so is `yield` inside a function literal
(`type.fn.4`). *(ADR 1073)*

**expr.yield.2** — `yield e` is a statement; `e` MUST have type `T`, and the
element is a copy (`own.val.1`): changes the generator makes later to the
variable are not seen in it. A unique `e` moves out. *(ADR 1073)*

**expr.yield.3** — The sequence ends when the body ends or runs a bare
`return`; every `next()` from then on gives `none`. `return e` in a generator
is a compile-time error. *(ADR 1073)*

**expr.yield.4** — A generator keeps its own copy of each argument, or takes
a `move` argument over, so the caller's later changes are not seen. A
generator MUST NOT have a `mut` parameter or `mut self`, `requires` or
`ensures`, or a `defer`, and `yield` MUST NOT be inside `par for` or `with`.
*(ADR 1073)*

```ail rules=expr.yield.1,expr.yield.2,expr.yield.3,expr.iter.1,expr.iter.2,expr.iter.3,expr.iter.4
fn count(n int) Seq[int]
    for i in 0..n
        yield i                   # the next element

fn evens(limit int) Seq[int]
    mut k := 0
    while true
        if k > limit
            return                # ends the sequence
        yield k
        k += 2

struct Countdown
    n int

fn Countdown.next(mut self) int?
    if n == 0
        return none
    n -= 1
    n + 1

fn main()
    for i in count(3)
        print(i)                  # 0 1 2
    print(evens(6).collect())     # [0, 2, 4, 6]
    print(count(10).filter(|x| x % 3 == 0).take(3).collect())   # [0, 3, 6]
    mut s := count(1)
    print(s.next() else -1)       # 0
    print(s.next() else -1)       # -1: ended
    c := Countdown(n: 2)
    for k in c
        print(k)                  # 2 1
    print(c.n)                    # 2: the loop read a copy
    print(count(5).fold(0, |acc, x| acc + x))   # 10
    print(count(5).count(|x| x > 1))            # 3
    print(count(5).skip_while(|x| x < 3).collect())   # [3, 4]
```

```ail error=expr.yield.1
fn main()
    yield 1                       # error: `main` doesn't return `Seq[T]`
```

```ail error=expr.yield.4
fn grow(mut xs List[int]) Seq[int]   # error: a generator can't take `mut`
    for x in xs
        yield x

fn main()
    mut a := [1]
    for x in grow(mut a)
        print(x)
```

```ail error=expr.iter.1
struct Bad
    n int

fn Bad.next(self) int?            # error at the loop: `next` needs `mut self`
    none

fn main()
    for x in Bad(n: 1)
        print(x)
```

## Examples

Conformance examples (ADR 0002) for the rules above. Each `rules=` list names
what its program demonstrates; `error=` blocks must be rejected with that
rule, and `norun` blocks are checked but not run because they trap on purpose.

```ail rules=expr.bind.1,expr.bind.2,expr.bind.3
fn main()
    x := 1                    # immutable: never reassigned
    mut y := 2                # mutable
    y = y + x                 # `=` assigns to an existing mut binding
    print(y)                  # 3
```

```ail error=expr.bind.3
fn main()
    x := 1
    x = 2                     # error: x is immutable
    print(x)
```

```ail rules=expr.bind.4
fn main()
    x := 1
    if x > 0
        y := x + 1            # a new name in an inner scope is fine when unused outside
        print(y)
    z := 2                    # sibling blocks may reuse names; this is a new name
    print(z)
```

**expr.bind.16** — A parameter of a function literal (`|x| e`, `type.fn.4`), the
parameter of a key function (`expr.key.2`) and a comprehension's variables
(`expr.comp.2`) MAY reuse the name of a local that is visible where the literal
or comprehension is written. The new name hides that local from its
declaration to the end of the literal's body or of the comprehension; after it
the name is the outer local again, unchanged. Two parameters of one literal, or
two variables of one comprehension, MUST NOT share a name (`expr.bind.4`), and
a local whose type is still open (`type.coll.13`, `type.opt.9`) can't be
hidden. A statement's name (`:=`, a `for` or `match` binding, a parameter of a
local `fn`) never hides a local. *(ADR 1180)*

```ail rules=expr.bind.16
fn main()
    x := 10
    xs := [1, 2, 3]
    print(xs.map(|x| x * 2))          # [2, 4, 6]: the parameter hides the local
    print([x + 1 for x in xs])        # [2, 3, 4]
    print(x)                          # 10
```

```ail error=expr.bind.5
struct Point
    x int

fn main()
    Point := 3                # error: a local may not take a type's name
    print(Point)
```

```ail rules=expr.bind.5
use std.path

fn helper() int
    4

fn main()
    helper := helper() + 1    # the initializer still calls the function
    print(helper)             # 5
    path := path.stem("notes.txt")  # a local may hide an imported module
    print(path)               # notes
```

```ail rules=expr.bind.6
fn main()
    mut total := 0
    total = total + 2
    total += 3                # p op= e: p is a mutable place of type int
    print(total)              # 5
```

```ail error=expr.bind.6
fn main()
    mut total := 0
    total = "two"             # error: the value must have the type of total
    print(total)
```

```ail error=expr.name.1
fn main()
    total := 1
    print(totl)               # error: no visible name `totl`
```

```ail error=expr.name.1 rules=expr.name.2
fn main()
    xs := [1, 2]
    print(len(xs))            # error: the help names xs.len()
```

```ail rules=expr.call.1,expr.call.2
fn scale(n int, by int) int
    n * by

fn main()
    print(scale(4, 3))        # 12: one argument per parameter, each of its type
```

```ail error=expr.call.1
fn scale(n int, by int) int
    n * by

fn main()
    print(scale(4))           # error: one argument per parameter
```

```ail error=expr.call.2
fn scale(n int, by int) int
    n * by

fn main()
    print(scale(4, "3"))      # error: the argument is str, not int
```

```ail rules=expr.block.1,expr.block.2,expr.block.3
fn sign(n int) int
    if n < 0
        return -1             # return leaves the function early
    1                         # the body's last expression is the result

fn tail(start int) int
    mut n := start
    while true                # diverges: the function ends here
        if n > 0
            return n
        n = n + 1

fn main()
    y := sign(-5)
    print(y)                  # -1
    print(tail(3))            # 3
```

```ail rules=expr.if.1,expr.if.2,expr.if.3,expr.if.4
fn grade(n int) str
    kind := if n >= 90
        "A"
    elif n >= 50
        "C"
    else
        "F"
    kind

fn main()
    print(grade(95))          # A
    print(grade(10))          # F
```

```ail error=expr.if.2
fn main()
    n := 3
    if n                      # error: the condition must be bool
        print(n)
```

```ail error=expr.if.4
fn main()
    n := 3
    kind := if n > 2          # error: a value-producing if needs an else
        "big"
    print(kind)
```

```ail rules=expr.if.5,expr.if.6
fn find(id int) int?
    if id > 0
        return id * 10
    none

error enum ParseError
    BadNumber(text str) "bad number {text}"

fn parse(s str) int ! ParseError
    n := s.to_int() else return BadNumber(text: s)
    n

fn main()
    if v := find(2)
        print(v)              # 20
    else
        print("missing")
    if w := parse("x")
        print(w)
    else err
        print(err)            # bad: x
```

```ail rules=expr.loop.1,expr.loop.2,expr.loop.3,expr.loop.4
fn main()
    mut sum := 0
    for x in [1, 2, 3, 4, 5]
        if x == 2
            continue          # next iteration
        if x == 5
            break             # leaves the innermost loop
        sum += x
    print(sum)                # 1 + 3 + 4 = 8
    for i in 0..3
        print(i)              # 0 1 2
    for i in 0..=2
        print(i)              # 0 1 2
    mut n := 0
    while n < 3
        n += 1
    print(n)                  # 3
    for s in {"a", "b"}
        print(s)              # a b: insertion order
```

```ail rules=expr.loop.8
fn main()
    mut stack := [1, 2, 3]
    while top := stack.pop()
        print(top)            # 3 2 1
    print(stack.len())        # 0
```

```ail error=expr.loop.8
fn main()
    mut stack := [1, 2]
    while let top = stack.pop()   # error: write `while top := stack.pop()`
        print(top)
```

```ail error=type.map.5
fn main()
    m := {"a": 1}
    for k in m               # error: iterate m.keys()
        print(k)
```

```ail error=expr.loop.5
fn main()
    break                     # error: no enclosing loop
```

```ail rules=expr.loop.6
fn bump(mut xs List[int])
    for mut x in xs
        x += 1                # each element is a mutable place
    print(xs)

fn main()
    mut nums := [1, 2, 3]
    for mut x in nums
        x = x * 10
    print(nums)               # [10, 20, 30]
    bump(mut nums)
```

```ail error=expr.loop.6
fn main()
    for mut x in [1, 2, 3]    # error: a list literal is not a mutable place
        x += 1
```

```ail rules=expr.loop.7
fn find_first(xs List[int], target int) int
    mut i := 0
    while true
        if xs[i] == target
            break
        i += 1
    i

fn main()
    print(find_first([4, 5, 6], 6))   # 2
```

```ail rules=expr.loop.9,expr.loop.10
fn first_pair(grid List[List[int]], want int) int
    rows: for r in 0..grid.len()
        for c in 0..grid[r].len()
            if grid[r][c] == want
                return r * 10 + c
    -1

fn main()
    mut found := 0
    outer: for i in 0..5
        for j in 0..5
            if i * j == 6
                found = i * 10 + j
                break outer           # leaves both loops
    print(found)                      # 23
    mut sum := 0
    rows: for i in 0..4
        for j in 0..4
            if j > i
                continue rows         # the next iteration of the outer loop
            sum += 1
    print(sum)                        # 10
    print(first_pair([[1, 2], [3, 4]], 4))   # 11
```

```ail error=expr.loop.10
fn main()
    for i in 0..3
        for j in 0..3
            if i + j == 2
                break outer           # error: no loop is labeled `outer`
```

```ail error=expr.loop.9
fn main()
    outer: for i in 0..3
        outer: for j in 0..3          # error: an enclosing loop is already `outer`
            print(i + j)
```

```ail rules=expr.prec.1,expr.op.1,expr.op.2,expr.op.3
fn main()
    print(2 + 3 * 4)                  # 14
    print(-2 * 3)                     # -6: unary minus binds tighter
    print(not false and true)         # true
    print(false or true and false)    # false: and binds tighter than or
    mut k := 1
    k += 4
    print(k.abs())                    # 5: a method call always has ()
```

```ail error=expr.prec.2
fn main()
    print(1 < 2 < 3)                  # error: two comparisons in sequence
```

```ail error=expr.op.3
fn main()
    xs := [1, 2]
    print(xs.len)                     # error: the method call needs ()
```

```ail rules=expr.cmp.1
enum Dir
    North
    East

fn main()
    d := Dir.East
    print(d == East)                  # true: the expected type qualifies East
    print(2.0 < 3.0)
    print("a" <= "b")
```

```ail error=expr.cmp.3
fn main()
    xs := [4, 5]
    i := xs.index_of(9)
    if i == -1                        # error: index_of gives none when absent
        print("absent")
```

```ail rules=expr.match.1,expr.match.2,expr.match.4,expr.match.6
enum Shape
    Circle(r int)
    Rect(w int, h int)
    Empty

fn area(s Shape) int
    match s
        Circle(r) => 3 * r * r
        Rect(w, _) => w * w       # `_` ignores a field
        Empty => 0

fn main()
    print(area(Shape.Circle(r: 2)))   # 12
    print(area(Empty))                # 0
```

```ail error=expr.match.2
enum Dir
    North
    East
    South

fn name(d Dir) str
    match d                      # error: South is not covered
        North => "n"
        East => "e"
```

```ail rules=expr.match.3
enum Dir
    North
    East
    South

fn name(d Dir) str
    match d
        North => "n"
        _ => "other"              # warns: the catch-all covers East and South

fn main()
    print(name(Dir.South))
```

```ail error=expr.match.5
fn main()
    n := 65
    match n
        65 => print("A")
        65 => print("again")     # error: already covered by the arm above
        _ => print("other")
```

```ail error=expr.match.6
enum Shape
    Rect(w int, h int)

fn main()
    s := Shape.Rect(w: 1, h: 2)
    match s
        Rect(w) => print(w)       # error: one pattern per field
```

```ail rules=expr.match.7,expr.match.8,expr.match.9
fn kind(c char) int
    match c
        '0' | '1' => 1
        'a' | 'b' | 'c' => 2
        '\n' => return 0
        _ => 3

fn main()
    print(kind('1'))          # 1
    print(kind('b'))          # 2
    print(kind('\n'))         # 0
    print(kind('z'))          # 3
```

```ail error=expr.match.7
fn main()
    x := 1.5
    match x                   # error: a float literal pattern
        1.5 => print("x")
        _ => print("other")
```

```ail rules=expr.match.10
enum Token
    Number(v int)
    Plus
    Label(s str)

fn kind(t Token) str
    match t
        Number | Label => "atom"   # bare: any fields, binds nothing
        Plus => "op"

fn main()
    print(kind(Token.Number(v: 4)))   # atom
    print(kind(Plus))                 # op
```

```ail rules=expr.comp.2,expr.comp.3
fn main()
    pairs := [[1, 2], [3, 4]]
    flat := [x for row in pairs for x in row if x % 2 == 0]
    print(flat)                       # [2, 4]
```

```ail error=expr.if.2 rules=expr.comp.2
fn main()
    xs := [1, 2]
    ys := [x * 2 for x in xs if x]    # error: the condition must be bool
    print(ys)
```

```ail rules=expr.arith.1,expr.arith.2,expr.arith.3,expr.arith.4
fn main()
    big := 9223372036854775807
    print(big.checked_add(1))         # none: the checked form reports overflow
    print(big.wrapping_add(1))        # explicit wrapping
    print(big.saturating_add(1))      # explicit saturation
    print(3 + 4)                      # the proven-safe add needs no check at run time
```

```ail norun rules=expr.arith.1
fn main()
    big := 9223372036854775807
    print(big + 1)                    # traps: the result doesn't fit in int
```

```ail norun rules=expr.arith.3
fn main()
    zero := 0
    print(10 / zero)                  # traps: division by zero
```

```ail rules=expr.arith.5
fn main()
    print(-7 / 2)                     # -3: rounds toward zero
    print(-7 % 2)                     # -1: the sign of a
    print(7 % -1)                     # 0 for every a
    a := -7
    b := 2
    print(a == (a / b) * b + a % b)   # true
```

```ail rules=expr.hole.1
fn area(w int, h int) int
    todo

fn main()
    print("started")
```

```ail norun rules=expr.hole.2
fn area(w int, h int) int
    todo

fn main()
    print(area(2, 3))                 # traps: evaluating the hole
```

```ail rules=expr.match.11,expr.match.12
fn sign(n int) str
    match n
        _ if n < 0 => "negative"
        0 => "zero"
        _ => "positive"

fn grade(c char) str
    match c
        'a'..='f' => "low"
        'g'..'z' => "high"
        _ => "other"

fn main()
    print(sign(-3))                   # negative
    print(sign(0))                    # zero
    print(grade('c'))                 # low
    print(grade('x'))                 # high
```
