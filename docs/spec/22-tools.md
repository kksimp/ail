# 22. Tools: api, explain, fix, budgets, diff, edit, replay, snapshots

Chapter keys: `api`, `explain`, `fix`, `budget`
Chapter keys: `api`, `explain`, `fix`, `snap`
Chapter keys: `api`, `explain`, `fix`, `budget`, `diff`

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
`FILE` in the context of the whole package `DIR`. `FILE` is a root file of
`DIR` or a file of a module the root imports. A file the package does not
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

**api.out.1** — The output is one JSON object with the keys `api` (the
shape's version, `1`), `file` (the path as given) and `items`, then
`references` with `--symbols`.

**api.out.2** — By default the object is written on one line with no
spaces outside strings. `--pretty` writes it with a two-space indent and a
space after each `:`. Either way every object's keys are in the order this
section lists them, an empty array is `[]`, and a newline ends the output.

**api.out.3** — `items` holds every declaration of the file, sorted by
position. Each item has `kind`, `name`, `id`, `line`, `col`, `doc` (the doc
comment, or `null`) and `signature` (the declaration's first line as
written, without its body; a constant's value only when it is a literal,
else `...`). Then, by kind:

**api.out.4** — `effects` lists the effects the function performs, sorted, from the inferred effect set (`eff.infer.*`, [ADR 0163](../decisions/0163-inferred-effect-sets.md)): `fail`, `io`, `mutates`, `spawn`, `unsafe` and the capabilities `fs`, `env`, `proc`, `clock`, `net`. `ail effects FILE` prints the same sets on their own.

**api.out.5** — A program with errors has no outline: `ail api` reports the
diagnostics as `ail check` would (with `--json`, as JSON lines) and exits 2.
A file without `main` (mod.main.1) is not an error here, so a library
file has an outline. Warnings are not reported and don't stop the outline.

**api.id.2** — When a method of an `impl` block would get the same id as
another method of the type, its id also names the trait:
`method:Circle.Shape.area`.

**api.id.3** — An id may name a standard declaration that isn't in the file,
such as `trait:Ord`.

**api.ref.1** — With `--symbols`, `references` lists every use of a declared
name as `{id, line, col}`, sorted by position then id: calls, method calls,
field reads, struct-literal and variant labels, variants (qualified, bare
and in patterns), constants, and type and trait mentions in signatures,
`impl` headers, bounds and type arguments. A local variable, a declaration
itself and a use inside the standard prelude are not references.

**api.ref.2** — A method call that is dispatched through a trait refers to
the trait's method (`method:Shape.area`), not to an implementation: which
implementation runs is known only from the value.

**api.doc.2** — The doc is the lines' text after `##` and one optional
space, joined with newlines. Structs, fields, enums, variants, traits,
functions, methods and constants take docs; an `impl` block doesn't.

**api.text.1** — `--text` prints one declaration per line: its line number
right-aligned in five columns, two spaces, four spaces per nesting level,
and its signature. Each doc line follows, indented one level deeper as
`## text`. A variant's fields are not repeated under it, since its
signature shows them.

**api.card.1** — The card of a package directory or a file has, in this
order: its name; `entry:` the `main` of the root module and its file;
`tests:` the `@test` functions' names, sorted; the lines of `ail.pkg`;
`examples:` the `.ail` files under `DIR/examples`, each with the first line
of its first `##` comment; then one block per module, the root module
first, then by import path, listing the module's declarations in source
order as `ail api --text` does (signatures, with `pub` where written),
each function's effects in `[...]`, its `requires`/`ensures` and, for an
inferred error set, `fails with`, and the first line of its doc after `##`.
`@test` functions are not in the module blocks.

**api.card.2** — With `--json` (or the global `--json`) the same content is
one JSON line: `api`, `card`, `entry`, `tests`, `test_count`, `manifest`,
`examples`, `modules` (each `module`, `count`, `items` with `kind`, `name`,
`pub`, `file`, `line`, `col`, `signature`, `doc`, `effects`, `requires`,
`ensures`, `fails_with`, `members`), `budget` and `dropped`. `file`, `line`
and `col` are where the declaration's name is, the file as the program
loaded it ([ADR 0338](../decisions/0338-card-json-positions.md)); the text
form doesn't show them.

**api.card.3** — The card fits `--budget` estimated tokens (default 4,000).
If the full card is over, it drops in this order until it fits: private
items; doc comments; parameters' names; fields, variants, contracts, error
sets and test names; signatures (kind and name only, then module names
with counts). A trimmed card's last line is `trimmed to fit N tokens (~M);
dropped: LIST`.

**api.card.4** — A program the checker rejects has no card: the
diagnostics are printed and the exit code is 2, as for api.out.5. A missing
path or a bad option is exit code 2.

**api.card.5** — `docs/design/cards/edit.md` and `std.md` are the output for
`tools/edit` and `bootstrap/src/std`; `cargo test --test api_card` fails
when either differs from the command's output. Regenerate with
`./ail api --card tools/edit > docs/design/cards/edit.md`.

**api.query.1** — A position with nothing to answer (`--type` or `--refs` off
a name, or a name with no recorded type or symbol) exits 1 and prints on
stderr one object `{"api":1,"severity":"error","message":...}`. A missing or
non-numeric `LINE COL` is a usage error, exit 2.

**api.loc.1** — `--locals` adds `locals`: every name in scope at the
position, as `{name, type, mut, kind, line, col}`. `kind` is `self`, `param`,
`loop` or `binding`; `line` and `col` are where the name is written (`null`
for `self`). The order is `self`, then by where each name is written. `type`
is the checker's, as in a diagnostic, or `"?"` where it is not known.

**api.loc.2** — A `:=` binding is in scope from the line after it. A
parameter, loop variable or pattern name is in scope from the first
statement of the block it scopes. A name stays in scope until a line indented
less than the line it started on; blank and comment lines never end a scope.
On an empty line the cursor's column gives the indent.

**api.type.1** — `--type` finds the name or number under (or just before) the
cursor and adds `text`, `start_col`, `end_col` (exclusive), `type` (the
innermost expression that starts there, or the type of a binding site) and
`symbol` (the id of the declaration it refers to, as `--symbols`, or `null`).
A qualified use of a module item (`geo.origin`) is one token at `geo`.

**api.type.2** — `callee` is the declaration's outline item when `symbol` is
a function or method (`signature`, `params`, `returns`, `effects`,
`requires`, `ensures`, `doc`; api.out.3), else `null`.

**api.refs.1** — `--refs` adds `symbol` and `refs`, a list of
`{file, line, col, def}` sorted by file order, line and column. `symbol` is
the id of the declaration the cursor is on or in a use of (api.ref.1) or
`local:NAME`. The definition has `def:true`; `file` is the source file of the
module the position is in, so a reference in an imported user module names
that module's file.

**api.refs.2** — A use of a module item is reported at the start of its
qualified name (`geo.origin` at the `g`). A method call through a trait refers to the trait's
method (api.ref.2).

**api.refs.3** — For a local, `refs` is its binding (`def:true`) and every
use of the name within its scope (api.loc.2). Locals can't be shadowed
(expr.bind.4), so the name identifies the binding. Only locals of the file
asked about are supported.

**api.pkg.1** — `ail api [--outline] [--symbols] [--text|--pretty] DIR` outlines
a whole package: the root module's files and every module they import
([ADR 0325](../decisions/0325-ail-api-on-package-directories.md)). The object
has `api`, `package` (the path as given), `items` and, with `--symbols`,
`references`. Items are ordered by file (root files first, then modules in
load order) and line. Each item has the single-file keys, with its `id` prefixed
by its module (`fn:geo.origin`, `type:geo.P`, `method:geo.P.shift`; api.id.1),
and ends with `module` (the import path, empty for the root) and `file`.
`line` counts from the top of the item's file. A reference has `id`, `line`,
`col` and `file`. `--text` prints a `# FILE` line before each file's items.

**api.pkg.3** — The cursor may be on the name after a module qualifier
(`geo.origin` at `origin`) as well as on the qualifier (api.refs.2); both ask
about the same symbol. This holds for `--refs` with or without `DIR`.

**api.pkg.4** — The single-file forms and their JSON are unchanged (api.out.1,
api.refs.1). `refs` always names the file each position is in.

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

**fix.edit.2** — A fix edit whose position the checker can't name by its
coordinates is anchored by rule and resolved once the diagnostics are settled:
either after the block below a position (whole lines, used to add match arms)
or at the end of a position's line. A fix whose anchor doesn't resolve, or
whose line isn't a plain case, is dropped rather than guessed
([ADR 0204](../decisions/0204-anchored-fix-edits.md),
[ADR 0335](../decisions/0335-explain-and-fix-coverage.md)).

**fix.edit.3** — For a fallible call used as a statement or as a `:=` value,
in a function whose error type already admits the call's errors (`err.use.1`),
the fix adds `?` at the end of the line
([ADR 0202](../decisions/0202-more-machine-applicable-fixes.md)).

**fix.edit.4** — For a `match` that misses enum variants, `true`/`false`, or
has no `_` arm (`expr.match.2`), the fix adds one arm per missing variant after
the last arm, each `=> todo`, so each arm is a typed hole
([ADR 0202](../decisions/0202-more-machine-applicable-fixes.md),
[ADR 0147](../decisions/0147-typed-holes-todo.md)).

**fix.apply.1** — `ail fix FILE` checks the file and takes the first fix of
every diagnostic that has one. It skips a fix whose edits touch the range of an
earlier-chosen fix, and applies the rest from last to first
([ADR 0203](../decisions/0203-ail-fix-command.md)).

**fix.apply.2** — `ail fix` checks the result again and repeats, up to 5
rounds, until no fixable diagnostic remains ([ADR 0203](../decisions/0203-ail-fix-command.md)).

**fix.apply.3** — If the file was already in canonical layout
([ADR 0169](../decisions/0169-canonical-formatting-style.md)) before `ail fix`
ran, the result is formatted too ([ADR 0203](../decisions/0203-ail-fix-command.md)).

**fix.apply.4** — `ail fix` writes the file. With `--dry-run` it prints a
unified diff and leaves the file unchanged. It prints each applied fix; with
`--json`, one `{"api":1,"applied":...}` object each, then a summary
([ADR 0203](../decisions/0203-ail-fix-command.md)).

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

## Token budgets

**budget.out.1** — Every command accepts `--budget N` before its file
(before a program's own arguments for `ail run`). The command runs as
usual; when its output, standard output and standard error together, is
estimated as for program cards (ADR 0311) at no more than `N` tokens, it is printed
unchanged. Otherwise records are printed by `budget.out.3` until the next
would pass `N`, then a note (`budget.out.4`). At least one record is shown.
The exit status is the command's own. `api --card` keeps its own
`--budget` (ADR 0311)
([ADR 0394](../decisions/0394-token-budgets-on-every-command.md)).

**budget.out.2** — A record is one diagnostic, trap or failing test with its
indented, `help:`, `note:` and `fix:` lines; one JSON line; or one other
line. A record stays on the stream it was written to.

**budget.out.3** — Records are shown in rank order, and in output order
within a rank: errors and traps, failed tests, warnings, other lines, passed
tests. A test summary (`N passed, M failed` or its JSON line) is always
shown, after the records.

**budget.out.4** — The note counts the records not shown by kind and gives
the number `K` of the first one not shown; `--from K` with the same command
and budget skips the first `K` records in rank order. With `--json` the note
is one line `{"budget","from","shown","total","tokens","omitted","next"}`,
`next` being `null` when nothing is left. The note is written to the stream
of the first record not shown.

**budget.out.5** — `--budget` without a number takes the command's default:
1,500 tokens for `check`, `fix`, `edit` and `test`; 2,000 for `run`,
`explain`, `effects` and `diff`; 4,000 otherwise. `--from` without
`--budget` uses the default too.

**budget.out.6** — A record is never cut, so a command whose whole output is
one line (`ail api FILE`, `ail explain --json`) is shown whole; its text forms
(`--text`) split into records.
## Snapshot updating
**snap.update.1** — `ail test --update FILE` runs the tests of `FILE` as
`ail test` does, and rewrites the expectations that failed so that they
hold (`snap.update.2–5`). It prints a unified diff of each file it changed,
then `updated N snapshots in M files`, then the test report as the tests
now stand; the exit status follows that report
([ADR 0395](../decisions/0395-ail-test-update-rewrites-snapshots.md)).
**snap.update.2** — An `assert_eq(actual, expected)` whose `expected` is a
literal (a string, character, number, `-` number or boolean, or a list
literal of these) does not fail under `--update` when `actual` has a
literal form of that shape: the literal is rewritten to it and the test
goes on. String literals of any form are rewritten as one-line `"..."`
literals with escapes (`lex.str.1`). Any other `assert_eq` and every
`assert` fail as usual.
**snap.update.3** — A literal reached more than once with different actual
values (in a loop, a helper or a property) is not rewritten, and the
assertion that found the second value fails as usual.
**snap.update.4** — A doc example (`test.doc.2`) whose output differs from
its `# =>` lines has those lines replaced by one `# =>` line per line of
output, with the first one's indentation. An example without `# =>` lines
is not changed.
**snap.update.5** — When `NAME.out` exists next to `FILE`, it is replaced by
what `ail run FILE` writes to standard output.
**snap.update.6** — `--dry-run` prints the same report and writes nothing
(`would update ...`). With `--json` each change is a line
`{"snapshot":"assert_eq"|"doc"|"out","file","line","col","old","new"}` and
the summary is `{"snapshots":true,"dry_run","updated","files","skipped"}`.

## Semantic diff

**diff.out.1** — `ail diff OLD NEW` compares two versions of a program: two
files, or two package directories. Each is loaded and checked as
`ail check` would; for files, the tests of each are run as `ail test` would
(a 10-second limit per test run), unless `--no-tests` is given. Packages'
tests are not run
([ADR 0396](../decisions/0396-ail-diff-semantic-diff.md)).

**diff.out.2** — Declarations are matched by id (`api.id.1`). An id only in
`NEW` is added, only in `OLD` removed. A matched declaration is changed when
any of these differ: its signature, whether it is `pub`, its effects, its
error types, its `requires`/`ensures`, the trait method it implements,
whether a trait method has a default, its doc comment, or, for functions and
methods, the text of its body (the lines after its first, to the next
declaration, without trailing blank and comment lines). Members (fields,
variants, methods) are declarations of their own. When either side is
rejected, declarations are not compared.

**diff.out.3** — Diagnostics are matched by rule, file (relative to the
compared path) and message, counting repeats; line numbers are shown but not
compared.

**diff.out.4** — A test is reported when its outcome differs, or when it
exists on one side only (`absent`).

**diff.out.5** — `ail diff --rev REV PATH` compares `PATH` as git has it at
revision `REV` (with the directory around a file, so its modules load)
against `PATH` now. An unknown revision or path is an error (exit 2).

**diff.out.6** — The text form has one line per added (`+`) or removed
(`-`) declaration, a `~ ID` line per changed one with `(doc, body changed)`
and an indented `KEY: OLD -> NEW` line for each other changed key, one
`+`/`-` line per diagnostic, one `test NAME: OLD -> NEW` line per test, and
a summary line. With `--json` each is a JSON line, a test that now fails
carries `"status":"fail"`, and the last line is
`{"summary":true,...}`. The exit status is 0 when nothing changed, 1 when
something did, 2 for an error.
## Examples
Conformance examples (ADR 0002). Each `tool=` block runs the named tool on the
example (`FILE` is the example, `DIR` its package directory) and checks that
its output contains the `expect=` text. The tools' output formats are
non-normative; these examples check the behaviour the rules describe.
`explain.cover.*` is also enforced by `bootstrap/tests/explain_coverage.rs`
in `./ail selftest`, and `api.card.5` (the generated cards match `ail api --card`)
by `bootstrap/tests/api_card.rs`; the harness cannot read the repository's cards.
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
```ail tool=api,--pretty,FILE expect=items rules=api.out.1,api.out.2,api.out.3,api.out.4
## Adds two numbers.
fn add(a int, b int) int
    a + b
fn log(msg str)
    print(msg)
fn main()
    log("{add(1, 2)}")
```
```ail tool=api,FILE expect=io rules=api.out.4
fn log(msg str)
    print(msg)
fn main()
    log("hi")
```
```ail tool=api,FILE expect=error status=2 rules=api.out.5,api.card.4
fn main()
    x := 1
    x = 2                              # a program with errors has no outline
```
```ail tool=api,FILE expect=method:Circle.Shape.area rules=api.id.2
trait Shape
    fn area(self) float
struct Circle
    r float
fn Circle.area(self) float
    3.0 * self.r * self.r
impl Shape for Circle
    fn area(self) float
        self.r * self.r
fn main()
    print(Shape.area(Circle(r: 1.0)))
```
```ail tool=api,FILE expect=fn:largest rules=api.id.3
fn largest(xs List[int]) int
    xs.max() else 0
fn main()
    print(largest([3, 9, 2]))
```
```ail tool=api,--refs,FILE,12,7 expect=method:Shape.area rules=api.ref.2
trait Shape
    fn area(self) float
struct Square
    side float
impl Shape for Square
    fn area(self) float
        self.side * self.side
fn total(s any Shape) float
    s.area()
fn main()
    sq := Square(side: 2.0)
    print(total(sq))
```
```ail tool=api,--refs,FILE,15,5 expect=refs rules=api.ref.1,api.refs.1,api.refs.3
trait Shape
    fn area(self) float
struct Square
    side float
impl Shape for Square
    fn area(self) float
        self.side * self.side
fn total(s any Shape) float
    s.area()
fn main()
    sq := Square(side: 2.0)
    print(total(sq))
```
```ail tool=api,--locals,FILE,2,5 expect=kind rules=api.loc.1,api.loc.2
fn add(a int, b int) int
    c := a + b
    c
fn main()
    print(add(1, 2))
```
```ail tool=api,--type,FILE,2,5 expect=int rules=api.type.1,api.type.2
fn add(a int, b int) int
    a + b
fn main()
    print(add(1, 2))
```
```ail tool=api,--refs,FILE,1,4 expect=add rules=api.query.1
fn add(a int, b int) int
    a + b
fn main()
    print(add(1, 2))
```
```ail tool=api,--text,FILE expect=add rules=api.text.1,api.doc.2
## Adds two numbers.
## Returns their sum.
fn add(a int, b int) int
    a + b
fn main()
    print(add(1, 2))
```
```ail tool=api,--card,FILE expect=entry rules=api.card.1,api.card.3
## Sums two numbers.
fn add(a int, b int) int
    a + b
fn main()
    print(add(1, 2))
```
```ail tool=api,--card,--json,FILE expect=card rules=api.card.2
fn main()
    print(1)
```
```ail tool=api,--card,--budget,30,FILE expect=entry rules=api.card.3
fn add(a int, b int) int
    a + b
fn main()
    print(add(1, 2))
```
```ail tool=api,--outline,DIR expect=geo pkg=geometry rules=api.pkg.1,api.pkg.4
```
```ail tool=api,--locals,DIR,FILE,2,5 expect=kind rules=api.pkg.2
fn add(a int, b int) int
    a + b
fn main()
    print(add(1, 2))
```
```ail tool=fix,--dry-run,FILE expect=match rules=fix.edit.2,fix.edit.4
enum Dir
    North
    South
fn name(d Dir) str
    match d
        North => "n"
fn main()
    print(name(Dir.North))
```
```ail tool=fix,--dry-run,FILE expect=load()? rules=fix.edit.3
error enum Oops
    Bad "bad"
fn load() int ! Oops
    Bad
fn run() int ! Oops
    load()
    1
fn main() ! Oops
    print(run()?)
```
```ail tool=fix,FILE expect=mut rules=fix.apply.1,fix.apply.2,fix.apply.3
fn main()
    x := 1
    x = 2
    print(x)
```
```ail tool=fix,--dry-run,FILE expect=--- rules=fix.apply.4
fn main()
    x := 1
    x = 2
    print(x)
```
```ail rules=type.opt.9
fn main()
    mut x := none
    x = 5
    print(x)                    # x is int?, fixed by its first assignment
```
```ail rules=type.opt.10
fn main()
    n := 3
    v := if n > 2
        3
    else
        none
    print(v)
```
```ail rules=expr.bit.1
fn main()
    print(6.bit_and(3))         # 2
    print(6.bit_or(3))          # 7
    print(6.bit_xor(3))         # 5
    print(0.bit_not())          # -1
    print(1.shl(4))             # 16
    print(-16.shr(2))           # -4: the sign bit is copied in
```
```ail norun rules=expr.bit.2
fn main()
    print(1.shl(64))            # traps: the shift count must be 0..=63
```
```ail tool=api,--refs,DIR,main.ail,16,12 expect=origin pkg=geometry rules=api.refs.2,api.pkg.2,api.pkg.3
```
```ail tool=api,--card,DIR expect=entry pkg=geometry rules=api.card.5
```
```ail tool=check,--budget,40,FILE expect=shown status=2 rules=budget.out.1,budget.out.2,budget.out.3,budget.out.4
fn main()
    x := 1
    x = 2
    y := 3
    y = 4
    z := 5
    z = 6
    print(x + y + z)
```
```ail tool=run,--budget,FILE expect=1 rules=budget.out.5,budget.out.6
fn main()
    print(1)
```
