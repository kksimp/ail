# AIL quick reference

> **Non-normative.** One page for people and AI agents writing AIL. The
> [spec](spec/) is authoritative. Items marked **(not yet)** are specified but
> not built in the native compiler yet.

## Running programs

```sh
./ail run [ARGS...] FILE.ail        # DIR runs a package; words after FILE are env.args()
./ail check [--json] FILE|DIR       # clean: one `ok FILE: N functions, M warnings` line, exit 0, then the tests an edit can affect run: `tests: 2 run, 2 pass (40 cached)`, a failing test is exit 1 (`--no-tests`: check only); errors: exit 2, each ends `next: ail fix FILE` or `next: ail explain RULE`
./ail build FILE.ail [-o OUT]       # native executable
./ail test [--json] FILE.ail        # @test functions; also --filter PATTERN, --list, --fuzz NAME, --mutate, --contracts
                                    # on an edited file it also prints `changed behaviour: f(x: 0) was 1, now 2`: old against new on generated inputs
./ail fix [--dry-run] FILE.ail      # apply the fixes diagnostics carry; check/build/run/test write certain fixes themselves (`--no-fix`, `AIL_FIX=0` to stop)
./ail fmt [--check] FILE.ail
./ail explain [RULE]                # a rule's meaning, ADRs and example; no RULE lists all
ail card [TOPIC]                  # the core card, `full` for all of it, or one topic (`std.fs`, `errors`); `ail ask std WORDS` finds a std function
./ail api [--json] [--card] FILE|DIR     # a program's public signatures
./ail ask FILE who writes field F   # also: who reads F, what calls f, what does f call, which effects reach f, who uses T
./ail context FILE f --budget 800   # what an edit to f needs, cut to 800 tokens
./ail edit FILE.ail [--test] < EDITS    # edit by declaration (also change_sig, extract, move)
./ail task start FILE GOAL...       # then: task verify | status | finish
./ail new [--no-agent] NAME         # a package with a first program, AGENTS.md and .mcp.json
./ail mcp                           # the compiler's tools as an MCP stdio server; `ail lsp` is a language server
```

`ail help [CMD] [--json]` lists every command with its usage; `ail doctor` checks the toolchain.

Options for each command are in [semantic-api.md](design/semantic-api.md) and the spec chapter for it (`ail explain` names it).

With `--json`, each diagnostic is one line of schema v1 (`"api":1`), with a
span and, where the fix is obvious, `fixes` to apply by line and column.

`todo` marks code not yet written: it has the expected type; `ail check` warns (`expr.hole.1`) and running it
traps (`expr.hole.2`). `uhwhatnow?` is the same hole, and its warning ranks what in scope fits (`expr.hole.3`).

A program needs `fn main()`. `print(value)` writes any value and a newline.

## Native builds

`ail run` builds natively and runs; `./ail build FILE.ail` writes a static executable. A rare construct the lowering cannot build yet is reported as
`error[native.subset.1]` with a `help:` line. `--target x86_64-linux`, `aarch64-linux`, `aarch64-macos`; an unchanged program rebuilds
from `~/.cache/ail/build` (`--no-cache` skips it; native.build.5). Guide: [design/native-guide.md](design/native-guide.md).

## Layout

- A statement ends at its line's end, unless the line ends inside `(`, `[`
  or `{`, or after a binary operator or comma. `#` comments, `##` doc comments.
- A line starting with `.name` continues the previous statement.
- Layout is what `ail fmt` writes ([formatting.md](design/formatting.md)).

## Functions

```ail
fn add(a int, b int) int     # `name Type`, no colon; return type after `)`
    a + b                    # the last expression is the result

fn greet(name str)           # no return type: returns nothing
    print("hello, {name}")
```

`return x` (or bare `return`) exits early.

`fn usage(why str) never` declares a function that never returns: its body ends in `io.exit`,
`unreachable` or another `never` call, `return` is an error in it, and a call fits where any value
is expected: `n := s.to_int() else usage("N")` (type.never.1 to 3, ADR 1119).

## Bindings

```ail
x := 1           # `:=` declares a new name (immutable)
mut n := 0       # mutable
n = 5            # `=` changes an existing `mut` name
n += 1           # also -= *= /= %=
```

## Types

`int`, `float`, `bool`, `str`, `bytes`, `char`, `List[T]`, `Map[K, V]`, `Set[T]`, tuples `(int, str)`, structs, enums, `T?`, `T ! E`, generics, `any Trait`.

A tuple `t := (1, "a")` is an `(int, str)`: read `t.0`, take it apart with
`n, s := t` (`_` skips; `a, mut b := t`), match arms like `(0, _) =>` and
`(k, w) =>`. `==`, `<`, map keys and JSON (arrays) work elementwise.

Constants are top-level `NAME := value`, computed at compile time by calls
with no I/O or capability (`ct.const.2`); one that can fail needs `else`, and
`if`, `match` and `?` go in a called function:

```ail
MAX_DEPTH := 64
KEYWORDS := {"if", "else", "while"}     # a Set[str]
SQUARES := squares(8)

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
    y int = 0                     # a default: Point(x: 1) is Point(x: 1, y: 0)

enum Color
    Red
    Green

fn Point.sum(self) int            # method; `self` is read-only
    self.x + self.y               # canonical; bare `x + y` works too

fn Point.shift(mut self, dx int)  # may change self
    self.x += dx

fn Point.origin() Point           # associated function: Point.origin()
    Point(x: 0, y: 0)

fn Point.scaled(self, k int = 2) Point   # a default: p.scaled() or p.scaled(k: 3), by name only
    Point(x: x * k, y: y * k)

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

Positional arguments work when no two fields they fill share a type; `Point(1, 2)` is an error when `x` and `y` are both `int`.

A bare variant works wherever the type is known (arguments, returns, fields,
arms, `c == Red`). Every value compares with `==` and prints: `Point(x: 1, y: 2)`.

Variants can carry fields, matched by position:

```ail
enum Shape
    Circle(r int)
    Rect(w int, h int)

fn area(s Shape) int
    match s
        Circle(r) => 3 * r * r
        Rect(w, _) => w * w       # `_` skips a field
