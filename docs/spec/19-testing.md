# 19. Testing

Chapter key: `test`

> **Status:** Drafted (partial). Specified: attributes on functions, `@test`
> functions, the `ail test` runner and its result format, test filtering and listing, fixtures, the test sandbox, stubs,
> `assert`, `assert_eq`, `assert_err` and `assert_near`, property tests, fuzzing, mutation testing,
> and doc examples as tests. Not yet designed: parallel runs and expected-failure tests.

## Attributes

**test.attr.1** — The attributes are `@test`, `@fixture`, `@needs`, `@pure`, `@inline`, `@deprecated`, `@allow`, `@tool` (`attach.tool.1`) and `@caps` (`eff.cap.6`). Any other
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
It also runs the parameterless `@test` functions of each module the file
imports from its package, under the name the import gives the module
(`mod.use.1`): `use a.b` runs those of `a/b` as `b.NAME`, and `use a.b as c`
as `c.NAME`. *(ADR 0167)*

```ail tool=test,FILE expect=passed rules=test.run.1
@test
fn one()
    assert(1 == 1)
```

```ail pkg=test_modules tool=test,--json,FILE expect="passed":3,"failed":0 rules=test.run.1
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

**test.affected.1** — `ail test --affected [--since REV] FILE` runs only the
`@test` functions that the changes since git revision `REV` (default `HEAD`)
can reach (`api.impact.3`) and says `affected: N of M tests` on standard
error. The tests of imported modules always run. With no changes no test
runs. Everything else is as `test.run.1-4`.
[ADR 0868](../decisions/0868-ai-first-tooling-after-rust-removal.md) item 3,
[ADR 0882](../decisions/0882-edit-impact.md).

```ail tool=test,--affected,--since,nosuch,FILE status=2 expect=api.impact.3 rules=test.affected.1
@test
fn adds()
    assert_eq(1 + 1, 2)
```

## Filtering and listing

**test.filter.1** — `ail test --filter PATTERN FILE` runs only the tests whose name matches
`PATTERN` and says `filter: N of M tests` on standard error. A pattern matches a name that
contains it; a `*` in the pattern stands for any run of characters. The name is the function's name,
`MODULE.NAME` for a test of an imported module, and `doc:NAME` for a doc example (`test.doc.3`).
`--filter` can be given several times, and a test that matches any pattern runs. Everything else is as
`test.run.1-4`. A pattern that matches no test is an error (exit 2) that says how many tests the file has.
([ADR 1075](../decisions/1075-test-filter-and-fixtures.md))

```ail tool=test,--filter,add,FILE expect=passed rules=test.filter.1
@test
fn adds()
    assert_eq(1 + 1, 2)

@test
fn fails()
    assert(false)
```

```ail tool=test,--filter,nosuch,FILE status=2 expect=test.filter.1 rules=test.filter.1
@test
fn adds()
    assert_eq(1 + 1, 2)
```

**test.filter.2** — With `--filter`, only the matching tests are built and run, and the rest are not
reported. A property test is matched by name like any other (`test.prop.1`). `--cases`, `--seed`,
`--json`, `--coverage` and `--trace` work as without it.

```ail tool=test,--filter,a*s,--json,FILE expect="test":"adds" rules=test.filter.2
@test
fn adds()
    assert_eq(1 + 1, 2)

@test
fn zero()
    assert(false)
```

**test.filter.3** — `--filter` can't be combined with `--update` or `--mutate`, which need every test: it
is an error (exit 2) naming `test.filter.3`.

```ail tool=test,--filter,a,--mutate,FILE status=2 expect=test.filter.3 rules=test.filter.3
@test
fn adds()
    assert_eq(1 + 1, 2)
