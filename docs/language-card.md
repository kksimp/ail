# AIL language card

> v1, 2026-10-20. Non-normative: the [spec](spec/) is authoritative and the
> [quick reference](quick-reference.md) has the long form. Budget 16,000 tokens (ADR 1110)
> (five-tokenizer mean, `tools/tokbench/run.sh docs/language-card.md`).

## Running programs

```sh
./ail run FILE.ail           # check, then run
./ail check --json FILE.ail  # diagnostics only; each cites a spec rule ID
./ail test FILE.ail          # run the `@test` functions; --filter PATTERN, --list
./ail fmt FILE.ail           # canonical layout; `ail fix FILE.ail` applies diagnostics' fixes
./ail api --text FILE.ail    # declarations and `##` docs without bodies
./ail explain RULE           # a rule's text, its ADRs and an example
./ail effects FILE.ail       # what each function may do
./ail run --trace --watch NAME FILE.ail   # events and a snapshot at a trap
./ail run --record R.json FILE.ail     # `--replay R.json` repeats the run
./ail CMD --budget N             # any command: about N tokens, errors first; `--from K` continues
./ail edit FILE.ail --test <<E   # change code by declaration name, then fmt, check, test
average                          # the declaration
-total / xs.len()                # one -/+ pair: replaces a unique part of a line
+total / xs.len().max(1)         # more lines: whole lines; ` line` keeps one; `rename f g`
E
./ail diff OLD NEW               # changed declarations with their lines, diagnostics, tests
```

`todo` is a typed hole: `ail check` warns with its expected type and the names in scope. `uhwhatnow?` also ranks what fits, with a fix each.

Modules: `use a.b` imports directory `a/b` as `b.item`; `pub` exports. Standard
modules need no `use` (`fs.read_text(p)`); a local of the same name hides one.

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

Tuples: `t := (1, "a")` is an `(int, str)`: `t.0`, `a, b := t`, arms like
`(0, s) =>`; `==`, `<` and map keys work elementwise.

Distinct types: `type UserId = int` (any built-in scalar base) is a new type
that never mixes with `int` or another id type. `UserId(3)` makes one,
`int(id)` gives the value back; same-type operators, `==`, `<`, text, map keys
and JSON work as the base's; no fields, methods or `impl`; free at run time.

`random.int(1, 6)` (inclusive), `random.float()`, `random.seed(n)`. Also `use std.csv` (`parse`, `records`, `write`), `use std.args` (`parse`), `use std.log` (`info`, `warn`, `err`), `use std.time.date` (`date.utc(ms).format("%Y-%m-%d")`); float math `x.sqrt()`, `floor()`, `ceil()`, `round()` (type.num.9), `x.sin()`, `cos`, `tan`, `atan`, `atan2(x)`, `exp`, `ln`, `log10`, `cbrt`, `hypot(y)`, `x.pow(y)` (type.num.16).

Regex (linear time; RE2 syntax, no backreferences): `re := regex.compile(r"(\d+)-(\d+)")?`, then `re.is_match(s)`, `re.find(s)` (`Match?`: `start`, `end`, `text`), `find_all`, `captures(s)` (`groups` `List[str?]`, `named`), `replace(s, "$2-$1")`, `replace_first`, `split(s)` (each is also `regex.NAME(re, s)`), `regex.group_all(re, s, n)` (group `n` of each match, `""` if unset), `regex.escape(s)`. Offsets count characters.

`t := tensor.from(xs, shape)` (or `tensor.zeros(shape)`): `t.matmul(u)`, `t + u`, `t * 2.0`, `t.dot(u)`, `t.sum(axis?)`, `t.transpose()`, `t.map(|x| x * 2.0)`, `t.to_list()`; SIMD and multi-core.

Standard input: `line := io.read_line() else break` gives a `str?` without its line ending (`\n` or `\r\n`; so `line.len()` does not count it), `none` at the end of input (std.io.1); `io.read_all()` gives all of it.

`fs.read_text(p)?`, `fs.write_text(p, s)?`, `fs.walk(dir)?`, `fs.glob("src/**/*.ail")`. Streaming: `use std.file`; `mut r := file.open(p)?`, `line := r.read_line()? else break`, `r.read_exact(n)?`, `r.seek(pos)`; `mut w := file.create(p)?` (or `file.append(p)?`), `w.write_line(s)?`, `w.close()?` (leaving scope flushes). `io.read_exact(n)` is `bytes?`.
`proc.run(cmd, args, input?, timeout_ms?)?` gives `.status`, `.out`, `.err`, `.ok()`, `.text()`.
`net.get(url)?`, `net.post(url, body, content_type)?` give `.status`, `.headers`, `.body` (`http://`, `https://`; `get_bytes`, `post_bytes`, `request_bytes` give `.data`); `net.request(method, url, opts)` with `net.options()`; servers: `srv := net.serve(addr)?`, `ex := srv.accept()?`, `ex.respond(net.Response(status: 200, headers: {}, body: "hi"))?`; raw TCP: `net.connect(host, port)?`.

