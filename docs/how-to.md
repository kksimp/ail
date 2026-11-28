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

There is no `+` for lists. `xs.concat(ys)` gives a new list and leaves both
as they are; `xs.extend(ys)` grows a `mut` list in place.

```ail fragment
xs := [1, 2]
both := [0].concat(xs).concat([3, 4])
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

## collect results from many tasks
Words: parallel fan out gather join all results spawn list of tasks handles wait all map concurrently workers threads errors first error fail fast errgroup

A task handle can't go in a list (conc.task.5). Run one call per element on
all cores and keep the results, in order, with a `par` comprehension. To
start tasks yourself and wait them all, add each to a `TaskGroup[T]` and call
`g.wait_all()?` for the `List[T]`, in the order added (conc.task.8).

```ail
fn count_to(n int) int
    mut total := 0
    for i in 0..n
        total += i
    total

fn main()
    sizes := [10, 20, 30]
    sums := [count_to(n) par for n in sizes]
    print(sums)
```

```ail
fn square(n int) int
    n * n

fn main() !
    g := TaskGroup[int]()
    for n in [3, 4, 5]
        g.add(spawn square(n))
    print(g.wait_all()?)
```

Tasks that can fail go in a `TaskGroup[T ! E]`, with the errors named.
`wait_all()` fails with the first error in the order the tasks were added and
cancels the tasks added after that one (conc.task.8).

```ail
error struct BadInput "bad input {n}"
    n int

fn parse(n int) int ! BadInput
    if n < 0
        return BadInput(n: n)
    n * 10

fn main() !
    g := TaskGroup[int ! BadInput]()
    for n in [1, 2, 3]
        g.add(spawn parse(n))
    print(g.wait_all()?)
```

## worker pool
Words: worker pool channel jobs queue fixed number of workers fan out fan in producer consumer spawn threads send recv close

K workers read jobs from one `Channel`, send answers on another, and stop when the
jobs channel is closed. One task feeds the jobs and is waited; each answer carries
its index so the results can be put back in order. For one call per element and no
pool, a `par` comprehension is shorter (see "collect results from many tasks").

```ail
struct Job
    idx int
    n int

struct Done
    idx int
    sum int

fn work(n int) int
    mut total := 0
    for i in 0..n
        total += i
    total

fn worker(jobs Channel[Job], results Channel[Done])
    for j in jobs
        results.send(Done(idx: j.idx, sum: work(j.n)))

fn feed(jobs Channel[Job], sizes List[int])
    for i, n in sizes.enumerate()
        jobs.send(Job(idx: i, n: n))
    jobs.close()

fn main() ! Trapped
    sizes := [10, 20, 30, 40]
    jobs := Channel[Job](8)
    results := Channel[Done](8)
    mut sums := [0 for _ in 0..sizes.len()]
    g := TaskGroup()
    for _ in 0..2
        g.add(spawn worker(jobs, results))
    feeder := spawn feed(jobs, sizes)
    for _ in 0..sizes.len()
        d := results.recv() else break
        sums[d.idx] = d.sum
    feeder.wait()?
    print(sums)
```

## write JSON text
Words: json serialize stringify dump encode to string text output save write

`j.to_text()` is the compact JSON text of a `Json`; `print(j)` and `"{j}"` write
the same. A value with the `ToJson` derive: `v.to_json().to_text()`, or
`print_json(v)` to print it.

```ail
fn main()
    mut doc := Json.Obj([])
    doc.set("id", Json.Num(7.0))
    doc.set("tags", Json.Arr([Json.Str("a")]))
    print(doc.to_text())
    print(doc)
```

## serve HTTP: a web server
Words: http server listen port handler respond web request response status bind accept

`http.run(srv, handler)` answers every request with `handler` (a function from
`net.Request` to `net.Response`) until `srv.close()`. `http.serve(addr, handler)?`
listens and runs in one call. `net.serve("127.0.0.1:0")` takes a free port;
`srv.addr()` is the address. A spawn takes a call of a declared function
(conc.task.1), so the loop sits in its own `fn`.

```ail
use std.net
use std.net.http

fn hello(req net.Request) net.Response
    http.text(200, "hello {req.path}")

# A spawn takes a call of a declared function (conc.task.1).
fn serve_all(srv net.Server) ! Trapped
    http.run(srv, hello)?

fn main() !
    srv := net.serve("127.0.0.1:0")?
    t := spawn serve_all(srv)
    print(net.get("http://{srv.addr()}/hi")?.body)
    srv.close()
    t.wait()?
```

## HTTP routes with parameters and JSON
Words: http router route get post put delete path parameter :id json api rest body request web service endpoint

`mut r := http.router()` collects routes in order; the first match answers. A
pattern's `:id` is `req.param("id")` (`str?`), `*rest` takes the rest of the
path, `req.json()?` and `req.form()` read the body, `http.json(status, j)`,
`http.text`, `http.html`, `http.redirect` and `http.not_found()` build the
response. No route is `404`, a wrong method `405` with `allow`. `r.run(srv)?`
serves on a `net.Server`; `r.serve(addr)?` listens too.

```ail
use std.net
use std.net.http

struct Note
    id int
    text str

impl ToJson for Note

fn list(notes List[Note]) net.Response
    http.json(200, notes.to_json())