```

**test.list.1** — `ail test --list [--filter PATTERN] FILE` checks the program and prints the name of each
test in source order, one per line, without building or running anything (exit 0). With `--json`, each
test is `{"test":NAME,"line":N}` (`line` is 0 for a test that can't be built) followed by
`{"summary":true,"listed":N}`. ([ADR 1075](../decisions/1075-test-filter-and-fixtures.md))

```ail tool=test,--list,FILE expect=adds rules=test.list.1
@test
fn adds()
    assert_eq(1 + 1, 2)

@test
fn fails()
    assert(false)
```

```ail tool=test,--list,--json,--filter,fail,FILE expect="listed":1 rules=test.list.1
@test
fn adds()
    assert_eq(1 + 1, 2)

@test
fn fails()
    assert(false)
```

## Fixtures

**test.fixture.1** — `@fixture` marks a top-level function with no parameters and no type parameters
that makes a value for tests: its return type is the value's type `T`, or `!T` when setup can fail.
`@fixture("END")` also names the teardown function `END` (`test.fixture.2`). A fixture is an ordinary
function in every other way and is not a test. *(ADR 1075)*

**test.fixture.2** — A teardown is a top-level function `fn END(value T)` that takes the fixture's value
and returns nothing, so it can't fail. *(ADR 1075)*

**test.fixture.3** — A `@test` function lists the fixtures it uses with `@needs("NAME")` (the name in quotes) after `@test`, one
attribute per fixture, and declares a parameter `NAME` of the fixture's type for each. These parameters
get the fixtures' values and are not generated, so a property test can take both. A fixture named
twice, a name that is not a `@fixture`, a missing or differently typed parameter, and `@needs` on a function
that is not a `@test` are compile-time errors. *(ADR 1075)*

**test.fixture.4** — Each run of a test makes its fixtures anew, in the order of the `@needs` attributes (a
property test makes them for each generated input), and tears them down in reverse order when the run
ends, whether it passed, failed, trapped or returned an error. A fixture whose setup fails ends
the test as `FAIL NAME: error: MESSAGE at LINE:COL`, at the position where the fixture first
returned the error, with the line `! context: setup of fixture F` under it; the test body does not run
and the fixtures made before it are torn
down. A fixture and its teardown must be in the same file as the test. *(ADR 1075)*

```ail tool=test,FILE expect=passed rules=test.fixture.1,test.fixture.2,test.fixture.3,test.fixture.4
@fixture("drop_file")
fn scratch_file() !str
    name := "{fs.temp_dir()}/fixture-demo.txt"
    fs.write_text(name, "hi")?
    name

fn drop_file(name str)
    fs.remove(name) else return

@test @needs("scratch_file")
fn reads_the_file(scratch_file str)
    assert_eq(fs.read_text(scratch_file)?, "hi")
```

```ail tool=test,FILE status=1 expect=context rules=test.fixture.4
error struct NoDatabase
    why str

@fixture
fn broken() !int
    return NoDatabase(why: "not running")

@test @needs("broken")
fn uses_it(broken int)
    assert_eq(broken, 1)
```

```ail error=test.fixture.1 rules=test.fixture.1
@fixture
fn needs(x int) int
    x

fn main()
    print("x")
```

```ail error=test.fixture.2 rules=test.fixture.2
@fixture("drop")
fn make() int
    1

fn drop(value str)
    print(value)

fn main()
    print("x")
```

```ail error=test.fixture.3 rules=test.fixture.3
@fixture
fn make() int
    1

@test @needs("make")
fn uses(x int)
    assert_eq(x, 1)

fn main()
    print("x")
```

## Sandbox

**test.sandbox.1** — `ail test` runs each test in a sandbox, a use of `eff.allow.1-9` that needs no flag. The
test may not use `net`, `net.serve` or `proc`. It may read files under the current directory, the directory
of the test file and the package root (the nearest directory above the test file with an `ail.pkg`). It may
write only in its own temporary directory, which is also where it may read what it wrote. That directory is
new for each run, is the test's `fs.temp_dir()` (the runner sets `TMPDIR` and `TEMP`), and is removed when the
run ends. `env`, `clock` and `random` are not limited. A property test gets a new directory for each generated
input. *(ADR 1100)*

```ail tool=test,FILE expect=passed rules=test.sandbox.1
@test fn writes_in_its_own_directory() !
    name := "{fs.temp_dir()}/note.txt"
    fs.write_text(name, "kept for this test only")?
    assert_eq(fs.read_text(name)?, "kept for this test only")
