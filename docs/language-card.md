# AIL language card

> v2, 2026-11-15 (0.2). Non-normative: the [spec](spec/) is authoritative. Budget 16,000 tokens (ADR 1110)
> (`tools/tokbench/run.sh docs/language-card.md`). If your tool cuts long output, read it in two parts. `ail card full` prints all of it
> and `ail card TOPIC` one section. `ail` is the installed command; in a checkout of the AIL repository, run `./ail`.

## Running programs

```sh
ail check FILE.ail         # every diagnostic cites a rule; `fix: write CODE` is the corrected line; `--json`: an `ok` line when clean, else exit 2; when clean it runs the tests your edit can affect (`tests: 2 run, 2 pass (40 cached)`, exit 1 if one fails)
ail run FILE.ail           # check, then run; `ail build FILE.ail -o NAME` writes a program
ail test FILE.ail          # run the `@test` functions; --filter PATTERN, --list
ail fmt FILE.ail           # canonical layout; `ail fix FILE.ail` applies diagnostics' fixes
ail explain RULE           # a rule's text and an example; `ail explain std.io` lists a family, `ail explain exit` the rules naming it
ail ask std "split a string"   # a std function or built-in method (`stderr`, `argv`, `exit code` work); `ail ask how "sort descending"` a checked how-to
ail ask std pad_left is_alpha  # names: each one's signature, its rule text (edge behaviour) and examples in one call; `strip`, `parseInt` give the AIL name
ail eval '"ab".repeat(2)' 'xs := [3, 1, 2]' 'xs.get(7)'   # each expression's value and type, no file: what a call does on an edge input
ail card TOPIC             # one section of this card by a word of its heading (`ail card modules`); `ail card std.fs` lists a module
```

A program needs `fn main()`. `print(value)` writes any value and a newline. The sections run from what
every program needs to what few do: read to "Modules" before writing a first program, and look up the rest.

## Command-line programs

```ail norun
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

Standard input: `io.read_all()`, or `line := io.read_line() else break` (`str?`, no line ending, `none` at the end). `s.lines()` and `io.read_line()` drop a `\r` before the line end (split with `s.split("\n")` to see it). No standard module needs `use` (std.env.1, std.io.1 to std.io.4, std.use.4). `io.exit(code)` and `unreachable("why")` (the message is required) never return, so nothing follows them. `int` overflow traps; big sums: `mut t := bigint.from_int(0)`, `t += bigint.from_int(n)` (std.bigint.4, `ail ask how overflow`).

## Habits from other languages

Each comment names a form that does not compile and that no fix rewrites; the list under the block is rewritten for you.

```ail
fn qty(s str) int
    s.trim().to_int() else 0

fn add_word(mut out List[str], w str)      # `mut` goes before the name (not `out mut List[str]`)
    out.push(w)

fn main()
    mut seen := Set[str]()
    seen.add("a")
    mut m := {"a": 1}
    m["b"] = 2                             # not `m.set`, `m.insert`: an entry is set by assignment
    n := m.get("z") else 0                 # `m[k]` is a plain value and traps on a missing key
    if m.contains("a") and not seen.contains("z")   # not `"a" in m`
        print("found")
    qty := qty(" 7 ")                      # a local may take a function's name; from here on `qty("8")` is an error
    mut total := 0
    total = "12".to_int() else 0           # after `=`, `else` takes a value only
    v := "5".to_int() else                 # an `else` block must leave (`return`, `break`, `continue`) or end in a value; `x = e else` + block works too (err.else.8)
        print("not a number")
        return
    label := if n > 0 then "pos" elif n < 0 then "neg" else "zero"   # one line: any `elif` arms, then a final `else`
    line := "{label}:{qty}"
    mut parts := List[str]()
    for i in 0..3                          # a range is only a `for` bound, a slice index or a pattern
        parts.push("{i}")
    add_word(mut parts, "end")             # and `mut` again at the call, on a `mut` variable
    text := parts.join(",")                # text from many pieces: push them, then `join`; `s += piece` is an error
    shown := "[{parts.join(", ")}]"        # inside `{ }` it is plain code: quotes are written as they are, never `\"`
    size := (parts.first() else "").len()  # a `T?` has no methods: take the value with `else` first
    idx := [i for i in 0..3]
    xs := [3, 1, 2]
    part := xs[0..2]                       # also `xs.slice(0, 2)`; `xs.slice(1)` runs to the end
    ks := seen.sorted()                    # a set as a list; insertion order: `seen.to_list()`; not `keys()`
    pad := "7".pad_left(3, '0')
    print("{line} {text} {shown} {size} {idx} {part} {ks} {pad} {total} {v}")
