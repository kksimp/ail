# AIL tutorial

> **Non-normative.** A first hour with AIL for people. You will run a program,
> write a small command-line tool, test it, and use the compiler's own help
> when something is wrong. The [spec](spec/README.md) is authoritative, and the
> [quick reference](quick-reference.md) is the one-page cheat sheet.

You need a checkout of this repository. Every command runs from its root. To run `ail` from
anywhere, link it onto your PATH (`ail doctor` checks the rest, `ail help` lists every command):

```sh
mkdir -p ~/.local/bin && ln -s "$PWD/ail" ~/.local/bin/ail   # macOS and Linux
ail doctor --path                                            # the profile line, if ~/.local/bin is not on PATH
```

On Windows, put the clone's directory on `PATH` (`ail doctor --path` prints the PowerShell line) and use `ail.cmd`.

## 1. Hello

```ail
fn main()
    print("Hello, world!")
```

Save that as `hello.ail`, or use `examples/hello.ail`, and run it:

```sh
./ail run examples/hello.ail
```

The first run downloads the published seed compiler and builds the native
`ail` into `.native/`; later runs start at once. On Windows use
`.\ail.ps1 run examples\hello.ail`.

Three things to notice:

- A block is **indented by four spaces**. There are no braces and no semicolons.
- `fn main()` is the entry point. `print` writes any value and a newline.
- The last expression of a function is its result, so `fn add(a int, b int) int`
  needs no `return` (the type comes after the name, with no colon).

## 2. A small tool: word counts

We will write `wc.ail`: it reads a text file named on the command line and
prints its most common words. Start with the counting, which is a plain
function:

```ail
fn count_words(text str) Map[str, int]
    mut counts := Map[str, int]()
    for word in text.words()
        counts[word.to_lower()] += 1
    counts

fn top(counts Map[str, int], n int) List[str]
    counts.keys().sorted(by: |k| (-counts[k], k)).take(n)

fn usage() never
    io.eprint("usage: ail run wc.ail FILE")
    io.exit(2)

fn main() !
    path := env.args().first() else usage()
    text := fs.read_text(path)? "reading {path}"
    counts := count_words(text)
    for word in top(counts, 5)
        print("{word:<12}{counts[word]:>4}")
```

What each part does:

- `x := value` declares a name that cannot change. `mut x := value` declares
  one that can, and `x = 5` or `x += 1` changes it.
- `Map[str, int]()` is an empty map. A missing key starts at zero, so
  `counts[w] += 1` counts.
- `sorted(by: |k| (-counts[k], k))` sorts by a key. The key is a tuple: biggest
  count first, then the word, so ties come out in a fixed order.
- `env.args().first()` is an `str?`: a string or `none`. `else` supplies a
  fallback. Here the fallback is `usage()`, a function declared `never`: it ends
  the program (`io.exit`), so it fits where a value is expected and `path` is
  always a string after the line. `io.eprint` writes a line to standard error.
- `fn main() !` means `main` can fail. `fs.read_text(path)?` passes a failure
  up to the caller, with the text after it as context. If the file is missing,
  the program stops and reports the error. It does not crash.
- `"{word:<12}{counts[word]:>4}"` interpolates any expression. After the colon
  go the width and alignment, as in `<12` (left, 12 wide) and `>4` (right).

Words after the file name are the program's arguments:

```sh
./ail run wc.ail README.md
```

## 3. Failing and falling back

Three more forms you will use at once. A function can fail with plain text:
declare it `T ! str` and `return` the message. A fallback can be a block, and
an `if` can be one line:

```ail
fn parse_port(s str) int ! str
    n := s.to_int() else return "not a number: {s}"
    if n < 1 or n > 65535
        return "port out of range: {n}"
    n

fn main() !
    port := parse_port("8080")?
    kind := if port < 1024 then "privileged" elif port < 49152 then "registered" else "dynamic"
    print("{port} is {kind}")
    mut total := 0
    for s in ["4", "x", "6"]
        total += s.to_int() else
            print("skip {s}")
            continue
    print(total)
```

- `int ! str` fails with a `str`: `return "text"` is the failure and any other
  `return` or the last expression is the result. The caller's `?` passes it up
  as text. With no result, or a `str` result, write `! Fail` and
  `return Fail(msg: "text")` instead (`err.fail.1`); `e.msg` is the text.
- `x += e else` followed by a block (`err.else.8`) runs the block only when `e`
  has no value. The block must leave the loop or function, as `continue` does
  here, or end in a value.
- `if c then a elif d then b else c` is one expression (`expr.if.8`); the final
  `else` is required.
- The standard modules (`fs`, `env`, `io`, `time`, `bigint` and the rest) need no
  `use` line; you write `use` only for your own modules.

## 4. A test