```

A task started by the test has the same limits and the same directory, so a file the test wrote can be read by it:

```ail tool=test,FILE expect=passed rules=test.sandbox.1
fn read_into(p str, out Channel[str])
    out.send(fs.read_text(p) else "EMPTY")

@test fn tasks_share_the_directory() !
    p := "{fs.temp_dir()}/shared.txt"
    fs.write_text(p, "hello")?
    out := Channel[str](4)
    g := TaskGroup()
    g.add(spawn read_into(p, out))
    g.add(spawn read_into(p, out))
    g.wait_all()?
    out.close()
    assert_eq([x for x in out].join(","), "hello,hello")
```

The runner's own imports use hidden names, so a program may name a parameter or a local `env` or `io`:

```ail tool=test,FILE expect=passed rules=test.run.1
fn show(env str, io int) str
    "{env}{io}"

@test fn names_may_be_env_and_io()
    assert_eq(show("a", 1), "a1")
```

**test.sandbox.2** — `@needs(CAP)` after `@test` lifts the limit for one test: `@needs(fs)` allows every file
(read and write), `@needs(net)` the network, `@needs(net.serve)` listening, and `@needs(proc)` running programs.
Several are written `@needs(net, proc)`. A capability name in bare words is a capability; a name in quotes is a
fixture (`test.fixture.3`) and can sit beside it. A word that is not one of the four, a capability listed twice,
and a quoted capability name that is not a fixture are compile-time errors. *(ADR 1100)*

```ail tool=test,FILE expect=passed rules=test.sandbox.2
@test @needs(fs)
fn may_write_beside_the_program() !
    fs.write_text(".ail-sandbox-demo.txt", "x")?
    fs.remove(".ail-sandbox-demo.txt")?
```

```ail error=test.sandbox.2 rules=test.sandbox.2
@test @needs(gpu)
fn t()
    assert(true)

fn main()
    print("x")
```

**test.sandbox.3** — A call the sandbox refuses fails that test only, with `trap[eff.allow.N]`, the call chain,
and the line to write: `add @needs(CAP) to the test, or run ail test --allow all`. Other tests are not
affected. *(ADR 1100)*

```ail tool=test,FILE status=1 expect=test.sandbox.3 rules=test.sandbox.3
@test fn writes_beside_the_program() !
    fs.write_text(".ail-sandbox-denied.txt", "x")?
```

**test.sandbox.4** — `ail test --allow all` runs every test with no sandbox, as before `test.sandbox.1` (the test
still gets its temporary directory as `fs.temp_dir()`, for stubs). It takes
no other list (`@needs` is the way to widen one test), and `ail task verify` runs without it. *(ADR 1100)*

```ail tool=test,--allow,all,FILE expect=passed rules=test.sandbox.4
@test fn writes_beside_the_program() !
    fs.write_text(".ail-sandbox-all.txt", "x")?
    fs.remove(".ail-sandbox-all.txt")?
