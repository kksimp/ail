# AIL language card

> v1, 2026-10-20. Non-normative: the [spec](spec/) is authoritative and the
> [quick reference](quick-reference.md) has the long form. Budget 8,000 tokens
> on every tokenizer (`tools/tokbench/run.sh docs/language-card.md`).

## Running programs

```sh
./ail run FILE.ail           # check, then run
./ail check --json FILE.ail  # diagnostics only; each cites a spec rule ID
```

Modules: `use a.b` imports directory `a/b` as `b.item`; `pub` exports. The
standard modules (`fs`, `env`, `io`, `net`, `path`, `proc`, `random`, `tensor`, `time`, `ui`)
need no `use`: `fs.read_text(p)`. A local of the same name hides one.

A program needs `fn main()`. `print(value)` writes any value and a newline.

## Layout

- Blocks are indented by exactly 4 spaces. No tabs, braces or `;`.
- A statement ends at the end of its line, unless the line ends inside
  `(`, `[` or `{`, or after a binary operator or a comma.
- `#` starts a comment. `##` starts a doc comment.

## Functions

```ail
fn add(a int, b int) int     # `name Type`, no colon; return type after `)`
    a + b                    # the last expression is the result

fn greet(name str)           # no return type: returns nothing
    print("hello, {name}")

fn half(n int) int
    requires n >= 0          # contract: a pure bool; false traps
    ensures result * 2 <= n  # `result` is the returned value
    n / 2
```

`return x` or bare `return` exits early. In `ensures`, `old(e)` is `e` on entry. A trait
method's contracts bind its impls; a struct's `invariant COND` lines hold after
construction and each `mut self` method.

## Bindings

```ail
x := 1           # `:=` declares a new name (immutable)
mut n := 0       # mutable
n = 5            # `=` changes an existing `mut` name
n += 1           # also -= *= /= %=
```

A name can't be declared twice in one function, even in an inner block: to
update a `mut` variable in a loop, use `=`.

## Types

`int` (64-bit), `float` (64-bit IEEE), `bool` (`true`, `false`), `str`,
`char` (`'a'`), `List[T]`, `Map[K, V]`, `Set[T]`, structs, enums, optionals
`T?`, results that can fail, `T ! E`, generic types, and `any Trait`.

`random.int(1, 6)` (inclusive), `random.float()`, `random.seed(n)`.

`t := tensor.from([1.0, 2.0, 3.0, 4.0], [2, 2])`: `t.matmul(t)`, `t.add(u)`, `t.mul(u)`, `t.dot(u)`, `t.sum()`, `t.map(|x| x * 2.0)`, `t.to_list()`; SIMD and multi-core.

`fs.read_text(p)?`, `fs.write_text(p, s)?`, `fs.walk(dir)?` (files, sorted), `fs.glob("src/**/*.ail")`.
`proc.run(cmd, args, input?, timeout_ms?)?` gives `.status`, `.out`, `.err`.
`net.get(url)?`, `net.post(url, body, content_type)?` give `.status`, `.headers`, `.body` (`http://` only).

Constants: top-level `NAME := value`, computed at compile time (pure calls allowed).

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

fn main()
    mut p := Point(y: 2, x: 1)    # name fields, in any order
    p.shift(3)                    # `mut self` methods need a `mut` variable
    print(p.sum())                # method calls always have ()
    c := Color.Red                # qualified, unless the type is known: `c = Green`
    p.x = 10                      # fields of a `mut` variable can be assigned
```

`==` and `print` work on every value: `print(p)` shows `Point(x: 1, y: 2)`.

Variants can carry fields. Build them like structs; match them by position:

```ail
enum Shape
    Circle(r int)
    Rect(w int, h int)

fn area(s Shape) int
    match s
        Circle(r) => 3 * r * r
        Rect(w, _) => w * w       # `_` skips a field; bare `Rect =>` skips all
```

## Absence and errors

```ail
fn find(names List[str], wanted str) int?   # an int, or `none`
    for i in 0..names.len()
        if names[i] == wanted
            return i
    none