Constants: top-level `NAME := value`, computed at compile time.

## Structs, enums and methods

```ail
struct Point
    x int
    y int

enum Color
    Red
    Green

fn Point.sum(self) int            # method; `self` is read-only
    self.x + self.y               # canonical; bare `x + y` works too

fn Point.shift(mut self, dx int)  # may change self
    self.x += dx

fn Point.origin() Point           # associated function: Point.origin()
    Point(x: 0, y: 0)

fn main()
    mut p := Point(y: 2, x: 1)    # name fields, in any order
    p.shift(3)                    # `mut self` methods need a `mut` variable
    p.shift(dx: 3)                # arguments may be named, after positional ones
    print(p.sum())                # method calls always have ()
    c := Color.Red                # qualified, unless the type is known: `c = Green`
    p.x = 10                      # fields of a `mut` variable can be assigned
```

`==` and `print` work on every value: `print(p)` shows `Point(x: 1, y: 2)`.
An enum with no fields has `Color.variants()` (a `List[Color]`), `Color.from_str("Red")` (a `Color?`, exact variant name) and `c.name()` (`"Red"`) (type.enum.20).

A keyword may name a field, a method or a label: `error str`, `o.error`,
`Outcome(error: "")`, `fn T.any(self)`. In a method reach such a field as
`self.error`; anywhere else a keyword is not a name.

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

Patterns nest in fields: `Add(Num(a), Num(b)) => a + b`, `Word("fn", (1, _))`. A bare name in a field binds it, so write a fieldless variant qualified there (`Cons(_, Items.Nil)`); a `match` that nests names each missing case, e.g. `Add(Num(_), Neg(_))`. Structs match by field name, as they are built: `Point(x: 0, y) =>` (a bare field name binds it, unnamed fields match anything); variants take names too, `Rect(w: 0) =>`.

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
  `x := ... else ...` gives a fallback or leaves (`return`, `continue`, `break`);
  a function can end with `m.get(k) else return Missing(key: k)` (`err.else.7`).
- Several error types: `fn load() Config ! IoError | ParseError`. No success
  value: `fn save() ! IoError` (or `fn save() !`); `save() else return` leaves.
- Which one failed: `match err` with `IoError =>` (there `err` is an
  `IoError`) and `ParseError.Empty =>`; every member must be covered.
- Standard errors carry a `kind` enum: `if err.kind == NotFound` or
  `match err.kind` (`fs.IoKind`, `proc.ProcKind`, `net.NetKind`); never a string.
  So are `app.Event.kind` (`ev.kind == Press`) and keys (`key == Escape`,
  `Text("q")`).

## Traits and generics

