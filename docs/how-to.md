# How to

Short, checked answers to the things programs do most. `ail ask how WORDS`
finds one by its title or its words (`api.find.6`); every block is checked by
`tools/check-doc-snippets.py`. Each section is a `##` title, a `Words:` line the
search also reads, and one block.

## sort descending
Words: sort order largest first reverse biggest highest

`sorted(by:)` takes a key; negate a number to flip the order. For any other
type, sort and then reverse in place.

```ail fragment
xs := [3, 1, 2]
down := xs.sorted(by: |x| -x)
mut names := ["b", "a", "c"].sorted()
names.reverse()
print(down)
print(names)
```

## sort descending, ties in input order
Words: sort descending largest first ties stable order reverse biggest highest

`xs.sorted().reversed()` reverses tied elements too. Sort by a key that
flips the order and the sort (stable) keeps ties in input order. `-x` traps on
the smallest `int`; `~x` (`-x - 1`) flips the order and never overflows.

```ail
struct Row
    name str
    n int

fn main()
    rows := [Row(name: "a", n: 2), Row(name: "b", n: 5), Row(name: "c", n: 2), Row(name: "d", n: 5)]
    for r in rows.sorted(by: |r| ~r.n)
        print("{r.name} {r.n}")
```

## fail with text
Words: error str fail text message no result void Err raise throw Fail

`int ! str` fails with `return "text"`. With no result, or a `str` result, use
the standard `Fail` (err.fail.1): `! Fail` and `return Fail(msg: "text")`;
`e.msg` is the text. `str ! str` is not allowed.

```ail
fn check(n int) ! Fail
    if n < 0
        return Fail(msg: "negative: {n}")

fn parse(s str) int ! str
    n := s.to_int() else return "not a number: {s}"
    n

fn main() !
    check(1)?
    print(parse("7")?)
```

## option has no map
Words: optional map option some none transform unwrap default

An optional has no `map` and none of its value's methods. Bind it, or give a
default; a default in parentheses takes a method.

```ail fragment
o := "42".to_int()
if v := o
    print(v + 1)
n := o else 0
print(n)
names := ["ann", "bob"]
print((names.last() else "").len())
```

## sort by key
Words: sort list order by field property custom comparator

```ail fragment
words := ["ccc", "a", "bb"]
print(words.sorted(by: |w| w.len()))
```

## loop with index
Words: for each element index position counter enumerate

```ail fragment
xs := ["a", "b"]
for i, x in xs.enumerate()
    print("{i}: {x}")
```

## command-line program: arguments, stderr, exit code
Words: cli command line argv args arguments parameters flags usage stderr exit status code tool

`env.args()` is the words after the program; the program name is not included.
`io.eprint` writes to standard error and adds a newline. `io.exit(code)` ends the
program. A helper declared `never` ends in `io.exit`, and its call fits where a
value is expected (type.never.2).

```ail
fn usage(why str) never
    io.eprint("usage: add A B ({why})")
    io.exit(2)

fn main()
    args := env.args()
    if args.len() != 2
        usage("two numbers")
    a := args[0].to_int() else usage("not a number: {args[0]}")
    b := args[1].to_int() else 0
    print(a + b)
```

## exit with a code
Words: exit status code quit stop terminate end program fail process

`main` returning gives status 0; `fn main() !` failing gives 1. `io.exit(code)`
ends at once with `code` (0 to 255) after flushing standard output.

```ail fragment
ok := false
if not ok
    io.exit(3)
```

## print to stderr
Words: stderr standard error eprint print error message warning log diagnostics

`io.eprint(value)` is `print` for standard error: it adds a newline.

```ail fragment
io.eprint("warning: no input")
print("result")
```

## sum that may overflow int (BigInt)
Words: overflow big integer bigint large numbers 64-bit int128 i128 u128 huge sum factorial

`int` is 64-bit and overflow traps. `x.checked_add(y)` gives `none` on overflow;
`bigint.BigInt` never overflows and needs no `use`. It takes `+ - * / %`, unary `-`,
`<` and `"{b}"`; `bigint.from_int(n)` brings an `int` in and `as_int()` gives an
`int?` back (std.bigint.4).

```ail
fn main()
    mut total := bigint.from_int(0)
    for n in [9000000000000000000, 9000000000000000000]
        total += bigint.from_int(n)
    print(total)
    print(total > bigint.from_int(0))
    print(total.as_int() else 0)
```

## read a file by lines
Words: read open text file lines each line

```ail fragment
text := fs.read_text("data.txt") else
    print("cannot read data.txt")
    return
for line in text.lines()
    print(line)
```

## read lines from standard input
Words: stdin standard input console lines read all

```ail fragment
for line in io.read_all().lines()
    print(line)
```

## write a file
Words: write save create text file

