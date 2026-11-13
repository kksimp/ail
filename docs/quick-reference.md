# AIL quick reference

> **Non-normative.** One page for people and AI agents writing AIL. The
> [spec](spec/) is authoritative. Items marked **(not yet)** are specified but
> not built in the native compiler yet.

## Running programs

```sh
./ail help [CMD] [--json]            # every command, or one command's usage and rule (also `ail --help`, `ail CMD --help`)
./ail doctor [--json] [--path]      # check the toolchain; --path prints the profile line that puts `ail` on PATH
./ail run [ARGS...] FILE.ail        # DIR runs a package; words after FILE are env.args()
./ail check [--json] [--budget N [--from K]] FILE|DIR
./ail build FILE.ail [-o OUT]       # native executable
./ail test [--json] FILE.ail        # @test functions; exit 0 if all pass; also --filter PATTERN, --list, --fuzz NAME, --mutate, --contracts
./ail card [TOPIC]                  # the 970-token core card, or one topic (`std.fs`, `errors`); `ail ask std WORDS` finds a std function
./ail pkg [COMMAND]                 # a program's packages: fetch, lock, list
./ail header FILE.ail [-o OUT.h]    # a C header of the exports
./ail effects [--json] FILE.ail
./ail explain [RULE]                # a rule's meaning, ADRs and example; no RULE lists all
./ail api [--json] [--text] [--card [--budget N]] FILE|DIR
./ail ask FILE who writes field F   # also: who reads F, what calls f, what does f call, which effects reach f, who uses T
./ail context FILE f --budget 800   # what an edit to f needs: body, callee and caller signatures, types, tests, rules; cut to 800 tokens
./ail review main..HEAD --budget 400   # a change by meaning: new capabilities, API breaks, contracts, untested code
./ail infer --contracts f FILE [--apply]   # proposes up to 3 requires/ensures lines for f from the calls its tests make (patch for ail edit)
./ail prove FILE                # each requires/ensures check the checker proves (and leaves out of the build) or keeps
./ail task start FILE GOAL...     # then: task verify | status | finish (one change, recorded base, `next` in every answer; `task verify --fix`; `task log` reads `.ail/task-log.jsonl`)
./ail merge BASE OURS THEIRS       # merge by declaration (a git merge driver; `ail merge --install`); exit 1 on a conflict hunk
./ail merge --check OURS THEIRS [PATH]   # merge two branches in scratch worktrees and check: an error neither side had is a `semantic:` conflict (also BASE OURS THEIRS files)
./ail verify --diff --attest C.json FILE   # change certificate; `ail verify --check C.json` re-runs it, exit 1 on a difference
./ail attach screen role=button budget=200   # read a running std.app program: observe, screen, heap; `ail run --attach-input` lets `attach input kind=press id=B` act; `./ail attach` lists them
./ail try FILE --set A.edit --set B.edit   # check and test patch sets on scratch copies, one verdict line each; --apply writes one clean set
./ail mcp                       # the compiler's tools as an MCP stdio server (23 tools: check, test, verify, ask, impact, context, edit, fix, explain, api, review, card, find, try, prove, infer, merge_check, verify_check, task_*)
./ail new [--no-agent] NAME     # a package with a first program, AGENTS.md and .mcp.json (the agent kit)
./ail lsp                       # a language server on stdio for any editor (diagnostics, hover, definition, references, symbols, completion, rename, quick fixes)
./ail sanitize FILE              # run on the fixed and seeded random schedules; report results that differ (san.sched.1)
./ail leak FILE                  # run with the leak report: growth by kind and the roots that hold it (san.leak.1-2); AIL_LEAK works on any native exe
./ail perf --explain FN FILE      # per loop of FN: bounds checks removed or why not (with a fix), list headers, vector kernel, parallel verdict, calls left (tool.perf.1-2)
./ail fix [--dry-run] FILE.ail      # apply the fixes diagnostics carry; `ail check --fix FILE` fixes, then checks, in one call
./ail diff OLD NEW                  # what changed by meaning
./ail edit FILE.ail [--test] < EDITS    # also change_sig, extract, move (edit.op.8-10)
./ail fmt [--check] FILE.ail
```

Options for each command are in [semantic-api.md](design/semantic-api.md) and the spec chapter for it (`ail explain` names it).