error enum ParseError                       # only `error` types can be errors
    Empty "empty input"                     # optional message: its display text
    BadNumber(text str) "bad number {text}"

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
        print("bad input: {err}")           # bad input: bad number x
        0
    print("{i} {n} {t}")
    if v := parse("7")                      # handle both outcomes, then go on
        print("got {v}")
    else err
        print("error: {err}")
```

- `?` on an optional before an `else` goes to it: `xs.first()?.name else ""`.
- In a function with inferred errors (`!T`, `!`) and in `@test` functions,
  `?` on `none` fails with the error `Absent`: `port := text.to_int()? "port"`.
- A result that can fail must be handled: with `?`, `else`, or by returning it.
  `x := ... else ...` gives a fallback or leaves (`return`, `continue`, `break`).
- Several error types: `fn load() Config ! IoError | ParseError`. No success
  value: `fn save() ! IoError` (or `fn save() !`); `save() else return` leaves.

## Traits and generics

```ail
trait Shape
    fn area(self) int
    fn describe(self) str            # a default; an `impl` may replace it
        "shape: {area()}"            # defaults call other methods on `self`

struct Circle
    r int

impl Shape for Circle                # defines every method without a default
    fn area(self) int
        3 * r * r

fn largest[T Ord](items List[T]) T   # `T`: any type that implements `Ord`
    mut best := items[0]
    for x in items
        if x > best
            best = x
    best

struct Stack[T]                      # a generic type; its methods use `T`
    items List[T]                    # and `Self` (for `Stack[T]`) directly

fn main()
    c := Circle(r: 2)
    print(c.describe())              # shape: 12
    print(largest([3, 9, 4]))        # type arguments are inferred
    mut shapes := List[any Shape]()  # `any`: values of different types
    shapes.push(c)
```

- `impl Ord for Version` with no body compares the fields in order. `<` and
  friends work on `int`, `str`, lists and `Ord` types. Customise with
  `impl ToStr` (`fn to_str`), `impl Eq` (`fn eq`), or `fn cmp`.

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
sign := if n < 0 then "-" else "+"   # the one-line form

while n > 0
    n -= 1

for x in xs          # also: for i in 0..n (excludes n), for i in 1..=n
    if x == 0
        continue
    if x > 9
        break

label := match c     # must cover every variant; `_` covers the rest
    Red => "red"
    Green => "green"  # an arm can also be an indented block

match ch             # literals and `|`; ints, strings and chars need `_`
    '+' | '-' => continue           # a one-line statement arm
    '0' => total += 1
    _ => return ch
```

`if` and `match` work as statements or values; a function can end with either.

## Operators and strings

- `+ - * / %` on `int`. Overflow and division by zero stop the program (a
  trap). `/` rounds toward zero.
- `== != < <= > >=` (see above for which types). Comparisons can't be
  chained: write `a < b and b < c`.
- `and`, `or`, `not` (not `&&`, `||`, `!`). Conditions must be `bool`: write
  `if n != 0`, not `if n`.
- `float`: `+ - * /`, never traps; no mixing with `int`: `n.to_float()`,
  `x.to_int()`. An int literal works where a float is expected.
- Sized numbers `i8`…`u64`, `f32`: typed by context, same-type operands only,
  overflow traps; `x.to_u8()` converts.
- `int`: `min`, `max`, `abs`, `to_hex`, `checked_add` (`int?`).

## Strings and characters