Put a test next to the code. A `@test` function takes no arguments and fails
if an `assert` fails or it returns an error:

```ail
fn count_words(text str) Map[str, int]
    mut counts := Map[str, int]()
    for word in text.words()
        counts[word.to_lower()] += 1
    counts

fn top(counts Map[str, int], n int) List[str]
    counts.keys().sorted(by: |k| (-counts[k], k)).take(n)

@test
fn counts_ignore_case()
    c := count_words("the cat the Dog The cat")
    assert_eq(c["the"], 3)
    assert_eq(top(c, 2), ["the", "cat"])

fn main()
    print(top(count_words("a b a"), 1))
```

```sh
./ail test wc.ail
```

The report has one line per test, `ok NAME` or `FAIL NAME: MESSAGE at LINE:COL`,
then a count ([test.run.3](spec/19-testing.md)). `ail test` exits 0 only if
every test passed, and 1 if one failed. A file with no tests prints
`no tests found in FILE` before the `0 passed, 0 failed` line and exits 0, since
nothing failed ([check.exit.1](spec/22-tools.md)). `ail run` ignores tests, so they cost nothing in a shipped
program. A test with parameters is a **property**: `ail test` generates inputs
and shrinks a failure to the smallest one.

## 5. When the compiler complains

Every diagnostic carries a **rule ID** from the spec. Make a typo:

```text
fn main()
    prnt("hi")
```

```sh
./ail check typo.ail
```

```text
typo.ail:2:5: error[expr.name.1]: unknown name `prnt`
        prnt("hi")
        ^
help: did you mean `print`?
fix: write `print("hi")`
next: ail fix typo.ail
```

Every error ends with a `next:` line: `ail fix FILE` when the diagnostic carries a
fix, otherwise `ail explain RULE` ([check.out.2](spec/22-tools.md)). When a
file has no error, `ail check` says so, so a clean run is not mistaken for one that
did not run:

```text
ok wc.ail: 3 functions, 0 warnings
```

`ail check` exits 0 when there is no error (warnings are allowed) and 2 when
there is one. `ail run` of a program that does not compile prints the same
diagnostics and exits 2 without running it; a program that traps exits 1
([check.exit.1](spec/22-tools.md)).

The ID in the brackets is the same one you find in the spec, in the code that
enforces it, and in the tests. Two commands use it:

```sh
./ail explain expr.name.1     # the rule's text, the ADRs behind it, an example
./ail explain                 # no rule: list them all
```

`ail explain` answers from the specification embedded in the compiler, so it
works offline and always matches the version you run.

Where a diagnostic carries a fix, the compiler can apply it for you:

```sh
./ail fix --dry-run typo.ail  # show what would change
./ail fix typo.ail            # rewrite the file
```

Some mistakes are another language's spelling of an AIL form: `eprint`, `else if`, `&&`,
`m.contains_key(k)`, a missing `mut`. These have one possible meaning, so `ail check`, `ail run`,
`ail build` and `ail test` correct them in your file, print a `fixed FILE:LINE:COL: [RULE] TITLE`
line on standard error, and go on ([fix.auto.1-3](spec/22-tools.md)). Read the file again before
you edit it. `ail check --no-fix` (or `AIL_FIX=0`) turns this off. A mistake with more than one
plausible fix, such as the misspelled name above, is only reported; `ail fix` or `ail check --fix`
applies it.

`ail check --json` gives the same diagnostics, one JSON object per line on
standard error, with the exact edits, and a summary record on standard output,
`{"api":1,"summary":true,"errors":1,"warnings":0,"fixable":1}`. The record's
shape is [check-json.schema.json](design/check-json.schema.json). This is what
editors and AI agents read.

## 6. One layout

AIL has one canonical layout, and `ail fmt` writes it, so there is nothing to
argue about in review:

```sh
./ail fmt wc.ail              # rewrite the file
./ail fmt --check wc.ail      # exit 1 if it would change, e.g. in a hook
```

The layout rules are in [design/formatting.md](design/formatting.md).

## 7. Where next

- Read real programs in [`examples/`](../examples/README.md): `csvreport.ail`,
  `logscan.ail` and `shapes.ail` are good next steps.
- [for-agents.md](for-agents.md) is the same language written for AI agents: the
  loop, the canonical patterns, the mistakes that cost the most and their fixes.
- `ail ask std "read a file by lines"` finds a std function, and `ail ask how "sort descending"`
  a checked answer to a common task ([how-to.md](how-to.md)).
- [quick-reference.md](quick-reference.md) covers structs, enums, `match`,
  traits, generics, errors and the standard library on one page.
- [`ail build FILE.ail`](quick-reference.md) makes a static native executable.
- `ail lsp` is a language server for any editor.
- [status.md](status.md) lists what is implemented today. AIL is pre-1.0 and
  some things are experimental.