With `--json`, each diagnostic is one line of schema v1 (`"api":1`), with a
span and, where the fix is obvious, `fixes` to apply by line and column.

`todo` marks code not yet written: it has the expected type; `ail check` warns
(`expr.hole.1`) and running it traps (`expr.hole.2`). `uhwhatnow?` is the same hole, but
its warning ranks what in scope fits, best first, each with a fix (`expr.hole.3`).

```ail
fn area(w int, h int) int
    todo     # warning: expected `int`; in scope: `h`, `w`
```

A program needs `fn main()`. `print(value)` writes any value and a newline.

## Native builds

```sh
./ail build examples/native_fib.ail   # writes ./native_fib, a static executable
./native_fib                          # same output as ./ail run
```

- Native is the only backend: `ail run` builds the program natively and runs it, so
  both commands accept the same language (structs, enums, methods, traits, maps,
  generics, errors, tasks). A rare construct the lowering cannot build yet is reported
  as `error[native.subset.1]` with its position and a `help:` line (native.subset.1).
- A trap prints the same first line as `ail run` and exits 1.
- Targets: `--target x86_64-linux`, `aarch64-linux`, `aarch64-macos`.
- An unchanged program rebuilds from `~/.cache/ail/build`; `--no-cache` skips it (native.build.5).
- Guide: [design/native-guide.md](design/native-guide.md).

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

Positional arguments work when no two fields they fill share a type
(`Problem(7, "bad")`); `Point(1, 2)` is an error when `x` and `y` are both `int`.

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
fields is written qualified there (`Cons(_, Items.Nil)`).

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
- Several error types (`! IoError | ParseError`): `match err` takes member types
  and their variants, and covers every member (err.match.1-3): `IoError =>`
  (`err` is an `IoError` there), `ParseError.Empty =>`, `_ =>`.
- `print` of a `T?` shows the value or `none`; `return i` needs no wrapping.
- `mut best := none` gets its type at the first assignment (`best = 3`);
  `x := if c then 4 else none` is an `int?`.
- A local may shadow a function's name; calling that name inside its block is an error.
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
    print(bigger(c, Square(side: 1)))
    print(Shape.area(c))             # the qualified form of `c.area()`