```ail
trait Shape
    fn area(self) int
    fn describe(self) str           # a default; defaults call other methods on `self`
        "shape: {area()}"

struct Circle
    r int

struct Square
    s int

impl Shape for Circle                # defines every method without a default
    fn area(self) int
        3 * r * r

impl Shape for Square
    fn area(self) int
        s * s
    fn describe(self) str           # an `impl` replaces a default
        "square {s}"

struct Version
    major int
    minor int

impl Ord for Version                # no body: compares the fields in order, so `<` works

struct Stack[T]                     # a generic type; its methods use `T` and `Self`
    items List[T]

fn Stack[T].push_twice(mut self, x T)
    items.push(x)
    items.push(x)

fn Stack[T].top(self) T?
    items.last()

enum Color
    Red
    Green
    Rgb(r int, g int, b int)

fn is_red(c Color) bool
    match c
        Red => true
        Rgb => false                # bare `Rgb` matches any fields
        Green => false

fn main()
    mut shapes := List[any Shape]()
    shapes.push(Circle(r: 2))
    shapes.push(Square(s: 3))
    for s in shapes
        print(s.describe())
    print(Version(major: 1, minor: 2) < Version(major: 1, minor: 3))
    mut st := Stack[int](items: [])
    st.push_twice(4)
    print(st.top())
    print(is_red(Color.Red))

fn largest[T Ord](items List[T]) T  # `T`: any type that implements `Ord`
    mut best := items[0]
    for x in items
        if x > best
            best = x
    best
```

- `impl Ord for Version` with no body compares the fields in order. `<` and
  friends work on `int`, `str`, lists and `Ord` types. Customise with
  `impl ToStr` (`fn to_str`), `impl Eq` (`fn eq`), or `fn cmp`.
- `impl ToJson for T` and `impl FromJson for T` with no body derive
  `t.to_json()` and `T.from_json(j) T ! JsonError` (fields by name; lists,
  `Map[str, T]`, `T?`, enum variants as `{"kind": "Name", ...}`); maps and
  optionals of `ToJson` values are `ToJson` themselves (`print_json(m)`). A trait
  `@derive(f) trait X` names a generator `fn f(t TypeInfo) str ! DeriveError`
  that writes an empty `impl`'s methods; `ail api --expanded` shows them.
  Generic types derive through bounds: `impl FromJson for Pair[A FromJson,
  B FromJson]`. `impl Default for T` derives `T.default()`.
- `trait Ord Eq` names supertraits (joined with `+`); a bound gives their
  methods. `fn zero() Self` in a trait is called `T.zero()` in generic code
  and `Type.zero()` on a type; `x.zero()` is an error.

Optionals nest and `else` works inside expressions:

```ail
fn first_word(s str) str?
    s.split(" ").first()

fn pick(xs List[int?]) int??
    xs.first()                      # `int??`: no element, or an element that is `none`

fn sign(n int) str
    if n < 0
        return "neg"
    if n >= 0
        return "non-neg"
    unreachable("n is an int")      # stops the program; has any type

fn main()
    a := first_word("") else "-"            # `else` inside an expression
    n := (["7"].first() else "0").to_int() else 0
    print("{a} {n + (first_word("a b") else "").len()} {sign(n)}")
    if v := pick([none, 3])                 # peels one level: v is `int?`
        print(v)
```

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
sign := if n < 0 then "-" else "+"   # the one-line form

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
    Green => "green"  # an arm can also be an indented block

match ch             # literals and `|`; ints, strings and chars need `_`
    '+' | '-' => continue           # a one-line statement arm
    '0' => total += 1
    'a'..='z' => lower += 1         # `lo..hi` or `lo..=hi`, int or char
    _ if ch == ' ' => skip += 1     # guard; a guarded arm covers nothing
    _ => return ch