```ail
s := "total: {a + b}"      # interpolates any expression; `\{` is a literal brace
c := 'x'                   # a char; '\n', '\'', '\u{e9}'
print("{n:>6}|{r:.2}|{s:*^8}")   # width, decimals, fill and align
```

- Format specs are `[[fill]align][width][.decimals][x]` and nothing else: `<` `>` `^`
  align (numbers default right, others left); `.N` needs a float, `x` an int.
- `"a,b".split(",")` gives `["a", "b"]`; `"12".to_int()` gives an `int?`,
  `"2.5".to_float()` a `float?`.
- `s.len()` counts characters, `s.byte_len()` bytes. `s.chars()` (a
  `List[char]`; strings have no indexing or `for`), `s.lines()`, `s.trim()`, `s.contains(x)`, `s.starts_with(x)`,
  `s.ends_with(x)` (`x` a `str` or `char`), `s.strip_prefix(p)` (gives
  `str?`), `s.pad_left(width, ' ')`, `pad_right`, `s.slice(a, b)` (characters), `s.char_at(i)` (`char?`), `s.find(x)` (`int?`), `s.replace(a, b)`, `s.to_lower()`, `s.to_upper()`.
- `xs.join(", ")` joins any list (`cs.join("")` for chars).
- `char`: `c.is_digit()`, `c.is_alpha()`, `c.is_space()`, `c.to_upper()`,
  `c.to_digit(16)` (gives `int?`), `c.code()`, `char.from_code(n)`.
- There's no `+` on strings and no `{{` escape: use interpolation and `\{`.

## Lists

```ail
mut xs := [3, 1, 2]    # a trailing comma is allowed; `[]` needs a known type
first := xs[0]         # out of bounds stops the program
part := xs[1..3]       # a copy of elements 1 and 2; also xs[1..], xs[..2]
xs.push(4)             # push, pop, sort and swap need a `mut` variable
xs.swap(0, 1)
xs[0] = 7              # elements can be assigned, including fields: xs[i].qty = 0
n := xs.len()          # int
mut out := []          # typed by its first use: out.push("a") makes a List[str]
```

Also `insert(i, x)`, `remove_at(i)`, `extend(ys)`, `clear()`, `reverse()`; `pop()`/`first()`/`last()` (give `T?`), `contains`, `index_of` (gives `int?`),
`sorted()`/`sort()`, `min()`/`max()` (give `T?`), `sum()`, `take(n)`. Slices
don't clamp and negative indexes are errors. Order by a key (stable):
`xs.sorted(by: |p| -p.age)`, also `sort`, `min`, `max`.

Comprehensions: `[x * x for x in xs if x > 0]`, sets `{x for ...}`, maps
`{k: v for ...}`.

`for x in xs` gives a copy of each element. To change the elements
themselves: `for mut item in items` (`items` must be `mut`), then `item.qty += 1`.

## Maps and sets

```ail
mut ages := {"ann": 31, "bob": 27}     # Map[str, int]; keeps insertion order
ages["cy"] = 40                        # inserts or replaces
ages["dan"] += 1                       # `op=` on a missing key starts at 0
mut counts := {}                       # typed by first use: counts[w] += 1
b := ages.get("dan") else 0            # `ages["dan"]` traps if missing
for name in ages.keys()                # `for k in m` is an error; also values()
    print("{name}: {ages[name]}")
mut seen := Set[str]()                 # or a literal: {"a", "b"}
if seen.add("x")                       # true if it wasn't there
    print("new")
```

Also `contains`, `remove`, `len`, `sorted`. Keys can't be floats.

## Values and `mut`

Assigning or passing copies: changing `b` after `b := a` never changes `a`.

```ail
fn bump(mut p Point)   # the caller sees the change
    p.x += 1

bump(mut q)            # `mut` again at the call, on a `mut` variable
```

Two `mut` arguments of one call can't refer to the same value.
`ref x` is a cell copies share. `unique` types move: `f(move x)`.

## Tasks

```ail
t := spawn load(1)          # a task; args are copies
p := t.wait()?              # result, or fails with Trapped
g := TaskGroup()            # tasks returning nothing; waited at block end
g.add(spawn work(2))
ch := Channel[int](8)       # send waits while full; close() when done
v := ch.recv() else break   # none once closed and empty
m := Mutex(0)               # get, set, swap; x := m.lock() then m.unlock(x + 1)
```

Wait or `add` each task exactly once, on every path. A deadlock traps.
