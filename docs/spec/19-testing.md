# 19. Testing

Chapter key: `test`

> **Status:** Drafted (partial). Specified: attributes on functions, `@test`
> functions, the `ail test` runner and its result format, and `assert` and
> `assert_eq`. Not yet designed: fixtures, test filtering, parallel runs,
> expected-failure tests, and spec-example extraction (see the plan, section 6).

## Attributes

**test.attr.1** — The only attributes are `@test` and `@pure`. Any other
attribute name is a compile-time error that lists the known attributes. The
attribute syntax itself is `gram.attr.1–2`; an attribute on a line of its own
is `gram.attr.2`. *(ADR 0083, ADR 0167)*

**test.attr.2** — A `@test` function is a free top-level function with no
parameters and no type parameters. Its return type is absent or `!`.
*(ADR 0167)*

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

**test.run.2** — Each test runs in isolation. A trap (including a failed
assertion) or a returned error ends that test only. Constants are computed
once. What a test prints is captured, and shown under its `FAIL` line.
*(ADR 0167)*

**test.run.3** — The text report has one line per test, `ok NAME` or
`FAIL NAME: MESSAGE at LINE:COL`, then `N passed, M failed`. The message is
`trap[RULE]: text` for a trap and `error: text` for a returned error. The
position is where the trap occurred or the error was first returned. The exit
status is 0 if every test passed and 1 otherwise. *(ADR 0167)*

**test.run.4** — With `--json`, standard output has one line per test,
`{"test":NAME,"status":"pass"|"fail","line":N,"col":N,"message":TEXT}`, then
`{"summary":true,"passed":N,"failed":M}`. For a passing test, `line` and `col`
are the test's name and `message` is empty. Order and text are deterministic.
*(ADR 0167)*

## Assertions

**test.assert.1** — `assert(cond)` does nothing when `cond` is `true` and traps
otherwise. `assert_eq(a, b)` does nothing when `a == b` and traps otherwise,
with a message showing both values' text forms. Both are in the prelude and
are allowed anywhere, not only in tests: outside a test, the trap ends the
program (`err.trap.3`). *(ADR 0168)*

**test.assert.2** — `assert` takes one `bool`. `assert_eq` takes two values of
one type that supports `==` and has a text form. Anything else is a
compile-time error. *(ADR 0168)*