```

- `< <= > >=` work on `int`, `str`, lists, and types with `impl Ord for T`
  (fields in order, or your `fn cmp(self, other Self) Ordering`).
- Customise: `impl ToStr` (`fn to_str(self) str`), `impl Eq` (`fn eq`).
- Constrain one method of a generic type: `fn Stack[T Ord].max(self) T?`.
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
sign := if n < 0 then "-" else "+"   # one line: `then` and `else`, no `elif`
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
- `float`: `+ - * /` (no `%`), never traps (`1.0 / 0.0` is `inf`). Math methods (`std.math`, type.num.16): `x.sin()`, `cos`, `tan`, `atan`, `exp`, `ln`, `log10`, `cbrt`; `x.atan2(y)`, `x.pow(y)`, `x.hypot(y)` (`x` first). No mixing
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
- Bits on `int` (64-bit two's complement): `a & b`, `a | b`, `a ^ b`, `~a`, `a << k`,
  `a >> k` (arithmetic; `k` in `0..=63`, else trap `expr.bit.2`); the methods `bit_and`,
  `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr` mean the same. `x &= e` (also `|= ^= <<= >>=`)
  works on a plain name. A bit operator never mixes with another operator without
  parentheses (`expr.bit.4`): write `(x & 1) == 0`, `(a + b) & m`, `a | (b & c)`.

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

- Format specs: `[[fill]align][+][0][width][,][.decimals][x|b|e]` only; `.N` and `e` need a float, `x` and `b` an int, `+` `0` `,` a number.
- `"a,b".split(",")` gives `["a", "b"]`, `"k=v=w".split("=", 2)` gives `["k", "v=w"]`; `"12".to_int()` gives an `int?`,
  `"2.5".to_float()` a `float?`.
- `s.chars()` (a `List[char]`; strings have no indexing or `for`), `s.len()` (characters; `s.byte_len()` is UTF-8 bytes),
  `s.lines()`, `s.trim()`, `s.contains(x)`, `s.starts_with(x)`,
  `s.ends_with(x)` (`x` a `str` or `char`), `s.strip_prefix(p)` (gives
  `str?`), `s.repeat(n)`, `s.pad_left(width, ' ')`, `s.pad_right(width, '0')`, `s.center(width, ' ')`, `s.words()` (split on whitespace), `s.trim_start()`, `s.trim_end()`, `s.count(sub)`, `s.rfind(sub)`, `s.slice(a, b)` (characters), `s.char_at(i)` (`char?`), `s.find(x)` (`int?`), `s.replace(a, b)`, `s.to_lower()`, `s.to_upper()`.
- `xs.join(", ")` joins any list. `char`: `is_digit()`, `is_alpha()`,
  `is_space()`, `to_upper()`, `to_digit(16)` (`int?`), `code()`, `char.from_code(n)`.
- `bytes` is immutable binary data (ADR 0412): `b"GET /\r\n\x00"` (ASCII, escapes `\xHH`),
  `s.to_bytes()`, `b.len()`, `b[i]` (a `u8`), `b[a..b]`, `a + b`, `for x in b`,
  `b.to_str()` (`str?`, strict UTF-8), `b.text()` (lossy `str`), `b.to_list()`,
  `b.find(x)` (`int?`), `contains`, `starts_with`, `ends_with`, `b.hex()` (`b.hex(sep)` between pairs),
  `b.base64()`, `bytes.from_list(xs)`, `bytes.from_hex(s)` and
  `bytes.from_base64(s)` (`bytes?`), `bytes.zeros(n)`. No `b[i] = x`.

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

Also: `insert(i, x)`, `remove_at(i)`, `extend(ys)`, `clear()`, `reverse()`,
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

`Grid[T]`, a flat 2-D list: `mut g := Grid.fill(rows, cols, v)` (or
`Grid.from(xs)`), `g[i, j] += 1`, `g.rows`, `g.cols`, `g.cells`, `g.row(i)`.

`Heap[T]`, a priority queue: `h := Heap[int]()` (smallest first; `max_first: true`
for largest), `h.push(x)`, `h.pop()`/`h.peek()` (`T?`), `h.len()`, `Heap.from(xs)`.
`Deque[T]`: `push_front`, `push_back`, `pop_front()`, `pop_back()` (`T?`), `first()`,
`last()`, `d.at(i)`, `d.to_list()`.

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
ages["ann"] += 1                       # `op=` on a missing key starts from 0
mut counts := {}                       # typed by first use: counts[w] += 1
counts["x"] += 1
a := ages["bob"]                       # traps if missing...
b := ages.get("dan") else 0            # ...`get` gives an optional
for name, age in ages                  # each entry as a pair; also keys(), values()
    print("{name}: {age}")
mut seen := Set[str]()                 # or a literal: {"a", "b"}
if seen.add("x")                       # true if it wasn't there
    print("new")
```

Also `m.contains(k)`, `m.remove(k)` (`V?`), `m.len()`, `s.remove(x)`, `s.contains(x)`,
`s.sorted()`, `s.union(t)`, `s.intersect(t)`, `s.difference(t)` (also on two `keys()`
lists), and `m.push(k, x)` for `Map[K, List[T]]`. Keys and elements: any type with
automatic `==` (ints, strings, chars, structs and enums of them), not floats.

## JSON

`Json` is built in, with no import (`json.type.1`). It is read only when the
program uses it.

```ail skip=context
doc := parse_json("\{\"a\": [1, \"x\"]}")?   # Json ! JsonError; err.at is the char offset
doc.get("a")?.at(1)?.as_str()                  # str?  (get, at, as_str/as_num/as_bool/as_arr/as_obj)
doc.str("id")                                  # str?: also num, bool, arr (a member of that kind)
doc.set("id", Json.Str("x"))                   # replace or add a member (mut doc)
doc.to_text()                                  # {"a":[1.0,"x"]}, compact, members in document order
doc.text("id", "none")                         # str: the text member, or the default
doc.flag("ok", false)                          # bool: likewise
parse_json_lines("1\nbad\n\"x\"\n")          # List[Json]: every line that parses
load_json[List[U]](path)?                      # a file decoded as T (FromJson); print_json(v) prints v as JSON (json.load.1-2)
```

Variants: `Null`, `Bool(b)`, `Num(n float)`, `Str(s)`, `Arr(items)`,
`Obj(members)`; a program's own `Json` wins. Derive conversions of your own types (`json.derive.1-2`):

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

