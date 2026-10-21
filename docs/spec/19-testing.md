# 19. Testing

Chapter key: `test`

> **Status:** Drafted (partial). Specified: attributes on functions, `@test`
> functions, the `ail test` runner and its result format, `assert` and
> `assert_eq`, property tests, and doc examples as tests. Not yet designed:
> fixtures, test filtering, parallel runs, and expected-failure tests.

## Attributes

**test.attr.1** — The only attributes are `@test` and `@pure`. Any other
attribute name is a compile-time error that lists the known attributes. The
attribute syntax itself is `gram.attr.1–2`; an attribute on a line of its own
is `gram.attr.2`. *(ADR 0083, ADR 0167)*

**test.attr.2** — A `@test` function is a free top-level function with no
type parameters. Its return type is absent or `!`. Its parameters make it a
property (`test.prop.1`). *(ADR 0167, ADR 0198)*

**test.attr.4** — A `@test` function written with no return type can fail
as if it were declared `!` (its errors are inferred), so `?` works in it on
results and on optionals (`err.prop.5`); a failure ends the test as
`test.run.2` says. *(ADR 0279)*

**test.attr.3** — A `@test` function can't be called from code. `ail run`
ignores tests, so a program's output is the same with or without them.
*(ADR 0167)*

```ail fragment
fn double(x int) int
    x * 2

@test fn doubles()
    assert_eq(double(4), 8)
```

## Running tests

**test.run.1** — `ail test FILE` checks the program, then runs every `@test`
function in source order. A file of tests needs no `main`. A program the
checker rejects is reported as for `ail check` (exit 2) and no test runs.
*(ADR 0167)*

```ail tool=test,FILE expect=passed rules=test.run.1
@test
fn one()
    assert(1 == 1)
```

**test.run.2** — Each test runs in isolation. A trap (including a failed
assertion) or a returned error ends that test only. Constants are computed
once. What a test prints to standard output or standard error is captured;
a failing test shows it under its `FAIL` line, standard output plain and
standard error marked `!`. *(ADR 0693)*
*(ADR 0167)*

```ail tool=test,FILE status=1 expect=failed rules=test.run.2
@test
fn one()
    assert(1 == 1)

@test
fn two()
    assert(false)
```

**test.run.3** — The text report has one line per test, `ok NAME` or
`FAIL NAME: MESSAGE at LINE:COL`, then `N passed, M failed`. The message is
`trap[RULE]: text` for a trap and `error: text` for a returned error. The
position is where the trap occurred or the error was first returned. The exit
status is 0 if every test passed and 1 otherwise. *(ADR 0167)*

```ail tool=test,FILE status=1 expect=FAIL rules=test.run.3
@test
fn one()
    assert(1 == 1)

@test
fn two()
    assert(false)
```

**test.run.4** — With `--json`, standard output has one line per test,
`{"test":NAME,"status":"pass"|"fail","line":N,"col":N,"message":TEXT}`, then
`{"summary":true,"passed":N,"failed":M}`. For a passing test, `line` and `col`
are the test's name and `message` is empty. Order and text are deterministic.
*(ADR 0167)*

```ail tool=test,--json,FILE status=1 expect="status":"fail" rules=test.run.4
@test
fn one()
    assert(1 == 1)

@test
fn two()
    assert(false)
```

## Assertions

**test.assert.1** — `assert(cond)` does nothing when `cond` is `true` and traps
otherwise. `assert_eq(a, b)` does nothing when `a == b` and traps otherwise,
with a message showing both values' text forms. Both are in the prelude and
are allowed anywhere, not only in tests: outside a test, the trap ends the
program (`err.trap.3`). *(ADR 0168)*

```ail tool=test,FILE status=1 expect=test.assert.1 rules=test.assert.1
@test
fn one()
    assert(1 == 1)

@test
fn two()
    assert(false)
```

**test.assert.2** — `assert` takes one `bool`. `assert_eq` takes two values of
one type that supports `==` and has a text form. Anything else is a
compile-time error. *(ADR 0168)*

```ail rules=test.assert.2
fn main()
    assert(1 < 2)
    assert_eq("a", "a")
    print("ok")
```

## Property tests

**test.prop.1** — A `@test` function with parameters is a property. `ail test`
runs it once per generated input, as many as `--cases N` (default 100), and it
passes if every run passes. The parameters are not `mut`. *(ADR 0198)*

**test.prop.2** — Each parameter's type is one of `int`, `float`, `bool`,
`char`, `str`, `List[T]`, `Map[K, V]`, `Set[T]`, `T?`, or a non-generic
`struct` or `enum` (not an `error` type) whose parts are generatable. Any other
type is a compile-time error naming the part that can't be generated.
*(ADR 0198)*

**test.prop.3** — Inputs come from a deterministic generator started at a
seed. The seed is `--seed S` if given, otherwise a hash of the test's name; it
is always reported. The same seed and `--cases` give the same inputs on every
platform. Inputs grow from small to larger over the run: ints stay within about
±2^31, floats are finite, and lists, strings, maps and sets stay short.
*(ADR 0199)*

**test.prop.4** — On a failure `ail test` stops that property, shrinks the
input (`test.prop.5`), and reports the smallest failing input as
`name = literal, ...` using the values' text forms (`trait.auto.2`), with the
number of cases run and the seed. The text report adds `(N cases, seed S)` to
the test's line and an `input:` line on a failure. With `--json` the test's
line also has `"cases":N,"seed":S` and, on a failure, `"input":TEXT`. Plain
tests have none of these fields. *(ADR 0199)*

**test.prop.5** — Shrinking tries simpler values, simplest first, and keeps
the first that still fails, until none does or 2000 extra runs were spent.
Ints go to 0, halve and step toward 0; floats to 0 and toward 0; `true` to
`false`; chars to `'a'`; strings and lists to empty, to each half, and without
one element; sets and maps likewise, shrinking values too; optionals to
`none`; enums to an earlier variant; structs field by field. *(ADR 0199)*

