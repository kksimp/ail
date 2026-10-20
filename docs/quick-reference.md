# AIL quick reference

> **Non-normative.** One page for people and AI agents writing AIL. The
> [spec](spec/) is authoritative. Items marked **(not yet)** are specified but
> not supported by the bootstrap interpreter.

## Running programs

```sh
./ail run FILE.ail           # check, then run
./ail run DIR                # a package: DIR's files are the root module
./ail run FILE.ail a b       # words after FILE are the program's: env.args()
./ail check --json FILE.ail  # diagnostics only; each cites a spec rule ID
./ail test [--json] FILE.ail # run the `@test` functions; exit 0 if all pass
./ail effects [--json] FILE.ail       # each function's inferred effects
./ail run --deny fs,proc FILE.ail     # reject if `main` reaches fs or proc (or `all`)
./ail explain RULE           # what a rule ID means, with its ADRs and an example
./ail api --text FILE.ail    # declarations and ## docs, no bodies (JSON without --text)
./ail fmt FILE.ail           # rewrite in canonical layout (--check: diff, exit 1; --stdout)
```

With `--json`, each diagnostic is one line of schema v1 (`"api":1`), with a
span and, where the fix is obvious, `fixes` to apply by line and column
([semantic-api.md](design/semantic-api.md)).

Write `todo` for code you haven't written yet. It has whatever type is
expected; `ail check` warns (`expr.hole.1`) with that type and the names in
scope, and running it traps (`expr.hole.2`).

```ail
fn area(w int, h int) int
    todo     # warning: hole: expected `int`; in scope: `h int`, `w int`
```

A program needs `fn main()`. `print(value)` writes any value and a newline.

## Layout

- Blocks are indented by exactly 4 spaces. No tabs, braces or `;`.
- A statement ends at the end of its line, unless the line ends inside
  `(`, `[` or `{`, or after a binary operator or a comma.
- `#` starts a comment. `##` starts a doc comment.
- Write the layout `ail fmt` writes ([formatting.md](design/formatting.md)):
  one space around binary operators and after `,` and `:`; none inside
  brackets, around `.` and `..`, or before a call's `(`; lines up to 100
  characters, with a longer list split one element per line plus a trailing
  comma; one blank line between multi-line top-level declarations.

## Functions

```ail
fn add(a int, b int) int     # `name Type`, no colon; return type after `)`
    a + b                    # the last expression is the result

fn greet(name str)           # no return type: returns nothing
    print("hello, {name}")
```

`return x` (or bare `return`) exits early.

## Bindings

```ail
x := 1           # `:=` declares a new name (immutable)
mut n := 0       # mutable
n = 5            # `=` changes an existing `mut` name
n += 1           # also -= *= /= %=
```

A name can't be declared twice in one function, even in an inner block
(sibling blocks may reuse a name). This includes parameters and loop
variables. Locals can't reuse a function or type name. To update a `mut`
variable inside a loop, use `=`, not `:=`.

## Types

`int` (64-bit), `float` (64-bit IEEE), `bool` (`true`, `false`), `str`,
`char` (`'a'`), `List[T]`, `Map[K, V]`, `Set[T]`, structs, enums, optionals
`T?`, results that can fail, `T ! E`, generic types, and `any Trait`.

Constants are top-level `NAME := value`, built from literals, other
constants, collection and struct literals and operators (no calls):

```ail
MAX_DEPTH := 64
KEYWORDS := {"if", "else", "while"}     # a Set[str]
```

## Structs, enums and methods

```ail
struct Point
    x int
    y int

enum Color
    Red
    Green

fn Point.sum(self) int            # method; `self` is read-only
    x + y                         # fields and methods by bare name (self.x)

fn Point.shift(mut self, dx int)  # may change self
    x += dx

fn Point.origin() Point           # associated function: Point.origin()
    Point(x: 0, y: 0)

fn is_red(c Color) bool
    match c
        Red => true
        Green => false

fn main()
    mut p := Point(y: 2, x: 1)    # name fields, in any order
    p.shift(3)                    # `mut self` methods need a `mut` variable
    print(p.sum())                # method calls always have ()
    c := Color.Red                # qualify a variant...
    print(is_red(Green))          # ...unless its type is known from context
    p.x = 10                      # fields of a `mut` variable can be assigned
```

