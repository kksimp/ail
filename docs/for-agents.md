# AIL for agents

> **Non-normative.** A dense guide for an AI agent that writes, changes or reviews
> AIL. People can read it too: it is the shortest route from "I know another language"
> to "this compiles on the first try". The [language card](language-card.md) has the
> whole language, the [spec](spec/README.md) is authoritative, and every claim here
> cites a rule ID you can read with `ail explain RULE`.

Load order: this guide, then `ail card` (about 3,000 tokens) or the
[full card](language-card.md) (within 16,000 tokens, ADR 1110), or both in one file:
[llms-full.txt](../llms-full.txt). Do not load the spec whole; look rules up.

## 1. The loop

```sh
ail check FILE.ail       # after every edit; exit 0 clean, 2 on an error
ail fix FILE.ail         # apply the fixes diagnostics carry (`--dry-run` shows them)
ail run FILE.ail         # check, then run; a compile error exits 2, a trap exits 1
ail test FILE.ail        # run `@test` functions; exit 1 if one fails
ail fmt FILE.ail         # the one canonical layout
```

What `ail check` prints (check.out.1 to 3, check.exit.1, [ADR 1113](decisions/1113-check-output-for-agents.md)):

| Situation | Output | Exit |
|---|---|---|
| No error | `ok PATH: N functions, M warnings` on stdout | 0 |
| An error | `PATH:LINE:COL: error[RULE]: MESSAGE`, a `help:` line, a `fix: write CODE` line with the corrected line when there is a fix, and always a last `next:` line | 2 |
| Warning only | on stderr, no `next:` line; the `ok` line still prints | 0 |
| `--json` | one object per diagnostic on stderr, then `{"api":1,"summary":true,"errors":N,"warnings":M,"fixable":K}` on stdout | as above |

- Silence is not success. A clean `ail check` always prints the `ok` line; no output
  means the command did not run.
- `next:` tells you what to run: `next: ail fix FILE` when a fix is settled, else
  `next: ail explain RULE`.
- `--json` records follow [check-json.schema.json](design/check-json.schema.json): `rule`, `file`, `line`,
  `col`, `end_line`, `end_col` (exclusive), `message`, and when present `help`, `notes`, `fixes`
  (each a `title` and `edits` with `line`, `col`, `end_line`, `end_col`, `text`) and `next`.
  Apply a fix's edits from the last to the first so the positions stay valid, or run `ail fix`.
- A file with no tests: `ail test` prints `no tests found in FILE`, then `0 passed, 0 failed`, and
  exits 0. Do not read that as a failure, and do not read it as proof.
- Any command takes `--budget N` to print about N tokens, errors first; `--from K` continues.

Fix the first error in the list first: later ones can be its echo.

## 2. Look it up, never guess

| You need | Run |
|---|---|
| A std function or built-in method | `ail ask std "read a file by lines"`, or `ail ask std --sig "str -> int?"` |
| The usual way to do a task | `ail ask how "sort descending"` (every answer in [how-to.md](how-to.md) is checked) |
| One topic of the card | `ail card errors`, `ail card std.fs` |
| Names and signatures in a program | `ail api --text FILE.ail` (declarations and `##` docs, no bodies) |
| Who calls, writes or reads | `ail ask FILE what calls FN`, `ail ask FILE who writes field F` |
| The code around one function | `ail context FILE FN --budget 800` |
| What a rule means | `ail explain RULE` (text, ADRs, an example) |
| The rules of one area | `ail explain std.io` (a family, one line each), `ail explain exit` (rules naming a word) |

Never grep for a callee or an API; the answers above are exact and fit a budget.

## 3. Canonical patterns

One way each. These are what `ail fmt`, the card and the how-tos use.