Enums: a bare variant is its name (`"Red"`), one with fields
`{"kind": "Circle", "r": 2.0}`. Generic types need bounds
(`impl FromJson for Pair[A FromJson, B FromJson]`).

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

```ail
fn adder(n int) fn(int) int
    |x| x + n                         # copies n

fn main()
    xs := [3, 8, 5, 12]
    lo := 4
    print(xs.filter(|x| x > lo))      # [8, 5, 12]
    print(xs.map(adder(1)))           # [4, 9, 6, 13]
    print(xs.any(|x| x > 10))         # true; also all, find (T?), group_by
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

A `move self` method consumes its receiver; the call has no mark, and the
receiver can't be used after it (own.move.6):

```ail skip=context
fn File.finish(move self) int
    fd

n := f.finish()        # f is moved
```

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

```ail fragment
pub struct Point         # `pub` exports; everything else is private
    pub x int            # fields too; a private field means only this
    y int                # module can build a `Point`
```

No item or wildcard imports, no `pub use`, no import cycles. `fn T.m` must be in
`T`'s module. Text forms show the path: `geo.Point(x: 1, y: 2)`.

A package root may hold `ail.pkg` (spec `mod.pkg.*`). Each `dep` is imported by
name (`use geo`, `use geo.shapes`) and reaches only its `caps` (none if omitted);
the `package` line's `caps` limits the package. Violations are compile errors.

```
package notes caps fs
dep geo path ../geo caps fs
dep textkit path vendor/textkit
dep json git "https://example.com/json.git" tag v1.2 caps fs
```

A `git` dep needs `rev COMMIT` or `tag TAG`; it is fetched into `~/.ail/deps`
(or `$AIL_HOME/deps`) and pinned in `ail.lock` (commit it). `ail pkg update [NAME]`
re-pins; `--deterministic` rejects a stale lock (`mod.pkg.6–9`). User guide: `docs/packages.md`.

## Files, processes and the system

Standard modules are in scope without `use` (spec 17-std, ADR 0277); a local or item with the module's name hides it. Failures are values: `fs.IoError(path, kind)`, `proc.ProcError(cmd, kind)`, `net.NetError(url, kind, message)`. Every `kind` is an enum (ADR 1068): `fs.IoKind` is `NotFound`, `Permission`, `Exists`, `IsDir`, `NotDir`, `NotEmpty`, `InvalidUtf8`, `Other`; `proc.ProcKind` adds `TimedOut`, `Cancelled`; test with `err.kind == NotFound` or an exhaustive `match err.kind` (a string there is type.enum.7, and `ail fix` rewrites it).

| Module | Functions |
|---|---|
| `std.compress` | `gzip(data) bytes`, `deflate(data)`, `zlib(data)`; `gunzip(b)`, `inflate(b)`, `unzlib(b)` `bytes ! compress.CompressError { kind CompressKind, message }`; `gzip_file(src, dst)`, `gunzip_file`, `zip_dir(dir, zipfile)`, `unzip(zipfile, dir)` `! CompressError`, effects `fs`, `io` |
| `std.hash` | `sha256(data)`, `sha384(data)`, `sha512(data)` (hex `str`), `hmac_sha256(key, data)`, `hmac_sha384(key, data)`, `hmac_sha512(key, data)`, `sha3_256(data)`, `sha3_512(data)`, `sha1_legacy(data)`, `md5_legacy(data)` (broken: for old formats only, never for security), `crc32(data) int`, `hash64(data) int` (`bytes` or `str`; not secure) |
| `std.crypto` | `ct_eq(a, b) bool` (constant time; use it, never `==`, for MACs and tags), `random_bytes(n) bytes` (system generator, effect `random`), `hkdf_sha256(ikm, salt, info, len) bytes`, `pbkdf2_sha256(password, salt, iterations, len) bytes` (all `bytes`) |
| `std.fs` | `read_text(p) str`, `write_text(p, s)`, `append_text(p, s)`, `read_bytes(p) bytes`, `write_bytes(p, b)`, `append_bytes(p, b)`, `exists(p) bool`, `list_dir(p) List[str]`, `remove(p)`, `make_dir(p)`, `walk(dir) List[str]`, `metadata(p) fs.Metadata` (`size`, `modified_ms`, `is_dir`), `scan(root, same_device) fs.Scan` (`entries` of `fs.Entry`: `path`, `size`, `alloc`, `modified_ms`, `inode`, `device`, `links`, `is_dir`, `is_link`; `skipped` paths), `scan_count() fs.ScanCount` (`entries`, `bytes` listed by every scan so far; read from another task for progress), `volume(p) fs.Volume` (`total`, `free` bytes), `read_range(p, off, len) bytes`, `map(p) bytes` (read-only view of the file's pages, no copy), `trash(p)` (recoverable delete; never permanent), `temp_dir() str` (`TMPDIR`, else `TEMP`, else `/tmp`), `make_temp_dir(prefix) str` (a new empty directory there, owner only; you remove it), `mode(p) int`, `set_mode(p, bits)`, `set_modified(p, ms)`, `read_link(p) str`, `symlink(target, p)`, `truncate(p, len)`, `write_at(p, off, b)`, `rename(a, b)`, `copy(a, b)` (directories copy their tree); all but `exists`, `glob`, `scan_count` and `temp_dir` are `! fs.IoError`; `glob(pat) List[str]` (`*`, `?`, `**`) |
| `std.env` | `args() List[str]`, `get(name) str?`, `vars() Map[str, str]`, `cwd() str`, `program() str`, `home() str?` |
| `std.path` | `dir(p)`, `base(p)`, `ext(p)`, `stem(p)`, `join(a, b)`; text only, no effects |
| `std.file` | `use std.file`; handles read or written a piece at a time, `unique` (never copied); `mut r := file.open(p)?` then `r.read(n)`, `r.read_exact(n)` (`bytes?`, none and no move when fewer remain), `r.read_line()` (`str?`), `r.read_complete_line()` (only a line whose `\n` is written: follow a log), `r.tell()`, `r.seek(pos)`, `r.size()`, `r.close()`; `mut w := file.create(p)?` (or `file.append(p)?`, `file.update(p)?` to patch in place), `w.write(text)?`, `w.write_line(text)?`, `w.write_bytes(b)?` (buffered to 64 KiB), `w.flush()?`, `w.tell()`, `w.seek(pos)?`, `w.close()?` (a writer leaving scope is flushed, but cannot report a failure); loop `line := r.read_line()? else break`; all `! fs.IoError` |
| `std.io` | `read_line() str?`, `read_all() str`, `read_bytes() bytes`, `read_exact(n) bytes?` (none if input ends first), `write_bytes(b)` (raw bytes to stdout), `eprint(value)`, `exit(code)` |
| `std.args` | `use std.args`; specs `args.flag("verbose", "v", help)`, `args.opt("out", "o", help, default)`, `args.pos("input", help)`, `args.rest("more", help)`; `a := args.parse("tool", "about", specs, env.args())?` (`ArgsError { why, usage }`); `a.flag("verbose")`, `a.get("out")`, `a.rest`; or `impl FromArgs for Opts` (`bool` switch, `T?` option, else positional, `##` help): `Opts.parse("tool", "about", env.args())?`; `--help` prints usage and exits 0; `--out=V`, `-o V`, `--` |
| `std.log` | `use std.log`; `log.debug(m)`, `log.info(m)`, `log.warn(m)`, `log.err(m)` write `LEVEL m` to stderr; env `LOG_LEVEL` (`debug`, `info`, `warn`, `error`, `off`; default `info`) |
| `std.proc` | `run(cmd, args, input?, timeout_ms?) proc.Output ! proc.ProcError`; `Output { status int, out str, err str }`; `run_bytes(cmd, args, input bytes?, timeout_ms?) proc.RawOutput ! proc.ProcError` (`RawOutput { status int, out bytes, err bytes }`); `which(name) str?`; `start(cmd, args, timeout_ms?) proc.Child ! proc.ProcError`, `wait(move c) int ! proc.ProcError` (`timed_out`), `kill(move c) int`; `Child { pid int, stdin, out, err Channel[str] }` (`stdin.send(text)`, `stdin.close()`) |
| `std.csv` | `use std.csv`; `parse(text) List[List[str]] ! csv.CsvError { why, row }`, `parse_sep(text, ';')`, `records(text) List[Map[str, str]]`, `write(rows) str`, `write_records(header, recs)`, `quote(f)` |
| `std.mcp` | `use std.mcp`; `connect(cmd, args) Client ! mcp.McpError { kind, message }`; `c.tools() List[Tool]`, `c.call(name, Json) Json` (`structuredContent` or the result; `mcp.text(r)` joins text blocks), `c.close()` |
| `std.toml` | `use std.toml`; `parse(text) Json ! toml.TomlError { why, line }`; tables are `Obj`, arrays `Arr`, numbers `Num`, dates `Str` (read with `get`, `str`, `num`) |
| `std.yaml` | `use std.yaml`; `parse(text) Json ! yaml.YamlError { why, line }`; maps `Obj` (string keys), lists `Arr`, `yes`/`no` stay strings; anchors, aliases, tags, several documents and multi-line plain scalars are refused with a message |
| `std.term` | `use std.term`; `mut t := term.open(mouse) ! term.TermError` (raw mode, alternate screen; `defer t.close()`), `t.poll(ms) Event?` (`Key(key, ctrl, alt, shift)` with `term.KeyCode` as `ui.KeyCode`, `Mouse(col, row, button, kind)`, `Scroll(lines)`, `Resize(cols, rows)`), `t.size()`, `t.write(s)`; pure: `term.parse(bytes)`, `fg(n)`, `bg(n)`, `rgb(r, g, b)`, `goto(col, row)`, `clear()`, `reset()`; macOS, Linux and Windows (std.term.1-8) |
| `std.bigint` | `use std.bigint`; `from_int(n)`, `parse(text) BigInt ! bigint.BigIntError { why }`, `factorial(n)`; methods `add sub mul divmod pow cmp negate abs to_text as_int is_zero is_negative` (no operators) |
| `std.random` | `int(lo, hi)` (both inclusive), `float()` in [0, 1), `seed(n)`, `shuffle(mut xs)` |
| `std.regex` | `compile(pat) regex.Regex ! regex.RegexError` (`kind RegexKind`, `at`, `rule`, `message`); `is_match(re, s)`, `find(re, s) Match?`, `find_all(re, s)`, `group_all(re, s, n)`, `captures(re, s) Captures?`, `replace(re, s, rep)` / `replace_first` (`$1`, `${name}`, `$0`, `$$`; unknown group traps), `split(re, s)`, `escape(s)`; the same as methods: `re.is_match(s)`; RE2 syntax (`\d \w \s \b`, `\p{L}`, `(?i)`, lazy `*?`, `(?P<n>..)`), no backreferences or lookaround; offsets in characters; patterns as `r"..."` |
| `std.tensor` | `from(xs)`, `from(xs, shape)`, `from_fn(shape, \|i\| e)` (`i` is the flat index), `zeros(shape)`: `tensor.Tensor[int]` or `[float]`; `a + b`, `a - b`, `a * b`, `a / b` (float only), `-t`, scalars on either side (`t * 2.0`); `Tensor[f32]` (`to_f32()`, `to_float()`; mixing with `float` is an error); `add(b)`, `mul(b)`, `dot(b)`, `sum()`/`max()`/`mean()` (or `(axis)`), `t[i]` (rows; `item()` for 1-D), `t[lo..hi]`; `relu()`, `sigmoid()`, `tanh()`, `softmax()` (float; softmax along the last axis); `transpose()`, `matmul(b)` (2-D), `map(\|x\| e)`, `reshape(shape)`, `shape()`, `len()`, `to_list()`; shape mismatch and `int` overflow trap |
| `std.time` | `now_ms() int`, `sleep_ms(ms)` |
| `std.time.date` | `date.utc(ms)`, `date.at(ms, zone) ! DateError`, `date.now()`, `date.now_local()`, `date.parse_iso(text)`, `date.parse_in(text, zone)`, `date.parse_date(text) Date`, `date.Date.new(y, m, d) Date?`, `is_leap(y)`, `days_in_month(y, m)`; durations are `int` ms (`date.HOUR`); zones `"UTC"`, `"+05:30"`, `"America/New_York"`. `Date`: `days()`, `weekday()` (1 = Monday), `yday()`, `add_days/months/years(n)`, `since(d)`, `iso()`, `format(p)`, `<`. `DateTime` (`date`, `hour`, `minute`, `second`, `ms`, `offset`, `zone`): `unix_ms()`, `add_ms(n)` (elapsed), `add_days/months/years(n)` (wall clock), `in_zone(z)`, `before(o)`, `since(o)`, `iso()`, `format(p)` (strftime: `%Y %y %m %d %e %j %H %I %p %M %S %f %a %A %b %B %z %:z %Z %s %%`) |
| `std.os` | `signals(names) Channel[str]` (`"HUP"`, `"INT"`, `"QUIT"`, `"TERM"`, `"WINCH"`); effect `proc` |
| `std.tz` | `offset(zone, secs) int?`, `system() str`; effects `fs`, `env` |
| `std.net` | `get(url)`, `post(url, body, content_type)`: `net.Response ! net.NetError`; `Response { status int, headers Map[str, str], body str }`; any status succeeds; `http://`, `https://` (TLS 1.3, system roots or `SSL_CERT_FILE`; kind `Tls`); `get_bytes(url)`, `post_bytes(url, data, content_type)`, `request_bytes(method, url, opts)` give `net.RawResponse { status, headers, data bytes }`; `request(method, url, opts)` with `mut o := net.options()` (`o.headers`, `o.query`, `o.body`, `o.data`, `o.timeout_ms`, `o.max_redirects`, `o.connect_ms`, `o.total_ms`; `o.headers["Connection"] = "close"` opens a fresh connection; `"Transfer-Encoding": "chunked"` sends chunks; connections are kept alive); cookies: `use std.net.jar`, `mut j := jar.new()`, `j.get(url)?`, `j.request(method, url, o)?`, `j.header(url)`, `j.header_from(url, from_page, navigation)`, `j.set_cookie(url, line)`, `j.store(url, r)`, `j.value(url, name)`; gzip and deflate are decoded unless you set `Accept-Encoding`; `b := net.open(method, url, o)?` (`net.Body`: `b.status`, `b.headers`, `b.read_bytes(max)?`, `b.close()`); server: `srv := net.serve(addr)?` (`srv.addr()`, `srv.close()`, `srv.shutdown()`), `ex := srv.accept()?` (`ex.request`: `method`, `path`, `query`, `headers`, `body`, `peer`; `ex.request.data` is the body as bytes), `ex.respond(net.Response(status: 200, headers: {}, body: "hi"))?`, `ex.respond_bytes(200, {}, data)?`; run workers with `spawn`; raw TCP: `net.listen(addr)?`, `net.connect(host, port)?`, `net.connect_tls(host, port)?`, `Conn`: `read(max)`, `read_bytes(max)`, `read_line()`, `write(text)`, `write_bytes(data)`, `set_timeout(ms)`, `close()`; `net.NetKind`: `BadUrl`, `BadRequest`, `Tls`, `NotFound`, `Refused`, `TimedOut`, `Closed`, `Bind`, `Protocol`, `Other`; `Response.all_headers Map[str, List[str]]` (repeated headers, e.g. `set-cookie`); handler form: `use std.net.http`, `http.serve(addr, handler)` or `http.run(srv, handler)` with `fn handler(r net.Request) net.Response`; `r.stop()` ends the server after its answer; `http.serve_dir(addr, dir)` / `http.run_dir(srv, dir)` serve files |
| `std.ui` | `open(title, cols, rows) ui.Window ! ui.UiError` (`kind`: `NoDisplay`, `Closed`, `Other`); `Window`: `size()`, `clear(bg)`, `put(col, row, text, fg, bg)`, `put_styled(..., ui.Style(bold, italic, underline))`, `cursor(col, row)`, `set_cursor(col, row, Bar\|Block\|Underline)`, `present()`, `next_event() ui.Event`, `poll_event() ui.Event?`, `modifiers()`, `set_font_size(px)` (10 to 40), `font_size()`, `set_title(t)`, `close()`; `Event`: `Key(key, ctrl, alt, shift)` (`key` a `ui.KeyCode`: `Text(t)`, `Enter`, `Escape`, `Up`, `PageDown`, `F(n)`, ...; `key.name()`, `key.text()`), `Resize(cols, rows)`, `Close`, `Mouse(col, row, button, kind)` (`ui.MouseButton` `Left`/`Middle`/`Right`, `ui.MouseKind` `Down`/`Up`/`Drag`), `Scroll(lines)`; `clipboard_get() str?`, `clipboard_set(text)`; colors `0xRRGGBB` or `ui.RED` and nine others |
| `std.ui.widgets` | `TextField`, `ListView` (fuzzy `filter`), `Button`, `Dialog`, `Pane`, `Tree`: `render(rect, theme) List[widgets.Run]`, `draw(mut w, rect, theme)`, `on_event(ev[, rect]) widgets.Action` (`Ignored`, `Changed`, `Submit`, `Cancel`); `widgets.Rect(col, row, cols, rows)`; `text_rows(runs, cols, rows)` prints a screen headless |