Arguments can also be positional when no two of the fields they fill have one
type (`Problem(7, "bad")` for `line int, msg str`), or a local named like the
field (`Point(x, y)`). `Point(1, 2)` is an error: `x` and `y` are both `int`.
In a method, a local can't take a field's name.

A bare variant is allowed wherever the type is known: arguments, return
values, struct fields, `match` arms, assignments to an existing variable, list
elements after the first, and the right side of a comparison (`c == Red`).

Every value can be compared with `==` (structs and lists field by field) and
printed: `print(p)` shows `Point(x: 1, y: 2)`, and `"{c}"` gives `Color.Red`.

Variants can carry fields. Build them like structs; match them by position:

```ail
enum Shape
    Circle(r int)
    Rect(w int, h int)

fn area(s Shape) int
    match s
        Circle(r) => 3 * r * r
        Rect(w, _) => w * w       # `_` skips a field
```

## Absence and errors

```ail
fn find(names List[str], wanted str) int?   # an int, or `none`
    for i in 0..names.len()
        if names[i] == wanted
            return i
    none

error enum ParseError                       # only `error` types can be errors
    Empty
    BadNumber(text str)

fn parse(s str) int ! ParseError            # an int, or a ParseError
    if s == ""
        return Empty                        # returning an error value fails
    n := s.to_int() else return BadNumber(text: s)
    n

fn total(items List[str]) !int              # `!int`: the errors are inferred
    mut sum := 0
    for s in items
        sum += parse(s)? "item {s}"         # `?` passes a failure on, with context
    sum

fn main() !                                 # main can fail; the error is reported
    i := find(["a", "b"], "b") else -1      # `else`: a fallback value...
    n := parse("12") else return            # ...or leave...
    t := total(["1", "x"]) else err         # ...or handle the error
        print("bad input: {err}")           # ParseError.BadNumber(text: "x")
        0
    print("{i} {n} {t}")
    if v := parse("7")                      # do one thing or the other, then go on
        print("got {v}")
    else err
        print("error: {err}")
```

- A result that can fail must be handled: with `?`, with `else`, or by
  returning it. Calling `parse(s)` on its own line is an error.
- `x := ... else ...` must give a fallback value or leave (`return`,
  `continue`, `break`). To handle success and failure and then carry on, use
  `if x := ...` with `else err`.
- `if x := e` also works on optionals (`else` without a name), and in `elif`.
  On a result that can fail, it needs an `else`.
- `else` also works inside an expression: `Min => nums.min() else 0`,
  `(m.get(k) else 0) + 1`, `f(s.to_int() else return none)`.
- `?` on an optional works only in a function that returns an optional.
- `print` of a `T?` shows the value, or `none`. A value is returned as a
  `T?` without wrapping: `return i`.
- Several error types: `fn load() Config ! IoError | ParseError`.
- No success value: `fn save() ! IoError` (or `fn save() !` to infer it).
  As a statement, `save() else err` + block handles the failure and goes on;
  `save() else return` leaves.
- `unreachable("why")` stops the program; it fits any type, like `return`.

## Traits and generics

