# AIL core card

> Non-normative; budget 3,000 tokens (ADR 1110) (`tools/tokbench/run.sh docs/language-card-core.md`).
> More for agents: [docs/for-agents.md](for-agents.md). The [full card](language-card.md) has the rest: `ail card TOPIC` prints one of its
> sections (`ail card errors`), `ail ask std "read a file by lines"` finds a std function or built-in method, `ail ask how "sort descending"` a checked how-to,
> `ail explain RULE` gives the text of any rule a diagnostic cites.

## Loop

```sh
ail check FILE.ail   # diagnostics, each with a rule ID and often a fix; `ail fix` applies them
ail run FILE.ail     # check, then run; `ail test FILE.ail` runs the `@test` functions
ail fmt FILE.ail     # canonical layout
```

## Syntax

```ail skip=context
fn add(a int, b int) int         # `name Type`, no colon; return type after `)`, no arrow
    a + b                        # blocks: exactly 4 spaces; the last expression is the result

x := 1                           # `:=` declares, immutable
mut n := 0                       # `mut` can change; `n = 5`, `n += 1`
if a < b and not c               # `and` `or` `not`; conditions are `bool`
    print("less {a + b}")        # `{expr}` interpolates; `\{` is a literal brace
elif a == b
    return
else
    n += 1
for i in 0..n                    # excludes n; `0..=n` includes it; `for x in xs`
    continue
while n > 0
    n -= 1
```

- No braces, `;`, `//`, `def`, `class`, `import`, `try`, `&&`, `||`, `x++`, `?:`, `f"..."`
  (gram.foreign.1): `#` comments, `##` docs, `use a.b`, `if/else`, `match`.
- Types: `int` `float` `bool` `str` `char` `List[T]` `Map[K, V]` `Set[T]` `T?` `T ! E`.
  `int` and `float` never mix: `n.to_float()`. No `+` on strings: interpolate.
  `type UserId = int` is a distinct type: `UserId(3)` in, `int(id)` out (type.distinct.1).
- Collections: `xs := [1, 2]`, `xs.push(3)`, `xs.len()`, `m := {"a": 1}`, `m["b"] = 2`.
  Read a map with `m.get(k) else 0` (`V?`) or test `m.contains(k)`; `m[k]` traps when `k` is missing.

## Structs, enums, match

```ail skip=context
struct Point
    x int
    y int

enum Shape
    Circle(r int)
    Rect(w int, h int)

fn Point.sum(self) int           # method; `mut self` may change self
    self.x + self.y              # canonical; bare `x + y` works too

fn area(s Shape) int
    match s                      # must cover every variant; `_` covers the rest
        Circle(r) => 3 * r * r
        Rect(w, h) => w * h

p := Point(x: 1, y: 2)           # fields by name, any order
print(p.sum())                   # calls always have `()`
```

Patterns nest in fields, `Add(Num(a), Num(b)) =>`; there a bare name binds, so write a fieldless variant qualified (`Expr.Nil`). Structs match by field name: `Point(x: 0, y) =>`.

## Absence and errors

```ail skip=context
fn parse(s str) int ! ParseError     # an int, or a failure of type ParseError
    n := s.to_int() else return BadNumber(text: s)   # `int?`: handle with `else`
    n

error enum ParseError                # only `error` types can fail
    BadNumber(text str) "bad number {text}"

n := parse("7")? "reading n"         # `?` passes a failure on; or `else`:
m := parse("x") else err
    0
```

A `T ! E` or `T?` must be handled (`?`, `else`, or returned); it is never a plain `T`.
`fn main() !` may fail. `if v := parse("7")` runs its block only on success; `none` is absence.

## Command-line programs

```ail skip=context
fn main()
    args := env.args()                   # the words after the program; the program name is not included
    if args.len() != 1
        io.eprint("usage: count FILE")   # to standard error; adds a newline like `print`
        io.exit(2)                       # ends the program now with status 2
        return                           # `exit` does not end a block for the checker yet
    text := fs.read_text(args[0]) else
        io.eprint("cannot read {args[0]}")
        io.exit(1)
        return
    print(text.lines().len())            # status 0 when `main` returns
```

Standard input: `io.read_all()` gives all of it, `io.read_line()` one line as `str?` (`none` at the end).
`env`, `io`, `fs` need no `use`. `int` overflow traps: sum big values with `use std.bigint` (`ail ask how overflow`).

## Ten mistakes that cost the most

1. An unknown name (expr.name.1): check spelling against `ail api --text FILE`; std modules
   need no `use` (`fs.read_text(p)?`), others do (`use std.csv`).
2. A construct from another language (gram.foreign.1): the message says the AIL spelling.
3. A std function qualified wrongly (std.use.2): `fs.read_text(p)`, not `read_text(p)`;
   look it up with `ail ask std "words"`.
4. Operators between different types (trait.op.3): no implicit conversion, even `int` and `float`.
5. A task handle left unwaited (conc.task.5): `.wait()` every `spawn` before its block ends.
6. Wrong argument count or type (expr.call.1, expr.call.2): one per parameter, in order.
   A parameter may default (`retries int = 3`, expr.call.4): leave it out, or pass it by name only.
7. A swap-prone positional constructor (type.struct.7): name the fields when two share a type.
   A field may default (`retries int = 3`, type.struct.8): a literal may leave it out; the default is a plain constant.
8. `=` on a name that is undeclared or immutable (expr.bind.3): declare with `:=`, or `mut`.
9. A failure left unhandled (err.use.1): `?`, `else`, or return it.
10. A parameter written `name: Type` or an arrow (gram.fn.2): `fn f(name Type) Ret`.