```

## Stubs

**test.stub.1** — `use std.test` provides stubs for one test: `files`, `env`, `clock`, `advance` and `http`. A stub is
active from the call until the test ends, and each test starts with none. They are for tests run by `ail test`
(`test.run.1`); anywhere else a call fails with a message that says so. A stub answers without the sandbox
(`test.sandbox.1`) asking: a stubbed request needs no `@needs(net)`. *(ADR 1101)*

**test.stub.2** — `test.files(content Map[str, str])` makes `fs.read_text`, `fs.read_bytes` and `fs.exists` of each
path answer with the text, and writes nothing to disk. Paths are compared as `eff.allow.3` compares them, so `a.txt`
and `./a.txt` are one file. A path that is not stubbed reads the real file system as before; `fs.list_dir`,
`fs.walk` and the other calls do not see stubbed files. A later call for a path replaces its text. *(ADR 1101)*

**test.stub.3** — `test.env(vars Map[str, str])` makes `env.get(name)` return the value for each name. Other
variables, `env.vars()` and `env.home()` are unchanged. *(ADR 1101)*

**test.stub.4** — `test.clock(ms int)` fixes the clock: `time.now_ms()` and everything built on it return `ms`, which
stands still. `time.sleep_ms(n)` then returns at once and moves the clock on by `n`, and `test.advance(ms)` moves it
on by `ms`. `test.advance` before `test.clock`, and a negative time, are failures of the test. The monotonic clock
that deadlines use is not stubbed. *(ADR 1101)*

**test.stub.5** — `test.http(call str, status int, body str)` makes a request `call`, written `METHOD URL` with the
method in capitals and the URL as the program writes it (`"GET https://api.example.com/items"`), answer with `status`,
`body` and no headers, with no connection. Any call of `std.net` that sends that method to that URL gets it; a request
with no stub goes on as without stubs (and is refused by the sandbox unless the test has `@needs(net)`). A `call`
that is not `METHOD URL` is a failure of the test. *(ADR 1101)*

**test.stub.6** — Stubs are made by `ail test` and kept in a file in the test's temporary directory, which is
`fs.temp_dir()` for that test (`test.sandbox.1`). Nothing outlives the test, and a stub can't leak into another. *(ADR 1101)*

```ail tool=test,FILE expect=passed rules=test.stub.1,test.stub.2,test.stub.3,test.stub.6
use std.test

@test fn reads_a_stubbed_file_and_variable() !
    test.files({"config.txt": "mode=fast"})
    test.env({"APP_MODE": "test"})
    assert(fs.exists("./config.txt"))
    assert_eq(fs.read_text("config.txt")?, "mode=fast")
    assert_eq(env.get("APP_MODE") else "", "test")

@test fn the_next_test_starts_clean()
    assert(not fs.exists("config.txt"))
    assert_eq(env.get("APP_MODE") else "none", "none")
```

```ail tool=test,FILE expect=passed rules=test.stub.4
use std.test

@test fn the_clock_moves_only_when_told()
    test.clock(1000)
    assert_eq(time.now_ms(), 1000)
    time.sleep_ms(500)
    assert_eq(time.now_ms(), 1500)
    test.advance(250)
    assert_eq(time.now_ms(), 1750)
```

```ail tool=test,FILE expect=passed rules=test.stub.5
use std.test

@test fn a_stubbed_request_needs_no_network() !
    test.http("GET https://api.example.com/items", 200, "one,two")
    r := net.get("https://api.example.com/items")?
    assert_eq(r.status, 200)
    assert_eq(r.body, "one,two")
```

```ail tool=test,FILE status=1 expect=test.stub.5 rules=test.stub.5
use std.test

@test fn the_call_needs_a_method()
    test.http("https://api.example.com/items", 200, "x")
```

## Assertions

**test.assert.1** — `assert(cond)` does nothing when `cond` is `true` and traps
otherwise. `assert(cond, "message")` traps with `assertion failed: message`;
the message is evaluated only when `cond` is false. `assert_eq(a, b)` does
nothing when `a == b` and traps otherwise, with a message showing both
values' text forms. All of them are in the prelude and are allowed anywhere,
not only in tests: outside a test, the trap ends the program (`err.trap.3`).
*(ADR 0168, ADR 1087)*

```ail tool=test,FILE status=1 expect=test.assert.1 rules=test.assert.1
@test
fn one()
    assert(1 == 1)

@test
fn two()
    assert(false, "two is false")
```

**test.assert.2** — `assert` takes one `bool` and, optionally, a `str`
message. `assert_eq` takes two values of one type that supports `==` and has a
text form. `assert_near` takes three `float`s. Anything else is a compile-time
error. *(ADR 0168, ADR 1087)*

```ail rules=test.assert.2
fn main()
    assert(1 < 2)
    assert(1 < 2, "one is less than two")
    assert_eq("a", "a")
    print("ok")