```ail
trait Shape
    fn area(self) int
    fn name(self) str                # a default; an `impl` may replace it
        "shape"
    fn describe(self) str            # defaults call other methods on `self`
        "{name()}: {area()}"

struct Circle
    r int

struct Square
    side int

impl Shape for Circle                # defines every method without a default
    fn area(self) int
        3 * r * r

impl Shape for Square
    fn area(self) int
        side * side
    fn name(self) str
        "square"

struct Version
    major int
    minor int

impl Ord for Version                 # no methods: compares major, then minor

fn largest[T Ord](items List[T]) T   # `T`: any type that implements `Ord`
    mut best := items[0]
    for x in items
        if x > best
            best = x
    best

fn first[T](xs List[T]) T?
    if xs.len() == 0
        return none
    xs[0]

fn bigger(a Shape, b Shape) bool     # a trait as a parameter type
    a.area() > b.area()

struct Stack[T]                      # a generic type
    items List[T]

fn Stack.new() Self                  # its methods use `T` directly;
    Stack(items: [])                 # `Self` means `Stack[T]`

fn Stack.push(mut self, x T)
    items.push(x)

struct Scene
    shapes List[any Shape]           # `any`: values of different types

fn main()
    c := Circle(r: 2)
    print(c.describe())              # shape: 12
    print(largest([3, 9, 4]))        # type arguments are inferred...
    print(first[int]([]))            # ...or written when nothing gives them
    mut s := Stack[int].new()
    s.push(1)
    scene := Scene(shapes: [c, Square(side: 3)])
    for sh in scene.shapes
        print(sh.describe())
    print(Version(major: 1, minor: 9) < Version(major: 2, minor: 0))
    print(bigger(c, Square(side: 1)))
    print(Shape.area(c))             # the qualified form of `c.area()`
```

- `<`, `<=`, `>`, `>=` work on `int`, `str`, lists, and types with
  `impl Ord for T`, which compares the fields in order, or uses your
  `fn cmp(self, other Self) Ordering` (returning `Less`, `Equal` or
  `Greater`).
- To customise: `impl ToStr for Color` with `fn to_str(self) str`, or
  `impl Eq for T` with `fn eq(self, other Self) bool`.
- To constrain one method of a generic type: `fn Stack[T Ord].max(self) T?`.
- On an `any Shape` value, you can call only the methods whose parameters
  don't use `Self`.

## Control flow

```ail
if a < b
    print("less")
elif a == b
    print("equal")
else
    print("greater")

kind := if n < 0     # `if` used as a value needs `else`;
    "negative"       # elif/else line up with the line the `if` starts on
else
    "non-negative"

while n > 0
    n -= 1

for x in xs          # also: for i in 0..n (excludes n), for i in 1..=n
    if x == 0
        continue
    if x > 9
        break

label := match c     # must cover every variant; `_` covers the rest
    Red => "red"
    Green =>
        part := "gr"
        "{part}een"

match c              # as a statement, arms can just do things
    Red => print("stop")
    Green =>
        count += 1
        print("go")

match ch             # literals and `|`; ints, strings and chars need `_`
    '+' | '-' => continue           # a one-line statement arm
    '0' => total += 1
    _ => return ch
```

`if` and `match` work as statements or as values. A function can end with
either one, and its value is the function's result. A `while true` loop with
no `break` can end a function too.

## Operators and strings

- `+ - * / %` on `int`. Overflow and division by zero stop the program (a
  trap). `/` rounds toward zero.
- `== != < <= > >=` (see above for which types). Comparisons can't be
  chained: write `a < b and b < c`.
- `and`, `or`, `not` (not `&&`, `||`, `!`). Conditions must be `bool`: write
  `if n != 0`, not `if n`.
- `float`: `+ - * /` (no `%`), never traps (`1.0 / 0.0` is `inf`). No mixing
  with `int`: `n.to_float()`, `x.to_int()` (truncates; traps on NaN). An int
  literal works where a float is expected: `x * 2`, `total: 0`. `round()`
  rounds halves away from zero and gives a float: `2.5.round()` is `3.0`.
  Floats print like Python: `0.1 + 0.2` is `0.30000000000000004`, `1e16` is
  `1e+16`.
- `int` extras: `a.min(b)`, `a.max(b)`, `n.abs()`, `n.to_hex()` (`ff`, no `0x`),
  `checked_add` (gives `int?`), `wrapping_add`, `saturating_add`, and so on.

## Strings and characters

