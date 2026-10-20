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
./ail build FILE.ail [-o OUT] # native executable for the host (or --target x86_64-linux|aarch64-linux|aarch64-macos): numbers, strings, lists, loops, free functions, env.args() (native.subset.1)
./ail test [--json] FILE.ail # run the `@test` functions; exit 0 if all pass
./ail test --coverage[=lines] FILE.ail # lines and functions the tests ran; =lines lists the unrun ranges
./ail run --timeout MS --max-memory MB --max-steps N FILE.ail # a runaway run traps (run.limit.*) with a snapshot
./ail effects [--json] FILE.ail       # each function's inferred effects
./ail effects --parallel FILE.ail     # which loops may run on many cores, and what blocks the rest
./ail run --deterministic FILE.ail # deny clock, random, proc, net: same inputs, same output; loops on one thread
./ail run --parallel FILE.ail      # also pool loops proven independent; AIL_THREADS=N sets the pool size
./ail run --seed 42 FILE.ail         # seed std.random so the run repeats
./ail run --no-bytecode FILE.ail     # every function on the tree walker (the oracle), not the bytecode tier
./ail run --deny fs,proc FILE.ail     # reject if `main` reaches fs or proc (or `all`)
./ail explain RULE           # what a rule ID means, with its ADRs and an example
./ail explain                # all rules, with chapter and whether `ail fix` repairs them
./ail api --text FILE.ail    # declarations and ## docs, no bodies; `fails_with` lists a function's error types (JSON without --text)
./ail check --errors FILE.ail  # JSON: each function's error types and failure paths (`?`, `else`, `return`)
./ail api --card [--budget N] DIR|FILE   # the whole program on one page: modules, signatures, tests, ail.pkg, examples (default 4,000 tokens)
./ail api --type FILE.ail L C   # type at a position (also --locals, --refs; `ail api DIR` and `--refs DIR FILE L C` cover a whole package)
./ail api --expanded --text FILE.ail # also the code derives generated (ToJson, FromJson, @derive traits)
./ail api --tokens FILE.ail  # the lexer's tokens, one per line; --ast: the parse as JSON (--program: with prelude and imports)
./ail fix FILE.ail           # apply the fixes diagnostics carry (mut, ?, elif, match arms); --dry-run: diff
./ail fix FILE.ail           # apply the fixes diagnostics carry (mut, ?, elif, `&&`, `len(x)`, tabs, ...); --dry-run: diff
./ail check --budget 500 DIR  # any command: ~500 tokens, errors first, then `[budget ...; continue with --from K]`
./ail fmt FILE.ail           # rewrite in canonical layout (--check: diff, exit 1; --stdout)
./ail run --trace FILE.ail   # each call, return, effect, task, contract check on stderr (=calls, =effects; --trace-limit N; `ail test --trace` keeps a failing test's)
./ail run --watch n FILE.ail # each assignment to `mut` variable n (and `push`, `pop`, `xs[i] = e`...), with value and position
./ail run --verbose FILE.ail # a trap shows every frame's values (default: 10 lines; --json: one object)
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
- A line starting with `.name` continues the previous statement (chained calls); attributes may sit on their own lines above a `fn`.
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

Constants are top-level `NAME := value`, computed at compile time. The
value may call functions that do no I/O and use no capability
(`ct.const.2`); a value that can fail needs `else`. Put `if`, `match` and
`?` in a function and call it:

```ail
MAX_DEPTH := 64
KEYWORDS := {"if", "else", "while"}     # a Set[str]
WORDS := "fn if else".split(" ")
SQUARES := squares(8)                   # squares must be free of I/O

fn squares(n int) List[int]
    mut out := List[int]()
    for i in 0..n
        out.push(i * i)
    out
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

fn round(s Shape) bool
    match s
        Circle => true            # bare: any fields, binds none
        Rect => false
```

## Absence and errors