| Task | Write | Not |
|---|---|---|
| Function | `fn add(a int, b int) int` then an indented body; the last expression is the result | `fn add(a: int, b: int) -> int`, `return` at the end |
| Bind | `x := 1`; `mut n := 0` then `n += 1` | `let`, `var`, `x++` |
| Condition | `if a < b and not c` / `elif` / `else` | `&&`, `\|\|`, `!c`, `else if` |
| Loop | `for i in 0..n`, `for x in xs`, `for i, x in xs.enumerate()`, `while c` | C-style `for`, `range(n)` |
| Text | `"total: {a + b}"`; a literal brace is `\{` | `f"..."`, `+` on strings, `{{` |
| Build a string | push parts to a `List[str]`, `parts.join(", ")` | `s = s + t` |
| Parse a number | `n := text.to_int() else 0` (`to_int()` returns an `int?`) | using it as an `int` |
| Missing value | `T?`, `none`, `x else fallback`, `if v := f()` | `null`, `nil` |
| Failure | `fn f() int ! MyError`, `error enum MyError`, `x := f()? "context"` | `try`/`catch`, `raise`, exceptions |
| Inferred failure | `fn f() !int`, `fn main() !` | |
| Usage error and exit | `fn usage() never` ending in `io.exit(2)`; `n := s.to_int() else usage()` | `sys.exit`, `return` after `io.exit` |
| Several arms on one line | `if a then x elif b then y else z` | `a ? x : y`, `x if a else y` |
| Method | `fn Point.sum(self) int`, called `p.sum()` | `impl` blocks for plain methods, calls without `()` |
| Construct | `Point(x: 1, y: 2)`: fields by name, any order | `new Point(1, 2)` |
| Branch on a variant | `match s` covering every variant, `_` for the rest | `if` chains on a tag |
| Sort by key | `xs.sorted(by: \|x\| -x)` | comparator functions |
| Tasks | `t := spawn f(1)`, then `t.wait()?` | `async`, `await` |
| Use all cores | `[f(x) par for x in xs]`, `par for i in 0..n` | threads by hand |
| Test | `@test fn name()` with `assert_eq(a, b)` | a separate runner |

A complete program that uses the failure and absence patterns together:

```ail
fn find(names List[str], wanted str) int?
    for i in 0..names.len()
        if names[i] == wanted
            return i
    none

error enum ParseError
    Empty "empty input"
    BadNumber(text str) "bad number {text}"

fn parse(s str) int ! ParseError
    if s == ""
        return Empty
    n := s.to_int() else return BadNumber(text: s)
    n

fn total(items List[str]) !int
    mut sum := 0
    for s in items
        sum += parse(s)? "item {s}"
    sum

fn main() !
    i := find(["a", "b"], "b") else -1
    t := total(["1", "x"]) else err
        print("bad input: {err}")
        0
    print("{i} {t}")
```

## 4. Rules that are not obvious

- **A result is never a plain value.** A `T?` or `T ! E` must be handled with `?`, `else`, or by
  returning it (err.use.1). A function that cannot fail cannot call one that can without `else`.
- **`int` and `float` never mix** (trait.op.3). Convert: `n.to_float()`. There is no implicit conversion
  anywhere, and no `+` on strings or lists.
- **Immutable by default.** `x = 5` on a `:=` name is an error (expr.bind.3); declare it `mut`. A call
  that changes an argument marks it at the call site: `mut` in the callee's parameter, and the caller
  writes `mut` as well. Shared mutation goes only through `ref T`.
- **Calls always have `()`**, including methods: `xs.len()`, `p.sum()`.
- **Standard modules need no `use`**: `fs.read_text(p)`, `env.args()`, `csv.parse(text)`,
  `bigint.from_int(n)` (std.use.4). `use std.NAME` stays valid. A local with the same name hides a module.
- **A statement ends at the line end**, unless the line ends inside `(`, `[`, `{`, or after a binary
  operator or comma. Blocks are exactly four spaces; tabs are an error.
- **Wait every task** exactly once on every path (conc.task.5); `spawn` returns a handle that must be
  waited or added to a `TaskGroup`.
- **`match` covers every variant** or ends with `_`. A fieldless variant inside a nested pattern is
  written qualified (`Expr.Nil`), because a bare name there binds.
- **Keywords as names.** Words such as `error`, `any`, `match`, `todo` may be field,
  label and method names (lex.kw.3, [ADR 1085](decisions/1085-keywords-as-member-names.md)) but not
  variables or functions: use `err`, `found`, `pending`.
- **Shadowing.** A local can't have the name of a type or constant (expr.bind.5). It may take the
  name of a function, an imported module or a field of `self`, and hides it to the end of its block;
  its initializer still sees the original (`qty := qty(s)`, trait.method.7). `Task` is a
  built-in type: do not name a struct `Task`.

## 5. The mistakes that cost the most

Each row is a diagnostic you will see, the rule behind it, and the fix. `ail explain RULE` has more.