```

- `ail check`, `build`, `run` and `test` rewrite the spellings below in your file and print `fixed PATH:LINE:COL: [RULE] ...`
  for each (fix.auto.1). Write the form after `->` yourself: after a rewrite, re-read the file before your next edit
  (`--no-fix` or `AIL_FIX=0` turns rewriting off).
  `&&` `||` `!x` -> `and` `or` `not x`; `else if` -> `elif`; `let x = 1`, `const x = 1` -> `x := 1`; `var x = 1` -> `mut x := 1`;
  `println` -> `print`; `x.to_string()`, `str(x)` -> `x.to_str()` or `"{x}"`; `"a" + b` -> `"a{b}"`; `len(xs)` -> `xs.len()`;
  `xs.append(x)` -> `xs.push(x)`; `xs + ys` -> `xs.concat(ys)`; `max(a, b)` -> `a.max(b)`; `range(n)` -> `0..n`;
  `insert` on a set -> `s.add(x)`; `m.contains_key(k)`, `m.has(k)` -> `m.contains(k)`; `m[k] else d` -> `m.get(k) else d`;
  `m.push(k, [x])` -> `m.push(k, x)`; `o.unwrap_or(d)`, `o ?? d` -> `o else d`; `is_some` on an optional -> `o != none`; `"0"` as a fill -> `'0'`;
  a lone `"{"` -> `"\{"`; a missing `mut` is added.
- `null` is `none`; an optional has no `map` or `unwrap` and no methods of its value: `v := x else d`, `if v := x`, `(x else d).len()`.
  `x?` hands the absence or failure to the caller, so it needs a function whose own result is `T?` or can fail; anywhere else write `else`.
- A range has no `rev`, `step_by` or third bound. Count down or by a step with `while` (`mut i := n - 1`, `while i >= 0`, `i -= 1`);
  walk a list backwards with `for x in xs.reversed()`.
- No `+` on lists and no `[...xs, x]`: `xs.concat(ys)` is a new list (`xs.concat([x])`, `[x].concat(xs)`); `xs.extend(ys)` grows a `mut` list in place.
- `exit`, `os.exit`, `sys.exit`, `process.exit` are `io.exit(code)`; `eprintln`, `stderr.write` are `io.eprint(value)`; `argv` is `env.args()`.
  The checker names these and other guessed std names, and prints the corrected line as `fix: write CODE`.
- `fn f() int ! str` fails with text: `return "bad input: {s}"` is the error, `?` passes any error on as text, `else e` binds a `str` (err.str.1, err.decl.2); `!T` infers the errors.
  With no result or a `str` result, fail with the standard `Fail`: `fn f() ! Fail`, `return Fail(msg: "bad")`; `e.msg` is the text, and `?` passes a `T ! str` call's text on as a `Fail` (err.fail.1, err.fail.3).

## Compiles, but is not what you meant

Little reaches these: no fix, and a warning only where noted. Check them yourself.

- `n := m.get(k) else 0 + 1` is `else (0 + 1)`, not `(m.get(k) else 0) + 1`: `else` binds looser than every operator, so
  nothing is added to the entry. `check` warns (`expr.prec.1`) and suggests the parentheses; write `(m.get(k) else 0) + 1`.

- `it := m[k]` copies the entry: changing `it` leaves `m` alone. Write `m[k].qty += 1`, or store it back, `m[k] = it`.
- `for x in xs` gives copies; `for mut x in xs` changes the elements. A function changes its caller's value only
  through a `mut` parameter, `fn f(mut p T)`, called `f(mut q)`.
- `s.lines()` and `io.read_line()` drop a `\r` before the line end; to see it, split with `s.split("\n")`.
- `int` overflow and division by zero trap; `/` rounds toward zero (`-7 / 2` is `-3`) and `%` takes the sign of the left side (`-7 % 3` is `-1`).
- A whole `float` prints with `.0`: `print(4.0 / 2.0)` shows `2.0`; for `2` write `x.to_int()` (cuts toward zero).
  `x.round()`, `floor()` and `ceil()` give a `float` (`round` of 2.5 is `3.0`: halves go away from zero); `"{x:.2}"` rounds the exact value half to even.
- Padding never cuts: `s.pad_left(4, ' ')` and `"{s:>4}"` leave a longer `s` whole; cut it yourself with `s.slice(0, 4)`, which traps past `s.len()`.
- `s.split(" ")` keeps empty pieces (`"a  b"` gives `["a", "", "b"]`; `"".split(",")` is `[""]`); `s.words()` splits on runs of any whitespace and gives none.
- `c.is_alpha()`, `is_alnum()`, `is_upper()`, `is_lower()` and `is_space()` follow Unicode (`é` is alphabetic); only `is_digit()` is ASCII `0` to `9`.
  For ASCII letters compare: `'a' <= c and c <= 'z'`. A digit's value is `c.to_digit(10)` (`int?`); `c.code()` is the code point (55 for `'7'`).
- `print(Color.Red)` shows `Color.Red`; `c.name()` is `Red`. Inside a list, map or struct a `str` prints quoted: `print(["a"])` shows `["a"]`.
- `xs.count(f)` takes a function: count a value with `xs.count(|x| x == v)`; `xs.counts()` is a `Map[T, int]` of every element's count.
- `" 7".to_int()` and `"+7".to_int()` are `none`: an optional `-` and digits only, so trim first.
- Strings compare by bytes: `"B" < "a"`. `s.len()` counts characters; `s.byte_len()` counts UTF-8 bytes.

## Layout

- Blocks are indented by exactly 4 spaces. No tabs, braces or `;`.
- A statement ends at the end of its line, unless the line ends inside
  `(`, `[` or `{`, or after a binary operator or a comma.
- `#` starts a comment. `##` starts a doc comment.

## Functions

```ail
fn add(a int, b int) int     # `name Type`, no colon or arrow (gram.fn.2); return type after `)`
    a + b                    # the last expression is the result

fn greet(name str)           # no return type: returns nothing
    print("hello, {name}")

fn half(n int) int
    requires n >= 0          # contract: a pure bool; false traps
    ensures result * 2 <= n  # `result` is the returned value
    n / 2
```

`return x` or bare `return` exits early. `fn usage() never` never returns: its body ends in `io.exit(code)`, `unreachable("why")` or
another `never` call, and a call fits where any value is expected (`n := s.to_int() else usage()`, type.never.1 to 3).
A parameter may default (`retries int = 3`, expr.call.4): leave it out, or pass it by name only.
In `ensures`, `old(e)` is `e` on entry. A struct's `invariant COND` lines hold after construction and each `mut self` method.

## Bindings

```ail
x := 1           # `:=` declares a new name (immutable)
mut n := 0       # mutable
n = 5            # `=` changes an existing `mut` name
n += 1           # also -= *= /= %=
```

- A variable that is changed later needs `mut` where it is declared: assigning it, a field or an
  element, and every changing method (`push`, `add`, `sort`, `m[k] = v`) (expr.bind.3, trait.method.3).
- A name can't be declared twice in one function, even in an inner block: to
  update a `mut` variable in a loop, use `=`. Only `|x| e` and `[e for x in xs]` may reuse a local's name (expr.bind.16).
- A local can't have the name of a type or a constant (expr.bind.5); a function's, module's or (in a method) field's name it hides
  to the end of its block: `qty := qty(s)`, then `self.qty` (trait.method.7).
- A local takes no type annotation (not `idx: List[int] = ...`): an empty value is typed by its first use
  (`mut out := []`, `mut best := none`, typed by `best = v`, and only `best == none` may come before; a `{}` by `m[k] = v`, `m.add(x)`, `m.push(k, x)` or `m.get_or(k, d)`; `m.contains(k)` first only tests; `m[k] = (m.get(k) else d) + v` types it too) or names its type, `List[int]()`, `Map[str, int]()`.

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
sign := if n < 0 then "-" elif n == 0 then "0" else "+"   # the one-line form: any `elif` arms, a final `else`

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
    Green => "green"  # an arm can also be an indented block

match ch             # literals and `|`; ints, strings and chars need `_`
    '+' | '-' => continue           # a one-line statement arm
    '0' => total += 1
    'a'..='z' => lower += 1         # `lo..hi` or `lo..=hi`, int or char
    _ if ch == ' ' => skip += 1     # guard; a guarded arm covers nothing
    _ => return ch
```

`if` and `match` work as statements or values; a function can end with either.

Collections share one vocabulary (List, Set, Map, Deque, Seq): `map filter any all find count fold flat_map take skip take_while skip_while zip enumerate group_by`; a function takes the element, on a map `|k, v|`: `xs.filter(|x| x > lo)`, `xs.fold(0, |acc, x| acc + x)`.

## Types

`int` (64-bit), `float` (64-bit IEEE), `bool` (`true`, `false`), `str`,
`char` (`'a'`), `List[T]`, `Map[K, V]`, `Set[T]`, structs, enums, optionals
`T?`, results that can fail, `T ! E`, generic types, and `any Trait`.

Tuples: `t := (1, "a")` is an `(int, str)`: `t.0`, `a, b := t`, arms like
`(0, s) =>`; `==`, `<` and map keys work elementwise.

Constants: top-level `NAME := value`, computed at compile time.

## Operators and numbers

- `+ - * / %` on `int`. Overflow and division by zero stop the program (a trap). `/` rounds toward zero; `a % -1` is 0 (`checked_rem(-1)` is never `none`). Past 64 bits, and operators on your own types: see Big numbers and operator traits.
- `== != < <= > >=`. Comparisons can't be
  chained: write `a < b and b < c`.
- `and`, `or`, `not` (not `&&`, `||`, `!`). Conditions must be `bool`: write
  `if n != 0`, not `if n`.
- `float`: `+ - * /`, never traps; no mixing with `int`: `n.to_float()`,
  `x.to_int()`. An int literal works where a float is expected.

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

- Format specs (`{n:>6}`, `{x:.2}`): see Format specs and chars.
- `"a,b".split(",")` gives `["a", "b"]` (`n` separators give `n + 1` pieces, empty ones kept; an empty separator traps: use `chars()`; `split(sep, n)`: at most `n` pieces); `"12".to_int()` gives an `int?`: an optional `-` and digits only, so `" 7"` and `"+7"` are `none` (trim first; `"ff".to_int(16)` reads another base),
  `"2.5".to_float()` a `float?`.
- `s.len()` counts characters, `s.byte_len()` bytes. `s.chars()` (a
  `List[char]`; strings have no indexing or `for`), `s.lines()`, `s.trim()`, `s.contains(x)`, `s.starts_with(x)`,
  `s.ends_with(x)` (`x` a `str` or `char`), `s.strip_prefix(p)` (gives
  `str?`), `s.pad_left(width, ' ')`, `pad_right`, `center` (the fill `char` is required; a string already that wide is returned whole), `is_empty()` (also on collections), `words()` (pieces between runs of whitespace), `trim_start`, `trim_end` (`trim` removes any Unicode whitespace, `\r` and `\t` included), `count(sub)`, `rfind`, `s.slice(a, b)` (characters; `s.slice(a)` to the end), `s.char_at(i)` (`char?`), `s.find(x)` (`int?`; `s.find(x, from)` searches from character `from`), `s.replace(a, b)` (every occurrence), `s.to_lower()`, `s.to_upper()`, `s.strip_suffix(p)` (`str?`), `s.repeat(n)`.
- `xs.join(", ")` joins any list (`cs.join("")` for chars).
- There's no `+` or `+=` on strings: interpolate (`"{a}{b}"`), or collect pieces with `out.push(piece)` and
  finish with `out.join("\n")`. No `{{`: write `\{`, and `}` is never escaped: `print("\{\"id\": {n}}")` prints `{"id": 3}`; the text `{}` is `"\{}"`.
  Text of any value: `x.to_str()` or `"{x}"` (`n.to_str(2)` in another base).
  A fill or separator `char` is `'0'`, not `"0"`. The list is the receiver: `xs.join(", ")`, not `", ".join(xs)`.

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

Also `slice(a, b)` and `slice(a)` (the same as `xs[a..b]`, `xs[a..]`), `insert(i, x)`, `remove_at(i)`, `extend(ys)`, `concat(ys)` (a new list; lists have no `+`), `clear()`, `reverse()` (in place), `reversed()` (a new list; `sorted().reversed()` reverses ties: `ail ask how "sort descending"`); `pop()`/`first()`/`last()`/`get(i)` (give `T?`; `xs.get(i)` is `none` when `i` is outside the list, where `xs[i]` traps), `contains`, `index_of` (gives `int?`),
`sorted()`/`sort()` (stable), `min()`/`max()` (give `T?`), `sum()` (0 for an empty list), `counts()`, `take(n)` and `skip(n)` (stop at the end, no trap), `chunks(n)` and `windows(n)` (lists of `n` elements), `flatten()` (a `List[List[T]]` into one list). Slices
don't clamp and negative indexes are errors: the last element is `xs.last()` (`T?`) or `xs[xs.len() - 1]`. Order by a key (stable):
`xs.sorted(by: |p| -p.age)` or `by: |p Person| -p.age`, also `sort`, `min`, `max`; a tuple key `(a, b)` orders by `a`, then `b`.

Comprehensions: `[x * x for x in xs if x > 0]`, sets `{x for ...}`, maps
`{k: v for ...}`. Pairs: `for i, x in xs.enumerate()`, `for a, b in zip(xs, ys)`.

`for x in xs` gives a copy of each element. To change the elements
themselves: `for mut item in items` (`items` must be `mut`), then `item.qty += 1`.

## Maps and sets

```ail skip=context
mut ages := {"ann": 31, "bob": 27}     # Map[str, int]; keeps insertion order
a := ages.get("zed") else 0            # read a key that may be missing: `get` gives `V?`
if ages.contains("ann")                # test for a key
    print(ages["ann"])                 # `m[k]` traps when `k` is missing
ages["cy"] = 40                        # inserts or replaces
ages["dan"] += 1                       # `op=` on a missing key starts at 0 (number values only)
mut counts := {}                       # typed by first use: counts[w] += 1
for name, age in ages                  # each entry; also keys(), values()
    print("{name}: {age}")
mut seen := Set[str]()                 # or a literal: {"a", "b"}
if seen.add("x")                       # true if it wasn't there
    print("new")
```

Also `get_or(k, d)`, `remove`, `len`, `sorted`, `m.map_values(|k, v| e)`; a set has `s.to_list()` (insertion order) and no `keys()`; sets and `keys()` have `union`, `intersect`, `difference`, `is_subset(t)`; `m.push(k, x)` appends to a list-valued entry (a missing `k` starts `[]`); `m[k] += 1` counts up; `m[k].qty += 1` changes an existing entry in place (it traps if `k` is missing), but `it := m[k]` (or `m.get(k)`) is a copy: store it back with `m[k] = it` (a changed copy that is never stored or read warns, expr.bind.15); `m.top(n)` gives the `n` largest-count keys, ties in insertion order. Keys can't be floats. Sorted keys: `m.keys().sorted()`; `str` sorts by UTF-8 bytes, so `"B" < "a"`.

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

`==` and `print` work on every value: `print(p)` shows `Point(x: 1, y: 2)`. Name the fields when two share a type (type.struct.7);
a field may default to a plain constant, `retries int = 3` (type.struct.8).
An enum with no fields has `Color.variants()` (a `List[Color]`), `Color.from_str("Red")` (a `Color?`, exact variant name) and `c.name()` (`"Red"`) (type.enum.20).

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

- Test an optional without binding it: `x == none`, `x != none` (no `is some`); `m.contains(k)` tests a key.
- `?` on an optional before an `else` goes to it: `xs.first()?.name else ""`.
- `else` binds looser than every operator: inside a larger expression write `(a else 0) + 1`; an argument needs none: `f(x else d)` (expr.prec.1).
- In a function with inferred errors (`!T`, `!`) and in `@test` functions,
  `?` on `none` fails with the error `Absent`: `port := text.to_int()? "port"`.
- A result that can fail must be handled: with `?`, `else`, or by returning it.
  `x := ... else ...` gives a fallback or leaves (`return`, `continue`, `break`). A fallback has the
  value's type: `else none` and `else helper()`, where `helper` returns nothing, are errors (err.else.1);
  declare `helper` with the result type `never` and `else helper()` fits (type.never.2).
  An `else` followed by an indented block, after `:=` or `=`, must leave or end in a value (`x = e else` + block: err.else.8);
  a function can end with `m.get(k) else return Missing(key: k)` (`err.else.7`).
- A typed error carries its text; the message shows the fields, which are declared under it (a message with no field line is err.msg.3):
  `error struct Bad "line {n}: {why}"` then, indented, `n int` and `why str` on lines of their own; `return Bad(n: 3, why: "x")`.
- Several error types: `fn load() Config ! IoError | ParseError`. No success
  value: `fn save() ! IoError` (or `fn save() !`); `save() else return` leaves.
- Which one failed: `match err` with `IoError =>` (there `err` is an
  `IoError`) and `ParseError.Empty =>`; every member must be covered.
- Standard errors carry a `kind` enum: `if err.kind == NotFound` or
  `match err.kind` (`fs.IoKind`, `proc.ProcKind`, `net.NetKind`); never a string.
  So are `app.Event.kind` (`ev.kind == Press`) and keys (`key == Escape`,
  `Text("q")`).

## Values and `mut`

Assigning or passing copies: changing `b` after `b := a` never changes `a`.

```ail skip=context
fn bump(mut p Point)   # the caller sees the change
    p.x += 1

bump(mut q)            # `mut` again at the call, on a `mut` variable
```

Two `mut` arguments of one call can't refer to the same value.

## Modules

A module is a directory: every `.ail` file directly in it, and those files see each other's names
with no `use` (mod.unit.1). Paths start at the directory of the file given to `ail run` or `ail build`.

```text
src/main.ail           # the root module, with `fn main()`: `ail run src/main.ail`
src/store/store.ail    # module `store`; a file `src/store.ail` is not a module
src/store/report.ail   # also module `store`
```

```ail skip=context
# src/store/store.ail
pub struct Item              # `pub`: other modules may name it
    pub sku str              # each field used outside its module needs `pub` too
    pub qty int

pub fn load(text str) List[Item]
    [Item(sku: l, qty: 1) for l in text.lines()]

# src/main.ail
use store                    # the directory `src/store`; not `use src.store`, not `import`; `use` lines come first

fn total(items List[store.Item]) int     # another module's names are always qualified
    items.fold(0, |acc, it| acc + it.qty)
```

- `pub` goes before `fn`, `struct`, `enum`, `trait`, `type`, a constant, a method (`pub fn Item.total(self)`) and a
  struct field; never before a variant, a local or a function inside `impl` (mod.vis.1, mod.vis.2).
- A module is not a value: after `use parse`, call `parse.parse(line)`; types are `parse.Event`, variants
  `parse.Kind.Add`, or bare `Add` where the type is known (`e.kind == Add`). No importing of single names (mod.use.2).
- A local may take an imported module's name (`out`, `parse`) and hides the module to the end of its block (std.use.3).
- No standard module needs `use`: `fs`, `io`, `env`, `csv`, `bigint`, `args`, `date`, `log` and the rest are in scope
  under their last name; `use std.NAME` stays valid. Only parts of a module keep `use`: `use std.net.jar`, `use std.net.websocket`,
  `use std.ui.widgets` (std.use.1, std.use.4).
- To split a file, run `ail mv Item load store` in the program's directory: it moves the named functions, types and
  constants into module `store` (making `store/store.ail` if needed) and fixes `pub`, qualifiers and `use` lines (mv.run.1).

## Closures and function values

A declared function is a value of type `fn(int) int`: `fn apply(f fn(int) int, x int) int`
calls `f(x)`, and `apply(double, 4)` passes it (not generic or `mut` ones). A literal
`|x| x + n` is one too: one expression on its line, types from context or written
(`|x int| ...`). It copies the locals it names when made; they are read-only inside
(share via a `ref`). `fn(str) int ! E` can fail; `?` in a literal leaves the literal.
A `fn name(params) R` with a block inside a body is a local function: it copies the
same way, can call itself, `return`s from itself, and is a value (no `mut` params).

## Big numbers and operator traits

- Past 64 bits: `bigint.BigInt` takes `+ - * / %`, unary `-`, `<` and `"{b}"`;
  mix with `int` through `bigint.from_int(n)`; `/` traps on zero, `divmod` returns an error (std.bigint.4, 10).
  A map of BigInts has no zero to start from: `m[k] = (m.get(k) else bigint.from_int(0)) + x`, not `m[k] += x` on a missing key.
- Your own type gets an operator from a trait: `impl Add for V` with `fn add(self, o V) V` gives `a + b` and `a += b`;
  also `Sub`, `Mul`, `Div`, `Rem`, `Neg` (unary `-`, `fn neg(self) V`). Both operands have the same type (trait.op.1).

## Format specs and chars

- Format specs are `[[fill]align][+][0][width][,][.decimals][x|b|e]` and nothing else (lex.str.7): `<` `>` `^`
  align (numbers default right, others left); `0` before the width pads a number with zeros and can't combine with an
  align (text: `{s:0>8}`); `+` signs, `,` groups thousands; `.N` needs a float, `x` and `b` an int.
- `char`: `c.is_digit()` (ASCII `0` to `9`), `c.is_alpha()`, `c.is_space()`, `c.is_alnum()`, `c.is_upper()`, `c.is_lower()` (Unicode), `c.to_upper()`, `c.to_lower()`,
  `c.to_digit(10)` (the digit's value as an `int?`; `to_digit(16)` reads hex; there is no `c.to_int()`), `c.code()` (the code point), `char.from_code(n)` (`char?`), `c.to_str()`; chars compare with `<` by code point.

## More built-ins

Less common numbers, text, types, patterns and collections, after the part every first program reads.

- Float math: `x.sqrt()`, `abs()`, `floor()`, `ceil()`, `round()` (each a `float`; `x.to_int()` cuts toward zero and traps on `nan`; type.num.9), `x.sin()`, `cos`, `tan`, `atan`, `atan2(x)`, `exp`, `ln`, `log10`, `cbrt`, `hypot(y)`, `x.pow(y)` (type.num.16).
- Sized numbers `i8`…`u64`, `f32`: typed by context, same-type operands only,
  overflow traps; `x.to_u8()` converts.
- `int`: `min`, `max`, `abs`, `pow(e)` (an `int` power; traps on overflow), `to_hex`, `checked_add`, `checked_sub`, `checked_mul`, `checked_div`, `checked_rem`, `checked_neg` (`int?`, `none` on overflow or zero divide; `checked_rem(-1)` is always `some(0)` because `a % -1` is 0, so test the smallest value `%` -1 yourself, while `checked_div(-1)` of it is `none`); `wrapping_add`, `wrapping_sub`, `wrapping_mul`, `wrapping_neg` (two's complement); `saturating_add`, `saturating_sub`, `saturating_mul` (clamp); `to_u8()`, `to_u16()`, `to_u32()`, `to_u64()`, `to_i8()`, `to_i16()`, `to_i32()`, `to_f32()` (sized, type.num.15); bits: `a & b`, `a | b`, `a ^ b`, `~a`, `a << k`, `a >> k` (count 0-63, else trap), the same as the methods `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr`; `x &= e` (also `|= ^= <<= >>=`) on a name. A bit operator never mixes with another operator without parentheses: `(x & 1) == 0`, `(a + b) & m`, `a | (b & c)`.
- `bytes` (immutable): `b"GET /\r\n\x00"`, `s.to_bytes()`, `b.len()`, `b[i]` (`u8`), `b[a..b]`, `a + b`,
  `b.to_str()` (`str?`), `b.text()` (lossy), `b.find(x)`, `b.hex()` (`b.hex(" ")` spaces the pairs), `b.base64()`, `bytes.from_hex(s)` and `bytes.from_base64(s)` (`bytes?`), `bytes.from_list(xs)` (a `List[u8]`), `a.concat(b)`, `fs.read_bytes(p)?`, `fs.write_bytes(p, b)?`.

Distinct types (type.distinct.1): `type UserId = int` (any built-in scalar base) is a new type
that never mixes with `int` or another id type. `UserId(3)` makes one,
`int(id)` gives the value back; same-type operators, `==`, `<`, text, map keys
and JSON work as the base's; no fields, methods or `impl`; free at run time.

Generators: a function returning `Seq[T]` with `yield x` in its body
(`fn count(n int) Seq[int]`) runs lazily, up to each `yield`, per `next()`
(`T?`). `for` takes any type with `fn next(mut self) T?`, `Seq` included; a
`Seq` is unique, read once: `count(9).filter(|x| x > 2).map(f).take(3).collect()`,
also `Seq.of(xs)`, `take_while`, `skip`, `enumerate`, `zip`, `chain`, `count`.

A keyword may name a field, a method or a label (`error str`, `o.error`, `fn T.any(self)`); in a method reach the field as `self.error`.

Patterns nest in fields: `Add(Num(a), Num(b)) => a + b`, `Word("fn", (1, _))`. A bare name in a field binds it, so write a fieldless variant qualified there (`Cons(_, Items.Nil)`); a `match` that nests names each missing case, e.g. `Add(Num(_), Neg(_))`. Structs match by field name, as they are built: `Point(x: 0, y) =>` (a bare field name binds it, unnamed fields match anything); variants take names too, `Rect(w: 0) =>`.

Grids (2-D data): `mut g := Grid.fill(rows, cols, 0)`, `Grid.from(xs)`, `g[i, j] += 1`, `g.rows`, `g.cols`, `g.row(i)`, `g.to_list()`.

Priority queue: `mut h := Heap[int]()` (smallest first; `Heap[int](max_first: true)`
for largest), `h.push(x)`, `h.pop()` and `h.peek()` give `T?`, `h.len()`,
`h.is_empty()`, `Heap.from(xs)`, `Heap.from_max(xs)`, `h.to_sorted()` (all elements in pop order). Double-ended queue: `mut d := Deque[int]()`,
`push_front`, `push_back`, `pop_front()`, `pop_back()` (`T?`), `first()`, `last()`,
`d[i]`, `d.set(i, v)`, `for x in d`, `d.to_list()`.

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

fn largest[T Ord](items List[T]) T  # `T`: any type that implements `Ord`
    mut best := items[0]
    for x in items
        if x > best
            best = x
    best
```

- `impl Ord for Version` with no body compares the fields in order. `<` and
  friends work on `int`, `str`, lists and `Ord` types. Arithmetic operators come from `impl Add` and its
  siblings (see Operators and numbers). Customise with
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

fn main()
    a := first_word("") else "-"            # `else` inside an expression
    n := (["7"].first() else "0").to_int() else 0
    print("{a} {n + (first_word("a b") else "").len()}")
    if v := pick([none, 3])                 # peels one level: v is `int?`
        print(v)
```

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

After an edit, `ail test` also runs each changed pure function of the old (git `HEAD`, or `--against OLD.ail`) and new version on generated inputs: `changed behaviour: NAME(PARAM: VALUE, ...) was OLD, now NEW` or `same behaviour: NAMES` (test.oracle.1). Information, not a failure.

A `@fixture("drop") fn tmp() !str` builds a value per test run (`fn drop(v str)` always runs after it; a setup failure fails the test); `@test @needs("tmp") fn uses(tmp str)` takes it by the fixture's name.

A block fenced `ail` in a `##` doc comment is a test; `# => text` asserts printed output.

## Standard library

`ail ask std "words"` finds a function; `ail ask std NAME...` prints what each named function or method does exactly (api.find.9); `ail card std.fs` lists a module. No module needs `use`.

`random.int(1, 6)` (inclusive), `random.float()`, `random.seed(n)`. Also `csv` (`parse`, `records`, `write`), `ini` (`parse` to `Json`, `write`, `parse_env` for `.env`), `stats` (`mean`, `median`, `stdev`, `percentile`, on a `List[float]`; `none` when empty; std.stats.1 to std.stats.12), `args` (`parse`), `log` (`info`, `warn`, `err`), `date` (`std.time.date`: `date.utc(ms).format("%Y-%m-%d")`), `semver` (`parse`, `compare`, `satisfies`: `semver.satisfies(semver.parse("1.4.2")?, "^1.2")?`; std.semver.1 to std.semver.12), `textdiff` (`lines`, `unified(a, b, na, nb, ctx)`, `apply(text, patch)?`, `stat`, `distance`; std.textdiff.1 to std.textdiff.9).

Text layout: `textwrap.wrap(s, cols)` (`List[str]`), `fill`, `dedent`, `indent(s, prefix)`, `truncate(s, cols)`, `width(s)` (terminal columns), `pad_to_width(s, cols, align: 'r')`, `table(rows, align: "lr")`, `table_grid`, `sorted(by: |n| textwrap.natural_key(n))` (`img2` before `img10`; std.textwrap.1 to std.textwrap.12).

Regex (linear time; RE2 syntax, no backreferences): `re := regex.compile(r"(\d+)-(\d+)")?`, then `re.is_match(s)`, `re.find(s)` (`Match?`: `start`, `end`, `text`), `find_all`, `captures(s)` (`groups` `List[str?]`, `named`), `replace(s, "$2-$1")`, `replace_first`, `split(s)` (each is also `regex.NAME(re, s)`), `regex.group_all(re, s, n)` (group `n` of each match, `""` if unset), `regex.escape(s)`. Offsets count characters.

URL (`std.url`, RFC 3986): `u := url.parse(link)?` (`scheme`, `user`, `host`, `port` `int?`, `path`, `query`, `fragment`; percent-encoded as written), `print(u)`, `url.resolve(base, ref)?`, `url.join_path(base, parts)?`, `url.encode(s)`/`decode(s)` (`str?`), `encode_form`/`decode_form` (`+` is a space), `url.parse_query(q)` (`List[(str, str)]`), `query_map(q)`, `build_query(pairs)`. Don't name a local `url`: it hides the module (std.url.1 to std.url.11).

`t := tensor.from(xs, shape)` (or `tensor.zeros(shape)`): `matmul`, `+`, `* 2.0`, `dot`, `sum(axis?)`, `transpose`, `map`, `to_list`; SIMD and multi-core.

`fs.read_text(p)?`, `fs.write_text(p, s)?`, `fs.walk(dir)?`, `fs.glob("src/**/*.ail")`. Streaming: `mut r := file.open(p)?`, `line := r.read_line()? else break`, `r.read_exact(n)?`, `r.seek(pos)`; `mut w := file.create(p)?` (or `file.append(p)?`), `w.write_line(s)?`, `w.close()?` (leaving scope flushes). `io.read_exact(n)` is `bytes?`.
SQL (std.db.3): `mut d := db.open(path)?` (or `db.memory()`), `d.exec("INSERT INTO t (a, b) VALUES (?, ?)", [Text(s), Int(n)])?` (rows changed; `[]` when there is no `?`), `for r in d.query("SELECT a FROM t WHERE b > ?", [Int(1)])?` with `r.str("a") else ""`, `r.int`, `r.float`; `d.begin()?` ... `d.commit()?`. Columns are typed strictly; test `err.kind == Constraint`.
`proc.run(cmd, args, input?, timeout_ms?)?` gives `.status`, `.out`, `.err`, `.ok()`, `.text()`; for a working directory or environment: `mut o := proc.options()`, `o.cwd = dir`, `o.env = {"K": "v"}` (`clear_env`, `input`, `timeout_ms`), then `proc.run_with(cmd, args, o)?` (`run_bytes_with`, `start_with`).
`net.get(url)?`, `net.post(url, body, content_type)?` give `.status`, `.headers`, `.body` (`http://`, `https://`; `get_bytes`, `post_bytes`, `request_bytes` give `.data`); `net.request(method, url, opts)` with `net.options()`; servers: `srv := net.serve(addr)?`, `ex := srv.accept()?`, `ex.respond(net.Response(status: 200, headers: {}, body: "hi"))?`; raw TCP: `net.connect(host, port)?`.
Routing (`use std.net.http`, http.route.1-5): `mut r := http.router()`, `r.get("/users/:id", show)` (also `post`, `put`, `patch`, `delete`, `any`; `*rest` takes the rest), `r.static("/site", dir)`, then `spawn serve(r, srv)` with `fn serve(r http.Router, srv net.Server) ! Trapped` calling `r.run(srv)?` or `r.serve(addr)?`; a handler is `fn(http.Req) net.Response` with `req.param("id")` (`str?`), `req.json()?`, `req.form()`; reply with `http.text(200, s)`, `http.json(200, j)`, `http.redirect(to)`, `http.not_found()`; no match is `404`, a wrong method `405` with `allow`; `r.handle(req)` answers without a server.

`Json`, `parse_json(text)?` and the `ToJson`/`FromJson` derives read and write JSON. A member's payload is `j.str(k)`, `num`, `int`, `bool` or `arr`, `none` when missing (`j.int("retries") else 2`); `j.set(k, v)` (`mut`); `j.need(k)?.need_int()?` fails with `missing KEY`; `j.text(k, default)` and `j.flag(k, default)` never fail; `parse_json_lines(text)` gives each line that parses; `load_json[T](path)?`, `print_json(v)`; `j.to_text()` is the JSON text (`v.to_json().to_text()` for a `ToJson` value), and so are `j.to_str()` and `"{j}"`.

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
rs := TaskGroup[int]()      # tasks returning int
rs.add(spawn load(3))
all := rs.wait_all()?       # List[int] in the order added; `?` for Trapped
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

Wait or `add` each task exactly once, on every path (conc.task.5); a handle can't go in a list: put N handles in a `TaskGroup[T]` and `wait_all()?` gives their results in order, or write `[f(x) par for x in xs]`. A deadlock traps.

`par for i in 0..n` (asserts the iterations are independent) and `[f(x) par for x in xs]` use all cores; the result is what the sequential loop gives, and provable conflicts are errors. An `Atomic` is shared by tasks and `par for` without a lock.

## Contracts

`requires`, `ensures` (`result`, `old(e)`) and struct `invariant` are checked at run time (`ail run --no-contracts` skips them). A trait method's contracts bind its implementations.

## Packages, effects and the system

`ail.pkg` lists the package and its `dep`s, and may hold one line `ail 0.2` naming the language version (`mod.pkg.10`; `ail new` writes it); each `dep` reaches only its `caps` (spec `mod.pkg.*`). A new project's `package` line has `caps fs=.,fs=@tmp,env,clock,random`: `net`, `proc` or a file outside the project and the temp directories (`fs=@tmp`) is refused until the entry the message names is added there (`tool.new.6`).

Effects are inferred: `io`, `fail`, `mutates`, `spawn`, and the capabilities `fs`,
`env`, `proc`, `clock`, `random`, `net`, `net.serve`, `ui`. `@pure fn f()` is an error if it reaches
one; `@caps(fs)` limits a function to the listed capabilities and everything it calls, `@caps(none)` allows none (`eff.cap.6`). A call whose value is dropped and that has no effect is an error: `xs.sorted()` alone
means `xs.sort()`; write `_ := f()` to drop a value on purpose (`eff.use.1`).
`ail run --deterministic` denies `clock`, `random`, `proc` and `net`. `ail run --allow fs.read=./data,net=host:443` allows only what it names, and a use outside the scope ends with `trap[eff.allow.N]`.

`unsafe` allows C calls and `std.mem` pointers; `ail.pkg` needs `unsafe allow`, or `unsafe driver` for `std.hw`; see `docs/spec/14-ffi.md`. Structs of C-typed fields pass to C by value; a `fn(...)` parameter of an `extern c fn` takes a declared function's name as a callback (`mem.c_fn(f)` gives a `*u8`).

A window is a grid of text cells: `mut w := ui.open("Demo", 40, 10) else return` (fails headless), `w.put(col, row, text, fg, bg)`, `w.present()`, then `match w.next_event()` on `Key(key, ctrl, alt, shift)`, `Text(s)`, `Close`, `Resize(_, _)`, `Mouse(_, _, _, _)`, `Scroll(_)`; `ail card std.ui` lists the rest.

## Tools

```sh
ail api --text FILE.ail    # declarations and `##` docs without bodies
ail effects FILE.ail       # what each function may do
ail run --trace --watch NAME FILE.ail   # events and a snapshot at a trap
ail run --record R.json FILE.ail     # `--replay R.json` repeats the run
ail CMD --budget N             # any command: about N tokens, errors first; `--from K` continues
ail edit FILE.ail --test <<E   # a declaration name, then `-old` / `+new` lines; then fmt, check, test
E
ail diff OLD NEW               # changed declarations with their lines, diagnostics, tests
```

`todo` is a typed hole: `ail check` warns with its expected type and the names in scope. `uhwhatnow?` also ranks what fits, with a fix each.