```

**test.assert.3** — `assert_err(call)` does nothing when `call` (an expression
of type `T ! E`) fails, and traps when it returns a value. `assert_err(call,
E.Variant)` also requires the error to equal the second argument, and traps
otherwise; that argument must be one of the errors the call declares, else it
is a compile-time error. For a variant that carries data, `E.Variant` alone
checks the kind of the error and ignores its fields; to check them too, pass
the whole value, `E.Variant(field: x)`. `assert_near(a, b, eps)` does nothing when
`|a - b| <= eps` and traps otherwise (also when a value is NaN), showing the
three values. The error `assert_err` catches is handled: the test goes on.
*(ADR 1087)*

```ail rules=test.assert.3
error enum Fail
    Bad
    Missing(name str)

fn pick(n int) int ! Fail
    if n == 0
        return Bad
    if n < 0
        return Missing(name: "n")
    n

fn main()
    assert_err(pick(0))
    assert_err(pick(0), Fail.Bad)
    assert_err(pick(-1), Fail.Missing)
    assert_err(pick(-1), Fail.Missing(name: "n"))
    assert_near(0.1 + 0.2, 0.3, 0.0001)
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

## Fuzzing

**test.fuzz.1** — `ail test --fuzz NAME [--time MS] [--cases N] [--seed S] [--corpus DIR] FILE`
fuzzes the property test `NAME` ([ADR 1015](../decisions/1015-user-fuzzing.md)). Only that test
is built, and the build counts the statements run (`test.cover.4`). The run ends after `--time`
milliseconds (default 10000), or after `--cases` runs when that is given and `--time` is not, and
the exit status and the text and `--json` forms are those of a property test (`test.prop.4`,
`test.run.3-4`), with `cases` the number of runs. A name that is not a property test of the file
is an error (exit 2).

```ail tool=test,--fuzz,nosuch,FILE status=2 expect=test.fuzz.1 rules=test.fuzz.1
@test fn small(n int)
    assert(n < 1000000)

fn main()
    print("x")
```

**test.fuzz.2** — A run is coverage-guided. The first input is the simplest value of each
parameter. Every later input is, one time in ten, a fresh `test.prop.3` input, and otherwise a
mutation of an input kept earlier: one to three parameters changed by a small step or an edge
value (numbers), an inserted, dropped or changed character (`str`, `bytes`), an element added,
dropped or changed (collections), or a changed field (`struct`, `enum`, tuples, optionals). An
input is kept when it runs a statement that no earlier input did. The parameter types are those of
`test.prop.2`. The same `--seed` and the same hit pattern give the same inputs.

**test.fuzz.3** — With `--corpus DIR`, the directory is made if missing. Before the first mutation
every file in it that fits the parameters is run, then each kept input is written to
`DIR/id-HASH`, so a later run starts from the earlier run's coverage. A file is the arguments'
text tokens (`test.prop.4`), one `=TOKEN` line each, with `\`, newline and return written
`\\`, `\n` and `\r`. A file that does not fit the parameters is ignored.

**test.fuzz.4** — The first failing input is shrunk as in `test.prop.5` and reported as in
`test.prop.4`. With `--corpus DIR` it is also written to `DIR/crash-HASH`, which the next run
replays first, so a fixed bug stays a regression test. `--bundle-failure` records the failure as
for any test (`bundle.write.1`), and `ail replay --shrink` (`shrink.run.1`) shrinks that bundle.

**test.fuzz.5** — A run that finds nothing ends with the line `fuzz NAME: R runs, K kept, S
statements reached` on standard error. Only the property test's own code and what it calls count
as statements.

```ail tool=test,--fuzz,small,--cases,400,--corpus,.ail-test-fuzz-corpus,FILE status=1 expect=input: rules=test.fuzz.2,test.fuzz.3,test.fuzz.4
@test fn small(n int)
    assert(n < 100)

