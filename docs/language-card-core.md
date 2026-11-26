# AIL core card

> Non-normative; budget 3,000 tokens (ADR 1110) (`tools/tokbench/run.sh docs/language-card-core.md`).
> More: [for-agents.md](for-agents.md). The [full card](language-card.md) has the rest: `ail card errors` prints a section,
> `ail ask std "read a file by lines"` finds a std function or method, `ail ask how "sort descending"` a checked how-to, `ail explain RULE` a rule's text.

## Loop

```sh
ail check FILE.ail   # diagnostics, each with a rule ID; `fix: write CODE` is the corrected line; `ail fix` applies them
ail run FILE.ail     # check, then run; `ail test FILE.ail` runs the `@test` functions
ail fmt FILE.ail     # canonical layout
```

## A whole program

```ail skip=context
fn usage(why str) never                  # `never`: every call ends the program (type.never.1)
    io.eprint("usage: total FILE ({why})")   # to standard error; adds a newline
    io.exit(2)                           # ends now with status 2

fn main()
    args := env.args()                   # the words after the program, not its name
    if args.len() != 1
        usage("one argument")
    text := fs.read_text(args[0]) else usage("cannot read {args[0]}")   # a `never` call fits any value
    mut total := 0
    for line in text.lines()
        n := line.trim().to_int() else 0
        total += n
    print(total)                         # status 0 when `main` returns
```

Standard input: `io.read_all()`, or `io.read_line()` for one line (`str?`, `none` at the end). No standard module needs `use`.

## Syntax

```ail skip=context
fn add(a int, b int) int         # `name Type`, no colon; return type after `)`, no arrow
    a + b                        # blocks: exactly 4 spaces; the last expression is the result

x := 1                           # `:=` declares, immutable
mut n := 0                       # `mut` can change; `n = 5`, `n += 1`
if a < b and not c               # `and` `or` `not`; conditions are `bool`
    print("less {a + b}")        # `{expr}` interpolates
elif a == b
    return
else
    n += 1
for i in 0..n                    # excludes n; `0..=n` includes it; `for x in xs`
    continue
while n > 0
    n -= 1
```

- Not AIL (gram.foreign.1): braces, `;`, `//`, `def`, `class`, `import`, `try`, `x++`, `f"..."`. Use `#` comments, `##` docs, `use a.b`, `if/else`, `match`.
- Literal `{` in a string: `\{`; `}` is plain: `"\{\"id\": {n}}"`.
- Types: `int` `float` `bool` `str` `char` `List[T]` `Map[K, V]` `Set[T]` `T?` `T ! E`.
  `int` and `float` never mix: `n.to_float()`. No `+` on strings: interpolate.
- A local has no type annotation; an empty value is typed by its first use (`mut out := []`, `mut best := none`)
  or names its type: `List[int]()`, `Map[str, int]()`.

## Everyday calls

- Text: `s.split(",")`, `s.lines()`, `s.trim()`, `s.chars()`, `s.len()`, `s.slice(a, b)`, `s.contains(x)`, `s.starts_with(x)`,
  `s.to_int()` (`int?`), `xs.join(", ")`, formats `"{n:>6}|{x:.2}|{n:05}"`. No indexing: `s.char_at(i)` (`char?`).
- Lists: `xs := [1, 2]`, `xs.push(3)`, `xs.pop()` (`T?`), `xs[i]`, `xs[a..b]` or `xs.slice(a, b)`, `xs.len()`, `xs.contains(x)`,
  `xs.sort()` (in place), `xs.sorted()`, `xs.sorted(by: |p| -p.age)`, `xs.reversed()`, `xs.sum()`,
  `[x * x for x in xs if x > 0]`, `for i, x in xs.enumerate()`.
- Maps keep insertion order: `m := {"a": 1}`, `m[k] = v`, `m[k] += 1` (a missing key starts at 0), `m.get(k) else 0`,
  `m.contains(k)`, `m.remove(k)`, `m.keys()`, `for k, v in m`.
- Sets: `Set[str]()`, `s.add(x)` (true if new), `s.contains(x)`, `s.sorted()`, `s.to_list()`, `for x in s`.
- Anything else: `ail ask std "words"` finds it; `ail ask std pad_left is_alpha` prints each name's rule text and examples.

## Habits that do not compile

```ail skip=context
mut m := {"a": 1}
m["b"] = 2                         # not `m.set`, `m.insert`: an entry is set by assignment
n := m.get("z") else 0             # `m[k]` is a plain value and traps on a missing key
found := m.get("z") != none        # test an optional with `== none` / `!= none`; there is no `is some`
if m.contains("a")                 # not `"a" in m`
    print("found")
line := "{n}:{m}"                  # no `+=` on strings: push the pieces to a list, then `join`
idx := [i for i in 0..3]           # a range is only a `for` bound, a slice index or a pattern
```