```ail fragment
fs.write_text("out.txt", "hello\n") else
    print("cannot write out.txt")
```

## string to int
Words: parse convert number integer text str int

`to_int()` gives an `int?`; `else` supplies the fallback.

```ail fragment
n := "42".to_int() else 0
print(n)
```

## build a string in a loop
Words: string builder concatenate append join pieces accumulate

There is no `+` for strings. Push the parts to a list and join once.

```ail fragment
mut parts := List[str]()
for i in 0..3
    parts.push("item {i}")
print(parts.join(", "))
```

## leave nested loops
Words: break out of nested loops both loops exit early find pair label outer

Label the outer loop and `break` or `continue` it by name (expr.loop.10). In a
function, `return` leaves every loop.

```ail
fn main()
    xs := [1, 2, 3]
    mut pair := (0, 0)
    outer: for i in 0..xs.len()
        for j in 0..xs.len()
            if i != j and xs[i] + xs[j] == 5
                pair = (i, j)
                break outer
    print(pair)
```

## count down or by a step
Words: reverse range backwards descending loop step by every other decrement rev downto

A range only counts up by one, and has no `rev` or `step_by`. Use `while`, or
loop over `xs.reversed()`.

```ail fragment
mut i := 10
while i > 0
    print(i)
    i -= 3
for x in [1, 2, 3].reversed()
    print(x)
```

## digit value of a character
Words: char to int digit number value ord parse character numeric

`c.to_digit(10)` is the digit's value as an `int?`; `c.code()` is the code
point (55 for `'7'`). There is no `c.to_int()`.

```ail fragment
mut total := 0
for c in "a1b22".chars()
    d := c.to_digit(10) else continue
    total += d
print(total)
```

## join two lists
Words: concatenate append lists plus add prepend spread extend combine merge

There is no `+` for lists. Copy one and `extend` it; `insert(0, x)` puts an
element first.

```ail fragment
xs := [1, 2]
mut both := xs
both.extend([3, 4])
both.insert(0, 0)
print(both)
print(xs)
```

## print and round a float
Words: float format decimal places round truncate integer whole number precision

A whole `float` prints with `.0`. `round()` gives a `float` (halves away from
zero), `to_int()` cuts toward zero, and `{x:.2}` fixes the decimals.

```ail fragment
x := 7.0 / 2.0
print(x)               # 3.5
print(x.round())       # 4.0
print(x.to_int())      # 3
print("{x:.2}")        # 3.50
```

## fixed-width columns
Words: pad align column table width truncate cut left right justify report

Padding and width specs never cut a longer value; cut it first with `slice`.

```ail fragment
name := "widget-long-name"
cell := if name.len() > 8 then name.slice(0, 8) else name
print("[{cell.pad_right(8, ' ')}][{42:>6}]")
```

## text with braces and quotes
Words: json literal brace curly escape quote string interpolation object

A literal `{` is `\{`; `}` is plain. Inside `{ }` the code is plain code, so
its quotes are not escaped.

```ail fragment
id := 3
tags := ["a", "b"]
print("\{\"id\": {id}, \"tags\": \"{tags.join(", ")}\"}")
print("\{}")
```

## count occurrences
Words: count frequency histogram tally words how many each

```ail fragment
counts := "a b a".split(" ").counts()
print(counts)
```

## group by key
Words: group multimap list per key bucket

```ail fragment
mut groups := Map[str, List[int]]()
groups.push("even", 2)
groups.push("odd", 1)
groups.push("even", 4)
print(groups)
```

## default for a missing map key
Words: map get default missing key lookup else zero

`m.get(k)` takes only the key and gives a `V?`.

```ail fragment
ages := {"ann": 31}
age := ages.get("bob") else 0
print(age)
```

## split and trim text
Words: split text separator trim whitespace tokens words parse csv

```ail fragment
parts := "a, b ,c".split(",")
for p in parts
    print(p.trim())
```

## wait some time
Words: sleep delay pause wait milliseconds

```ail fragment
time.sleep_ms(100)
```

## one-line if with several arms
Words: ternary conditional expression inline elif else if chain one line

`if ... then ... elif ... then ... else ...` is one expression; the final `else`
is required (expr.if.8).

```ail fragment
n := -3
sign := if n > 0 then "pos" elif n < 0 then "neg" else "zero"
print(sign)
```

## operators on your own type
Words: operator overloading plus add vector arithmetic impl Add trait

`impl Add for V` with `fn add(self, o V) V` gives `a + b` and `a += b`. The
others are `Sub`, `Mul`, `Div`, `Rem` and `Neg` (unary `-`) (trait.op.1).

```ail
struct V
    x int
    y int

impl Add for V
    fn add(self, o V) V
        V(x: x + o.x, y: y + o.y)

fn main()
    mut a := V(x: 1, y: 2)
    a += V(x: 3, y: 4)
    print(a)
```