```

`if` and `match` work as statements or values; a function can end with either.

A declared function is a value of type `fn(int) int`: `fn apply(f fn(int) int, x int) int`
calls `f(x)`, and `apply(double, 4)` passes it (not generic or `mut` ones). A literal
`|x| x + n` is one too: one expression on its line, types from context or written
(`|x int| ...`). It copies the locals it names when made; they are read-only inside
(share via a `ref`). `fn(str) int ! E` can fail; `?` in a literal leaves the literal.
Collections share one vocabulary (List, Set, Map, Deque, Seq): `map filter any all find count fold flat_map take skip take_while skip_while zip enumerate group_by`; a function takes the element, on a map `|k, v|`: `xs.filter(|x| x > lo)`, `xs.fold(0, |acc, x| acc + x)`.
A `fn name(params) R` with a block inside a body is a local function: it copies the
same way, can call itself, `return`s from itself, and is a value (no `mut` params).

## Operators and strings

- `+ - * / %` on `int`. Overflow and division by zero stop the program (a
  trap). `/` rounds toward zero.
- `== != < <= > >=`. Comparisons can't be
  chained: write `a < b and b < c`.
- `and`, `or`, `not` (not `&&`, `||`, `!`). Conditions must be `bool`: write
  `if n != 0`, not `if n`.
- `float`: `+ - * /`, never traps; no mixing with `int`: `n.to_float()`,
  `x.to_int()`. An int literal works where a float is expected.
- Sized numbers `i8`…`u64`, `f32`: typed by context, same-type operands only,
  overflow traps; `x.to_u8()` converts.
- `int`: `min`, `max`, `abs`, `to_hex`, `checked_add`, `checked_sub`, `checked_mul`, `checked_div`, `checked_rem`, `checked_neg` (`int?`, `none` on overflow or zero divide); `wrapping_add`, `wrapping_sub`, `wrapping_mul`, `wrapping_neg` (two's complement); `saturating_add`, `saturating_sub`, `saturating_mul` (clamp); `to_u8()`, `to_u16()`, `to_u32()`, `to_u64()`, `to_i8()`, `to_i16()`, `to_i32()`, `to_f32()` (sized, type.num.15); bits: `a & b`, `a | b`, `a ^ b`, `~a`, `a << k`, `a >> k` (count 0-63, else trap), the same as the methods `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr`; `x &= e` (also `|= ^= <<= >>=`) on a name. A bit operator never mixes with another operator without parentheses: `(x & 1) == 0`, `(a + b) & m`, `a | (b & c)`.

## Strings and characters

```ail skip=context
s := "total: {a + b}"      # interpolates any expression; `\{` is a literal brace
m := """
    line one
      indented
    """                     # multi-line: "line one\n  indented"; the common indent is removed, the closing line adds no newline