| You wrote | Diagnostic | Fix |
|---|---|---|
| `prnt("hi")`, an unknown name | `expr.name.1` | Check spelling with `ail api --text FILE`; `ail fix` offers the nearest name |
| `def`, `class`, `import`, `try`, `&&`, `x++`, `a ? b : c`, `f"..."`, `//` | `gram.foreign.1` | The message names the AIL spelling: `fn`, `struct`, `use`, `?` or `else` for `try`, `and`/`or`, `+= 1`, `if/else`, `"{x}"`, `#` |
| `read_text(p)` | `std.use.2` | Qualify it: `fs.read_text(p)`; find it with `ail ask std "words"` |
| `n + 1.5` with `n int` | `trait.op.3` | `n.to_float() + 1.5` |
| `fn f(name: str) -> int` | `gram.fn.2` | `fn f(name str) int` |
| `t := spawn f()` never waited | `conc.task.5` | `t.wait()?` on every path |
| `Rect(3, 4)` when two fields share a type | `type.struct.7` | `Rect(w: 3, h: 4)` |
| `f(1)` for `fn f(a int, b int)` | `expr.call.1`, `expr.call.2` | One argument per parameter, in order; a defaulted parameter (`retries int = 3`) is passed by name only |
| `x = 5` where `x := 1` | `expr.bind.3` | `mut x := 1`, or a new name |
| `port := text.to_int()` used as an `int` | `err.use.1` | `text.to_int() else 80` or `text.to_int()? "port"` |
| `match` missing a variant | `expr.match.3` (a warning that names the variants to add) | Add the arms, or end with `_` |
| `"a" + b` | `trait.op.3` | `"a{b}"` |
| A bare `{` in a string | an interpolation error | `\{`; a raw string is `r"..."` |

Most of these carry a fix, so `ail check --fix FILE` repairs and re-checks in one call. When two fixes are
plausible, `ail try FILE` applies each to a copy and reports which one passes check and tests.

## 6. Tests

```text
@test fn doubles()
    assert_eq(double(4), 8)           # traps, showing both values
    assert_err(fs.read_text("no.txt"))  # traps unless the call fails
```

- Run one with `ail test FILE --filter NAME`; list them with `--list`.
- A `@test` with parameters is a property: `ail test` generates inputs (`--cases N`, `--seed S`) and
  shrinks a failure.
- `?` works inside a test. A test that returns an error fails.
- Failure text: `FAIL NAME: MESSAGE at LINE:COL`; `ail test` exits 1. A file with no tests exits 0.

## 7. Working in a project

`ail new NAME` writes an `AGENTS.md` and a `.mcp.json` for the project (`ail new .` does it in an existing
one). Each tool pays for itself in one situation; use it there, not as a ritual:

| Situation | Tool |
|---|---|
| A diagnostic `error[RULE]` | `ail check --fix FILE`, then `ail explain RULE` |
| Editing a function you did not write | `ail context FILE FN --budget 800` |
| Changing a signature or a type | `ail ask FILE what calls FN` (never grep) |
| Edits by declaration name, then fmt, check, test | `ail edit FILE --test` |
| Two candidate fixes | `ail try FILE` |
| A change across several functions, in a program with `@test` functions | `ail task start FILE "goal"`, `ail task verify`, `ail task finish` (writes `.ail/cert.json`) |

`ail task verify` runs `ail check` and the `@test` functions, nothing else. A project with its own build
or test command still needs that command run. A small one-file program with an outside test script gains
nothing from the task loop: build, run the script, read the diagnostics.

Read the verify report by its categories: *checked* is only the static check (not a proof), *contracts* are what `ail prove` discharged, *tested* has a state (`passed`, `failed`, `none-ran`, `could-not-run`, `not-tested`), and *unverified* lists what no result speaks for. The certificate records the environment and inputs, so `ail verify --check` differs when any of them changed ([ADR 1116](decisions/1116-verify-evidence-categories.md)).

The same tools are MCP calls (`ail mcp`, [mcp.tools.1](spec/22-tools.md)): `ail_check`, `ail_test`,
`ail_ask`, `ail_fix`, `ail_explain`, `ail_card`, `ail_find` and more.

## 8. Do not

- Do not invent std names; `ail ask std` finds the real one, and an unknown name is an error with a
  nearest-name fix.
- Do not trust a program because it compiles: the card's "Compiles, but is not what you meant" block
  lists the forms no diagnostic catches.