```

A bare variant pattern matches any fields and binds none (`Circle => true`).
Patterns nest in fields: `Add(Num(a), Num(b)) => a + b`, `Neg(Num(0..=9))`,
`Word("fn", (1, _))`. A bare name in a field binds it, so a variant without
fields is written qualified there (`Cons(_, Items.Nil)`). A struct is matched by
field name, `Point(x: 0, y) =>`: a bare field name binds that field and
unnamed fields match anything; `Rect(w: 0)` names a variant's fields.

A distinct type wraps a built-in scalar so IDs and units can't be swapped
(type.distinct.1-7): `type UserId = int`, made by `UserId(42)`, read back by
`int(id)`; it has its base's operators, text and JSON, and costs nothing at run time.

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

- A function may end with `s.to_int() else return BadNumber(text: s)`: as a block's last line, `else return` gives the value (`err.else.7`).
- A failing result must be handled: `?`, `else`, or returned. `parse(s)` alone
  on a line is an error.
- `x := ... else ...` gives a fallback or leaves (`return`, `continue`, `break`).
  To handle both outcomes and go on: `if x := ...` with `else err`; `if x := e`
  also works on optionals and in `elif`.
- `else` works in expressions: `Min => nums.min() else 0`, `(m.get(k) else 0) + 1`,
  `for x in j.arr("items") else []`.
- `?` on an optional works in a function returning an optional, or before an
  `else` that catches the `none`: `j.get("u")?.str("name") else "?"`.
- Inferred-error functions and tests: `?` on `none` fails with the prelude error
  `Absent`, with context: `text.to_int()? "port {text}"`. `T ! E` needs `else`.
- `error struct NotFound "nothing at {path}"` gives a struct a message.
- `T ! str` fails with plain text: `fn parse(s str) int ! str`, `return "bad input"` (any other `return` is the value);
  `?` passes it on and a handler binds the text (err.str.1-3, ADR 1129).
- `x = e else` takes an indented block like `x := e else` (err.else.8, ADR 1128).
- Failing with text and no result (or a `str` result): `fn f() ! Fail`,
  `return Fail(msg: "...")`, the prelude's error type (err.fail.1).
- Several error types (`! IoError | ParseError`): `match err` takes member types
  and their variants, and covers every member (err.match.1-3): `IoError =>`
  (`err` is an `IoError` there), `ParseError.Empty =>`, `_ =>`.
- `unreachable("why")` stops the program; it fits any type.

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

fn largest[T Ord](items List[T]) T   # `T`: any type with `impl Ord for T` (no methods: compares fields in order)
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

struct Scene
    shapes List[any Shape]           # `any`: values of different types

fn main()
    c := Circle(r: 2)
    print(c.describe())              # shape: 12
    print(largest([3, 9, 4]))        # type arguments are inferred...
    print(first[int]([]))            # ...or written when nothing gives them
    scene := Scene(shapes: [c, Square(side: 3)])
    for sh in scene.shapes
        print(sh.describe())
    print(bigger(c, Square(side: 1)))
    print(Shape.area(c))             # the qualified form of `c.area()`
```

- `< <= > >=` work on `int`, `str`, lists, and types with `impl Ord for T`
  (fields in order, or your `fn cmp(self, other Self) Ordering`).
- Customise: `impl ToStr` (`fn to_str(self) str`), `impl Eq` (`fn eq`).
- Arithmetic operators on your type: `impl Add for V` with `fn add(self, o V) V` gives `a + b` and
  `a += b`; also `Sub`, `Mul`, `Div`, `Rem` and `Neg` (`fn neg(self) V`, unary `-`). Both operands
  have the same type; built-in numbers keep their own operators (trait.op.1, ADR 1124).
- A generic type: `struct Stack[T]` with `fn Stack.new() Self` and `fn Stack.push(mut self, x T)` (`Self` is `Stack[T]`); supertraits `trait Sized Ord + Zero`; a bound on one method: `fn Stack[T Ord].max(self) T?`.
- On an `any Shape`, only methods whose parameters don't use `Self` are callable.

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
sign := if n < 0 then "-" elif n == 0 then "0" else "+"   # one line: `then`, any `elif` arms, a final `else`
pad := 2 + (if n < 0 then 1 else 0)  # a one-line `if` as an operand: in parentheses

while n > 0
    n -= 1
while top := xs.pop()      # loops while the T? or T ! E has a value, binding it
    print(top)

for x in xs          # also: for i in 0..n (excludes n), for i in 1..=n
    if x == 0
        continue
    if x > 9
        break

outer: for r in rows       # a label: `break outer` / `continue outer` from a nested loop
    for c in r
        if c == 0
            continue outer

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

### Generators and `Seq` (expr.yield.1-4, expr.iter.1-3)

```ail
fn count(n int) Seq[int]             # `yield` in the body makes it a generator
    for i in 0..n
        yield i                      # the next element; a bare `return` ends it

fn main()
    for i in count(3)                # runs the body up to each `yield`
        print(i)
    print(count(10).filter(|x| x % 2 == 0).map(|x| x * x).take(3).collect())   # [0, 4, 16]
    mut s := count(2)
    print(s.next() else -1)          # 0
```

A `Seq[T]` is unique: `for` and the adapters take it over, so it is read once.
Adapters (lazy): `Seq.of(xs)`, `map`, `filter`, `take_while`, `take`, `skip`,
`enumerate`, `zip`, `chain`; `collect()` and `count()` read to the end.
`Seq(step: f)` wraps any `fn() T?`. A generator keeps copies of its arguments,
so it takes no `mut` parameters, `mut self`, `defer` or contracts.

## Operators and strings

- `+ - * / %` on `int`. Overflow and division by zero stop the program (a
  trap). `/` rounds toward zero.
- `== != < <= > >=` (see above for which types). Comparisons can't be
  chained: write `a < b and b < c`.
- `and`, `or`, `not` (not `&&`, `||`, `!`). Conditions must be `bool`: write
  `if n != 0`, not `if n`.
- `float`: `+ - * /` (no `%`), never traps (`1.0 / 0.0` is `inf`). Math methods (`std.math`, type.num.16): `x.sin()`, `cos`, `tan`, `atan`, `exp`, `ln`, `log10`, `cbrt`; `x.atan2(y)`, `x.pow(y)`, `x.hypot(y)`, `x.log2()`; `x.is_nan()`, `is_finite()`, `is_infinite()`; constants `float.PI`, `E`, `TAU`, `INF`, `NAN` (type.num.18). `int`: `a.gcd(b)`, `lcm(b)`, `n.isqrt()`, `is_pow2()`, `n.log2()` (type.num.17). No mixing
  with `int`: `n.to_float()`, `x.to_int()` (truncates; traps on NaN); an int literal works where a float is expected.
  `2.5.round()` is `3.0` (halves away from zero). Floats print like Python: `0.1 + 0.2` is `0.30000000000000004`.