## Tests and assertions

```ail skip=context
@test fn doubles()           # `@test` on the same line
    assert_eq(double(4), 8)  # traps showing both values if unequal
    assert(double(1) == 2)   # traps if false
```

A test returns nothing or `!`, can't be called, and `ail run` ignores it. `?`
works in it: `assert_eq(parse("7")?, 7)`. `assert` and `assert_eq` also work
in ordinary code (a failure traps).

A test with parameters is a **property**: `ail test` runs generated inputs
(`--cases N`, default 100; `--seed S` replays) and shrinks a failure to the
smallest input. Parameters: `int`, `float`, `bool`, `char`, `str`, `List`, `Map`,
`Set`, optionals, and structs and enums of those.

`ail test --filter PATTERN FILE` runs the tests whose name contains PATTERN (`*` is
any text; repeat `--filter` for several); `--list` prints the names without running.

A **fixture** is shared setup made fresh for each run of a test: `@fixture fn tmp() T`
(or `!T` if setup can fail; a failure fails the test, with `context: setup of fixture tmp` shown).
`@fixture("drop")` names a teardown `fn drop(v T)` that always runs, in reverse
order, after pass, fail or trap. A test takes it with `@test @needs("tmp")` and a
parameter `tmp T`: `@test @needs("a") @needs("b") fn t(a A, b B, n int)`
(`n` is still a generated input).

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
# a `unique struct` may hold a `TaskGroup` field: its release waits the tasks
ch := Channel[int](8)         # bounded: send waits while full
ch.send(1)
v := ch.recv() else break     # none once closed and empty
w := ch.try_recv() else 0     # none if nothing is waiting now
u := ch.recv_for(50) else 0   # waits at most 50 ms
task.set_threads(4)           # cap the pool from the next loop; task.threads() reads it
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

