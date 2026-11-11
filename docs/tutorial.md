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

fn main() !
    path := env.args().first() else
        print("usage: ail run wc.ail FILE")
        return
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
  fallback, or here leaves the function when there is no argument.
- `fn main() !` means `main` can fail. `fs.read_text(path)?` passes a failure
  up to the caller, with the text after it as context. If the file is missing,
  the program stops and reports the error. It does not crash.
- `"{word:<12}{counts[word]:>4}"` interpolates any expression. After the colon
  go the width and alignment, as in `<12` (left, 12 wide) and `>4` (right).

Words after the file name are the program's arguments:

```sh
./ail run wc.ail README.md
```

## 3. A test

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
every test passed. `ail run` ignores tests, so they cost nothing in a shipped
program. A test with parameters is a **property**: `ail test` generates inputs
and shrinks a failure to the smallest one.

## 4. When the compiler complains

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
fix: replace `prnt` with `print`
```

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

`ail check --json` gives the same diagnostics, one JSON object per line, with
the exact edits. This is what editors and AI agents read.

## 5. One layout

AIL has one canonical layout, and `ail fmt` writes it, so there is nothing to
argue about in review:

```sh
./ail fmt wc.ail              # rewrite the file
./ail fmt --check wc.ail      # exit 1 if it would change, e.g. in a hook
```

The layout rules are in [design/formatting.md](design/formatting.md).

## 6. Where next

- Read real programs in [`examples/`](../examples/README.md): `csvreport.ail`,
  `logscan.ail` and `shapes.ail` are good next steps.
- [quick-reference.md](quick-reference.md) covers structs, enums, `match`,
  traits, generics, errors and the standard library on one page.
- [`ail build FILE.ail`](quick-reference.md) makes a static native executable.
- `ail lsp` is a language server for any editor.
- [status.md](status.md) lists what is implemented today. AIL is pre-1.0 and
  some things are experimental.