```ail
fn find(names List[str], wanted str) int?   # an int, or `none`
    for i in 0..names.len()
        if names[i] == wanted
            return i
    none

error enum ParseError                       # only `error` types can be errors
    Empty "empty input"                     # optional message: the display text,
    BadNumber(text str) "bad number {text}" # on every variant or none

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
  `(m.get(k) else 0) + 1`, `f(s.to_int() else return none)`, and in a `for`
  header: `for x in j.arr("items") else []`.
- `?` on an optional works in a function that returns an optional, or before
  an `else`, which then catches the `none`: `j.get("u")?.str("name") else "?"`.
- In a function whose errors are inferred (`!T`, `!`) and in a test, `?` on
  `none` fails with the prelude error `Absent` ("a value is absent"), with
  context if given: `n := text.to_int()? "port {text}"`. A function that
  states its errors (`T ! E`) needs `else` instead.
- `error struct NotFound "nothing at {path}"` gives a struct a message; it
  may then have no fields. Without messages, write `impl ToStr`.
- `print` of a `T?` shows the value, or `none`. A value is returned as a
  `T?` without wrapping: `return i`.
- `mut best := none` has no type until its first assignment: `best = 3`
  makes it an `int?`. `x := if c then 4 else none` is an `int?`.
- A local may have the name of a function (it shadows it); calling that name
  inside the local's block is an error.
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

trait Zero                           # a function without `self`
    fn zero() Self

trait Sized Ord + Zero               # supertraits: `[T Sized]` gives `<`, `==`, `T.zero()`
    fn size(self) int

fn zeros[T Zero](n int) List[T]
    [T.zero() for _ in 0..n]         # zeros[int](3); `Type.zero()` on a type

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

```ail skip=context
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
sign := if n < 0 then "-" else "+"   # one line: `then` and `else`, no `elif`
pad := 2 + (if n < 0 then 1 else 0)  # a one-line `if` as an operand: in parentheses

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
    'a'..='z' => lower += 1         # ranges: `lo..hi` or `lo..=hi`, ints or chars
    _ if ch == ' ' => skip += 1     # guard: `pattern if cond =>`; a guarded arm covers nothing
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
- Sized numbers `i8 i16 i32 u8 u16 u32 u64 f32` (`i64` is `int`, `f64` is
  `float`). A literal takes its type from context and must fit: `fn mix(a i32)`
  called `mix(-7)`, `r: 255` for a `u8` field. Operators need two values of
  one type and trap on overflow; no implicit widening. Convert with
  `x.to_u8()` (traps if it doesn't fit), `x.to_int()`, `x.to_float()`,
  `f.to_f32()`; `wrapping_add`, `checked_add` and `saturating_add` work on
  every sized integer. Unsigned types have no unary `-`. Locals aren't
  annotated: `x := 100.to_u8()`.
- `int` extras: `a.min(b)`, `a.max(b)`, `n.abs()`, `n.to_hex()` (`ff`, no `0x`),
  `checked_add` (gives `int?`), `wrapping_add`, `saturating_add`, and so on.
- Bits on `int` (64-bit two's complement): `a.bit_and(b)`, `bit_or`, `bit_xor`,
  `n.bit_not()`, `n.shl(k)`, `n.shr(k)` (arithmetic; `k` in `0..=63`, else trap `expr.bit.2`).

## Strings and characters

```ail skip=context
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
  `str?`), `s.repeat(n)`, `s.pad_left(width, ' ')`, `s.pad_right(width, '0')`, `s.slice(a, b)` (characters), `s.char_at(i)` (`char?`), `s.find(x)` (`int?`), `s.replace(a, b)`, `s.to_lower()`, `s.to_upper()`.
- Join any list: `xs.join(", ")`; chars back into a string: `cs.join("")`.
- `char`: `c.is_digit()`, `c.is_alpha()`, `c.is_space()`, `c.to_upper()`,
  `c.to_digit(16)` (gives `int?`), `c.code()`, `char.from_code(n)`.
- There's no `+` on strings and no `{{` escape: use interpolation and `\{`.

## Lists

```ail skip=context
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

Also: `xs.insert(i, x)`, `xs.remove_at(i)`, `xs.extend(ys)`, `xs.clear()`, `xs.reverse()`; `xs.pop()`, `xs.first()` and `xs.last()` (give `T?`), `xs.contains(x)`,
`xs.index_of(x)` (gives `int?`: `none`, not -1, when absent), `xs.sorted()`
(a copy) and `xs.sort()` (in place), `xs.min()` and `xs.max()` (give `T?`),
`xs.sum()` (ints or floats), `xs.take(n)` (the first `n`, or fewer). Slices
don't clamp, and negative indexes are errors: write `xs[xs.len() - 1]` or
`xs.last()`.

Sort or pick by a key with `by: |x| key` (only on `sorted`, `sort`, `min`,
`max`; stable, so sort by name first, then by count):

```ail fragment
counts := {"b": 2, "a": 2, "c": 5}
top := counts.keys().sorted().sorted(by: |k| -counts[k]).take(2)   # ["c", "a"]
```

`for x in xs` gives a copy of each element, and changing `xs` inside the loop
doesn't affect it. To change the elements themselves, use `for mut`:

```ail skip=context
for mut item in items      # `items` must be `mut`
    item.qty += 1          # changes the element
```

Inside a `for mut` loop, the body can't use the list itself (`items`).

A comprehension builds a list, set or map in one expression:

```ail fragment
xs := [3, 1, 4]
squares := [x * x for x in xs]             # [9, 1, 16]
big := [x for x in xs if x > 2]            # [3, 4]
pairs := [r * 10 + c for r in 0..2 for c in 0..2]   # clauses nest, outermost first
seen := {x % 2 for x in xs}                # a Set[int]
sizes := {w: w.chars().len() for w in ["a", "bc"]}  # a Map[str, int]
nums := [t.to_int() else continue for t in ["1", "x"]]  # continue skips: [1]
```

The loop variables are new names that end with the comprehension. `?` and
`return` leave the function; `break` ends the comprehension.

Loops use every core automatically when their iterations are independent:
no output or capability, no `break`/`return`/`?`, and writes only to the
iteration's own locals, to `out[i]` in a `for i in a..b` loop, or to an
`int` total with `+=`. Results are always the same as running in order, so
float totals use `xs.sum()` (a fixed tree) rather than `+=`. When you know
the iterations are independent but the checker can't see it (writes to
`out[perm[i]]`), write `par for`; a `par for` the checker can prove unsafe
(it prints, or breaks out) is an error. A parallel map is a comprehension
with `par for`: `lens := [steps(x) par for x in xs]` (one clause, a list;
the element must not print, use a handle or write outside itself):

```ail
fn main()
    perm := [1, 0]
    mut out := [0, 0]
    par for i in 0..2
        out[perm[i]] = i * 10    # par asserts perm has no repeats
    print(out)
```

## Maps and sets

```ail skip=context
mut ages := {"ann": 31, "bob": 27}     # Map[str, int]; keeps insertion order
ages["cy"] = 40                        # inserts or replaces
ages["ann"] += 1                       # `op=` on a missing key starts from 0
mut counts := {}                       # typed by first use: counts[w] += 1
a := ages["bob"]                       # traps if missing...
b := ages.get("dan") else 0            # ...`get` gives an optional
for name in ages.keys()                # `for k in m` is an error; also values()
    print("{name}: {ages[name]}")
mut seen := Set[str]()                 # or a literal: {"a", "b"}
if seen.add("x")                       # true if it wasn't there
    print("new")
```

Also `m.contains(k)`, `m.remove(k)` (gives `V?`), `m.len()`, `s.remove(x)`,
`s.contains(x)`, `s.sorted()`, `s.union(t)`, `s.intersect(t)`, `s.difference(t)` (also on two `keys()` lists; give a set), and `m.push(k, x)` for a `Map[K, List[T]]` (starts an empty list when `k` is missing; types an untyped `{}`). Keys and set elements can be any type whose
`==` is automatic (ints, strings, chars, structs and enums of them), but not
floats.

## JSON

`Json` is built in, with no import (`json.type.1`). It is read only when the
program uses it.

```ail skip=context
doc := parse_json("\{\"a\": [1, \"x\"]}")?   # Json ! JsonError; err.at is the char offset
doc.get("a")?.at(1)?.as_str()                  # str?  (get, at, as_str/as_num/as_bool/as_arr/as_obj)
doc.str("id")                                  # str?: also num, bool, arr (a member of that kind)
doc.set("id", Json.Str("x"))                   # replace or add a member (mut doc)
doc.to_text()                                  # {"a":[1.0,"x"]}, compact, members in document order
```

Variants: `Null`, `Bool(b)`, `Num(n float)`, `Str(s)`, `Arr(items)`,
`Obj(members)` (`JsonMember` has `key`, `value`). Write `Json.Num(1.0)` when
the type isn't known. A program that declares its own `Json` keeps it.

Derive the conversions of your own types (`json.derive.1-2`):

```ail skip=context
struct Todo
    id str
    hours float
    tags List[str]
    note str?              # may be missing; `none` writes `null`
    env Map[str, str]      # an object; lists, maps, optionals nest freely

impl ToJson for Todo       # t.to_json() Json: an object of the fields in order
impl FromJson for Todo     # Todo.from_json(j) Todo ! JsonError
impl Default for Todo      # Todo.default(): 0, "", false, [], {}, none per field

t := Todo.from_json(doc)?   # err.message names the path: "missing tags[2]", "id: expected a string, found number"
doc.need("id")?.need_str()?  # the same checks by hand: need, need_str/num/int/bool/arr/obj
```

Enums: a variant without fields is its name (`"Red"`), one with fields
`{"kind": "Circle", "r": 2.0}`. Generic types derive both with bounds
(`impl FromJson for Pair[A FromJson, B FromJson]`).
`ail api --expanded FILE` prints the code a derive generated.

## Values and `mut`

Assigning or passing a value copies it (cheaply): changing `b` after
`b := a` never changes `a`.

```ail skip=context
fn bump(mut p Point)   # the caller sees the change
    p.x += 1

bump(mut q)            # `mut` again at the call; q must be a `mut` variable
```

Two `mut` arguments of one call can't refer to the same value.

## Shared cells, resources and `move`

`ref T` is the one way to share a value: copies of a reference see each
other's changes, and fields and methods reach the contents directly.

```ail skip=context
root := ref Node(value: 1, children: [])
alias := root
alias.value = 2        # root.value is 2; any copy may change the cell
print(root == alias)   # true: same cell (`==` on refs is identity)
```

A `unique` type can't be copied: binding, assigning, returning or storing it
moves it, and using the old name afterwards (on any path) is an error. Its
`impl Drop` runs once, when its last owner's block ends, even on a trap.
`defer S` runs `S` when the block ends; cleanups and defers run last first.

```ail
unique struct File
    fd int

impl Drop for File
    fn drop(mut self)
        print("closing {fd}")

fn close(move f File)  # takes ownership; the caller writes `move` too
    print("bye {f.fd}")

f := File(fd: 3)
close(move f)          # f can't be used after this
```

A `move self` method consumes its receiver; the call has no mark, and the
receiver can't be used after it (own.move.6):

```ail skip=context
fn File.finish(move self) int
    fd

n := f.finish()        # f is moved
```

Lists and maps of a unique type are unique too (own.uniq.8-10).
`xs.push(move x)`, `insert(i, move x)` and `extend(move ys)` move in;
`pop()`, `remove_at(i)` and `m.remove(k)` move out; `xs[i]`, `m[k]` and
`for x in xs` borrow in place. `first`, `last`, `min`, `max`, `sorted`,
`take`, `get` and `values` would copy, so they are errors. A unique value
still can't be a map key, set element, `ref` contents or type argument;
task handles go in a `TaskGroup`, not a list.

## Modules

A module is a directory of `.ail` files sharing one namespace. `use a.b`
imports `ROOT/a/b`, where ROOT is the directory run (`ail run DIR`) or the
file's directory (`ail run FILE`, whose siblings are not included).

```ail skip=context
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

A package root may hold `ail.pkg` (spec `mod.pkg.*`). Each `dep` is imported
by its name (`use geo`, `use geo.shapes`) and may reach only the
capabilities in its `caps` (none if omitted); `caps` on the `package` line
limits the package itself. Violations are compile errors at the line.

```
package notes caps fs
dep geo path ../geo caps fs
dep textkit path vendor/textkit
dep json git "https://example.com/json.git" tag v1.2 caps fs
```

A `git` dep needs `rev COMMIT` or `tag TAG`; `ail run`/`test` fetch it into
`~/.ail/deps` (or `$AIL_HOME/deps`) and pin its commit in `ail.lock` (commit
it). `ail pkg update [NAME]` re-pins; `--deterministic` rejects a stale lock,
`--deny net` fetches nothing (`mod.pkg.6–9`).

## Files, processes and the system

The standard modules are in scope without `use`: call them by name,
`fs.read_text(p)` (spec 17-std, ADR 0277). A local, or the program's own item,
with a module's name hides it; `use std.fs as files` still works.
Failures are values: `fs.IoError(path, kind)` and
`proc.ProcError(cmd, kind)`, `kind` one of `not_found`, `permission`,
`exists`, `is_dir`, `not_dir`, `not_empty`, `invalid_utf8`, `timed_out`, `other`;
`net.NetError(url, kind, message)`.

```ail
fn main() !
    text := fs.read_text("in.txt")?          # str ! fs.IoError
    fs.write_text("out.txt", text)?           # also append_text
    names := fs.list_dir(".")?                # sorted names
    srcs := fs.glob("src/**/*.ail")            # also fs.walk(dir)?: every file, sorted
    r := proc.run("git", ["status"])?         # no shell; fails only if it can't start
    print("{r.status} {r.out} {r.err}")      # proc.Output
    s := proc.run("sort", [], text, 5000)?     # input, then a time limit (kind timed_out)
    c := proc.start("make", [])?              # returns at once; c.stdin, c.out, c.err are Channel[str]
    i := task.select([c.out, c.err])          # index of a channel with a value; none when all closed
    for line in c.out                         # lines as they arrive; ends when the stream does
        print(line)
    print(proc.wait(move c)?)                 # status; `kill(move c)` ends it (a unique value)
```

| Module | Functions |
|---|---|
| `std.fs` | `read_text(p) str`, `write_text(p, s)`, `append_text(p, s)`, `exists(p) bool`, `list_dir(p) List[str]`, `remove(p)` (file or empty dir), `make_dir(p)` (with parents), `walk(dir) List[str]` (files, any depth), `metadata(p) fs.Metadata` (`size`, `modified_ms`, `is_dir`), `rename(a, b)`, `copy(a, b)`; all but `exists` are `! fs.IoError`; `glob(pat) List[str]` (`*`, `?`, `**`) never fails |
| `std.compress` | `gzip(s) List[int]`, `deflate(s)`, `zlib(s)`: bytes as ints 0-255; `gunzip(b)`, `inflate(b)`, `unzlib(b)`: `str ! compress.CompressError { kind, message }`; `gzip_file(src, dst)`, `gunzip_file`, `zip_dir(dir, zipfile)`, `unzip(zipfile, dir)`: `! CompressError`, effects `fs`, `io`; others no effects |
| `std.hash` | `sha256(s)`, `sha384(s)`, `sha512(s)` (hex `str`), `hmac_sha256(key, s)` (hex), `crc32(s) int`, `hash64(s) int` (fast, not secure); no effects |
| `std.env` | `args() List[str]`, `get(name) str?`, `vars() Map[str, str]`, `cwd() str`, `program() str`, `home() str?` |
| `std.path` | `dir(p)` (`.` if none), `base(p)`, `ext(p)` (no dot), `join(a, b)`; text only, no effects |
| `std.io` | `read_line() str?`, `read_all() str`, `eprint(value)`, `exit(code)` |
| `std.proc` | `run(cmd, args, input?, timeout_ms?) proc.Output ! proc.ProcError`; `Output { status int, out str, err str }`; `which(name) str?`; `start(cmd, args, timeout_ms?) proc.Child ! proc.ProcError`, `wait(move c) int ! proc.ProcError` (`timed_out`), `kill(move c) int`; `Child { pid int, stdin, out, err Channel[str] }` (`stdin.send(text)`, `stdin.close()`) |
| `std.net` | `get(url)`, `post(url, body, content_type)`: `net.Response ! net.NetError`; `Response { status int, headers Map[str, str], body str }`; any status succeeds; `http://` and `https://` (TLS 1.3, certificates checked against the system roots or `SSL_CERT_FILE`; failures have kind `tls`) |
| `std.proc` | `run(cmd, args, input?, timeout_ms?) proc.Output ! proc.ProcError`; `Output { status int, out str, err str }`; `which(name) str?` |
| `std.net` | `get(url)`, `post(url, body, content_type)`: `net.Response ! net.NetError`; `Response { status int, headers Map[str, str], body str }`; any status succeeds; `http://` and `https://` (TLS 1.3, certificates checked against the system roots or `SSL_CERT_FILE`; failures have kind `tls`); `request(method, url, opts)` with `mut o := net.options()` (`o.headers`, `o.query`, `o.body`, `o.timeout_ms`, `o.max_redirects`); server: `srv := net.serve(addr)?` (`addr()`, `close()`), `ex := srv.accept()?` (`ex.request`: `method`, `path`, `query`, `headers`, `body`, `peer`), `ex.respond(net.Response(status: 200, headers: {}, body: "hi"))?`; run workers with `spawn`; raw TCP: `net.listen(addr)?`, `net.connect(host, port)?`, `net.connect_tls(host, port)?`, `Conn`: `read(max)`, `read_line()`, `write(text)`, `set_timeout(ms)`, `close()`; kinds also `bad_request`, `closed`, `bind` |
| `std.random` | `int(lo, hi)` (both inclusive), `float()` in [0, 1), `seed(n)`, `shuffle(mut xs)` |
| `std.tensor` | `from(xs)` (1-D), `from(xs, shape)`, `zeros(shape)`: `tensor.Tensor[int]` or `[float]`; `a + b`, `a - b`, `a * b`, `a / b` (float only; elementwise, or with a scalar on either side: `t * 2.0`), `-t`; methods `add(b)`, `mul(b)`, `dot(b)`, `sum()`/`max()`/`mean()` (or `(axis)` for a tensor without that axis), `relu()`, `sigmoid()`, `tanh()` (float); `transpose()` (2-D), `matmul(b)` (2-D; pooled at 64³), `map(\|x\| e)`, `reshape(shape)`, `shape()`, `len()`, `to_list()`; shape mismatch and `int` overflow trap; vector kernels (AVX2, NEON) |
| `std.tensor` | `from(xs)` (1-D), `from(xs, shape)`, `zeros(shape)`: `tensor.Tensor[int]` or `[float]`; `a + b`, `a - b`, `a * b` (elementwise, or with a scalar on either side: `t * 2.0`); methods `add(b)`, `mul(b)`, `dot(b)`, `sum()`/`max()`/`mean()` (or `(axis)` for a tensor without that axis), `transpose()` (2-D), `matmul(b)` (2-D; pooled at 64³), `map(\|x\| e)`, `reshape(shape)`, `shape()`, `len()`, `to_list()`; shape mismatch and `int` overflow trap; vector kernels (AVX2, NEON) |
| `std.regex` | `compile(pat) regex.Regex ! regex.RegexError` (`kind`, `at`, `rule`, `message`); `is_match(re, s)`, `find(re, s) Match?`, `find_all(re, s)`, `captures(re, s) Captures?`, `replace(re, s, rep)` / `replace_first` (`$1`, `${name}`, `$0`, `$$`; unknown group traps), `split(re, s)`, `escape(s)`; RE2 syntax (`\d \w \s \b`, `\p{L}`, `(?i)`, lazy `*?`, `(?P<n>..)`), linear time, no backreferences or lookaround; offsets in characters; write patterns as `r"..."` |
| `std.time` | `now_ms() int`, `sleep_ms(ms)` |
| `std.time.date` | `date.utc(ms)`, `date.at(ms, zone) ! DateError`, `date.now()`, `date.now_local()`, `date.parse_iso(text)` (offset required) and `date.parse_in(text, zone)` `: DateTime ! DateError`, `date.parse_date(text) Date`, `date.Date.new(y, m, d) Date?`, `is_leap(y)`, `days_in_month(y, m)`; durations are `int` ms (`date.HOUR`); a zone is `"UTC"`, `"+05:30"` or `"America/New_York"`. `Date`: `days()`, `weekday()` (1 = Monday), `yday()`, `add_days/months/years(n)` (months clamp the day), `since(d)`, `iso()`, `format(p)`, `<`. `DateTime` (`date`, `hour`, `minute`, `second`, `ms`, `offset`, `zone`): `unix_ms()`, `add_ms(n)` (elapsed), `add_days/months/years(n)` (wall clock), `in_zone(z)`, `before(o)`, `since(o)`, `iso()`, `format("%Y-%m-%d %H:%M:%S %z")` (strftime: `%Y %y %m %d %e %j %H %I %p %M %S %f %a %A %b %B %z %:z %Z %s %%`) |
| `std.tz` | `offset(zone, secs) int?` (none for an unknown zone), `system() str`; effects `fs`, `env` |
| `std.ui` | `open(title, cols, rows) ui.Window ! ui.UiError`; `Window`: `size() ui.Size`, `clear(bg)`, `put(col, row, text, fg, bg)`, `put_styled(col, row, text, fg, bg, ui.Style(bold, italic, underline))`, `cursor(col, row)`, `set_cursor(col, row, Bar|Block|Underline)`, `present()`, `next_event() ui.Event`, `poll_event() ui.Event?`, `modifiers() ui.Modifiers` (of the last event), `set_font_size(px)` (10 to 40; a `Resize` follows), `font_size() int`, `set_title(t)`, `close()`; `Event`: `Key(key, ctrl, alt, shift)`, `Resize(cols, rows)`, `Close`, `Mouse(col, row, button, kind)`, `Scroll(lines)`; `clipboard_get() str?`, `clipboard_set(text)`; colors `0xRRGGBB` or `ui.BLACK`, `BLUE`, `CYAN`, `GRAY`, `GREEN`, `MAGENTA`, `ORANGE`, `RED`, `WHITE`, `YELLOW` |
| `std.ui.widgets` | `TextField`, `ListView` (fuzzy `filter`), `Button`, `Dialog`, `Pane`, `Tree`: `render(rect, theme) List[widgets.Run]`, `draw(mut w, rect, theme)`, `on_event(ev[, rect]) widgets.Action` (`Ignored`, `Changed`, `Submit`, `Cancel`); `widgets.Rect(col, row, cols, rows)`; `text_rows(runs, cols, rows)` prints a screen headless |

A window is a grid of text cells; draw with `put`, then `present`:

```ail
fn main()
    mut w := ui.open("Demo", 40, 10) else return   # fails headless: kind "no_display"
    w.put(1, 1, "Hello", 0xFFFFFF, 0x000000)
    w.present()
    while true
        match w.next_event()
            Key(key, ctrl, _, _) =>                # "a", "enter", "f1"; Ctrl-S is ("s", true)
                if key == "escape"
                    break
            Close => break
            Resize(_, _) | Mouse(_, _, _, _) | Scroll(_) => w.present()
```

## Tests and assertions

```ail skip=context
@test fn doubles()           # `@test` on the same line
    assert_eq(double(4), 8)  # traps showing both values if unequal
    assert(double(1) == 2)   # traps if false
```

A test returns nothing or `!`, can't be called, and is ignored by `ail run`.
It can always fail, so `?` works in it: `assert_eq(parse("7")?, 7)` fails the
test if `parse` fails or gives `none`.
`assert` and `assert_eq` also work in ordinary code (a failure traps).

A test with parameters is a **property**: `ail test` runs it on generated
inputs (`--cases N`, default 100; `--seed S` replays), and shrinks a failure
to the smallest input, printed as AIL literals. Parameters may be `int`,
`float`, `bool`, `char`, `str`, `List`, `Map`, `Set`, optionals, and structs
and enums made of those.

```ail fragment
@test fn add_commutes(a int, b int)
    assert_eq(a + b, b + a)
```

A block fenced `ail` in a `##` doc comment runs as the test `doc:NAME`;
`# => text` after a `print` asserts its output:

```text
## Doubles a number.
##
## ```ail
## print(double(4))
## # => 8
## ```
fn double(x int) int
```

## Tasks and channels

```ail skip=context
t := spawn load(1)            # runs load(1) as a task; args are copies
p := t.wait()?                # its result; fails with Trapped if it trapped
g := TaskGroup()              # tasks that return nothing
g.add(spawn work(2))          # waited when g's block ends, or:
g.wait_all()?
ch := Channel[int](8)         # bounded: send waits while full
ch.send(1)
v := ch.recv() else break     # none once closed and empty
for v in ch                   # receives until closed and empty
    print(v)
all := [v for v in ch]        # also in comprehensions
ch.close()
m := Mutex(0)                 # shared state: get, set, swap
with m as mut x               # read-modify-write: holds m, stores x back
    x += 1                    # on any exit; a trap keeps the old value
n := Atomic(0)                # int or bool, shared by tasks and par for
n.add(1)                      # returns the old value; also load, store, cas
g.cancel()                    # or t.cancel(): ask tasks to stop
task.cancelled()              # in the task: was it cancelled?
task.check()?                 # fails with Cancelled once cancelled
n.add(1, order: Relaxed)      # weaker ordering: Relaxed Acquire Release AcqRel
```

A cancel reaches every task the cancelled task started. A cancelled task's
`recv` gives `none` (and `for v in ch` ends), a `send` on a full channel
drops its value, `time.sleep_ms` returns at once and `proc.run` fails; the
scope still waits for it. `ail test --schedule=random:SEED` runs tasks in an order picked from
`SEED` and prints the seed when a test fails, so the interleaving replays.

Each task handle must be waited or added to a group exactly once, on every
path. A handle can't be stored or passed; channels and mutexes can, and
copies are the same channel or mutex. If every task is blocked, the program
traps (deadlock).

## Effects

Effects are inferred, never written: `io` (`print`, `fs`, `io`, `proc`),
`fail` (`T ! E` result), `mutates` (`mut` parameter), `spawn`, and the capabilities
`fs`, `env`, `proc`, `clock`, `random`, `net`, `net.serve` (`net.serve`, `net.listen`), `ui`. A call through `any Trait` counts every
implementation.

```ail
@pure fn area(w int, h int) int   # error if it reaches print or a capability
    w * h
```

`@inline` is a speed hint, accepted and ignored for now. `@deprecated("use
area")` before a function warns at each call, with the reason (required).
Unknown attributes are errors.

## Unsafe

`unsafe` and a block allow unsafe operations; `unsafe fn` can be called only
there. The block yields its last expression and may `return`. A package
needs `unsafe allow` on its `package` or `dep` line in `ail.pkg`; a program
without `ail.pkg` needs nothing. A function with an `unsafe` block is not `@pure`.

```ail
unsafe fn twice(x int) int
    x * 2

fn safe_twice(x int) int
    unsafe
        twice(x)
```

## Contracts

Lines directly under the signature; conditions are pure `bool`s. `result` is
the returned value in `ensures`. A failed `requires` traps at the call
(`contract.req.1`), a failed `ensures` at its line (`contract.ens.1`), both
with the arguments' values. `ail run --no-contracts` skips the checks.
`old(e)` in `ensures` is `e` on entry. A trait method's contracts bind its
implementations (their own `requires` weaken, their own `ensures` add). A
struct's `invariant COND` lines hold after construction and after every
`mut self` method (`contract.inv.2`, `contract.inv.3`).

```ail
fn divide(a int, b int) int
    requires b != 0
    ensures result * b + a % b == a
    a / b

struct Account
    balance int
    invariant balance >= 0

fn Account.add(mut self, n int)
    ensures balance == old(balance) + n
    balance += n
```
