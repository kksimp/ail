# 03. Grammar

Chapter key: `gram`

> **Status:** Drafted (partial). Function-declaration shape, generic brackets,
> and attributes (`@inline`, `@deprecated`, and those of other chapters) are specified. The full grammar (in a machine-checkable form, per
> ADR 0002) will be written as the syntax decisions are completed.

## Syntax errors

**gram.syntax.1** — Source text that doesn't match the grammar is rejected. The
diagnostic names the construct that was expected, unless a more specific rule
applies. *(ADR 0108)*

**gram.foreign.1** — A construct that other languages have and AIL doesn't
(`//` comments, `def`, `class`, `import`, `try`, `switch`, `x++`, `a ? b : c`,
`<>`, `Color::Red`, `Point{...}`, `new`, braces around blocks, `:` after a
block header, a line-ending `\`, `f"..."`) is reported once, at its first token. The message says what AIL
has instead, and the diagnostic carries a fix when the AIL spelling can be
written without knowing more than the source shows. *(ADR 0262)*

```ail error=gram.foreign.1
fn main()
    mut n := 3
    n++
    print(n)
```

## Recovery and help

**gram.rec.1** — A line with several syntax errors yields one diagnostic, for
the leftmost. The fixes of the errors it absorbs join its own fix when their
edits don't overlap, so one application settles the line. A file indented by
2 or 3 spaces, or with tabs, yields one diagnostic for the file whose fix
re-indents every line. *(ADR 0263)*

**gram.rec.2** — After an error in a declaration, the checker still treats the
name as declared, so later uses don't add reports about it. *(ADR 0263)*

**gram.rec.3** — A "did you mean" suggestion, and the fix that comes with it,
is made only for names of 3 or more characters: with fewer, nearly any name is
one edit away, and applying the fix would change the program's meaning.
*(ADR 0263)*

**gram.help.1** — Every error diagnostic carries a `help` that says what to
write. A rule without a specific help has a default one. *(ADR 0262)*

```ail tool=check,FILE status=2 expect=help: rules=gram.help.1
fn main()
    x := 1
    x = 2
```

**gram.help.2** — `ail fix` resolves a mistake only when the result checks
without errors; a fix that leaves other errors is still a fix, but the
mistakes corpus records which files `ail fix` leaves clean. *(ADR 0262,
[audit](../design/diagnostics-audit.md))*

```ail tool=fix,--dry-run,FILE expect=mut rules=gram.help.2
fn main()
    x := 1
    x = 2
```

## Function declarations

**gram.fn.1** — A function declaration begins with the keyword `fn`, followed
by the function name, a parenthesised parameter list, and optionally a return
type. Its body is the indented block that follows. *(ADR 0042, ADR 0037, ADR 0033)*

**gram.fn.2** — Each parameter is written as a name followed by its type,
separated by whitespace, with no colon. The return type, if present, follows
the closing parenthesis, with no arrow. *(ADR 0037)*

```ail fragment
fn add(a i32, b i32) i32
    a + b
```

> **Note:** there is no parameter grouping (`a, b i32`): every parameter
> names its type. Function types are `fn(A) R` (`type.fn.1`), several results
> are a tuple (`type.tuple.1`), and arguments may be named (`expr.call.3`).

## Lists of items

**gram.list.1** — In argument lists, parameter lists, list literals and struct
literals, a comma MAY follow the last element. The canonical form has one only
when the list spans several lines. *(ADR 0118)*

```ail rules=gram.list.1
fn main()
    xs := [
        1,
        2,
    ]
    print(xs.len())
```

## Generic arguments

**gram.generic.1** — Generic type parameters and type arguments are enclosed
in square brackets `[` `]`. Angle brackets are never used for generics.
*(ADR 0041)*

```ail skip=context
fn load(path str) Result[Map[str, User], DbError]
    ...
```

> **Note:** square brackets also denote indexing. They are told apart by
> whether the operand is a type or generic function, or a value (ADR 0041).

## Attributes

**gram.attr.1** — An attribute is written `@name` or `@name(arguments)`.
*(ADR 0083)*

**gram.attr.2** — Attributes precede the declaration they apply to, on the same
line. The canonical formatter moves them to their own preceding line only when
the line would otherwise exceed the line-length limit. A newline may separate
an attribute from the next attribute or the declaration. *(ADR 0083, ADR 0324)*

```ail fragment
@inline fn hash(x u64) u64
    x * 31
```

**gram.attr.3** — `@inline` is a hint that the function is worth inlining. It
takes no arguments and never changes behaviour; an implementation MAY ignore
it. The bootstrap accepts it and ignores it. *(ADR 0196)*

**gram.attr.4** — `@deprecated("reason")` marks a function as deprecated. The
reason is one string literal and is required. Each call of the function is a
warning that names the function and gives the reason; the program still runs.
The attributes known to the bootstrap are `@test`, `@pure`, `@inline`,
`@deprecated` and `@allow`; any other attribute is an error (`test.attr.1`).
*(ADR 0197, ADR 0379)*

```ail fragment
@deprecated("use hash") fn old_hash(x u64) u64
    x * 17
```

**gram.attr.5** — `@allow("rule")` before a function allows the warning with
that rule ID anywhere in the function: the checker doesn't report it. The
argument is one string literal naming a warning that can be allowed; the
bootstrap allows `expr.match.3` only. A missing or other argument is a
compile-time error. *(ADR 0379)*

```ail fragment
enum Role
    Admin
    Editor
    Guest

@allow("expr.match.3") fn is_admin(r Role) bool
    match r
        Admin => true
        _ => false
```

**gram.attr.6** — `@observe` may precede a field in a struct body, on the
same line as the field: `@observe total int`. It marks the field as readable by
an attached agent (`attach.observe.1`) and changes nothing else. It is the only
attribute a field takes; any other is an error. *(ADR 0884)*

```ail rules=gram.attr.6
struct Cart
    @observe total int
    name str

fn main()
    c := Cart(total: 3, name: "tea")
    print(c.total)
```

## Runnable examples for the rules above

Blocks with `error=` must report their rule, and `rules=` lists the rules a
block demonstrates (ADR 0498). Where a rule describes how the checker
reports, the block shows one reported case; the rest of the rule is checked
by the mistakes corpus (`tests/mistakes/`).

```ail rules=gram.attr.5
enum Role
    Admin
    Editor
    Guest

@allow("expr.match.3") fn is_admin(r Role) bool
    match r
        Admin => true
        _ => false

fn main()
    print(is_admin(Admin))
```

```ail error=gram.attr.5
@allow("type.num.1")
fn f() int
    1

fn main()
    print(f())
```

```ail error=gram.syntax.1
fn main()
    x := 1 +            # an operator with nothing after it doesn't parse
```

```ail error=gram.foreign.1 rules=gram.rec.1
fn main()
    print(&1, &2)       # two foreign operators on one line: one diagnostic, for the first
```

```ail error=expr.name.1 rules=gram.rec.2
fn main()
    print(f(1))         # the name `f` is unknown here, reported once
    print(f(2))         # later uses of the same name add no more reports
```

```ail error=expr.name.1 rules=gram.rec.3
fn main()
    print(pr(1))        # two characters: no "did you mean" is offered
```

```ail error=gram.foreign.1 rules=gram.help.1
fn main()
    x := 1
    print(&x)           # the help says there are no references
```

```ail error=gram.foreign.1 rules=gram.help.2
fn main()
    n := 1
    n++                 # `ail fix` resolves this, and the result checks clean
```

```ail error=gram.fn.2 rules=gram.fn.2
fn add(a: int) int      # a colon between name and type is not allowed
    a
```

```ail rules=gram.fn.1,gram.fn.2
fn add(a int, b int) int    # `fn`, a name, parameters as name then type, a return type with no arrow
    a + b

fn main()
    print(add(1, 2))    # 3
```

```ail error=gram.fn.1 rules=gram.fn.1
fn id<T>(x T) T         # a generic function is written with brackets, and its keyword is still `fn`
    x
```

```ail rules=gram.list.1
fn main()
    xs := [
        1,
        2,              # a trailing comma is allowed when the list spans lines
    ]
    print(xs)           # [1, 2]
```

```ail error=gram.generic.1 rules=gram.generic.1
fn f(xs List<int>) int  # angle brackets are not generics; use `List[int]`
    xs.len()
```

```ail rules=gram.generic.1
fn f(xs List[int]) int
    xs.len()

fn main()
    print(f([1]))       # 1
```

```ail rules=gram.attr.1,gram.attr.2
@deprecated("use sq2")  # `@name(arguments)`, on the line before the declaration
fn sq(x int) int
    x * x

fn main()
    print(sq(3))        # 9, with a deprecation warning
```

```ail rules=gram.attr.3
@inline
fn sq(x int) int
    x * x

fn main()
    print(sq(3))        # 9; @inline never changes the result
```

```ail rules=gram.attr.4
@deprecated("use sq2")
fn sq(x int) int
    x * x

fn main()
    print(sq(3))        # warns: sq is deprecated: use sq2; prints 9
```

```ail error=gram.syntax.1 rules=gram.attr.2
fn main()
    @inline fn_x := 1   # an attribute precedes a declaration, not a statement
```