fn main()
    print("x")
```

```ail tool=test,--fuzz,small,--cases,50,FILE expect=cases rules=test.fuzz.5
@test fn small(n int)
    assert(n + 0 == n)

fn main()
    print("x")
```

## Mutation testing

**test.mutate.1** — `ail test --mutate [--max N] [--timeout MS] FILE` checks how well the tests of
`FILE` notice a change to the program ([ADR 1022](../decisions/1022-mutation-testing.md)). It runs
the tests once, with the build counting the statements run (`test.cover.4`), and prints their
results as usual. If a test fails, no test passed, or the tests don't build, no mutant is made, the
error names `test.mutate.1`, and the exit status is that of the run (1 when a test failed, 2 otherwise).

```ail tool=test,--mutate,FILE status=1 expect=test.mutate.1 rules=test.mutate.1
fn five() int
    5

@test fn bad()
    assert(five() == 6)

fn main()
    print("x")
```

**test.mutate.2** — A mutant is the program with one change to one line. A line `if C` or `elif C`
(with no `then`) has the mutant `if not (C)`. Then each place in a line where one of these
applies gives a mutant: `==`, `!=`, `<=`, `>=`, `+=` and `-=` become `!=`, `==`, `<`, `>`, `-=` and
`+=`; `<` and `>` become `<=` and `>=`; `+`, `-`, `*` and `/` become `-`, `+`, `+` and `*`; `and`,
`or`, `true` and `false` become `or`, `and`, `false` and `true`; `not` is dropped; a whole decimal
number N becomes N + 1, and 0 becomes 1. Strings, characters, comments, attribute lines, `use`,
`assert`, `requires` and `ensures` lines and lines at the top level are not changed.

**test.mutate.3** — Only a line that the run of `test.mutate.1` reached, in a function that is not
a test and not `main`, is changed. Mutants are in line order, then column order. There are at
most `--max` of them (default 50, and 0 means no limit); when there are more, they are
spread evenly (the Ith kept is number I times count over max), so the same program and tests give the same mutants.

**test.mutate.4** — Each mutant is written to a copy of the file beside it and run with `ail test
--json` and a per-test limit of `--timeout` (default 5000 ms). The mutant is *killed* when that run
exits 1, *survived* when it exits 0 and as many tests passed as in the first run, and *invalid*
otherwise (it doesn't type-check, doesn't build, or a test was left out). The copy is deleted at the end.

**test.mutate.5** — A surviving mutant is printed as `survived FILE:LINE: CHANGE (in FN)`; the
last line is `mutants: N run, K killed, S survived, I invalid, score P%`, where P is K over K plus
S (100 when both are 0). With `--json` each survivor is `{"mutant":true,"file":...,"line":...,
"fn":...,"mutation":...,"status":"survived"}` and the last line is `{"mutate":true,"mutants":N,
"killed":K,"survived":S,"invalid":I}`. The exit status is 1 when S is more than 0, otherwise 0.

```ail tool=test,--mutate,FILE status=1 expect=survived rules=test.mutate.2,test.mutate.3,test.mutate.4,test.mutate.5
fn sign(n int) int
    if n > 0
        return 1
    if n < 0
        return -1
    0

@test fn positive()
    assert(sign(5) == 1)

fn main()
    print("x")
```

```ail tool=test,--mutate,--max,3,FILE expect=mutants rules=test.mutate.3,test.mutate.5
fn double(n int) int
    n * 2

@test fn doubles()
    assert_eq(double(3), 6)
    assert_eq(double(0), 0)

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

**test.cover.4** — The native `ail test --coverage` is built on the
instrumentation of native.instr.1-3. The file's own `main` stays in the test
build, never called, so it counts like any function the tests do not run. *(ADR 0871)*

```ail tool=test,--coverage,FILE expect=coverage rules=test.cover.4
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