```ail
s := "total: {a + b}"      # interpolates any expression; `\{` is a literal brace
t := """
    line one
      indented
    """                    # "line one\n  indented": the closing line adds no newline
q := """say "hi""""        # on one line, for text with quotes: say "hi"
r := r"C:\dir\{x}"         # raw: no escapes, no interpolation
c := 'x'                   # a char; '\n', '\'', '\u{e9}'
print("{n:>6}|{r:.2}|{n:0>4x}|{s:^8}|{s:*<5}")  # width, fill, align, decimals, hex
```

- Format specs are `[[fill]align][width][.decimals][x]` and nothing else: `<` `>` `^`
  align (numbers default right, others left); `.N` needs a float, `x` an int.

- `"a,b".split(",")` gives `["a", "b"]`; `"12".to_int()` gives an `int?`,
  `"2.5".to_float()` a `float?`.
- `s.chars()` (a `List[char]`; strings have no indexing, `len()` or `for`),
  `s.lines()`, `s.trim()`, `s.contains(x)`, `s.starts_with(x)`,
  `s.ends_with(x)` (`x` a `str` or `char`), `s.strip_prefix(p)` (gives
  `str?`), `s.repeat(n)`, `s.pad_left(width, ' ')`, `s.pad_right(width, '0')`.
- Join any list: `xs.join(", ")`; chars back into a string: `cs.join("")`.
- `char`: `c.is_digit()`, `c.is_alpha()`, `c.is_space()`, `c.to_upper()`,
  `c.to_digit(16)` (gives `int?`), `c.code()`, `char.from_code(n)`.
- There's no `+` on strings and no `{{` escape: use interpolation and `\{`.

## Lists

```ail
mut xs := [3, 1, 2]    # a trailing comma is allowed; `[]` needs a known type
first := xs[0]         # out of bounds stops the program
part := xs[1..3]       # a copy of elements 1 and 2; also xs[1..] and xs[..2]
xs.push(4)             # push, pop, sort and swap need a `mut` variable
xs.swap(0, 1)
xs[0] = 7              # elements can be assigned, including fields: xs[i].qty = 0
n := xs.len()          # int
mut out := []          # typed by its first use: out.push("a") makes a List[str]
empty := List[int]()   # an empty list of a given type, anywhere
```

Also: `xs.pop()` and `xs.last()` (give `T?`), `xs.contains(x)`,
`xs.index_of(x)` (gives `int?`: `none`, not -1, when absent), `xs.sorted()`
(a copy) and `xs.sort()` (in place), `xs.min()` and `xs.max()` (give `T?`),
`xs.sum()` (ints or floats). Slices don't clamp, and negative indexes are
errors: write `xs[xs.len() - 1]` or `xs.last()`.

`for x in xs` gives a copy of each element, and changing `xs` inside the loop
doesn't affect it. To change the elements themselves, use `for mut`:

```ail
for mut item in items      # `items` must be `mut`
    item.qty += 1          # changes the element
```

Inside a `for mut` loop, the body can't use the list itself (`items`).

## Maps and sets

```ail
mut ages := {"ann": 31, "bob": 27}     # Map[str, int]; keeps insertion order
ages["cy"] = 40                        # inserts or replaces
ages["ann"] += 1                       # the key must exist, or the program stops
a := ages["bob"]                       # traps if missing...
b := ages.get("dan") else 0            # ...`get` gives an optional
for name in ages.keys()                # `for k in m` is an error; also values()
    print("{name}: {ages[name]}")
mut seen := Set[str]()                 # or a literal: {"a", "b"}
if seen.add("x")                       # true if it wasn't there
    print("new")
```

Also `m.contains(k)`, `m.remove(k)` (gives `V?`), `m.len()`, `s.remove(x)`,
`s.contains(x)`, `s.sorted()`. Keys and set elements can be any type whose
`==` is automatic (ints, strings, chars, structs and enums of them), but not
floats.

## JSON

`Json` is built in, with no import (`json.type.1`). It is read only when the
program uses it.

```ail
doc := parse_json("\{\"a\": [1, \"x\"]}")?   # Json ! JsonError; err.at is the char offset
doc.get("a")?.at(1)?.as_str()                  # str?  (get, at, as_str/as_num/as_bool/as_arr/as_obj)
doc.to_text()                                  # {"a":[1.0,"x"]}, compact, members in document order
```

Variants: `Null`, `Bool(b)`, `Num(n float)`, `Str(s)`, `Arr(items)`,
`Obj(members)` (`JsonMember` has `key`, `value`). Write `Json.Num(1.0)` when
the type isn't known. A program that declares its own `Json` keeps it.

## Values and `mut`

Assigning or passing a value copies it (cheaply): changing `b` after
`b := a` never changes `a`.

```ail
fn bump(mut p Point)   # the caller sees the change
    p.x += 1

bump(mut q)            # `mut` again at the call; q must be a `mut` variable
```

Two `mut` arguments of one call can't refer to the same value.

## Modules

A module is a directory of `.ail` files sharing one namespace. `use a.b`
imports `ROOT/a/b`, where ROOT is the directory run (`ail run DIR`) or the
file's directory (`ail run FILE`, whose siblings are not included).

```ail fragment
use geo                  # imports at the top of each file
use util.text as fmt     # `as` renames; names must not clash

fn main()
    p := geo.Point(x: 1, y: 2)   # items are always qualified
    print(geo.dist(p, geo.ORIGIN))
    match geo.classify(p)
        geo.Kind.Near => print(fmt.banner("near"))
        _ => print(p.x)
```

```ail fragment
pub struct Point         # `pub` exports; everything else is private
    pub x int            # fields too; a private field means only this
    y int                # module can build a `Point`
```

There are no item or wildcard imports, no `pub use`, and no import cycles.
`fn T.m` must be in `T`'s module. Text forms show the module path:
`geo.Point(x: 1, y: 2)`.

## Files, processes and the system

Import a standard module with `use std.NAME` and call it by its name
(spec 17-std). Failures are values: `fs.IoError(path, kind)` and
`proc.ProcError(cmd, kind)`, `kind` one of `not_found`, `permission`,
`exists`, `is_dir`, `not_dir`, `not_empty`, `invalid_utf8`, `other`.

```ail
use std.fs
use std.proc

fn main() !
    text := fs.read_text("in.txt")?          # str ! fs.IoError
    fs.write_text("out.txt", text)?           # also append_text
    names := fs.list_dir(".")?                # sorted names
    r := proc.run("git", ["status"])?         # no shell; fails only if it can't start
    print("{r.status} {r.out} {r.err}")      # proc.Output
```

| Module | Functions |
|---|---|
| `std.fs` | `read_text(p) str`, `write_text(p, s)`, `append_text(p, s)`, `exists(p) bool`, `list_dir(p) List[str]`, `remove(p)` (file or empty dir), `make_dir(p)` (with parents); all but `exists` are `! fs.IoError` |
| `std.env` | `args() List[str]`, `get(name) str?`, `cwd() str`, `program() str` |
| `std.io` | `read_line() str?`, `read_all() str`, `eprint(value)`, `exit(code)` |
| `std.proc` | `run(cmd, args) proc.Output ! proc.ProcError`; `Output { status int, out str, err str }` |
| `std.time` | `now_ms() int`, `sleep_ms(ms)` |

## Tests and assertions

```ail
@test fn doubles()           # `@test` on the same line; no parameters
    assert_eq(double(4), 8)  # traps showing both values if unequal
    assert(double(1) == 2)   # traps if false
```

A test returns nothing or `!`, can't be called, and is ignored by `ail run`.
`assert` and `assert_eq` also work in ordinary code (a failure traps).

## Effects

Effects are inferred, never written: `io` (`print`, `fs`, `io`, `proc`),
`fail` (`T ! E` result), `mutates` (`mut` parameter), and the capabilities
`fs`, `env`, `proc`, `clock`, `net`. A call through `any Trait` counts every
implementation.

```ail
@pure fn area(w int, h int) int   # error if it reaches print or a capability
    w * h
```