`exit`, `os.exit`, `sys.exit` are `io.exit(code)`; `eprintln` is `io.eprint(value)`; `argv` is `env.args()`; `null` is `none`;
`panic` is `unreachable("why")`. A range has no `rev` or step: count down with `while`. No `+` on lists: `xs.concat(ys)`.
`ail check`, `build`, `run` and `test` rewrite these in your file, printing `fixed PATH:LINE:COL: [RULE] ...` (fix.auto.1);
write the form after `->` yourself, and re-read the file after a rewrite:
`&&` `||` `!x` -> `and` `or` `not x`; `else if` -> `elif`; `let x = 1` -> `x := 1`; `var x = 1` -> `mut x := 1`; `println` -> `print`;
`x.to_string()` -> `x.to_str()`; `"a" + b` -> `"a{b}"`; `xs + ys` -> `xs.concat(ys)`; `max(a, b)` -> `a.max(b)`;
`m[k] else d` -> `m.get(k) else d`; `o.unwrap_or(d)` -> `o else d`; `out mut T` -> `mut out T`; a missing `mut` is added.

## Compiles, but is not what you meant

No diagnostic and no fix reaches these; check them yourself.

- `it := m[k]` copies the entry: changing `it` leaves `m` alone. Write `m[k].qty += 1`, or store it back, `m[k] = it`.
- `for x in xs` gives copies; `for mut x in xs` changes the elements. A function changes its caller's value only
  through a `mut` parameter, `fn f(mut p T)`, called `f(mut q)`.
- `s.lines()` and `io.read_line()` drop a `\r` before the line end; to see it, split with `s.split("\n")`.
- `int` overflow and division by zero trap; `/` rounds toward zero (`-7 / 2` is `-3`), `%` takes the left side's sign.
- A whole `float` prints as `2.0`; `x.round()` is a `float`, `x.to_int()` cuts toward zero.
- Padding never cuts: `s.pad_left(4, ' ')` and `"{s:>4}"` leave a longer `s` whole.
- `c.is_alpha()` follows Unicode, `c.is_digit()` is ASCII; a digit's value is `c.to_digit(10)` (`int?`), not `c.code()`.
- `" 7".to_int()` and `"+7".to_int()` are `none`: an optional `-` and digits only, so trim first.
- Strings compare by bytes: `"B" < "a"`. `s.len()` counts characters; `s.byte_len()` counts UTF-8 bytes.
- `else` binds loosest: `a else 0 + 1` is `a else (0 + 1)`; write `(a else 0) + 1`.

## Structs, enums, match

```ail skip=context
struct Point
    x int
    y int

enum Shape
    Circle(r int)
    Rect(w int, h int)

fn Point.sum(self) int           # method; `mut self` may change self
    self.x + self.y

fn area(s Shape) int
    match s                      # must cover every variant; `_` covers the rest
        Circle(r) => 3 * r * r
        Rect(w, h) => w * h

p := Point(x: 1, y: 2)           # fields by name, any order
print(p.sum())                   # calls always have `()`
```

In a nested pattern, `Add(Num(a), Num(b)) =>`, a bare name binds: write a fieldless variant qualified (`Expr.Nil`).

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
`fn main() !` may fail. `T ! str` fails with `return TEXT`; with no result or a `str` result: `fn f() ! Fail`, `return Fail(msg: "bad")`.
An optional has no `map` and no methods of its value: `v := o else d`, `if v := o`, `(o else "").len()`.
`x?` needs a function that itself returns `T?` or can fail; elsewhere write `else`. Inside `{ }` in a string, quotes are plain: `"{xs.join(", ")}"`.

## Modules

A module is a directory, not a file: `use store` in `src/main.ail` loads every `.ail` file directly in `src/store/`
(`src/store/store.ail`; a file `src/store.ail` is not a module). `use` lines come first.

```ail skip=context
pub struct Item              # in src/store/store.ail; `pub` lets other modules name it
    pub sku str              # each field used outside needs `pub` too
    pub qty int
```

Outside, names are always qualified: `store.Item`, `store.load(text)`.

## Mistakes that cost the most

1. An unknown name (expr.name.1) or a std function qualified wrongly (std.use.2): `fs.read_text(p)`, not
   `read_text(p)`; look it up, never guess.
2. A failure left unhandled (err.use.1): `?`, `else`, or return it. `else` handles only `T?` and `T ! E` (err.else.1);
   `E` is an `error` type or `str` (err.str.1).
3. A private name used from another module (mod.vis.1): mark the declaration and each field `pub`.
4. Wrong argument count or type (expr.call.1, expr.call.2, trait.op.3): one argument per parameter, in order; no implicit conversion.