fn one(r http.Req, notes List[Note]) net.Response
    id := (r.param("id") else "").to_int() else
        return http.text(400, "id must be a number")
    for n in notes
        if n.id == id
            return http.json(200, n.to_json())
    http.not_found()

fn add(r http.Req) net.Response
    body := r.json() else err
        return http.text(400, "bad json: {err.message}")
    text := body.str("text") else
        return http.text(422, "missing text")
    http.json(201, Note(id: 3, text: text).to_json())

fn serve_all(r http.Router, srv net.Server) ! Trapped
    r.run(srv)?

fn main() !
    notes := [Note(id: 1, text: "milk"), Note(id: 2, text: "eggs")]
    mut r := http.router()
    r.get("/notes", |q| list(notes))
    r.get("/notes/:id", |q| one(q, notes))
    r.post("/notes", add)
    srv := net.serve("127.0.0.1:0")?
    t := spawn serve_all(r, srv)
    base := "http://{srv.addr()}"
    print(net.get("{base}/notes/2")?.body)
    made := net.post("{base}/notes", "\{\"text\": \"tea\"}", "application/json")?
    print("{made.status} {made.body}")
    print(net.get("{base}/notes/9")?.status)
    srv.close()
    t.wait()?
```

## test an HTTP router in a unit test
Words: http router test request handle unit test no sockets mock status 404 405 allow response check

`r.handle(request)` is the response a router gives a `net.Request`, with no
socket, so a test builds the request and checks the status and body.
`net.Request` has all nine fields: `method`, `path`, `query`, `headers`, `body`,
`peer`, `data` and `server` (`-1` for none).

```ail
use std.net
use std.net.http

fn ask(method str, path str, body str) net.Request
    net.Request(method: method, path: path, query: {}, headers: {}, body: body, peer: "", data: b"", server: -1)

fn show(r http.Req) net.Response
    http.text(200, "user {r.param("id") else "?"}")

@test
fn routes_answer()
    mut r := http.router()
    r.get("/users/:id", show)
    ok := r.handle(ask("GET", "/users/7", ""))
    assert(ok.status == 200)
    assert_eq(ok.body, "user 7")
    assert(r.handle(ask("GET", "/nope", "")).status == 404)
    wrong := r.handle(ask("POST", "/users/7", ""))
    assert(wrong.status == 405)
    assert_eq(wrong.headers["allow"], "GET, HEAD")

fn main()
    mut r := http.router()
    r.get("/users/:id", show)
    print(r.handle(ask("GET", "/users/7", "")).body)
```

## JSON for your own struct
Words: json derive tojson fromjson struct serialize deserialize encode decode to_json from_json impl number Json.Num attribute

There is no `@derive`. An `impl ToJson for T` and an `impl FromJson for T`
with no body write the conversions (json.derive.1-2). `Json` has the variants
`Null`, `Bool`, `Num(float)`, `Str`, `Arr` and `Obj`, and no `Int`: a number is
`Json.Num(0.5)`, and `n.to_json()` turns an `int`. `j.set(k, v)` adds a member
to a `mut Json`.

```ail
struct Job
    id int
    title str
    tags List[str]
    done bool

impl ToJson for Job
impl FromJson for Job

fn main() !
    t := Job(id: 1, title: "ship", tags: ["a", "b"], done: false)
    text := t.to_json().to_text()
    print(text)
    back := Job.from_json(parse_json(text)?)?
    print(back.title)
    mut doc := Json.Obj([])
    doc.set("count", 3.to_json())
    doc.set("ratio", Json.Num(0.5))
    doc.set("name", Json.Str("x"))
    print(doc)
```

## keep data in a database file
Words: database db sql sqlite table insert select query rows store persist file open exec transaction

`std.db` is a small SQL database in one file of its own format (not SQLite: other
tools can't open it, and `std.sqlite` only reads SQLite files). `db.open(path)?`
makes the file when it is missing, `db.memory()` keeps it in memory. `d.exec(sql,
params)?` runs statements and gives the rows changed, `d.query(sql, params)?` gives
`List[Row]`. `?` takes the values in `params`: `Text(s)`, `Int(n)`, `Real(x)`,
`Blob(b)` and `Null`. A column is read as an optional: `r.int("id")`, `r.str`,
`r.float`, `r.bytes`, `r.bool`. `d.begin()?` and `d.commit()?` group changes.

```ail
fn main() !
    mut d := db.memory()
    d.exec("CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY, title TEXT NOT NULL, stars INTEGER)", [])?
    d.exec("INSERT INTO notes (title, stars) VALUES (?, ?)", [Text("milk"), Int(3)])?
    print(d.last_id())
    for r in d.query("SELECT id, title, stars FROM notes WHERE stars >= ? ORDER BY id", [Int(1)])?
        print("{r.int("id") else 0} {r.str("title") else ""}")
    d.close()
```

## sort a map's keys or values
Words: map sorted keys sort order dictionary count top by value largest sorted()

A map has no `sorted`. Sort its keys: `m.keys().sorted()` for key order,
`m.keys().sorted(by: |k| -m[k])` for the largest value first; `m.top(n)` gives
the `n` keys with the largest counts.

```ail
fn main()
    counts := {"pear": 2, "apple": 5, "fig": 2}
    for k in counts.keys().sorted()
        print("{k} {counts[k]}")
    by_count := counts.keys().sorted(by: |k| -counts[k])
    print(by_count)
```
