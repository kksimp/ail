# AIL core card

> Non-normative; budget 3,000 tokens (ADR 1110) (`tools/tokbench/run.sh docs/language-card-core.md`).
> More for agents: [docs/for-agents.md](for-agents.md). The [full card](language-card.md) has the rest: `ail card TOPIC` prints one of its
> sections (`ail card errors`), `ail ask std "read a file by lines"` finds a std function or built-in method, `ail ask how "sort descending"` a checked how-to,
> `ail explain RULE` gives the text of any rule a diagnostic cites.

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
    io.exit(2)                           # ends now with status 2 (0 to 255)

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

- Not AIL (gram.foreign.1): braces, `;`, `//`, `def`, `class`, `import`, `try`, `x++`, `f"..."`. Use `#` comments, `##` docs, `use a.b`, `if/else`, `match`.
- Literal `{` in a string: `\{`; `}` is plain: `"\{\"id\": {n}}"`.
- Types: `int` `float` `bool` `str` `char` `List[T]` `Map[K, V]` `Set[T]` `T?` `T ! E`.
  `int` and `float` never mix: `n.to_float()`. No `+` on strings: interpolate.
  `type UserId = int` is a distinct type: `UserId(3)` in, `int(id)` out (type.distinct.1).
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
- Anything else: `ail ask std "words"`.

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

`exit`, `os.exit`, `sys.exit` are `io.exit(code)`; `eprintln` is `io.eprint(value)`; `argv` is `env.args()`; `null` is `none`.
`ail check`, `build`, `run` and `test` rewrite these in your file, printing `fixed PATH:LINE:COL: [RULE] ...`
(fix.auto.1): `&&` `||` `!`, `else if`, `let`, `var`, `const`, `println`, `to_string()`, `"a" + b`, `(0..3).to_list()`,
`insert` on a set, `contains_key`, `has`, `m[k] else d`, `unwrap_or(d)`, `??`, a missing `mut`. Re-read the file before your next edit;
`--no-fix` or `AIL_FIX=0` turns this off.

## Compiles, but is not what you meant

No diagnostic and no fix reaches these; check them yourself.

- `it := m[k]` copies the entry: changing `it` leaves `m` alone. Write `m[k].qty += 1`, or store it back, `m[k] = it`.
- `for x in xs` gives copies; `for mut x in xs` changes the elements. A function changes its caller's value only
  through a `mut` parameter, `fn f(mut p T)`, called `f(mut q)`.
- `s.lines()` and `io.read_line()` drop a `\r` before the line end; to see it, split with `s.split("\n")`.
- `int` overflow and division by zero trap; `/` rounds toward zero (`-7 / 2` is `-3`).
- `a % -1` is `0` even for the smallest `int`, and `checked_rem(-1)` is `some(0)`, never `none`: to report overflow there, test `b == -1 and a == -9223372036854775808` yourself.
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
`fn main() !` may fail. `T ! str` fails with `return TEXT`; with no result or a `str` result: `fn f() ! Fail`, `return Fail(msg: "bad")`.
An optional has no `map`: `v := o else d`, or `if v := o`.

## Modules

A module is a directory, not a file: `use store` in `src/main.ail` loads every `.ail` file directly in `src/store/`
(`src/store/store.ail`; a file `src/store.ail` is not a module). `use` lines come first.

```ail skip=context
pub struct Item              # in src/store/store.ail; `pub` lets other modules name it
    pub sku str              # each field used outside needs `pub` too
    pub qty int

pub fn load(text str) List[Item]
```

Outside, names are always qualified: `store.Item`, `store.load(text)`; after `use parse`, `parse.parse(line)`.

## Mistakes that cost the most

Foreign constructs (gram.foreign.1), a missing `mut` (expr.bind.3) and `name: Type` (gram.fn.2) are rewritten for you.

1. An unknown name (expr.name.1) or a std function qualified wrongly (std.use.2): `fs.read_text(p)`, not
   `read_text(p)`; look it up with `ail ask std`, never guess a module.
2. A failure left unhandled (err.use.1): `?`, `else`, or return it. `else` handles only `T?` and `T ! E` (err.else.1);
   `E` is an `error` type or `str` alone (err.str.1), never `int` (err.decl.2).
3. A private name used from another module (mod.vis.1): mark the declaration and each field `pub`.
4. Operators between different types (trait.op.3): no implicit conversion, even `int` and `float`.
5. Wrong argument count or type (expr.call.1, expr.call.2): one per parameter, in order.
   A parameter may default (`retries int = 3`, expr.call.4): leave it out, or pass it by name only.
6. A swap-prone positional constructor (type.struct.7): name the fields when two share a type.
   A field may default (`retries int = 3`, type.struct.8): the default is a plain constant.
7. A task handle left unwaited (conc.task.5): `.wait()` every `spawn` before its block ends.