```ail fragment
fn add(a int, b int) int
    a + b

@test fn add_commutes(a int, b int)
    assert_eq(add(a, b), add(b, a))
```

```ail error=test.prop.2
struct Box[T]
    item T

@test fn bad(b Box[int])
    assert(true)

fn main()
    print("x")
```

## Doc examples

**test.doc.1** — In `ail test`, each block fenced exactly ```` ```ail ```` inside
a `##` doc comment (`api.doc.1`) on a declaration is compiled as a test with no
parameters, in the scope of the file, so it can call the documented function.
Blocks with any other tag (`ail fragment`, `text`) are not run. Doc examples
run after the file's own tests, in source order. A problem in an example is
reported at its line in the doc comment. *(ADR 0200)*

```ail tool=test,FILE expect=doc:add rules=test.doc.1,test.doc.3
## Adds two numbers.
##
## ```ail
## print(add(1, 2))
## # => 3
## ```
fn add(a int, b int) int
    a + b

fn main()
    print(add(1, 2))
```

**test.doc.2** — If an example has lines `# => text`, the text after `# =>`
(one leading space dropped, trailing spaces ignored), in order, must equal the
lines the example prints; otherwise the test fails with `trap[test.doc.2]`.
An example with no such lines only has to pass. *(ADR 0201)*

```ail tool=test,FILE status=1 expect=FAIL rules=test.doc.2
## Adds two numbers.
##
## ```ail
## print(add(1, 2))
## # => 4
## ```
fn add(a int, b int) int
    a + b

fn main()
    print(add(1, 2))
```

**test.doc.3** — A doc example is reported as `doc:NAME`, where `NAME` is the
declaration's name (`doc:Type.method` for a method, `doc:Enum.Variant` for a
variant). A second block on the same declaration is `doc:NAME#2`, and so on. A
failure is reported at the failing line inside the doc comment, or at the
opening fence for an output mismatch. *(ADR 0200)*

## Failure detail and coverage

**test.run.5** — When a trap fails a test, `ail test` prints the trap's
snapshot (trace.snap.1-2) under the failure: the source line, the values a
failed contract condition read, and the stack. With `--json`, the failing
line carries it as `"snapshot"`. A contract failure (contract.req.1,
contract.ens.1) names the contract in the failure message. *(ADR 0290)*

```ail tool=test,FILE status=1 expect=two() rules=test.run.5
@test
fn one()
    assert(1 == 1)

@test
fn two()
    assert(false)
```

**test.run.6** — `ail test --trace` adds the failing test's trace to its
report (trace.test.1-2). *(ADR 0326)*

```ail tool=test,--trace=calls,FILE status=1 expect=adds() rules=test.run.6
@test
fn adds()
    assert_eq(1 + 1, 3)
```

**test.cover.1** — `ail test --coverage` runs the tests and then prints, for
each source file of the program, the lines and functions they ran. A line
counts when it starts a statement of a function that is not a test. A
function counts as run when its first statement ran. The prelude and test
functions are not counted. *(ADR 0289)*

```ail tool=test,--coverage,FILE expect=coverage rules=test.cover.1
@test
fn one()
    assert(1 == 1)

fn main()
    print(1)
```

**test.cover.2** — The text form is one line per file:
`coverage FILE: lines H/T (P%), functions H/T`. `--coverage=lines` adds
`  not run: 7-8, 12` listing the unexecuted line ranges. *(ADR 0289)*

```ail tool=test,--coverage=lines,FILE expect=lines rules=test.cover.2
@test
fn one()
    assert(1 == 1)

fn main()
    print(1)
```

**test.cover.3** — With `--json`, each file is a line
`{"coverage":true,"file","lines","lines_hit","functions","functions_hit","missed":[[from,to],...]}`
after the summary line; `missed` is always present. Coverage is off unless
asked for and then costs one flag test per statement; a run without it is
unchanged. *(ADR 0289)*

```ail tool=test,--coverage,--json,FILE expect="coverage":true rules=test.cover.3
@test
fn one()
    assert(1 == 1)

fn main()
    print(1)
```

## Checked examples for the attribute and property rules

Each block runs `ail test` or checks the program, with the options on its fence (test.attr.1-4, test.prop.1, 3-5).

```ail error=test.attr.1 rules=test.attr.1
@slow fn main()
    print(1)
```

```ail tool=test,FILE expect=checks rules=test.attr.2
@test fn checks()
    assert_eq(1, 1)

fn main()
    print("ok")
```

```ail error=test.attr.3 rules=test.attr.3
@test fn checks()
    assert(true)

fn main()
    checks()
```

```ail tool=test,FILE status=1 expect=FAIL rules=test.attr.4
error struct Empty
    why str

fn parse(s str) int ! Empty
    if s == ""
        return Empty(why: "empty")
    7

@test fn reads()
    n := parse("")?
    assert(n == 7)
```

```ail tool=test,--cases,5,FILE expect=cases rules=test.prop.1
@test fn doubles(x int)
    assert(x * 2 >= x or x < 0)

fn main()
    print("ok")
```

```ail tool=test,--seed,7,FILE expect=seed rules=test.prop.3
@test fn doubles(x int)
    assert(x * 2 >= x or x < 0)

fn main()
    print("ok")
```

```ail tool=test,FILE status=1 expect=input rules=test.prop.4
@test fn small(x int)
    assert(x < 10)

fn main()
    print("ok")
```

```ail tool=test,FILE status=1 expect=10 rules=test.prop.5
@test fn small(x int)
    assert(x < 10)

fn main()
    print("ok")
```
