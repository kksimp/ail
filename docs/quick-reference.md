# AIL quick reference

> **Non-normative.** One page for people and AI agents writing AIL. The
> [spec](spec/) is authoritative. Items marked **(not yet)** are specified but
> not supported by the bootstrap interpreter.

## Running programs

```sh
./ail run FILE.ail           # check, then run
./ail check --json FILE.ail  # diagnostics only; each cites a spec rule ID
```

A program needs `fn main()`. `print(value)` writes an `int`, `bool` or `str`
and a newline.

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

A name can't be declared twice in one function, even in an inner block
(sibling blocks may reuse a name). This includes parameters and loop
variables. Locals can't reuse a function or type name. To update a `mut`
variable inside a loop, use `=`, not `:=`.

## Types

`int` (64-bit), `bool` (`true`, `false`), `str`, `List[T]`, structs, enums,
optionals `T?`, and results that can fail, `T ! E`.
**(not yet):** floats, `char`, `Map`, `Set`, traits, generics.

## Structs, enums and methods

```ail
struct Point
    x int
    y int

enum Color
    Red
    Green

fn Point.sum(self) int            # method; `self` is read-only
    self.x + self.y

fn Point.shift(mut self, dx int)  # may change self
    self.x += dx

fn Point.origin() Point           # associated function: Point.origin()
    Point(x: 0, y: 0)

fn is_red(c Color) bool
    match c
        Red => true
        Green => false

fn main()
    mut p := Point(y: 2, x: 1)    # name every field, in any order
    p.shift(3)                    # `mut self` methods need a `mut` variable
    print(p.sum())                # method calls always have ()
    c := Color.Red                # qualify a variant...
    print(is_red(Green))          # ...unless its type is known from context
    p.x = 10                      # fields of a `mut` variable can be assigned
```

A bare variant is allowed wherever the type is known: arguments, return
values, struct fields, `match` arms, assignments to an existing variable, and
list elements after the first. Enums have no `==` yet: compare them with
`match`.

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
- `?` on an optional works only in a function that returns an optional.
- Several error types: `fn load() Config ! IoError | ParseError`.

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
```

`if` and `match` work as statements or as values. A function can end with
either one, and its value is the function's result.

## Operators and strings

- `+ - * / %` on `int`. Overflow and division by zero stop the program (a
  trap). `/` rounds toward zero.
- `== != < <= > >=`. Comparisons can't be chained: write `a < b and b < c`.
- `and`, `or`, `not` (not `&&`, `||`, `!`). Conditions must be `bool`: write
  `if n != 0`, not `if n`.
- Strings: `"total: {a + b}"` interpolates any `int`, `bool` or `str`
  expression, or an error value. `\{` is a literal brace. There's no `+` on
  strings, and `str` supports only `==` and `!=`.
- `"a,b".split(",")` gives `["a", "b"]`; `"12".to_int()` gives an `int?`.

## Lists

```ail
mut xs := [3, 1, 2]    # a trailing comma is allowed; `[]` needs a known type
first := xs[0]         # out of bounds stops the program
part := xs[1..3]       # a copy of elements 1 and 2
xs.push(4)             # push and swap need a `mut` variable or parameter
xs.swap(0, 1)
xs[0] = 7              # elements can be assigned, including fields: xs[i].qty = 0
n := xs.len()          # int
```

`for x in xs` gives a copy of each element, and changing `xs` inside the loop
doesn't affect it. To change the elements themselves, use `for mut`:

```ail
for mut item in items      # `items` must be `mut`
    item.qty += 1          # changes the element
```

Inside a `for mut` loop, the body can't use the list itself (`items`).

## Values and `mut`

Assigning or passing a value copies it (cheaply): changing `b` after
`b := a` never changes `a`.

```ail
fn bump(mut p Point)   # the caller sees the change
    p.x += 1

bump(mut q)            # `mut` again at the call; q must be a `mut` variable
```

Two `mut` arguments of one call can't refer to the same value.