A cancel reaches every task the cancelled task started. Cancelled, `recv` gives
`none`, a full-channel `send` drops its value, `sleep_ms` returns at once, and
`proc.run` fails; the scope still waits. `ail test --schedule=random:SEED` replays an interleaving.

Each task handle is waited or added to a group exactly once, on every path. A
handle can't be stored or passed; channels and mutexes can, and copies share one.
If every task is blocked, the program traps (deadlock).

## Effects

Effects are inferred, never written: `io` (`print`, `fs`, `io`, `proc`),
`fail` (`T ! E` result), `mutates` (`mut` parameter), `spawn`, and the capabilities
`fs`, `env`, `proc`, `clock`, `random`, `net`, `net.serve` (`net.serve`, `net.listen`), `ui`. A call through `any Trait` counts every
implementation.

```ail
@pure fn area(w int, h int) int   # error if it reaches print or a capability
    w * h
```

A call whose value is dropped must do something (`eff.use.1`): `area(3, 4)` alone is an
error, `xs.sorted()` alone is fixed to `xs.sort()`, and `_ := area(3, 4)` drops a value on purpose.

`@inline` is accepted and ignored. `@deprecated("use area")` warns at each call.
Unknown attributes are errors.
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

C functions are declared, not imported. Types: `int`, `float`, `bool`,
`i8`...`u64`, `f32`, and `*T` pointers; at most 6 integer-like and 8 float
arguments; no `mut`. `std.mem`: `cstr(s) *u8`, `from_cstr(p) str`, `free(p)`.
A narrow C `int` result is declared `i32`.
Pointers: `p.read()`, `p.write(v)`, `p.offset(n)`, `p.is_null()`;
`mem.alloc(n, 0)` allocates `n` ints set to the example (`0.0` gives `*float`),
`mem.null()`, `mem.cast(p, example)`, `mem.free(p)`; bad accesses trap.

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
