# 22. Tool lookups: api, explain, fix

Chapter keys: `api`, `explain`, `fix`

> **Status:** Drafted (partial). These are the rules that diagnostics and other
> chapters cite. The output formats of `ail api` are non-normative and live in
> [semantic-api.md](../design/semantic-api.md); the examples of `ail fix` live in
> [ADR 0203](../decisions/0203-ail-fix-command.md) and
> [ADR 0204](../decisions/0204-anchored-fix-edits.md).

## Semantic API

**api.id.1** — An id names a declaration by its path, never by its position,
so it survives edits elsewhere in the file (`fn:add`, `type:Point`,
`method:Point.shift`; [ADR 0158](../decisions/0024-semantic-api-from-stage-2.md)).

**api.doc.1** — A doc comment is a run of `##` lines with nothing between it
and the declaration's line. A blank line or a plain `#` comment ends a run and
leaves it unattached.

**api.pkg.2** — `ail api --locals|--type|--refs DIR FILE LINE COL` asks about
`FILE` in the context of the whole package `DIR`. A file the package does not
load is an error, reported under this rule with exit code 2
([ADR 0325](../decisions/0325-ail-api-on-package-directories.md)).

**api.tok.1** — `ail api --tokens FILE` lists the tokens the lexer makes of
`FILE`, one per line, then its comments, then its lexical diagnostics, in
the order found. It is the comparison target of the lexer written in AIL
([ADR 0371](../decisions/0371-self-hosted-front-end-compared-by-dumps.md)).

**api.tok.2** — A token's line is `LINE:COL KIND`, with a value after a
space for `id`, `kw`, `sym` (the text), `str`, `char` (a JSON string),
`int` and `float` (as AIL prints the number); the other kinds are `intmin`,
`nl`, `indent`, `dedent` and `eof`. An interpolated string is `interp`,
then for each part `lit TEXT`, or `code` or `fmt SPEC` followed by that
part's tokens through their `eof`, then `end`.

**api.tok.3** — A comment is `LINE:COL comment TEXT` and a diagnostic
`LINE:COL diag RULE`. Strings are quoted as `Json.to_text` quotes them.

**api.ast.1** — `ail api --ast FILE` prints the parse of `FILE` alone (no
prelude, imports or later passes) as one line of JSON, then one
`LINE:COL diag RULE` line per lexical and syntax diagnostic, in the order
found ([ADR 0371](../decisions/0371-self-hosted-front-end-compared-by-dumps.md)).

**api.ast.2** — The JSON is what the derived `ToJson` of `compiler/ast/`
writes for an `ast.Program`, so `Program.from_json` reads it back
([ADR 0376](../decisions/0376-syntax-tree-in-ail-and-its-json.md)).

**api.ast.3** — `ail api --ast --program FILE|DIR` prints the program the
checker sees: the prelude (lines from 2^30), every imported module (lines
from each file's `base`), the derive output and the standard declarations.

## Fixes

**fix.edit.1** — A fix is a titled list of edits to the source, each replacing
a span with text; the edits of one fix apply together or not at all. A fix is
offered only when the result is the intended repair; where the line is not a
plain case, the diagnostic carries none (ADR 0145).

**fix.edit.6** — Some fixes read the line they repair: the operands of an
operator (`"a" + b`, `"ab" * 3`, `"n=%d" % n`, `1 in xs`), a C-style `for`
header, and `xs: List[int] = []`. Each applies only when the operands are
plain names, literals, calls or index expressions with nothing binding
tighter on either side, and the operator's result is the whole operand of the
enclosing expression. Otherwise the diagnostic has no fix
([ADR 0335](../decisions/0335-explain-and-fix-coverage.md)).

## Explain

**explain.show.1** — `ail explain RULE` prints the rule's text from
`docs/spec` and, when a chapter has a conformance example tagged
`error=RULE`, that example and the plain block that directly follows it
([ADR 0335](../decisions/0335-explain-and-fix-coverage.md)).

**explain.list.1** — `ail explain` with no rule lists every rule of
`docs/spec`: its ID, chapter key, file and whether `ail fix` repairs
diagnostics of that rule. With `--json` it is one object with `count`,
`fixable` and `rules`.

**explain.cover.1** — Every rule ID a diagnostic can carry is defined in
`docs/spec`, so `ail explain` has an entry for it. The test suite fails when
the sources name a rule that no chapter defines.

**explain.cover.2** — A rule ID cited in `docs/spec` is defined there. The
test suite fails when a chapter cites an undefined rule.

**explain.cover.3** — The list of rules with a fix (`explain.list.1`) equals
the set of rules whose diagnostics carry a fix in the test programs. The test
suite fails when they differ.

**explain.cover.4** — An entry is short enough to read in one step: the mean
`ail explain RULE` output stays under 120 tokens
([ADR 0336](../decisions/0336-explain-listing-and-length.md)).

## Examples

Conformance examples (ADR 0002). Each `tool=` block runs the named tool on the
example (`FILE` is the example, `DIR` its package directory) and checks that
its output contains the `expect=` text. The tools' output formats are
non-normative; these examples check the behaviour the rules describe.
`explain.cover.*` is also enforced by `bootstrap/tests/explain_coverage.rs`
in `./ail selftest`.

```ail tool=api,--text,FILE expect=add rules=api.id.1
fn add(a int, b int) int
    a + b

fn main()
    print(add(1, 2))
```

```ail tool=api,--text,FILE expect=Adds rules=api.doc.1
## Adds two numbers.
fn add(a int, b int) int
    a + b

# A plain comment ends the doc run above; it is not a doc comment.

fn main()
    print(add(1, 2))
```

```ail tool=api,--locals,DIR,FILE,2,5 expect=a rules=api.pkg.2
fn add(a int, b int) int
    a + b

fn main()
    print(add(1, 2))
```

```ail tool=api,--tokens,FILE expect=1:1 rules=api.tok.2
fn main()
    print(1)
```

```ail tool=api,--tokens,FILE expect=main rules=api.tok.1
fn main()
    print(1)
```

```ail tool=api,--tokens,FILE expect=comment rules=api.tok.3
# a comment line
fn main()
    print(1)
```

```ail tool=api,--ast,FILE expect="fns" rules=api.ast.1,api.ast.2
fn main()
    print(1)
```

```ail tool=api,--ast,--program,FILE expect=1073741843 rules=api.ast.3
fn main()
    print(1)
```

```ail tool=fix,--dry-run,FILE expect=mut rules=fix.edit.1
fn main()
    x := 1
    x = 2
    print(x)
```

```ail tool=fix,--dry-run,FILE expect={name} rules=fix.edit.6
fn main()
    name := "bo"
    print("hi " + name)
```

```ail tool=explain,expr.bind.4 expect=expr.bind.4 rules=explain.show.1
fn main()
    x := 1
    print(x)
```

```ail tool=explain,--json expect="count" rules=explain.list.1
fn main()
    print(1)
```

```ail tool=explain,--json expect="fixable" rules=explain.cover.1,explain.cover.2,explain.cover.3,explain.cover.4
fn main()
    print(1)
```
