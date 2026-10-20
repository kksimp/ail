# AIL language card

> Language card v1, 2026-10-20, bootstrap milestone 6. About 3,525 tokens
> (3,973 on the costliest tokenizer; measure with
> `tools/tokbench/run.sh docs/language-card.md`). Non-normative: the
> [spec](spec/) is authoritative and the [quick reference](quick-reference.md)
> has the long form. Everything here runs on the bootstrap interpreter.

## Running programs

```sh
./ail run FILE.ail           # check, then run
./ail check --json FILE.ail  # diagnostics only; each cites a spec rule ID
```

Modules: `use a.b` imports directory `a/b` as `b.item`; `pub` exports.

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
```

`return x` (or bare `return`) exits early.

## Bindings

```ail
x := 1           # `:=` declares a new name (immutable)
mut n := 0       # mutable
n = 5            # `=` changes an existing `mut` name
n += 1           # also -= *= /= %=
```

A name can't be declared twice in one function, even in an inner block. To
update a `mut` variable in a loop, use `=`, not `:=`.

## Types

`int` (64-bit), `float` (64-bit IEEE), `bool` (`true`, `false`), `str`,
`char` (`'a'`), `List[T]`, `Map[K, V]`, `Set[T]`, structs, enums, optionals
`T?`, results that can fail, `T ! E`, generic types, and `any Trait`.

Constants are top-level `NAME := value` (literals, constants, operators; no calls).

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

- A result that can fail must be handled: with `?`, `else`, or by returning it.
- `x := ... else ...` must give a fallback or leave (`return`, `continue`,
  `break`); to handle both outcomes and carry on, use `if x := ...` + `else err`.
- `else` also works inside an expression: `(m.get(k) else 0) + 1`.
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
    print(largest([3, 9, 4]))        # type arguments are inferred
    mut s := Stack[int].new()
    s.push(1)
    scene := Scene(shapes: [c, Square(side: 3)])
    for sh in scene.shapes
        print(sh.describe())
    print(Shape.area(c))             # the qualified form of `c.area()`
```

- `<`, `<=`, `>`, `>=` work on `int`, `str`, lists and types with `impl Ord`.
  Customise with `impl ToStr` (`fn to_str`), `impl Eq` (`fn eq`), or `fn cmp`.

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
- `int` extras: `a.min(b)`, `a.max(b)`, `n.abs()`, `n.to_hex()`, `checked_add`
  (gives `int?`), `wrapping_add`, `saturating_add`.

## Strings and characters

```ail
s := "total: {a + b}"      # interpolates any expression; `\{` is a literal brace
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

Also `pop()`/`last()` (give `T?`), `contains`, `index_of` (gives `int?`),
`sorted()`/`sort()`, `min()`/`max()` (give `T?`), `sum()`. Slices don't clamp
and negative indexes are errors.

`for x in xs` gives a copy of each element, and changing `xs` inside the loop
doesn't affect it. To change the elements themselves, use `for mut`:

```ail
for mut item in items      # `items` must be `mut`
    item.qty += 1          # changes the element
```

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

Also `contains`, `remove`, `len`, `sorted`. Keys can't be floats.

## Values and `mut`

Assigning or passing a value copies it (cheaply): changing `b` after
`b := a` never changes `a`.

```ail
fn bump(mut p Point)   # the caller sees the change
    p.x += 1

bump(mut q)            # `mut` again at the call; q must be a `mut` variable
```

Two `mut` arguments of one call can't refer to the same value.
