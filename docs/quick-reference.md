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
x := 1           # immutable
mut n := 0       # mutable
n = 5
n += 1           # also -= *= /= %=
```

A name can't be declared twice in one function, even in an inner block
(sibling blocks may reuse a name). Locals can't reuse a function or type name.

## Types

`int` (64-bit), `bool` (`true`, `false`), `str`, `List[T]`, structs and enums.
**(not yet):** floats, `char`, `Map`, `Set`, `T?`, `T ! E`, traits, generics.

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
```

Enums have no `==` yet: compare them with `match`. **(not yet):** variants
with data, such as `Circle(r int)`.

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
```

## Operators and strings

- `+ - * / %` on `int`. Overflow and division by zero stop the program (a
  trap). `/` rounds toward zero.
- `== != < <= > >=`. Comparisons can't be chained: write `a < b and b < c`.
- `and`, `or`, `not` (not `&&`, `||`, `!`). Conditions must be `bool`: write
  `if n != 0`, not `if n`.
- Strings: `"total: {a + b}"` interpolates any `int`, `bool` or `str`
  expression. `\{` is a literal brace. There's no `+` on strings, and `str`
  supports only `==` and `!=`.

## Lists

```ail
mut xs := [3, 1, 2]    # a trailing comma is allowed; `[]` needs a known type
first := xs[0]         # out of bounds stops the program
part := xs[1..3]       # a copy of elements 1 and 2
xs.push(4)             # push and swap need a `mut` variable
xs.swap(0, 1)
n := xs.len()          # int
```

## Values and `mut`

Assigning or passing a value copies it (cheaply): changing `b` after
`b := a` never changes `a`.

```ail
fn bump(mut p Point)   # the caller sees the change
    p.x += 1

bump(mut q)            # `mut` again at the call; q must be a `mut` variable
```

Two `mut` arguments of one call can't refer to the same value.