- Sized numbers `i8 i16 i32 u8 u16 u32 u64 f32` (`i64` is `int`, `f64` is `float`). A literal takes its
  type from context and must fit. Operators need two values of one type and trap on overflow; no implicit
  widening. Convert with `x.to_u8()` (traps if it doesn't fit), `to_int()`, `to_float()`, `to_f32()`;
  `wrapping_add`, `checked_add` (`int?`) and `saturating_add` work on every integer. `n.abs()`, `min`, `max`, `n.to_hex()`.
- Bits on `int` (64-bit two's complement): `& | ^ ~ << >>` (`>>` arithmetic; `k` in `0..=63`, else trap `expr.bit.2`);
  `x &= e` works on a plain name. A bit operator never mixes with another operator without parentheses
  (`expr.bit.4`): write `(x & 1) == 0`, `(a + b) & m`.

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
print("{n:>6}|{r:.2}|{n:08}|{n:b}|{n:,}|{r:e}|{n:+}|{s:^8}|{s:*<5}")  # width, decimals, zero pad, binary, thousands, exponent, sign, fill, align
```

- Format specs: `[[fill]align][+][0][width][,][.decimals][x|b|e]` only (the card lists them).
- `"a,b".split(",")` gives `["a", "b"]`, `"k=v=w".split("=", 2)` gives `["k", "v=w"]`; `"12".to_int()` gives an `int?`,
  `"2.5".to_float()` a `float?`.
- `s.chars()` (a `List[char]`; strings have no indexing or `for`), `s.len()` (characters; `s.byte_len()` is UTF-8 bytes),
  `lines`, `words`, `trim`, `trim_start`, `trim_end`, `contains`, `starts_with`, `ends_with` (`x` a `str` or `char`),
  `strip_prefix(p)` (`str?`), `repeat(n)`, `pad_left(width, ' ')`, `pad_right`, `center`, `count(sub)`, `find(x)` (`int?`),
  `rfind`, `slice(a, b)` (characters), `char_at(i)` (`char?`), `replace(a, b)`, `to_lower`, `to_upper`.
- `xs.join(", ")` joins any list. `char`: `is_digit()` (ASCII), `is_alpha()`, `is_space()`, `to_upper()`, `to_digit(10)` (`int?`), `code()`, `char.from_code(n)`. Padding never cuts; a whole `float` prints `2.0`.
- `bytes` is immutable binary data (ADR 0412): `b"GET /\r\n\x00"`, `s.to_bytes()`, `b.len()`, `b[i]` (a `u8`), `b[a..b]`, `a + b`,
  `for x in b`, `b.to_str()` (`str?`), `b.text()` (lossy), `find`, `hex()`, `base64()`, `bytes.from_hex(s)` (`bytes?`). No `b[i] = x`.
- Binary data (type.bytes.15-18): `b.u32_le(at)`, `b.i16_be(at)`, `b.f64_le(at)` and their kin read a number at an offset (`int`; `u64_*` a `u64`, `f32_*`/`f64_*` a `float`), `b.uvarint(at)` is `(value, length)?`; build with `mut xs := List[u8]()`, `xs.push_u32_le(x)` (same names with `push_`), `xs.push_bytes(b)`, `xs.push_uvarint(x)`, then `bytes.from_list(xs)`.

## Lists

```ail
mut xs := [3, 1, 2]    # a trailing comma is allowed; `[]` needs a known type
first := xs[0]         # out of bounds stops the program
part := xs[1..3]       # a copy of elements 1 and 2; also xs[1..] and xs[..2]
xs.push(4)             # push, pop, sort and swap need a `mut` variable
xs[0] = 7              # elements can be assigned, including fields: xs[i].qty = 0
n := xs.len()          # int
mut out := []          # typed by its first use: out.push("a") makes a List[str]
out.push("a")
empty := List[int]()   # an empty list of a given type, anywhere
```

Also: `slice(a, b)` and `slice(a)` (the same as `xs[a..b]`, `xs[a..]`), `insert(i, x)`, `remove_at(i)`, `extend(ys)`, `clear()`, `reverse()`,
`pop()`/`first()`/`last()`/`min()`/`max()` (`T?`), `contains(x)`, `index_of(x)`
(`int?`), `sorted()` (copy) and `sort()` (in place), `sum()`,
`take(n)`, `chunks(n)`/`windows(n)`, `counts()` (`Map[T, int]`),
`flatten()`, `enumerate()` (`for i, x in xs.enumerate()`), `zip(xs, ys)` (pairs, to the shorter);
`is_empty()` on strings, lists, maps and sets. Slices don't clamp; negative indexes are errors: use `xs.last()`.

Sort or pick by a key with `by: |x| key` (on `sorted`, `sort`, `min`, `max`;
stable); a tuple key sorts by several:

```ail fragment
counts := {"b": 2, "a": 2, "c": 5}
top := counts.keys().sorted(by: |k| (-counts[k], k)).take(2)   # ["c", "a"]
```

```ail skip=context
for mut item in items      # `items` must be `mut`
    item.qty += 1          # changes the element
```

Inside a `for mut` loop the body can't use the list itself.

`Grid[T]` (flat 2-D list), `Heap[T]` (priority queue) and `Deque[T]` are built in: `Grid.fill(rows, cols, v)`,
`g[i, j] += 1`; `Heap[int]()` with `push`, `pop`, `peek`; `Deque` with `push_front`, `push_back`, `pop_front`, `pop_back`.
The card lists every method.

A comprehension builds a list, set or map in one expression:

```ail fragment
xs := [3, 1, 4]
squares := [x * x for x in xs]             # [9, 1, 16]
big := [x for x in xs if x > 2]            # [3, 4]
seen := {x % 2 for x in xs}                # a Set[int]
sizes := {w: w.chars().len() for w in ["a", "bc"]}  # a Map[str, int]
```

Loop variables end with it; `?` and `return` leave the function, `break` ends it.

Independent loops use every core automatically (no output, no
`break`/`return`/`?`; writes only to locals, `out[i]` or an `int` total with
`+=`; float totals use `xs.sum()`). `par for` forces one the checker can't
prove; a parallel map is `[f(x) par for x in xs]`.

## Maps and sets

```ail
mut ages := {"ann": 31, "bob": 27}     # Map[str, int]; keeps insertion order
ages["cy"] = 40                        # inserts or replaces
ages["ann"] += 1                       # `op=` on a missing key starts from 0 (number values only)
mut counts := {}                       # typed by first use: counts[w] += 1
counts["x"] += 1
b := ages.get("dan") else 0            # a key that may be missing: `get` gives an optional
a := ages["bob"]                       # traps if missing; test with `ages.contains(k)`
for name, age in ages                  # each entry as a pair; also keys(), values()
    print("{name}: {age}")
mut seen := Set[str]()                 # or a literal: {"a", "b"}
if seen.add("x")                       # true if it wasn't there
    print("new")
```

Also `m.contains(k)`, `m.remove(k)` (`V?`), `m.len()`, `s.remove(x)`, `s.contains(x)`,
`s.sorted()`, `s.to_list()` (insertion order), `s.union(t)`, `s.intersect(t)`, `s.difference(t)` (also on two `keys()`
lists), and `m.push(k, x)` for `Map[K, List[T]]`. Keys and elements: any type with
automatic `==` (ints, strings, chars, structs and enums of them), not floats.

## JSON

`Json` is built in, with no import (`json.type.1`). It is read only when the
program uses it.

```ail skip=context
doc := parse_json("\{\"a\": [1, \"x\"]}")?   # Json ! JsonError; err.at is the char offset
doc.get("a")?.at(1)?.as_str()                  # str?  (get, at, as_str/as_num/as_bool/as_arr/as_obj)
doc.str("id")                                  # str?: also num, bool, arr (a member of that kind)
doc.set("id", Json.Str("x"))                   # replace or add a member (mut doc); also to_text(), text(k, default), flag(k, default)
parse_json_lines("1\nbad\n\"x\"\n")          # List[Json]: every line that parses
load_json[List[U]](path)?                      # a file decoded as T (FromJson); print_json(v) prints v as JSON (json.load.1-2)
```

Variants: `Null`, `Bool(b)`, `Num(n float)`, `Str(s)`, `Arr(items)`,
`Obj(members)`; a program's own `Json` wins. `impl ToJson for T`, `impl FromJson for T`
and `impl Default for T` with no body derive the conversions from the fields
(`json.derive.1-2`): `t := Todo.from_json(doc)?` (`err.message` names the path:
`missing tags[2]`), `t.to_json()`; `doc.need("id")?.need_str()?` checks by hand. Fields
may be lists, maps and optionals (`note str?`; `none` writes `null`). Enums: a bare
variant is its name (`"Red"`), one with fields `{"kind": "Circle", "r": 2.0}`.

## Values and `mut`

Assigning or passing copies a value (cheaply): changing `b` after `b := a`
never changes `a`.

```ail skip=context
fn bump(mut p Point)   # the caller sees the change
    p.x += 1

bump(mut q)            # `mut` again at the call; q must be a `mut` variable
```

Two `mut` arguments of one call can't be the same value.

Your functions' arguments can be named (`show(tag, scale: 3, n: 4)`): positional
first, then named in any order. Built-ins take no names.

A declared function is a value of type `fn(A, B) R` (`f := double`, `f(21)`).
Not generic functions, methods, or ones with `mut` parameters.

A function literal `|a, b| e` is a value too (`|| e` takes nothing). The body
is one expression on the same line. Parameter types come from the expected
`fn(...)` type, or are written: `|x int| x * 2`. It copies the locals its body
names when it is made, and can't change them; to share state, capture a `ref`.
`fn(A) R ! E` is a function that can fail; `?` in a literal leaves the literal.
A `fn` declared inside a function body is a local function: a block body that
copies the outer locals it names, can call itself, and is a value. It isn't
generic, and its parameters aren't `mut`.

```ail
fn adder(n int) fn(int) int
    |x| x + n                         # copies n

fn main()
    xs := [3, 8, 5, 12]
    lo := 4
    print(xs.filter(|x| x > lo))      # [8, 5, 12]
    print(xs.map(adder(1)))           # [4, 9, 6, 13]
    print(xs.any(|x| x > 10))         # true; also all, find (T?), group_by, count, flat_map, skip_while
    print(xs.fold(0, |acc, x| acc + x))   # 28; Set, Map (|k, v|), Deque, Seq take the same names
```

## Shared cells, resources and `move`

`ref T` is the one way to share: copies see each other's changes.

```ail skip=context
root := ref Node(value: 1, children: [])
alias := root
alias.value = 2        # root.value is 2; any copy may change the cell
print(root == alias)   # true: same cell (`==` on refs is identity)
```

A `unique` type can't be copied: binding, assigning, returning or storing it
moves it, and using the old name afterwards is an error. Its `impl Drop` runs
once, when its last owner's block ends, even on a trap. `defer S` runs `S` at
block end, last first.

```ail
unique struct File
    fd int

impl Drop for File
    fn drop(mut self)
        print("closing {fd}")

fn close(move f File)  # takes ownership; the caller writes `move` too
    print("bye {f.fd}")

fn main()
    f := File(fd: 3)
    close(move f)          # f can't be used after this
```

A `move self` method consumes its receiver (own.move.6): `fn File.finish(move self) int`; the call has no mark.

Lists and maps of a unique type are unique too (own.uniq.8-10): `push(move x)`,
`insert(i, move x)`, `extend(move ys)` move in; `pop()`, `remove_at(i)`,
`m.remove(k)` move out; `xs[i]`, `m[k]`, `for x in xs` borrow. `first`, `last`,
`min`, `max`, `sorted`, `take`, `get`, `values` would copy, so they are errors.
A unique value can't be a map key, set element, `ref` contents or type argument.

## Modules

A module is a directory of `.ail` files sharing one namespace. `use a.b` imports
`ROOT/a/b`, ROOT being the run directory (`ail run DIR`) or the file's directory.

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

`pub` exports an item or a field (`pub struct Point`, `pub x int`); everything else is private, and a private
field means only its module can build the struct.

No item or wildcard imports, no `pub use`, no import cycles. `fn T.m` must be in
`T`'s module. Text forms show the path: `geo.Point(x: 1, y: 2)`.

A package root may hold `ail.pkg` (spec `mod.pkg.*`): `package notes caps fs`, `dep geo path ../geo caps fs`,
`dep json git "URL" tag v1.2 caps fs`. A `dep` is imported by name (`use geo`) and reaches only its `caps`
(none if omitted); violations are compile errors. `ail new` writes
`package NAME caps fs=.,fs=@tmp,env,clock,random`: runs reach files under the project only, until `caps` lists more. A `git` dep needs `rev` or `tag`, is pinned in `ail.lock`
(commit it); `ail pkg update [NAME]` re-pins. User guide: [packages.md](packages.md).

## Files, processes and the system

Standard modules are in scope without `use` (spec 17-std, ADR 0277); a local or item with the module's name hides it. Failures are values: `fs.IoError(path, kind)`, `proc.ProcError(cmd, kind)`, `net.NetError(url, kind, message)`. Every `kind` is an enum (ADR 1068): `fs.IoKind` is `NotFound`, `Permission`, `Exists`, `IsDir`, `NotDir`, `NotEmpty`, `InvalidUtf8`, `Other`; `proc.ProcKind` adds `TimedOut`, `Cancelled`; test with `err.kind == NotFound` or an exhaustive `match err.kind` (a string there is type.enum.7, and `ail fix` rewrites it).

Every signature is one command away: `ail card std.MODULE` lists a module's functions, `ail ask std WORDS` finds one by what it does. This table names each module and what is not in its signatures.

| Module | Use, and what the signatures do not say |
|---|---|
| `std.fs` | whole files (`read_text(p)?`, `write_text(p, s)?`, `read_bytes`, `walk`, `glob(pat)`, `copy`, `rename`, `remove`, `make_dir`, `metadata`, `scan`, `temp_dir`); all but `exists`, `glob`, `scan_count`, `temp_dir` are `! fs.IoError`; `trash(p)` is recoverable, never permanent; `sync(p)` survives power loss; `watch(dir)` is a `Channel[str]` of changed paths |
| `std.file` | `unique` handles (never copied): `mut r := file.open(p)?`, `r.read_line()` (`str?`), `r.read_exact(n)` (`bytes?`), `r.seek(pos)`; `mut w := file.create(p)?` (or `append`, `update`), `w.write_line(s)?`; a writer leaving scope is flushed but cannot report a failure, so `w.close()?` |
| `std.mem`, `std.task` | `mem.cstr`, `from_cstr`, `alloc`, `free` for C calls (`unsafe`); `task.threads()`, `task.set_threads(n)`, `task.cancelled()`, `task.check()?` |
| `std.io`, `std.env`, `std.path` | `io.read_line() str?`, `read_all()`, `eprint(v)` (stderr), `exit(code)` (flushes stdout first); `env.args()` is the words after the program; `env.get(name) str?`; `path.join(a, b)`, `base`, `dir`, `ext`, `stem`, `clean`, `parts`, `join_all(xs)`, `with_ext(p, e)`, `split_ext(p)` (a pair), `is_abs`, `rel(base, p) str?`, `match(pattern, p)` (`*`, `?`, `[a-z]`, `**`) are text only; `path.abs(p)` and `expand_home(p)` read the environment |
| `std.args`, `std.log` | `args.parse("tool", "about", specs, env.args())?` or `impl FromArgs for Opts` (`bool` switch, `T?` option, else positional); `a.has`, `a.opt`, `a.int`, `a.float`, `a.opt_int`, `a.opt_float` read typed values; `--help` prints usage and exits 0; `log.info(m)` writes `LEVEL m` to stderr, `LOG_LEVEL` filters; `log.info_kv(m, {"k": "v"})` adds `k=v`; `log.fields(map)` or `log.to_file(path)?` gives a `Logger` with the same methods; `LOG_FORMAT=json`, `LOG_FILE`, `LOG_FILE_MAX`+`LOG_FILE_KEEP` (rotate to `path.1`..), `LOG_TIME` |
| `std.proc`, `std.os` | `proc.run(cmd, args, input?, timeout_ms?)? proc.Output` (`status`, `out`, `err`); `start` gives a `Child` with `stdin`, `out`, `err` channels; `which(name) str?`; `proc.options()` (`cwd`, `env`, `clear_env`, `input`, `data`, `timeout_ms`) for `run_with`, `run_bytes_with`, `start_with`; `proc.pid()`; `os.hostname()`; `os.signals(["INT", "TERM"])` is a `Channel[str]` |
| `std.net`, `std.net.http`, `std.net.jar`, `std.net.web`, `std.net.smtp` | `net.get(url)?`, `post(url, body, content_type)?` give `Response { status, headers, body }`; any status succeeds; `http://` and `https://` (TLS 1.3); `request(method, url, opts)` with `net.options()`; `net.serve(addr)?`, `srv.accept()?`, `ex.respond(...)?`; `std.net.http` handler form (`http.serve(addr, handler)`); router `mut r := http.router()`, `r.get("/users/:id", f)`, `req.param("id")`, `http.json(200, j)`, `spawn serve(r, srv)` with `fn serve(r http.Router, srv net.Server) ! Trapped` calling `r.run(srv)?`; cookies `std.net.jar` (`jar.new()`, `j.get(url)?`); server side `use std.net.web`: `web.cookies(req)`, `web.set_cookie(name, value, web.CookieOpts())?` with `web.with_cookie(resp, line)`, `web.sign(secret, name, v)?` and `unsign`, sessions `web.with_session(web.memory_store(10000), web.SessionOpts(), handler)` with `handler(req http.Req, s web.Session)` and `s.get(k)`, `s.set(k, v)`, `s.regenerate()`, `s.destroy()`, `web.csrf_guard(handler)`, `web.cors(web.CorsOpts(), handler)` with `web.preflight(mut r, opts)`, uploads `web.multipart(req, web.UploadLimits())?` (`form.fields`, `form.file(name)`) and `web.safe_filename(name)`; raw TCP `net.connect(host, port)?`; WebSocket `use std.net.websocket`: `websocket.connect("ws://host/path")?`, `ws.send_text(s)?`, `m := ws.recv()?` (`m.kind` is `Kind.Text`, `Binary`, `Close`, `Ping`, `Pong`; `m.text()`), `ws.close(1000, "bye")`, server `websocket.listen(addr)?` and `srv.accept()?`; mail `use std.net.smtp`: `mut c := smtp.connect(host, 587, smtp.Security.StartTls, "me.example")?`, `c.auth(user, pw)?`, `m := smtp.message(from, [to], subject)` with `m.text`, `m.html`, `m.attach(name, type, data)`, `c.send(m)?`, `c.quit()`; `m.build()?` is the MIME bytes; `c.start_tls(host)?` upgrades a plain `Conn`; Server-Sent Events `use std.net.sse`: server `sse.listen(addr)?`, `r := srv.accept()?`, `st := r.open()?` (or `r.refuse(404, "no")`), `st.send(sse.event("x"))?`, `st.pump(ch, 15000)?` from a `Channel[sse.Event]`; client `mut c := sse.connect(url)?`, `ev := c.next()?` (`ev.name`, `ev.data`, `ev.id`; reconnects with `Last-Event-ID`, `c.set_retries(n)`) |
| `std.hash`, `std.crypto` | `sha256(data)` (hex `str`), `hmac_sha256`, `crc32`; `md5_legacy` and `sha1_legacy` are broken: old formats only; `crypto.ct_eq(a, b)` is constant time: use it, never `==`, for MACs and tags; `random_bytes(n)` needs effect `random` |
| `std.compress`, `std.csv`, `std.toml`, `std.yaml` | `gzip`/`gunzip`, `deflate`/`inflate`, `zip_dir`/`unzip`, `unbrotli` (decode only) `! compress.CompressError`; `csv.parse(text)?`, `records`, `write`; `toml.parse` and `yaml.parse` give `Json` (`Obj`, `Arr`, `Num`, `Str`; yaml refuses anchors, tags and several documents); `toml.write(j)?` and `yaml.write(j)` write it back (toml refuses `Null`) |
| `std.db` | `mut d := db.open(path)?` or `db.memory()` (`unique`); `d.exec(sql, [Text(s), Int(n), Real(x), Blob(b), Null])?` runs statements and returns the rows changed (`[]` with no `?`); `d.query(sql, params)?` gives `List[Row]` with `r.int(col)`, `str`, `float`, `bytes`, `bool` (each `T?`, `none` for NULL), `r.to_json()`; `begin`, `commit`, `rollback`, `last_id`, `compact`, `sync`; strict column types; `DbError.kind` (`Constraint`, `Syntax`, ...) |
| `std.kv`, `std.bigint`, `std.random`, `std.tensor` | `mut s := kv.open(path)?` (`unique`; `get`, `set`, `remove`, `sync`, sorted `scan(prefix)` and `range(lo, hi)`, all-or-nothing `b := kv.batch()` with `b.set`/`b.remove` then `s.apply(b)?`); `bigint.parse(text)?` with `+ - * / %` (`/` and `%` trap on zero); `random.int(lo, hi)` (both inclusive), `float()`; `tensor.from(xs, shape)` with `+ *`, `matmul`, `dot`, `sum` (shape mismatch and overflow trap) |
| `std.markdown` | `markdown.html(text)` (CommonMark and GFM tables, `~~strike~~`, bare links; raw HTML shown as text, `javascript:` links dropped), `markdown.parse(text)` gives `List[Block]` (`Heading`, `Para`, `Rule`, `Quote`, `CodeBlock`, `ListBlock`, `Table`; `Inline`: `Text`, `Code`, `Emph`, `Strong`, `Strike`, `Link`, `Image`), `markdown.render(blocks)`, `plain(text)`, `headings(text)` (`Head` `level`, `text`); no reference links |
| `std.xml` | `xml.parse(text)? Node` (`Elem`/`Text`; DOCTYPE refused, no entity expansion), `n.find("a/b")`, `find_all("a/*")`, `"a/@id"` gives text nodes; `n.name()`, `text()`, `attr(k) str?`, `elements()`; `xml.write(n, indent)`, `escape(s)`, `escape_attr(s)` (also HTML escape) |
| `std.template` | `template.render(tpl, data)?` fills `{{name}}` (HTML escaped), `{{{name}}}` and `{{&name}}` (raw), `{{a.b.0}}`, `{{#each xs}}` with `this`, `@index`, `@first`, `@last`, `@key`, `{{#if x}}...{{else}}...{{/if}}`, `{{#unless x}}`, `{{! note }}` from a `Json` (`v.to_json()` for a struct); `render_text` does not escape; `compile(tpl)?` then `t.render(data)?`; `escape_html(s)`; a name with no value fails with `TemplateKind.Missing`. Write the template in `r"""..."""` or read it from a file, since `{{` is not a plain string |
| `std.textwrap` | `textwrap.wrap(s, 40)` (lines), `fill`, `dedent`, `indent(s, "> ")`, `truncate(s, 20)`, `width(s)` (terminal columns: CJK 2, marks 0, ANSI 0), `strip_ansi`, `pad_to_width(s, 10, align: 'r')`, `table(rows, align: "lr")`, `table_grid`; `names.sorted(by: \|n\| textwrap.natural_key(n))` orders `img2` before `img10` |
| `std.unicode` | `unicode.nfc(s)`, `nfd`, `nfkc`, `nfkd` (normal forms; `is_nfc` and the rest test); `casefold(s)` (compare without case: `ß` gives `ss`); `caseless_eq(a, b)` (case and canonical forms ignored); `strip_marks("crème")` gives `creme`; `graphemes(s)` (`List[str]`, a flag or `e` with an accent is one) and `grapheme_count(s)` (what a reader counts; `s.len()` counts characters) |
| `std.textdiff` | `textdiff.lines(a, b)` gives `Edit`s (`kind` `Keep`/`Add`/`Remove`, `text`, `a`, `b`); `patience(a, b)` is the patience line diff and `words(a, b)` a word diff, both giving `Edit`s; `unified(a, b, name_a, name_b, context)` is `diff -u` text; `apply(text, patch)? str` (`! PatchError`); `stat(a, b)` is `(added, removed)`; `distance(a, b)` (Levenshtein) and `similarity(a, b)` |
| `std.lru` | `mut c := lru.Cache[str, int].new(100)`; `c.put(k, v)` (a full cache evicts the least recently used), `c.get(k)` (`V?`, marks it used), `peek`, `contains`, `remove`, `len`, `keys()` (newest first), `clear`; `lru.Ttl[K, V].new(capacity, ttl_ms)` also expires entries (`get_at`/`put_at`/`purge_at` take a clock reading `time.mono_ns()` for tests) |
| `std.decimal` | `decimal.parse("19.99")?` gives an exact `Decimal` (`print` shows `12.50` with its scale); `+ - *` are exact, `==` and `<` compare by value, there is no `/`: `a.divide(b, 2, decimal.Rounding.HalfEven)?`; `d.round(2, mode)?`, `pow`, `normalize`, `decimal.sum(xs)`, `from_int`; modes `Down Up Floor Ceiling HalfUp HalfDown HalfEven`; errors are `DecimalError` with a `kind` |
| `std.ordcoll` | `m := ordcoll.OrdMap[int, str].new()`, `m.put(k, v)`, `get`, `remove`, `first`/`last`, `floor(k)`/`ceiling(k)`/`lower`/`higher` (`(K, V)?`), `range(lo, hi)`, `keys()`, `rank`, `nth`; `OrdSet[K].new()` with `add`, `contains`; `IndexHeap[int].new()`: `push(id, prio)` (updates), `decrease_key`, `pop()` gives `(id, prio)?`, `remove(id)` |
| `std.semver` | `semver.parse("1.4.2-rc.1")?` gives a `Version` (`major`, `minor`, `patch`, `pre`, `build`; `print` shows the text, `<` and `sorted()` follow SemVer precedence, build ignored); `compare(a, b)` is -1, 0 or 1; `bump(v, semver.Part.Minor)`; `parse_range("^1.2 || >=3")?` then `r.matches(v)`, `semver.satisfies(v, text)?`, `max_satisfying(vs, r)`; `resolve(roots, packages)?` picks one version of each package (`Dep`, `Package`, `Pin`) so every requirement holds; errors are `SemverError` with a `kind` |
| `std.throttle` | `mut b := throttle.bucket(5, 10)?` (5 a second, burst 10; `none` for a bad rate), `b.take(n)` true or false, `b.wait(n)` sleeps for tokens, `b.available()`; `mut l := throttle.limiter(100, 1000)?` then `l.allow()`; `throttle.pace(in_ch, out_ch, per_sec, burst)` forwards a channel at a bounded rate; `_at(now_ns)` twins take `time.mono_ns()` readings for tests |
| `std.uuid`, `std.base32`, `std.base58` | `uuid.v4()`, `uuid.v7()` (time-ordered: sorts by creation; `uuid.v7_at(ms)?`), `uuid.parse(text)?`, `u.to_str()`, `u.version()`, `u.unix_ms()` (`int?`), `<` and `sorted()` order by bytes; `base32.encode(b)`, `decode(text)?` (RFC 4648), `encode_hex`, `encode_crockford`; `base58.encode(b)`, `decode(text)?` (Bitcoin alphabet); errors have a `kind` |
| `std.cron` | `s := cron.parse("*/15 9-17 * * MON-FRI")?` (five fields, six with seconds first, or `@daily` `@hourly` `@weekly` `@monthly` `@yearly`; names `MON` `JAN`; `, - / *`); `s.next(ms)` and `s.prev(ms)` give the firing strictly after or before as `int?` (none when never); `s.upcoming(ms, 5)`; `s.in_zone("America/New_York")?` reads the fields in a zone (a skipped wall time does not fire, a repeated one fires once); `s.matches(dt)`; errors are `CronError` with a `kind` |
| `std.humanize` | `humanize.int_text(1234567)` (`1,234,567`), `float_text(x, 2)`, `percent(0.256, 1)` (`25.6%`), `compact(1234, 1)` (`1.2K`), `size_si(n, 1)` (`1.5 MB`), `size_iec(n, 1)` (`1.5 MiB`), `ordinal(22)` (`22nd`), `relative(then_ms, now_ms)` (`3 minutes ago`); another style: `humanize.de().float_text(x, 2)`, `humanize.locale("fr")?`, `humanize.custom(" ", ",")` |
| `std.oauth2` | `c := oauth2.Client(client_id: id, token_url: u, auth_url: a, redirect_uri: r, scope: "x y")`; `p := oauth2.pkce()`, `c.authorize_url(oauth2.random_state(), challenge: p.challenge)?`, `oauth2.callback_params(link)?`, `c.exchange_code(code, verifier: p.verifier)?` gives a `Token` (`access_token`, `refresh_token`, `expires_at_ms`, `t.header()`, `t.is_expired()`); `c.client_credentials()?`, `c.refresh(t)?`, `c.fresh(t)?`; device flow `c.device_start()?` then `c.device_wait(d)?`; errors are `OAuthError` (`kind`, `code`, `status`) |
| `std.stats` | `stats.mean(xs)`, `median`, `mode`, `stdev`, `pstdev`, `variance`, `percentile(xs, 90.0)`, `quantiles(xs, 4)`, `min_max`, `histogram(xs, 5)`, `correlation(xs, ys)`, `moving_average(xs, 3)` take a `List[float]` (`stats.floats(ints)` converts) and give `float?` etc.: `none` for empty input, never a trap |
| `std.bitset`, `std.bloom` | `mut s := bitset.new(n)` is a set of `0..n-1` in 64-bit words: `s.set(i)`, `clear`, `flip` (trap outside the range), `s.test(i)` (false outside), `count()`, `members()`, `next_set(from)` (`int?`), `fill()`, `reset()`, `complement()`, `union`/`intersect`/`difference`/`xor(o)` (new sets), `is_subset`, `intersects`, `bitset.from_list(n, xs)`; `mut f := bloom.new(100000, 0.01)` then `f.add(key)`/`f.has(key)` (`str`; `add_int`/`has_int` for `int`), never a false negative, `bits()`, `hashes()`, `count()`, `false_positive_rate()`, `reset()`, `union(o)` (`Bloom?`: `none` when the shapes differ) |
| `std.tar` | `tar.write(entries, gzip: true)?` gives bytes and `tar.read(bytes)?` the `List[Entry]` back (`name`, `kind`, `mode`, `mtime`, `data`, `link`; gzip detected); `tar.new_file(name, data)`, `new_dir`, `new_symlink`; files: `tar.list(path)?`, `tar.extract(path, dir)?` (refuses `..`, absolute paths and links out of `dir` before writing anything), `tar.create(dir, out)?` (`.gz` or `.tgz` gzips); errors are `TarError` with a `kind` |
| `std.regex` | `regex.compile(r"(\d+)")?` then `re.is_match(s)`, `find`, `find_all`, `captures`, `replace(s, "$1")`, `split`; RE2 syntax, linear time, no backreferences or lookaround; offsets count characters |
| `std.jwt` | (unaudited) `jwt.sign(claims, key, jwt.Alg.HS256)?` makes `A.B.C`; `jwt.verify(tok, key, [jwt.Alg.HS256], jwt.options())?` gives a `Token` (`header`, `claims`, `alg`); you pick the allowed algorithms, `none` is refused; `o := jwt.options()` has `leeway`, `issuer`, `audience`, `now`, `require_exp`; errors are `JwtError` with a `kind`; `sign_with`/`verify_with` take a function for `EdDSA`, `ES256`; `decode_unverified` trusts nothing |
| `std.ini` | `ini.parse(text)?` gives `Json` (keys before a section on the object, each `[name]` an `Obj`, every value a `Str`; `;` or `#` comments, `"..."` and `'...'` values; a duplicate key fails); `ini.write(j)`; `ini.parse_env(text)?` gives `Map[str, str]` from `.env` text (`export`, quotes, `${VAR}` and `${VAR:-x}` from earlier keys of the file only) |
| `std.cbor` | `cbor.encode(j)?` gives `bytes`, `cbor.decode(data)?` a `Json` (CBOR, RFC 8949; whole `Num`s are integers, byte strings read as base64 `Str`, tags dropped, map keys must be text; errors have `kind` `Truncated`, `Malformed`, `Unsupported`, `Trailing`, `TooDeep`) |
| `std.msgpack` | `msgpack.encode(j)?` gives `bytes`, `msgpack.decode(data)?` a `Json` (same mapping as `std.cbor`: `bin` reads as base64 `Str`, `ext` and non-`str` keys fail with `Unsupported`) |
| `std.url` | `u := url.parse(link)?` gives `scheme`, `user`, `host`, `port` (`int?`), `path`, `query`, `fragment` (kept percent-encoded); `print(u)`; `url.resolve(base, ref)?` (RFC 3986); `join_path(base, parts)?`; `encode`/`decode` (`str?`), `encode_form`/`decode_form` (`+` is a space); `parse_query(q)` (pairs), `query_map(q)`, `build_query(pairs)`; a local named `url` hides the module |
| `std.time`, `std.time.date`, `std.tz` | `time.now_ms()`, `sleep_ms(ms)`; `date.utc(ms)`, `date.now()`, `parse_iso(text)`, `format("%Y-%m-%d")` (also `%F %T %-d %V`), `parse_format(text, "%d/%m/%Y %H:%M %z")`, `parse_format_in(text, pattern, zone)`, `parse_rfc3339`, `parse_rfc2822`, `dt.rfc2822()`, `dt.http_date()`; durations are `int` ms; zones are `"UTC"`, `"+05:30"` or a name; `tz.system()` |
| `std.time`, `std.time.date`, `std.tz` | `time.now_ms()`, `sleep_ms(ms)`, `mono_ns()` (monotonic, for elapsed time), `since_ms(start_ns)`; `date.utc(ms)`, `date.now()`, `parse_iso(text)`, `format("%Y-%m-%d")`; durations are `int` ms (`date.parse_duration("1h30m")`, `date.duration_text(ms)`); zones are `"UTC"`, `"+05:30"` or a name; `tz.system()` |
| `std.test`, `std.mcp` | `std.test` stubs `files`, `env`, `clock`, `http` for one `ail test` run, and gives `test.temp_dir()`, `test.temp_files(map)` and `test.golden(name, text)` (`testdata/NAME.golden`, `AIL_UPDATE_GOLDEN=1` rewrites); tests are sandboxed (no net, proc or files outside the package) unless `@needs(net)`, `@needs(proc)`, `@needs(fs)` (test.sandbox.1-4, test.stub.1-6, test.temp.1-2, test.golden.1-4); `mcp.connect(cmd, args)?` calls another tool's MCP server (`c.set_timeout(ms)`, default 30 s) |
| `std.term`, `std.ui`, `std.ui.widgets` | `term.open(mouse)?` raw terminal (`defer t.close()`), `t.poll(ms) Event?`; style and cursor helpers (`bold`, `fg`, `rgb`, `clear_line`, `hide_cursor`, `enter_alt`, `is_tty()`); `term.green(s)`, `red`, `strong`, `paint(s, term.fg(n))` honour `NO_COLOR` and pipes; `term.progress(label, total)` (`inc`, `set`, `finish`) and `term.spinner(label)` (`tick`, `finish`) print only on a terminal; prompts `term.confirm(q, default)`, `ask(q, default)`, `choose(q, options) int?`, `password(q) str?`; `ui.open(title, cols, rows)? ui.Window` with `put`, `present`, `next_event`; widgets `TextField`, `ListView`, `Button`, `Dialog`, `Pane`, `Tree`; `widgets.text_rows` prints a screen headless |
| `std.hw`, `std.app`, `std.gfx`, `std.gfx.host`, `std.gfx.watch`, `std.screen`, `std.screen.cells`, `std.screen.pixels`, `std.screen.view`, `std.audio`, `std.audio.device`, `std.audio.opus`, `std.image`, `std.image.ops`, `std.image.png`, `std.attach`, `std.attach.serve`, `std.media.webm`, `std.video.vp9` | `hw.at(addr, 0)`, `hw.fence()`, `hw.fence_io()` (`unsafe driver`); the rest have a spec chapter each (the `std-` files in the spec folder): `ail card std.NAME`, `ail ask std WORDS` |

## Tests and assertions

```ail skip=context
@test fn doubles()           # `@test` on the same line
    assert_eq(double(4), 8)  # traps showing both values if unequal
    assert(double(1) == 2)   # traps if false
```

A test returns nothing or `!`, can't be called, and `ail run` ignores it. `?`
works in it: `assert_eq(parse("7")?, 7)`. `assert` and `assert_eq` also work
in ordinary code (a failure traps).

A test with parameters is a **property**: `ail test` runs generated inputs (`--cases N`, `--seed S` replays) and
shrinks a failure to the smallest input. `--filter PATTERN` selects tests by name, `--list` prints them.

A **fixture** is setup made fresh for each test: `@fixture fn tmp() T` (or `!T`); `@fixture("drop")` names a teardown
`fn drop(v T)` that always runs. A test takes it with `@test @needs("tmp")` and a parameter `tmp T`.

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
rs := TaskGroup[int]()         # tasks that return an int
rs.add(spawn load(3))
sums := rs.wait_all()?        # List[int], in the order added
fs := TaskGroup[int ! Bad]()   # tasks that return int ! Bad
xs := fs.wait_all()?          # fails with the first error in add order; the rest are cancelled
ch := Channel[int](8)         # bounded: send waits while full
ch.send(1)
v := ch.recv() else break     # none once closed and empty
w := ch.try_recv() else 0     # none if nothing is waiting now
for v in ch                   # receives until closed and empty
    print(v)
select                        # waits; runs the first arm in order that can go
    n := nums.recv() =>       # a value from nums, as an int (not int?)
        total += n
    out.send(job) => sent += 1  # room in out: job is queued
    after 50 => print("idle") # no arm ready for 50 ms; `after 0` never waits
    closed => break           # every recv channel closed and empty (required with recv)
ch.close()
m := Mutex(0)                 # shared state: get, set, swap
with m as mut x               # read-modify-write: holds m, stores x back
    x += 1                    # on any exit; a trap keeps the old value
n := Atomic(0)                # int or bool, shared by tasks and par for
n.add(1)                      # returns the old value; also load, store, cas
g.cancel()                    # or t.cancel(): ask tasks to stop
task.cancelled()              # in the task: was it cancelled?
task.check()?                 # fails with Cancelled once cancelled
```

A cancel reaches every task the cancelled task started; `recv` gives `none`, `sleep_ms` returns at once, the scope still waits.
`ail test --schedule=random:SEED` replays an interleaving.

Each task handle is waited or added to a group exactly once, on every path. A
handle can't be stored or passed; channels and mutexes can, and copies share one.
If every task is blocked, the program traps (deadlock).

## Effects

Effects are inferred, never written: `io` (`print`, `fs`, `io`, `proc`),
`fail` (`T ! E` result), `mutates` (`mut` parameter), `spawn`, and the capabilities
`fs`, `env`, `proc`, `clock`, `random`, `net`, `net.serve` (`net.serve`, `net.listen`), `ui`. A call through `any Trait` counts every
implementation. `ail run --allow LIST FILE` runs with only the listed capabilities (`fs.read=PATH`, `net=HOST[:PORT]`, `proc=NAME`, ...; `none` allows nothing); a use outside them ends the program with `trap[eff.allow.N]` naming the entry to add (`eff.allow.1-9`).

```ail
@pure fn area(w int, h int) int   # error if it reaches print or a capability
    w * h
```

A call whose value is dropped must do something (`eff.use.1`): `area(3, 4)` alone is an
error, `xs.sorted()` alone is fixed to `xs.sort()`, and `_ := area(3, 4)` drops a value on purpose.

`@inline` is a speed hint, accepted and ignored for now. `@deprecated("use
area")` before a function warns at each call, with the reason (required).
`@allow("expr.match.3")` silences the warning for `_` arms that cover enum
variants in that function. Unknown attributes are errors.

## Unsafe

`unsafe` blocks allow unsafe operations; `unsafe fn` is callable only there. A package needs `unsafe allow` on its `package`
or `dep` line in `ail.pkg`; a program without `ail.pkg` needs nothing. Hardware
(`hw.at(addr, 0)`, `p.read_volatile()`, `p.write_volatile(v)`, `hw.fence()`)
needs `unsafe driver` there, or `--unsafe driver`.

```ail
unsafe fn twice(x int) int
    x * 2

fn safe_twice(x int) int
    unsafe
        twice(x)
```

C functions are declared, not imported: types `int`, `float`, `bool`, `i8`...`u64`, `f32`, `*T`; at most
6 integer-like and 8 float arguments; no `mut`; a struct of those types passes by value in the platform's
C layout; a `fn(...)` parameter is a C callback (a declared function's name, or `mem.c_fn(f)`). A narrow C
`int` result is declared `i32`. `std.mem`: `cstr(s) *u8`, `from_cstr(p) str`, `free(p)`, `alloc(n, 0)`,
`null()`, `cast(p, example)`; pointers `p.read()`, `p.write(v)`, `p.offset(n)`, `p.is_null()`; bad accesses trap.

```ail
use std.mem

extern c "libm" fn hypot(a float, b float) float
extern c fn strlen(s *u8) int

fn c_length(text str) int
    unsafe
        p := mem.cstr(text)
        n := strlen(p)
        mem.free(p)
        n
```

## Contracts

Lines under the signature pure `bool`s. `result` is the returned value in
`ensures`; `old(e)` is `e` on entry. A failed `requires` traps at the call
(`contract.req.1`), a failed `ensures` at its line (`contract.ens.1`). Trait
contracts bind implementations (their `requires` weaken, their `ensures` add).
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