r := r"C:\dir\{x}"          # raw: no escapes, no interpolation
c := 'x'                   # a char; '\n', '\'', '\u{e9}'
print("{n:>6}|{r:.2}|{n:08}|{n:b}|{n:,}|{s:*^8}")   # width, decimals, zero pad, binary, thousands, fill and align
```

- Format specs are `[[fill]align][width][.decimals][x]` and nothing else: `<` `>` `^`
  align (numbers default right, others left); `.N` needs a float, `x` an int.
- `"a,b".split(",")` gives `["a", "b"]` (`split(sep, n)`: at most `n` pieces); `"12".to_int()` gives an `int?` (`"ff".to_int(16)` reads another base),
  `"2.5".to_float()` a `float?`.
- `s.len()` counts characters, `s.byte_len()` bytes. `s.chars()` (a
  `List[char]`; strings have no indexing or `for`), `s.lines()`, `s.trim()`, `s.contains(x)`, `s.starts_with(x)`,
  `s.ends_with(x)` (`x` a `str` or `char`), `s.strip_prefix(p)` (gives
  `str?`), `s.pad_left(width, ' ')`, `pad_right`, `center`, `is_empty()` (also on collections), `words()`, `trim_start`, `trim_end`, `count(sub)`, `rfind`, `s.slice(a, b)` (characters), `s.char_at(i)` (`char?`), `s.find(x)` (`int?`), `s.replace(a, b)`, `s.to_lower()`, `s.to_upper()`, `s.strip_suffix(p)` (`str?`), `s.repeat(n)`.
- `xs.join(", ")` joins any list (`cs.join("")` for chars).
- `char`: `c.is_digit()`, `c.is_alpha()`, `c.is_space()`, `c.is_alnum()`, `c.is_upper()`, `c.is_lower()`, `c.to_upper()`,
  `c.to_digit(16)` (gives `int?`), `c.code()`, `char.from_code(n)`.
- There's no `+` on strings and no `{{`; write `\{`.
- `bytes` (immutable): `b"GET /\r\n\x00"`, `s.to_bytes()`, `b.len()`, `b[i]` (`u8`), `b[a..b]`, `a + b`,
  `b.to_str()` (`str?`), `b.text()` (lossy), `b.find(x)`, `b.hex()` (`b.hex(" ")` spaces the pairs), `b.base64()`, `bytes.from_hex(s)` and `bytes.from_base64(s)` (`bytes?`), `bytes.from_list(xs)` (a `List[u8]`), `a.concat(b)`, `fs.read_bytes(p)?`, `fs.write_bytes(p, b)?`.

## Lists

```ail skip=context
mut xs := [3, 1, 2]    # a trailing comma is allowed; `[]` needs a known type
first := xs[0]         # out of bounds stops the program
part := xs[1..3]       # a copy of elements 1 and 2; also xs[1..], xs[..2]
xs.push(4)             # push, pop, sort and swap need a `mut` variable
xs.swap(0, 1)
xs[0] = 7              # elements can be assigned, including fields: xs[i].qty = 0
n := xs.len()          # int
mut out := []          # typed by its first use: out.push("a") makes a List[str]
```

Also `insert(i, x)`, `remove_at(i)`, `extend(ys)`, `clear()`, `reverse()` (in place), `reversed()` (a new list; descending is `xs.sorted().reversed()`); `pop()`/`first()`/`last()` (give `T?`), `contains`, `index_of` (gives `int?`),
`sorted()`/`sort()`, `min()`/`max()` (give `T?`), `sum()`, `take(n)`, `chunks(n)` and `windows(n)` (lists of `n` elements), `flatten()` (a `List[List[T]]` into one list). Slices
don't clamp and negative indexes are errors. Order by a key (stable):
`xs.sorted(by: |p| -p.age)`, also `sort`, `min`, `max`.

Comprehensions: `[x * x for x in xs if x > 0]`, sets `{x for ...}`, maps
`{k: v for ...}`. Pairs: `for i, x in xs.enumerate()`, `for a, b in zip(xs, ys)`.

Generators: a function returning `Seq[T]` with `yield x` in its body
(`fn count(n int) Seq[int]`) runs lazily, up to each `yield`, per `next()`
(`T?`). `for` takes any type with `fn next(mut self) T?`, `Seq` included; a
`Seq` is unique, read once: `count(9).filter(|x| x > 2).map(f).take(3).collect()`,
also `Seq.of(xs)`, `take_while`, `skip`, `enumerate`, `zip`, `chain`, `count`.

`for x in xs` gives a copy of each element. To change the elements
themselves: `for mut item in items` (`items` must be `mut`), then `item.qty += 1`.

Grids (one flat block, for 2-D data): `mut g := Grid.fill(rows, cols, 0)` or
`Grid.from(xs)`, `g[i, j] += 1`, `g.rows`, `g.cols`, `g.row(i)`, `g.to_list()`.

Priority queue: `h := Heap[int]()` (smallest first; `Heap[int](max_first: true)`
for largest), `h.push(x)`, `h.pop()` and `h.peek()` give `T?`, `h.len()`,
`h.is_empty()`, `Heap.from(xs)`, `Heap.from_max(xs)`, `h.to_sorted()` (all elements in pop order). Double-ended queue: `d := Deque[int]()`,
`push_front`, `push_back`, `pop_front()`, `pop_back()` (`T?`), `first()`, `last()`,
`d[i]`, `d.set(i, v)`, `for x in d`, `d.to_list()`.

## Maps and sets

```ail skip=context
mut ages := {"ann": 31, "bob": 27}     # Map[str, int]; keeps insertion order
ages["cy"] = 40                        # inserts or replaces
ages["dan"] += 1                       # `op=` on a missing key starts at 0
mut counts := {}                       # typed by first use: counts[w] += 1
b := ages.get("dan") else 0            # `ages["dan"]` traps if missing; same: `ages.get_or("dan", 0)`
for name, age in ages                  # each entry; also keys(), values()
    print("{name}: {age}")
mut seen := Set[str]()                 # or a literal: {"a", "b"}
if seen.add("x")                       # true if it wasn't there
    print("new")
```

Also `contains`, `remove`, `len`, `sorted`, `m.map_values(|v| e)`; sets and `keys()` have `union`, `intersect`, `difference`, `is_subset(t)`; `m.push(k, x)` appends to a list-valued entry (a missing `k` starts `[]`); `m[k] += 1` counts up; `m.top(n)` gives the `n` largest-count keys, ties in insertion order. Keys can't be floats.

## Values and `mut`

Assigning or passing copies: changing `b` after `b := a` never changes `a`.

```ail skip=context
fn bump(mut p Point)   # the caller sees the change
    p.x += 1

bump(mut q)            # `mut` again at the call, on a `mut` variable
```

Two `mut` arguments of one call can't refer to the same value.

## Ownership

`unique` types are never copied; they move, and `Drop` cleans them up. `ref T` is a
cell that copies share. Lists of unique values move in with `xs.push(move x)`,
out with `pop`, and `xs[i]` borrows.

```ail
unique struct Conn
    id int

impl Drop for Conn                  # runs when the value goes away
    fn drop(mut self)
        print("close {id}")

fn Conn.finish(move self) int       # consumes the receiver: `c.finish()`
    id

fn release(move c Conn)             # `move` parameter: the caller writes `move c`
    print("release {c.id}")

struct Node
    value int
    children List[ref Node]         # `ref T`: a shared, mutable cell

fn main()
    defer print("done")             # runs when the block ends, in reverse order
    a := Conn(id: 1)                # a `unique` value is never copied
    b := Conn(id: 2)
    release(move a)                 # `a` is unusable afterwards
    n := b.finish()                 # so is `b`
    root := ref Node(value: 1, children: [])
    other := root                   # a copy of the reference, same cell
    other.value = 2                 # root.value is now 2
    print("{n} {root.value}")
```

## Tasks

```ail skip=context
t := spawn load(1)          # a task; args are copies
p := t.wait()?              # result, or fails with Trapped
g := TaskGroup()            # tasks returning nothing; waited at block end
g.add(spawn work(2))
ch := Channel[int](8)       # send waits while full; close() when done
v := ch.recv() else break   # none once closed and empty
w := ch.try_recv() else 0   # none if nothing now; recv_for(ms) waits at most ms
for v in ch                 # receives until closed and empty
    print(v)
select                      # first ready arm in order; channels of any types
    n := nums.recv() => total += n
    out.send(job) => sent += 1
    after 50 => print("idle")   # optional; `after 0` never waits
    closed => break         # required with a recv arm: all recv channels closed
m := Mutex(0)               # get, set, swap
with m as mut x             # holds m; stores x back on any exit
    x += 1
n := Atomic(0)              # int/bool: load, store, add (old value), cas
g.cancel()                  # task.cancelled(), task.check()? in the task
ys := [f(x) par for x in xs]   # elements on all cores, in order
```

Wait or `add` each task exactly once, on every path. A deadlock traps.

`par for` and `par` comprehensions use all cores; the result is what the sequential
loop gives, and provable conflicts are errors:

```ail
fn main()
    xs := [1, 2, 3, 4]
    mut out := [0, 0, 0, 0]
    par for i in 0..4               # asserts the iterations are independent
        out[i] = xs[i] * xs[i]
    sq := [x * x par for x in xs]   # a list computed on the thread pool
    n := Atomic(0)                  # shared by tasks and `par for` without a lock
    par for x in xs
        n.add(x)
    print("{out} {sq} {n.load()}")
```

## Contracts

`requires`, `ensures` (`result`, `old(e)`) and struct `invariant` are checked at run time (`ail run --no-contracts` skips them). A trait method's contracts bind its implementations.

## Tests

```ail
use std.fs

fn double(x int) int
    x * 2

@test fn doubles()                  # `ail test`; `?` works inside a test
    assert_eq(double(4), 8)         # traps showing both values
    assert(double(1) == 2, "message")   # optional `str` message
    assert_near(0.1 + 0.2, 0.3, 0.0001) # three floats: |a - b| <= eps
    assert_err(fs.read_text("no.txt"))  # traps unless the call fails; a second argument `E.Variant` checks which error (kind alone for a variant with data; `E.V(f: x)` the payload too)

@test fn add_commutes(a int, b int) # parameters: a property on generated inputs
    assert_eq(a + b, b + a)         # --cases N, --seed S replays; failures shrink
```

```ail
@fixture("drop")                    # made per test run, `drop` always runs after it
fn tmp() !str                      # `T` or `!T`; a setup failure fails the test
    "x"

fn drop(v str)                      # takes the value, returns nothing
    print("dropped {v}")

@test @needs("tmp")
fn uses(tmp str)                    # parameter named like the fixture
    assert(tmp == "x")
```

A block fenced `ail` in a `##` doc comment is a test; `# => text` asserts printed output.

## Packages, effects and the system

`ail.pkg` lists the package and its `dep`s, and may hold one line `ail 0.1` naming the language version (`mod.pkg.10`; `ail new` writes it); each `dep` reaches only its `caps` (spec `mod.pkg.*`).

Effects are inferred: `io`, `fail`, `mutates`, `spawn`, and the capabilities `fs`,
`env`, `proc`, `clock`, `random`, `net`, `net.serve`, `ui`. `@pure fn f()` is an error if it reaches
one; `@caps(fs)` limits a function to the listed capabilities and everything it calls, `@caps(none)` allows none (`eff.cap.6`). A call whose value is dropped and that has no effect is an error: `xs.sorted()` alone
means `xs.sort()`; write `_ := f()` to drop a value on purpose (`eff.use.1`).
`ail run --deterministic` denies `clock`, `random`, `proc` and `net`. `ail run --allow fs.read=./data,net=host:443` allows only what it names, and a use outside the scope ends with `trap[eff.allow.N]`.

`unsafe` allows C calls and `std.mem` pointers; `ail.pkg` needs `unsafe allow`, or `unsafe driver` for `std.hw`; see `docs/spec/14-ffi.md`. Structs of C-typed fields pass to C by value; a `fn(...)` parameter of an `extern c fn` takes a declared function's name as a callback (`mem.c_fn(f)` gives a `*u8`).

A window is a grid of text cells, drawn with `w.put(col, row, text, fg, bg)` and `w.present()`:

```ail
fn main()
    mut w := ui.open("Demo", 40, 10) else return   # fails headless
    w.put(1, 1, "Hello", 0xFFFFFF, 0x000000)      # col, row, text, fg, bg
    w.present()
    while true
        match w.next_event()
            Key(key, _, _, _) =>                   # Text("a"), Enter; (key, ctrl, alt, shift)
                if key == Escape
                    break
            Close => break
            Resize(_, _) | Mouse(_, _, _, _) | Scroll(_) => w.present()
```

`Json`, `parse_json(text)?` and the `ToJson`/`FromJson` derives read and write JSON. A member's payload is `j.str(k)`, `num`, `int`, `bool` or `arr`, `none` when missing (`j.int("retries") else 2`); `j.set(k, v)` (`mut`); `j.need(k)?.need_int()?` fails with `missing KEY`; `j.text(k, default)` and `j.flag(k, default)` never fail; `parse_json_lines(text)` gives each line that parses; `load_json[T](path)?`, `print_json(v)`.
