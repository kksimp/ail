# 22. Tools: api, explain, fix, budgets, diff, edit, replay, bundles, snapshots, infer

Chapter keys: `api`, `explain`, `fix`, `budget`, `snap`, `diff`, `edit`, `replay`, `bundle`, `verify`, `shrink`, `infer`, `prove`

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
from each file's `base`), the derive output and the standard declarations. Its
`unsafe_allowed` lists the packages whose manifest allows `unsafe` (none
when the program has no manifest), so a checker reading the dump can report
unsafe.block.3 ([ADR 0740](../decisions/0740-self-hosted-checker-parity-gate.md)).

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

**api.card.5** — `docs/design/cards/compiler.md` and `std.md` are the output for
`compiler` and `std`; the `api_card` gate suite fails
when either differs from the command's output. Regenerate with
`./ail api --card compiler > docs/design/cards/compiler.md`.

**api.card.6** — `--module NAME` prints the card of one module of the
package, with the same levels and budget as api.card.3. NAME is a module
name as the index lists it, or `(root)` for the package's own module. The
entry, manifest and examples appear only on the root module's card, and
the tests only those of that module. An unknown module, or `--module` with
`--index`, is exit code 2.

**api.card.7** — `--index` prints one line per module, `NAME  N items`, under
a `# PACKAGE` line; `--index --json` prints `api`, `card` and `modules` (each
`module` and `count`). An agent reads the index, then one module card. The
index is not trimmed.

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

**fix.check.1** — `ail check --fix FILE` applies every safe fix to a fixed point
as `ail fix` does (`fix.apply.1-3`), writes the file, then checks it as `ail check`
does. It prints no per-fix lines; when at least one fix applied it prints one line
first, `fixed N: RULE, RULE xM` (the rules in first-seen order, `xM` when a rule
fixed more than once), on standard output. With `--json` that line is one object
`{"api":1,"fixed":N,"line":"..."}`. The diagnostics that remain and the exit status are
those of the check after the fixes. Plain `ail check` never writes a file
([ADR 1047](../decisions/1047-ail-check-fix.md)).

**fix.check.2** — `ail task verify --fix` runs its check stage as
`ail check --fix PROGRAM`, shows the `fixed` line first and reports it as `fixed`
(`--json`). The MCP tools `ail_check` and `task_verify` take an optional `fix`
argument; `true` adds the flag ([ADR 1047](../decisions/1047-ail-check-fix.md)).

```ail tool=check,--fix,FILE expect=expr.op.1 rules=fix.check.1
fn main()
    a := true
    b := false
    if a && b
        print("both")
```

```ail pkg=task_fix tool=task,verify,--fix status=1 expect=expr.op.1 rules=fix.check.2
```

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
is one line `{"budget","from","shown","full","total","tokens","omitted","next"}`
(`full` counts the records shown in full, `budget.out.7`),
`next` being `null` when nothing is left. The note is written to the stream
of the first record not shown.

**budget.out.5** — `--budget` without a number takes the command's default:
1,500 tokens for `check`, `fix`, `edit` and `test`; 2,000 for `run`,
`explain`, `effects` and `diff`; 4,000 otherwise. `--from` without
`--budget` uses the default too.

**budget.out.6** — A record is never cut, so a command whose whole output is
one line (`ail api FILE`, `ail explain --json`) is shown whole; its text forms
(`--text`) split into records.

**budget.out.7** — Each record has an expanded form, the whole record, and a
one-line form, its first line (for a diagnostic, `FILE:L:C: severity[rule]:
message`). When the output doesn't fit, the first record shown is expanded,
the rest are counted against the budget in their one-line form, and then
records are expanded in rank order while the budget lasts; the expanded
records come first. The note says which records are in one line and gives
`--from K`, which shows record `K + 1` expanded. A JSON line is its own
one-line form ([ADR 0868](../decisions/0868-ai-first-tooling-after-rust-removal.md) item 6, [ADR 0880](../decisions/0880-budgeted-diagnostic-forms.md)).

```ail tool=check,--budget,200,FILE expect=budget status=2 rules=budget.out.7
fn main()
    x := 1
    x = 2
    y := 3
    y = 4
    z := 5
    z = 6
    print(x + y + z)
```

The first error is shown with its source line, help, note and fix; the other
two as their first lines; the note ends
`records 1-3 of 3 shown, 2-3 in one line (~285 tokens in all); --from 1 shows 2 in full`.

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
variants, methods) are declarations of their own. When either side has
syntax or load errors, declarations are not compared (`diff.out.8` for check errors).

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
**diff.out.7** — Under each changed function or method whose body changed,
the text form shows the body's removed (`-`) and added (`+`) lines,
without the body's own indentation and in the order of a longest common
subsequence, at most 12 lines and then `... N more changed lines`. With
`--json` they are the object's `"lines"`
([ADR 0399](../decisions/0399-compact-edits-and-diff-hunks.md)).

```ail pkg=diff_pair tool=diff,old/a.ail,new/a.ail status=1 expect=-assert_eq rules=diff.out.7
```

```ail pkg=diff_pair tool=diff,old/a.ail,new/a.ail status=1 expect=adds: rules=diff.out.1,diff.out.2,diff.out.3,diff.out.4,diff.out.6
```

```ail pkg=diff_pair tool=diff,--json,--no-tests,old/a.ail,new/a.ail status=1 expect="summary":true rules=diff.out.6
```

```ail pkg=diff_pair tool=diff,--rev,nosuch,main.ail status=2 expect=diff.out.5 rules=diff.out.5
```

**diff.out.8** — A side with check errors but no syntax or load errors is still
compared, as far as its declarations resolve: the report starts with `api:
partial, NEW has errors; compared as far as they allow` (JSON: `{"change":"api",
"op":"partial","reason":...}`) and its diagnostics are listed as usual. Tests are
not run on a side with errors.

```ail pkg=diff_partial tool=diff,--no-tests,old/a.ail,new/a.ail status=1 expect=partial, rules=diff.out.8
```

**diff.out.9** — An added function or method that is a removed one under another
name (same kind, same signature once the name is left out, same body text) is a
*rename*: one line `> fn:old -> fn:new (renamed)` (JSON: `"op":"renamed"` with `from`
and `id`), counted as `N renamed` in the summary when there is any, and not as an
addition and a removal. A function whose body also mentions its own name differs
in body, so it stays an addition and a removal.

```ail pkg=diff_rename tool=diff,--no-tests,old/a.ail,new/a.ail status=1 expect=(renamed) rules=diff.out.9
```

**api.diff.1** — `ail api --diff OLD NEW` lists what changed in the public
API between two versions of a file or package directory. `ail api --diff
--rev REV PATH` compares `PATH` as git has it at `REV` against `PATH` now,
as `diff.out.5` does. An unknown revision or path, a missing argument or an
unknown option is exit code 2
([ADR 0729](../decisions/0729-api-diff-command.md)).

**api.diff.2** — Only public declarations are compared, matched by id
(`api.id.1`): those written `pub`, the variants of a public enum with their
fields, the methods of a public trait, and the `impl` blocks of a public
type. A declaration is changed when its signature, effects, error types,
`requires`, `ensures` or trait-method default differs. A body, a doc comment
or a private declaration is not part of the API and never shows.

**api.diff.3** — A change is marked `breaking:` with the reasons when a
caller or implementer can stop compiling or lose a guarantee: a removed
declaration; a changed signature; an effect or error type that is new; a
`requires` that is new or changed; an `ensures` that went away; a trait
method that lost its default; and an added enum variant or required trait
method (a trait method with no default). Other additions are not breaking
([ADR 0730](../decisions/0730-api-diff-breaking-rules.md)).

**api.diff.4** — Each side is checked as `ail check` would, except that a
single file needs no `main`. When either side has errors, nothing is
compared: the error is exit code 2.

**api.diff.5** — The text form has one line per added (`+ ID SIGNATURE`) or
removed (`- ID SIGNATURE`) declaration, a `~ ID` line per changed one with an
indented `KEY: OLD -> NEW` line for each changed key, `  breaking: WHY` at
the end of a breaking line, and a last line
`N added, N removed, N changed; N breaking`. With `--json` each is a JSON
line with `"change":"api"` and the last is `{"summary":true,...}`. The exit
status is 0 when the API is the same, 1 when it changed, 2 for an error. A
`--budget` bounds the output as for the other tools, keeping the `breaking`
lines first ([ADR 0731](../decisions/0731-api-diff-output-and-budget.md)).

```ail pkg=api_diff_pair tool=api,--diff,old/a.ail,new/a.ail status=1 expect=breaking: rules=api.diff.2,api.diff.3,api.diff.5
```

```ail pkg=api_diff_pair tool=api,--diff,old/a.ail,old/a.ail status=0 expect=added, rules=api.diff.2,api.diff.5
```

```ail pkg=api_diff_pair tool=api,--diff,--json,old/a.ail,new/a.ail status=1 expect="summary":true rules=api.diff.3,api.diff.5
```

```ail pkg=api_diff_pair tool=api,--diff,--rev,nosuch,old/a.ail status=2 expect=api.diff.1 rules=api.diff.1
```

```ail pkg=api_diff_pair tool=api,--diff,--budget,60,old/a.ail,new/a.ail status=1 expect=breaking: rules=api.diff.5
```

```ail pkg=api_diff_pair tool=api,--diff,bad/a.ail,new/a.ail status=2 expect=api.diff.4 rules=api.diff.4
```
## Semantic questions

`ail ask PROGRAM QUESTION...` answers a question about a program from the
checker's graph of it, with no grep and no file reading
([ADR 0868](../decisions/0868-ai-first-tooling-after-rust-removal.md) item 1,
[ADR 0881](../decisions/0881-ail-ask.md)).

**api.ask.1** — The question is plain words after the program: `who writes
field total`, `what calls pay`, `what does main call`, `which effects reach
main`, `who uses Cart?`. The verb picks the question (`writes`, `reads`,
`calls`, `callees`, `effects`, `uses`); filler words (`who`, `what`, `which`,
`does`, `field`, `the`, ...) and a trailing `?` say nothing; the last other
word is the subject. A subject names a declaration by its id (`field:Cart.total`),
its qualified name (`Cart.total`) or its bare name (`total`); a bare name that
fits several declarations answers for each. Exit status is 0 when there are
answers, 1 when there are none, 2 for an error. `--json` prints the answers
as JSON.

**api.ask.2** — `writes` lists every place that sets a field, `reads` every
place that only reads it, each with its function, file, line and column. A
write is an assignment, a `mut` argument, or a `mut self` or built-in mutating
method called on a place that reaches the field (`a.b.c = x` writes `c` and
`b`); a field named in a struct literal is an `init`. Anything else that
names the field is a read.

**api.ask.3** — `calls` lists the functions that call the subject, each call
once, and `callees` lists the functions the subject calls, each at its first
call.

**api.ask.4** — `effects` lists the effects of the subject function (`eff.infer.1`),
and for each the chain of calls down to the function that has the effect of
its own: `io: fn:main > fn:pay` says `main` has `io` through `pay`, which has
it itself.

**api.ask.5** — `uses` lists every use of the subject: a type in a signature,
field or expression, a function or method call, a field access, a constant or
variant.

**api.ask.6** — A program that has errors can't be asked about: the errors
are shown and the status is 2, as `api.diff.4`. A subject that names no
declaration of the right kind, or a question with no known verb, is an error
(status 2) that says so.

```ail tool=ask,FILE,who,writes,field,total expect=fn:add rules=api.ask.1,api.ask.2
struct Cart
    total int
fn add(mut c Cart, n int)
    c.total += n
fn main()
    mut c := Cart(total: 0)
    add(mut c, 5)
    print(c.total)
```

```ail tool=ask,FILE,who,reads,field,total? expect=fn:main rules=api.ask.2
struct Cart
    total int
fn main()
    c := Cart(total: 3)
    print(c.total)
```

```ail tool=ask,FILE,what,calls,show expect=fn:pay rules=api.ask.3
fn show(n int) str
    "{n}"
fn pay(n int) str
    show(n)
fn main()
    print(pay(1))
```

```ail tool=ask,FILE,what,does,main,call expect=fn:pay rules=api.ask.3
fn pay(n int) int
    n + 1
fn main()
    print(pay(1))
```

```ail tool=ask,FILE,which,effects,reach,main expect=fn:log rules=api.ask.4
fn log(s str)
    print(s)
fn main()
    log("hi")
```

```ail tool=ask,FILE,who,uses,Cart expect=type:Cart rules=api.ask.5
struct Cart
    total int
fn size(c Cart) int
    c.total
fn main()
    print(size(Cart(total: 1)))
```

```ail tool=ask,FILE,who,writes,field,nosuch status=2 expect=api.ask.1 rules=api.ask.1
struct Cart
    total int
fn main()
    print(Cart(total: 1).total)
```

```ail tool=ask,FILE,who,writes,field,total status=2 expect=api.ask.6 rules=api.ask.6
struct Cart
    total int
fn main()
    print(missing)
```

## Cards and std lookup

`ail card` prints the core card or one topic of the full card, and `ail ask std`
finds a standard-library function by words or by shape, so an agent can look up
what it would otherwise recall from memory
([ADR 1051](../decisions/1051-core-card-and-std-lookup.md)).

**api.find.1** — `ail ask std WORDS...` prints at most ten lines, best first,
each `MODULE.SIGNATURE   (RULE)` with the signature as the spec writes it
(`fs.read_text(path str) str ! IoError   (std.fs.1)`). A built-in method
(`api.find.5`) prints as `TYPE.SIGNATURE   (RULE)`
(`List.sorted(by: |x| key) List[T]   (type.coll.11)`). Exit status is 0 when
there are lines, 1 when there are none, 2 for an error.

**api.find.2** — The words are matched against the standard library's rules in
the spec: a rule `**std.M.N**` whose text starts with a backticked signature
`name(params) ret ! Error` describes one function `M.name`. A function scores 6
for each word that is its name, 3 for each word its name contains (of three
letters or more), 2 for each word that is its module, and 1 for each word (or
its plural or singular) in the rule's text. Filler words (`a`, `the`, `of`,
`to`, `by`, `in`, `from`, `and`, `std`, ...) say nothing. A function is
printed only when it is answered, so one stray word no longer answers: some
word is its name, a `_` piece of its name, or the start of one of at least four
letters (`sort` for `sorted`), or at least two different words hit it in any
way above, or the only word is its module. A word that names a type or a common
noun (`list`, `map`, `str`, `key`, `file`, `text`, ...) never answers by hitting a
name.

**api.find.3** — `ail ask std --sig "(PARAMS) RET"` matches by shape. Parameter
names are optional (`(path str)` and `(str)` are the same), spaces say nothing
and `! Error` is ignored. A function scores 6 when its parameter types equal
the asked ones in order, else 1 for each asked type among its parameters, and 4
when its result type equals the asked one.

**api.find.4** — A query that nothing answers exits 1 and prints
`nothing in std matches WORDS` (`api.find.5`). No words, or an unknown option,
is a usage error (status 2). The index is read from the repository's `docs/spec`
and `docs/design`, found as `ail explain` finds it.

**api.find.5** — `ail ask std WORDS...` also searches the built-in methods of
`str`, `char`, `int`, `float`, `bytes`, `List`, `Map`, `Set`, `Grid`, optionals
and tuples, which `docs/design/builtin-methods.txt` lists one per line
(`SIGNATURE ; RULE ; WORDS`, the rule being the `type.*` rule that defines it).
A method is matched like a function, its type is its module, and its words are
those of its line. On a miss, the line after `nothing in std matches WORDS` is
the answer of the first row of `docs/design/not-in-std.txt` (`WORDS|WORDS ->
ANSWER`) one of whose alternatives has all its words in the query; no row, no
second line. `tools/check-method-table.py` checks that every rule of the table
exists and that every built-in method of the checker is in it.

**api.find.6** — `ail ask how WORDS...` searches `docs/how-to.md`, whose `##`
sections are one task each (`sort descending`, `read a file by lines`): a title,
a `Words:` line and a block of code the docs check keeps compiling. A section is
answered when a word is a non-generic word of its title or two words are among
its title and `Words:` line. The best section is printed, as its title and
`(docs/how-to.md)`, its text and its code, then `also: TITLE; TITLE` for up to
two more. No answer prints `no how-to matches WORDS` and the titles, status 1.
No words lists the titles.

**api.find.7** — The MCP tool `ail_find` takes `words`, `sig` or `how` (words
for `api.find.6`).

**tool.card.1** — `ail card` prints the core card (`docs/language-card-core.md`):
the syntax, the ten rules diagnostics cite most often, and where to look up the
rest. It is at most 2,000 tokens by `tools/tokbench`. `ail card core` prints the
same.

**tool.card.2** — `ail card TOPIC` prints the section of the full card
(`docs/language-card.md`) whose `##` heading contains the topic, ignoring case
(`ail card errors`). `ail card std.MODULE` prints the signatures of that std
module, as `api.find.1` writes them.

**tool.card.3** — A topic that names no section or module prints an error that
lists the topics and exits 1. A missing card file is an error (status 2).

```ail tool=ask,std,read,text,file expect=read_text(path rules=api.find.1,api.find.2
fn main()
    print(1)
```

```ail tool=ask,std,--sig,(str)bytes expect=read_bytes(path rules=api.find.3
fn main()
    print(1)
```

```ail tool=ask,std status=2 expect=usage: rules=api.find.4
fn main()
    print(1)
```

```ail tool=ask,std,zzqqxx status=1 expect=nothing rules=api.find.4,api.find.5
fn main()
    print(1)
```

```ail tool=ask,std,sort,list,by,key expect=List.sorted(by: rules=api.find.5
fn main()
    print(1)
```

```ail tool=ask,std,sqlite,database status=1 expect=nothing rules=api.find.5
fn main()
    print(1)
```

```ail tool=ask,how,sort,descending expect=sorted(by: rules=api.find.6
fn main()
    print(1)
```

```ail tool=card expect=Ten rules=tool.card.1
fn main()
    print(1)
```

```ail tool=card,errors expect=Absence rules=tool.card.2
fn main()
    print(1)
```

```ail tool=card,std.fs expect=read_text(path rules=tool.card.2
fn main()
    print(1)
```

```ail tool=card,nosuchtopic status=1 expect=topics rules=tool.card.3
fn main()
    print(1)
```

## Context for an edit

`ail context PROGRAM SYMBOL` prints the smallest bundle of the program that an
edit to one function needs, cut to a token budget
([ADR 1020](../decisions/1020-ail-context.md), which implements
[ADR 0891](../decisions/0891-ail-context-budgeted-pack.md)). One call replaces
the file reads before an edit. It reads the same graph as `ail ask`.

**api.ctx.1** — `ail context PROGRAM SYMBOL [--budget N] [--json]`. SYMBOL names
one function or method as `api.ask.1` does: by its id (`fn:pay`), its qualified
name (`Cart.add`) or its bare name. A bare name that fits several functions is an
error that lists their ids. N is a positive number of tokens; a bare `--budget`
and no `--budget` both mean 2000.

**api.ctx.2** — The pack has these sections, each of whole items: `signature`
(where the function is declared, its signature, contracts and doc comment), `body`
(its lines as written, comments included), `effects` (each effect with the chain
of calls that carries it, as `api.ask.4`), `callees` (the signatures of the
functions it calls), `callers` (the signatures of the functions that call it, tests
excluded), `types` (the types its signature and body name, with their fields,
at most 12 lines each), `tests` (the signatures of the `@test` functions that reach
it, directly or through callers) and `rules` (the spec rule IDs its comments and
doc cite). Standard modules and generated code are not part of a pack.

**api.ctx.3** — Items are kept in the order the sections are listed in
`api.ctx.2`; within a section, in source order.

**api.ctx.4** — The pack is counted with the same estimate as `ail api --card`.
It keeps the longest prefix of the items, in the order of `api.ctx.3`, that fits
the budget, so what is cut is always the least needed. An item is never cut in
the middle. The signature is shown even when it alone is over the budget. When
anything is cut, the last line says what: `cut: callers 2, tests 3; --budget N
shows all`, where N is the budget that shows everything. The output is the same
for the same program and arguments.

**api.ctx.5** — A rule ID is a word in a `#` comment of the body, or in the doc
comment, spelled as three or more lowercase parts and a number last
(`api.ask.3`, `api.ask.4`). Each is listed once, in the order it appears.

**api.ctx.6** — A program that has errors can't be read for context: the errors
are shown and the status is 2. A SYMBOL that names no function, or fits several,
is an error with status 2. A pack that is printed has status 0.

**api.ctx.7** — With `--json` the pack is one object: `target` (the function id),
`budget`, `needs` (the budget that shows everything), `sections` (the section
names that are shown, each with the array of its items, an item's lines joined
with newlines) and `cut` (each section with items left out, with their count).

```ail tool=context,FILE,pay expect=callees rules=api.ctx.1,api.ctx.2,api.ctx.3
fn show(n int) str
    "{n}"
fn pay(n int) str
    show(n)
fn main()
    print(pay(1))
```

```ail tool=context,FILE,fn:pay,--budget,1 expect=cut: rules=api.ctx.4
fn show(n int) str
    "{n}"
fn pay(n int) str
    show(n)
fn main()
    print(pay(1))
```

```ail tool=context,FILE,pay,--budget,1,--json expect="cut":{"body":1,"callees":1,"callers":1} rules=api.ctx.4,api.ctx.7
fn show(n int) str
    "{n}"
fn pay(n int) str
    show(n)
fn main()
    print(pay(1))
```

```ail tool=context,FILE,log expect=effects rules=api.ctx.2
fn log(s str)
    print(s)
fn main()
    log("hi")
```

```ail tool=context,FILE,total expect=types rules=api.ctx.2
struct Cart
    n int
fn total(c Cart) int
    c.n
fn main()
    print(total(Cart(n: 1)))
```

```ail tool=context,FILE,pay expect=pays_one rules=api.ctx.2
fn pay(n int) int
    n + 1
@test fn pays_one()
    assert_eq(pay(1), 2)
fn main()
    print(pay(1))
```

```ail tool=context,FILE,pay,--json expect="rules":["api.ask.3"] rules=api.ctx.5
fn pay(n int) int
    # api.ask.3: callers are listed once.
    n + 1
fn main()
    print(pay(1))
```

```ail tool=context,FILE,nosuch status=2 expect=api.ctx.6 rules=api.ctx.6
fn show(n int) str
    "{n}"
fn pay(n int) str
    show(n)
fn main()
    print(pay(1))
```

```ail tool=context,FILE,pay,--budget,0 status=2 expect=positive rules=api.ctx.1
fn show(n int) str
    "{n}"
fn pay(n int) str
    show(n)
fn main()
    print(pay(1))
```

```ail tool=context,FILE,pay status=2 expect=api.ctx.6 rules=api.ctx.6
fn pay(n int) int
    n + 1
fn main()
    print(missing)
```

## Review of a change

`ail review BASE..HEAD` summarises a change by meaning, in one budgeted report
([ADR 1024](../decisions/1024-ail-review.md), which builds the Proposed
[ADR 0894](../decisions/0894-effect-changes-in-ail-diff.md)). A reviewer, human
or agent, judges a pull request from a few hundred tokens instead of reading its
diff. It compares two versions the way `ail diff` does and reads them through the
same checker graph as `ail ask`.

**api.rev.1** — `ail review BASE..HEAD [PATH] [--budget N] [--json] [--deny-new]`
compares `PATH` (a program file or a directory with `.ail` files at its top; default `.`) as git has it at revision `BASE` with `PATH` at
`HEAD`, each copied out as `diff.out.5` does. `ail review --files OLD NEW` compares
two files or two package directories instead. An argument that is not `BASE..HEAD`,
an unknown revision or path and an unknown option are errors with status 2.

**api.rev.2** — N is a positive number of tokens; a bare `--budget` and no
`--budget` both mean 500.

**api.rev.3** — The report has these sections, each of whole lines: `effects`,
`api`, `contracts`, `perf` and `untested`; a section with no lines is left out.
Within a section the lines that matter most come first (`NEW CAPABILITY`, then the
rest; `BREAKING`, then the rest). The report keeps the longest prefix of its lines,
in that order, that fits the budget, counted with the estimate of `ail api --card`
(`api.card.3`). A line is never cut in the middle. When anything is cut, the last line
says what: `cut: untested 2; --budget N shows all`, where N is the budget that
shows everything. The output is the same for the same program and arguments.

**api.rev.4** — The first line is always shown: `review RANGE: effects N (C new
capability), api N (B breaking), contracts N, perf N, untested N`, with
`; partial, a version has check errors` added when either version has check errors
(it is then read as far as it resolves, as `api.impact.5` does). A version with
syntax or load errors can't be compared: the errors are shown and the status is 2.

**api.rev.5** — With `--json` the report is one object: `range`, `budget`, `needs`
(the budget that shows everything), `new_capabilities`, `breaking`, `untested` (the
counts), `partial`, `sections` (each shown section with the array of its lines) and
`cut` (each section with lines left out, with their count).

**api.rev.6** — The status is 0 when the report was printed. With `--deny-new` it is 1
instead when `effects` has a `NEW CAPABILITY` line, so a gate without hosted CI
([ADR 0085](../decisions/0085-interim-local-testing-gate.md)) can refuse a change that widens what
a program can do. Status 2 is an error.

**api.rev.7** — `effects` has a line `ID: +net -fs` for each function whose inferred
effect set differs (`eff.infer.*`), callers that inherit the change included, since
their sets differ too. A function that is `main` or `pub` and gains an effect widens
what the program can do: its line starts `NEW CAPABILITY`.

**api.rev.8** — `api` has one line for each public declaration that was added
(`+ ID SIGNATURE`), removed (`- ID`) or changed (`~ ID: KEYS`), as
`api.diff.1-3` find them. A line that can break a caller ends `BREAKING`, followed
for an addition or a removal by the reason.

**api.rev.9** — `contracts` has a line `ID requires: OLD -> NEW` or `ID ensures: OLD
-> NEW` for each function or method whose precondition or postcondition differs or
is new. `(none)` stands for an absent one.

**api.rev.10** — `perf` has a line for each loop whose `ail perf --explain` verdict
(ADR 1021) differs between the versions. It is
empty until that command exists.

**api.rev.11** — `untested` has the id of each changed or new function, other than
`main` and `@test` functions, that no `@test` function reaches by calls or by
naming it (`api.impact.2`).

```ail pkg=review_pair tool=review,--files,old/a.ail,new/a.ail status=0 expect=CAPABILITY rules=api.rev.1,api.rev.4,api.rev.7
```

```ail pkg=review_pair tool=review,--files,old/a.ail,new/a.ail,--deny-new status=1 expect=CAPABILITY rules=api.rev.6
```

```ail pkg=review_pair tool=review,--files,old/a.ail,new/a.ail status=0 expect=BREAKING rules=api.rev.3,api.rev.8
```

```ail pkg=review_pair tool=review,--files,old/a.ail,new/a.ail status=0 expect=requires: rules=api.rev.9
```

```ail pkg=review_pair tool=review,--files,old/a.ail,new/a.ail status=0 expect=fn:helper rules=api.rev.11
```

```ail pkg=review_pair tool=review,--files,old/a.ail,new/a.ail,--budget,1 status=0 expect=cut: rules=api.rev.2,api.rev.3
```

```ail pkg=review_pair tool=review,--files,old/a.ail,new/a.ail,--json status=0 expect="new_capabilities":1 rules=api.rev.5
```

```ail pkg=review_pair tool=review,--files,old/a.ail,old/a.ail status=0 expect=perf rules=api.rev.10
```

```ail pkg=review_pair tool=review,nosuch status=2 expect=BASE..HEAD rules=api.rev.1
```

```ail pkg=review_pair tool=review,--files,old/a.ail,new/a.ail,--budget,0 status=2 expect=positive rules=api.rev.2
```

## Edit impact

What an edit can break, from the same graph as `ail ask`
([ADR 0868](../decisions/0868-ai-first-tooling-after-rust-removal.md) item 3,
[ADR 0882](../decisions/0882-edit-impact.md)).

**api.impact.1** — A declaration is *changed* when it is new or its text
differs from before, ignoring blank lines, comments, doc comments and
attributes. The declarations compared are functions, methods, types, traits
and constants.

**api.impact.2** — A function is *affected* when it is a changed function, or
names a changed declaration (a call, a field or variant of a changed type, a
constant), or calls an affected function. A function that fills in a trait's
method also reaches the callers of that trait method. The result is
conservative: it may name a test the change can't alter, but never misses one
that it can.

**api.impact.3** — `ail ask PROGRAM --affected [--rev REV]` prints the name of
every affected `@test` function, one per line: the changes are those of the
program as written now against the same path at git revision `REV` (default
`HEAD`), as `api.diff.1` reads a revision. `--json` prints the whole impact
(`api.impact.4`) instead. Both versions must check without errors, and an
unknown revision is an error: exit status 2.

**api.impact.4** — The impact is the changed declarations, the affected
functions, the affected tests and the *effects* changes: each function whose
effects (`eff.infer.1`) differ, with the effects before and after. As JSON:
`{"changed":[ids],"affected":[ids],"tests":[names],"effects":[{"id","old":[...],"new":[...]}]}`,
and, only when they apply, `"via":{id:next}` (for each affected function, the declaration it
names or calls on its way to a changed one), `"reaches":{test:[ids]}` (the changed declaration
each affected test reaches) and `"partial":true` (`api.impact.5`). Text output adds one
`via: fn:test > fn:caller > fn:changed` line per affected test.

**api.impact.5** — When the edited version has check errors (not syntax or load
errors) the impact is still printed, read from the graph as far as it resolves, with
`"partial":true` (text: a `partial:` line), and the errors are shown. `ail ask
--affected` and `ail edit --impact` do this for the edited version only: a signature
change that breaks its callers lists those callers as affected, which are the
functions that now need an edit. The version at the revision must still check. The
exit status stays 2 for `ail edit` while it has errors.

**api.impact.6** — A changed type reaches its members by line. When only the text of
existing fields or variants changed, only those members are seeded: a function that
reads another field is not affected, and neither is one that only names the type.
When a field or variant was added or removed, or the type's own line changed, the
type and all its members are seeded, as constructors and matches must change. The
changed list is the same either way. *(ADR 0868 item 3)*

```ail pkg=edit_impact_field tool=edit,--dry-run,--impact,main.ail stdin=edits.json expect=via: rules=edit.out.5,api.impact.6
```

```ail tool=ask,FILE,--affected,--rev,nosuch status=2 expect=api.impact.3 rules=api.impact.3
fn main()
    print(1)
```

## Agent tasks

`ail task` records one change from start to finish
([ADR 0917](../decisions/0917-ail-task-protocol.md)).

**task.state.1** — The open task is the file `.ail-task` of the current directory: `program`, `goal`, `rev` and `last` (`none`, `green` or `red`) as `key=value` lines.

**task.state.2** — A directory has one open task. `start` with one open, and any other subcommand with none, is an error: exit status 2.

**task.start.1** — `ail task start PROGRAM GOAL... [--rev REV]` opens a task whose base is `REV` (default `HEAD`). The program must compare against the base as `api.impact.3` does; if it can't, no task is opened: exit status 2.

**task.verify.1** — `ail task verify` runs `ail check PROGRAM`, then, if that passes, `ail test PROGRAM --affected --since REV`, and records `green` only when both pass.

**task.verify.2** — Exit status is 0 for green and 1 for red.

**task.verify.3** — A red result shows the stage that failed (`check` or `test`) and the last 20 non-blank lines of its output.

**task.verify.4** — The test stage runs with `--bundle-failure` (`bundle.write.1`). A red test stage lists the bundle files it wrote as `bundles` (`--json`), and `next` names `ail test --bundle PATH` for the first.

**task.verify.5** — A green `verify` also runs `ail verify --diff --json --rev REV PROGRAM` (`verify.run.1`) and reports its JSON as `diff`, `null` when it prints none.

**task.status.1** — `ail task status` prints the goal, the last verify result and the impact since the base (`api.impact.4`).

**task.finish.1** — `ail task finish` is refused (exit status 1) unless the last verify was green.

**task.finish.2** — A finished or aborted task removes `.ail-task`.

**task.finish.3** — `ail task finish` also writes the change certificate, `ail verify --diff --rev REV --attest .ail/cert.json PROGRAM` (verify.cert.1), before it closes the task, and names its path (`certificate` with `--json`; `certificate: PATH` in the text answer). A certificate that fails exits 1 with the reason and leaves the task open. The task's own files (`.ail-task` and everything under `.ail/`) are not part of the change the certificate lists. *(ADR 1046)*

**task.out.1** — Every answer names the one command to run next (`next`). `--json` prints one object with the same fields.

**task.log.1** — `ail task verify` appends one line to `.ail/task-log.jsonl`, creating it: the object `{"stage","result","tokens","fixed","rules","ms"}` with `rules` the rule IDs named by the toolchain output (`error[R]`, `warning[R]`, `trap[R]`) and their counts in first-seen order, `tokens` the estimate of `api.card.3` of what `verify` printed, `fixed` the fixes `--fix` applied and `ms` last. Every field but `ms` is the same for the same change. Nothing is sent anywhere ([ADR 1048](../decisions/1048-task-ledger.md)).

**task.log.2** — `.ail-task` records `mark`, the lines the ledger held at `start`. `ail task finish --json` adds `summary`, `{"rounds","red","fixed","tokens","rules"}`, over the lines after `mark`; the text answer ends with the same object.

**task.log.3** — `ail task log [--json]` prints the ledger lines after `mark` (all of them when no task is open), then their summary; `--json` prints `{"log":[...],"summary":{...}}`.

```ail pkg=task_log tool=task,log,--json status=0 expect="rounds":2,"red":1,"fixed":1,"tokens":50,"rules":{"expr.op.1":1,"expr.name.1":1} rules=task.log.1,task.log.2,task.log.3
```

```ail skip=bootstrap tool=task,verify,--json rules=task.state.2,task.start.1,task.verify.1,task.verify.2,task.verify.3,task.verify.4,task.verify.5,task.status.1,task.finish.1,task.finish.2,task.finish.3,task.out.1
fn main()
    print(1)
```

```ail pkg=task_red tool=task,verify status=1 expect=red rules=task.state.1,task.verify.1,task.verify.2,task.verify.3
```

## Merge by declaration

`ail merge` is a three-way merge for `.ail` files that works on declarations, so
two branches that change different functions never conflict
([ADR 1019](../decisions/1019-ail-merge-driver.md)).

**merge.split.1** — A file is cut into declarations. A line at column 0 starts one; the indented lines after it, and blank lines followed by an indented line, belong to it. Comment (`#`) and attribute (`@`) lines at column 0 belong to the declaration after them, unless a blank line separates them. Blank lines after a declaration are kept as its gap.

**merge.split.2** — A declaration's key is its kind and name (`fn Point.sum`, `struct Point`, `const NAME`); `pub`, `unique` and `error` are not part of it. Any other declaration (`use`, `impl`, `test`, a loose comment) is keyed by its first line. A repeated key gets `#2`, `#3`. A rename is a removal and an addition.

**merge.decl.1** — A declaration whose text differs from the base on one side only takes that side's text, or is dropped if that side removed it. Text is compared exactly.

**merge.decl.2** — A declaration that is the same on both sides, added, changed or removed, appears once.

**merge.order.1** — The merged file keeps the order of OURS. A declaration only THEIRS has goes after the declaration before it in THEIRS.

**merge.line.1** — A declaration changed on both sides is merged line by line as `git merge-file` would. If that merges cleanly, the result is used.

**merge.conflict.1** — Otherwise the declaration is one hunk: `
**merge.cli.1** — `ail merge BASE OURS THEIRS` writes the result to OURS, the way a git merge driver is called; further arguments (git's `%L` and `%P`) are ignored. `--stdout` prints it instead and writes nothing. The exit status is 0 for a clean merge, 1 for conflicts, 2 for an error.

**merge.cli.2** — A file that can't be read is an error: exit status 2.

**merge.cli.3** — With `--json` the answer is one object on standard output, `{"merged":BOOL,"conflicts":[KEY...]}`.

**merge.install.1** — `ail merge --install` adds `*.ail merge=ail` to `.gitattributes` unless it is there, and sets `merge.ail.driver` to `ail merge %O %A %B %L %P` (`./ail` when the current directory has one) in the repository's git config. Outside a git repository it is an error: exit status 2.

```ail pkg=merge_disjoint tool=merge,--stdout,base.ail,ours.ail,theirs.ail status=0 expect=quadruple rules=merge.split.1,merge.split.2,merge.decl.1,merge.order.1,merge.cli.1
```

```ail pkg=merge_same tool=merge,--stdout,base.ail,ours.ail,theirs.ail status=0 expect=two rules=merge.decl.2
```

```ail pkg=merge_lines tool=merge,--stdout,base.ail,ours.ail,theirs.ail status=0 expect=60 rules=merge.line.1
```

```ail pkg=merge_conflict tool=merge,base.ail,ours.ail,theirs.ail status=1 expect=conflict: rules=merge.conflict.1,merge.cli.1
```

```ail pkg=merge_conflict tool=merge,--stdout,--json,base.ail,ours.ail,theirs.ail status=1 expect="merged":false rules=merge.cli.3
```

```ail pkg=merge_conflict tool=merge,nosuch.ail,ours.ail,theirs.ail status=2 expect=merge.cli.2 rules=merge.cli.2
```

```ail pkg=merge_disjoint tool=merge,--check,base.ail,ours.ail,theirs.ail status=0 expect=ok: rules=merge.check.1
```

```ail pkg=merge_conflict tool=merge,--check,base.ail,ours.ail,theirs.ail status=1 expect=conflict: rules=merge.check.1
```

One side adds a parameter to `add` and updates its caller; the other adds a new caller with the old arity. Git and merge.decl.1 merge both cleanly; the check does not.

```ail pkg=merge_semantic tool=merge,--check,base.ail,ours.ail,theirs.ail status=1 expect=(ours): rules=merge.check.2
```

```ail pkg=merge_semantic tool=merge,--check,--json,base.ail,ours.ail,theirs.ail status=1 expect="semantic":[{ rules=merge.check.4
```

**merge.check.1** — `ail merge --check BASE OURS THEIRS`, when BASE names a file, merges as above and writes nothing. If the merge has conflict hunks they are reported (merge.conflict.1) and nothing is checked. Otherwise OURS, THEIRS and the merged text are each checked as a program with `ail check`, and an error the merge has that neither side has is a *semantic conflict*. Errors are compared by file, rule and message with digits removed, so an error that only moved lines is not new. A clean result prints `ok: merged and checked, no new errors (merge.check.1)`.

**merge.check.2** — Each semantic conflict is reported on standard error as `semantic: FILE:LINE: KEY (SIDE) with KEY2 (SIDE2), ...: RULE message (merge.check.2)`. KEY is the declaration that holds the error in the merged text and SIDE the side that changed it (`ours`, `theirs`, `both`, or `base` when neither did). The `with` list names the declarations the other side added, changed or removed whose name appears in KEY's text or in the message; it is left out when there are none. The exit status is 1 for any conflict, textual or semantic.

**merge.check.3** — `ail merge --check OURS_REV THEIRS_REV [PATH]`, when the first argument is not a file, merges two git revisions: it adds scratch worktrees of OURS_REV, THEIRS_REV and their merge (made by `git merge` with this driver for `*.ail`), checks PATH (default `.`) in each, and reports as merge.check.2 with paths relative to the repository. A file git can't merge is reported as `conflict: FILE`. The worktrees are removed after; the repository's branches, index and files are unchanged. A revision that isn't one, or two with no common ancestor, is an error: exit status 2.

**merge.check.4** — With `--json` the `--check` answer is one object on standard output, `{"merged":BOOL,"conflicts":[KEY...],"semantic":[{"file","line","decl","side","with":[KEY...],"rule","message"}...]}`.

`--install` changes the git configuration of the repository it runs in, and the revision form needs a git repository, so their examples are counted, not run.

```ail skip=context rules=merge.install.1,merge.check.3
fn main()
    print(1)
```

## MCP server

`ail mcp` serves the compiler's tools to an agent
([ADR 0918](../decisions/0918-ail-mcp-compiler-tools.md)); `ail serve --mcp` is the
different feature of [ADR 0892](../decisions/0892-tool-functions-mcp-server.md).

**mcp.proto.1** — The server reads one JSON-RPC 2.0 message per line of standard input and writes one per line of standard output. It answers `initialize` (protocol version echoed, capability `tools`, server `ail`), `ping`, `tools/list` and `tools/call`; any other method is error -32601, text that isn't JSON is error -32700.

**mcp.proto.2** — A message with no `id` (a notification) gets no reply.

**mcp.tools.1** — The tools are `ail_check`, `ail_test`, `ail_ask`, `ail_impact`, `ail_verify`, `ail_context`, `ail_edit`, `ail_fix`, `ail_explain`, `ail_api`, `ail_review`, `ail_card`, `ail_find`, `ail_try`, `ail_prove`, `ail_infer`, `ail_merge_check`, `ail_verify_check`, `task_start`, `task_status`, `task_verify`, `task_finish` and `task_abort`. Each has an `inputSchema` of string properties and the required ones, except that `budget` is an integer and the flags `json`, `test`, `impact` and `dry_run` are booleans (a text `"true"` is also accepted). `ail_test` runs with `--bundle-failure`, and with `bundle` set reruns that bundle (`ail test --bundle PATH`) instead. The six that wrap commands with no tool of their own before ([ADR 0918](../decisions/0918-ail-mcp-compiler-tools.md), amended 2026-11-10) run: `ail_context` (`program`, `symbol`, `budget`, `json`) as `ail context PROGRAM SYMBOL`; `ail_edit` (`program`, `edits`, `test`, `impact`, `dry_run`, `budget`) as `ail edit [--test] [--impact] [--dry-run] PROGRAM` with `edits` on standard input (text, or any JSON value, which is sent as its JSON text); `ail_fix` (`program`, `dry_run`, `budget`) as `ail fix`; `ail_explain` (`rule`, `budget`, `json`) as `ail explain [RULE]`; `ail_api` (`program`, `form`, `module`, `budget`, `json`) as `ail api --FORM`, where `form` is `card` (the default), `index` (`--card --index`), `outline`, `symbols`, `text` or `pretty`, and any other form is error -32602; `ail_review` (`range`, `path`, `budget`, `json`) as `ail review RANGE [PATH]`. The seven added later run: `ail_card` (`topic`) as `ail card [TOPIC]`; `ail_find` (`words`, or `sig`) as `ail ask std WORDS` or `ail ask std --sig SIG`, and with neither it is error -32602; `ail_try` (`program`, `sets`: a list of patch sets, or one text) as `ail try PROGRAM --sets-json` with the sets as a JSON array on standard input, and never `--apply`; `ail_prove` (`program`) as `ail prove`; `ail_infer` (`program`, `function`) as `ail infer --contracts FUNCTION PROGRAM`, and never `--apply`; `ail_merge_check` (`ours`, `theirs`, and `base` or `path`) as `ail merge --check BASE OURS THEIRS` when `base` is given, else `ail merge --check OURS THEIRS [PATH]`; `ail_verify_check` (`cert`) as `ail verify --check CERT`. A `budget` is passed as `--budget N`; without one the command uses its own default.

**mcp.tools.3** — Every description is one sentence that says when to use the tool, and `required` is left out of a schema that has none, so the whole `tools/list` answer stays under 1,500 tokens (the `agent_tools` suite fails above 6,000 bytes). *(ADR 0918, amended 2026-11-11)*

**mcp.call.1** — A call runs the matching `ail` command in the server's directory and returns its stdout then stderr as one text content; `isError` is true when the exit status is not 0.

**mcp.call.2** — A call that lacks a required argument, or names no tool, is error -32602 and runs nothing.

**mcp.call.3** ([ADR 0918](../decisions/0918-ail-mcp-compiler-tools.md), amended) — When the command ran with `--json` (the always-JSON tools `ail_verify`, `ail_impact` and `task_*`, and the others when `json` is true) and its standard output is one JSON object, the result also carries that object as `structuredContent`. The text content is unchanged.

**mcp.call.4** ([ADR 0918](../decisions/0918-ail-mcp-compiler-tools.md), amended) — A call whose `program`, `symbol`, `rule`, `range`, `path`, `module`, `form`, `topic`, `words`, `sig`, `function`, `base`, `ours`, `theirs` or `cert` starts with `-` is error -32602 and runs nothing, so a value cannot become an option.

```ail tool=mcp rules=mcp.proto.1,mcp.proto.2,mcp.tools.1,mcp.tools.3,mcp.call.1,mcp.call.2,mcp.call.3,mcp.call.4
fn main()
    print(1)
```

### Warm calls

`ail mcp` keeps no compiler in memory; it saves the launcher's start and repeated answers
([ADR 1056](../decisions/1056-warm-ail-mcp.md), amending ADR 0918).

**mcp.warm.1** — A call whose command the launcher would only hand to the native CLI (`check` without `--budget`, `--from` or `--errors`, as `check-program`; `api` without `--budget`, `--from` or `--diff`; `ask`, `card`, `context`, `explain`, `edit`, `fix`, `prove` and `review`) runs that CLI directly, the one named by `AIL_MCP_NATIVE`, which the `ail mcp` launcher sets. Every other call runs `ail` as `mcp.call.1` says. The answer is the same either way.

**mcp.warm.2** — `ail_check`, `ail_ask`, `ail_context`, `ail_api`, `ail_explain`, `ail_card`, `ail_find`, `ail_prove` and `ail_fix` with `dry_run` are read-only. A read-only call with the same command line as an earlier one in the session, while the toolchain and the program's inputs are unchanged, gets that call's answer without running anything. The inputs are the contents of the `.ail` and `.h` files under the program's root (its directory, or the package), its `ail.pkg` and its `ail.lock`; for `ail_explain`, `ail_find` and the card topics of `ail_card`, the spec and the cards. A package whose `ail.pkg` names a path with `..`, or a root with more than 4000 such files, is not memoized.

**mcp.warm.3** — When a file under `compiler/` or `std/` is newer than the native CLI, the call goes through `ail`, which rebuilds it, and the memo is emptied.

**mcp.warm.4** — Each tool result carries `_meta` with `ail/warm`: `memo`, `direct` or `launcher`. With `AIL_MCP_COLD` set, every call goes through `ail` and nothing is memoized.

```ail tool=mcp rules=mcp.warm.1,mcp.warm.2,mcp.warm.3,mcp.warm.4
fn main()
    print(1)
```

## Language server

`ail lsp` serves the semantic API to any editor as a Language Server
([ADR 0966](../decisions/0966-ail-lsp-language-server.md)). It runs the matching
`ail` commands and reshapes their JSON; it has no analysis of its own.

**lsp.proto.1** — The server speaks LSP 3.17 over standard input and output with `Content-Length` framing. It answers `initialize` with the capabilities `textDocumentSync` (open, close, full-text change, save), `hoverProvider`, `definitionProvider`, `referencesProvider`, `documentSymbolProvider`, `completionProvider` (trigger `.`), `renameProvider` (with prepare) and `codeActionProvider` (`quickfix`, `source.fixAll.ail`).

**lsp.proto.2** — A request is answered; a notification is not. An unknown request is error -32601, text that isn't JSON is error -32700, an unknown notification is ignored. `shutdown` then `exit` ends the process with status 0, `exit` alone with status 1.

**lsp.sync.1** — The server keeps the text of each open document. A change replaces the whole text.

**lsp.sync.2** — A command sees unsaved text: when an open document differs from the file on disk, the server copies the `.ail` files of its directory to a scratch directory under the temporary directory, lays every open buffer of that directory over them, and runs the command there. Packages with subdirectories see the saved text of files outside the directory.

**lsp.diag.1** — On open and on save the server publishes the diagnostics of `ail check --json` for that document. The rule id is the diagnostic's `code`, the source is `ail`, an error is severity 1 and a warning severity 2, and a `help` is appended to the message. Closing a document publishes an empty list.

**lsp.hover.1** — Hover is `ail api --type`: the type under the cursor, then for a call the callee's signature and doc comment, as Markdown.

**lsp.def.1** — Definition is the `def` entry of `ail api --refs`; references are all its entries. A use through a module resolves to the declaring file.

**lsp.sym.1** — Document symbols are the `ail api --outline` items, nested: fields, variants and methods are children. `fn` is Function, `method` Method, `struct` Struct, `enum` Enum, `trait` Interface, `impl` Namespace, `const` Constant, `field` Field, `variant` EnumMember.

**lsp.complete.1** — `textDocument/completion` answers a complete list of names from `ail api`: the locals in scope at the cursor (`api --locals`; Variable, with the type as detail), then the functions, structs, enums, traits and constants of the file (`api --outline`). Only names that start with the text typed so far are returned.

**lsp.complete.2** — After a `.` the names are the fields (Field) and methods (Method) of the type of the value before the dot (`api --type`), looked up among the declarations of the file. A buffer being typed rarely checks, so before asking, the server rewrites the cursor's line in its scratch copy: to a blank line, or after a `.` to a binding of the dotted name before it (`ail_probe := p`), whose type is asked. When `api` refuses the copy (a later line used what the rewritten line bound), the names come from the text instead: the parameters and the `name :=` and `for name` bindings above the cursor in its function, and for a receiver that is one name, the type its parameter, `self` or `name := Type(...)` binding writes, looked up in the last outline that checked. A receiver that is not one name, and a type not declared in the file, give no members. `AIL_LSP_DEBUG` set makes the server print each command it runs, its status and its standard error to standard error. When the copy still has errors, the last outline that checked answers. Types that are not declared in the file give no members.

**lsp.rename.1** — `textDocument/rename` rewrites every reference of `ail api --refs` to the new name, in every file the references are in, as a `changes` edit. A use through a module qualifier (`geo.origin`) rewrites only the name after the qualifier.

**lsp.rename.2** — `textDocument/prepareRename` answers the range and name under the cursor when it is a declared name, else null. A new name that is not an identifier, a position with no name and a name the program does not declare (a standard item) are error -32602 and change nothing. The result is not re-checked: run `ail check` after.

**lsp.action.1** — `textDocument/codeAction` offers each fix of `ail check --json` for a diagnostic at the requested lines as a `quickfix` whose edit applies it. The first fix is preferred and the diagnostic is attached.

**lsp.action.2** — With two or more fixable diagnostics the server also offers `source.fixAll.ail`: the first fix of every diagnostic whose edits touch none already taken, in one edit.

**lsp.pos.1** — Positions are zero-based lines and characters; a character is one Unicode scalar (equal to a UTF-16 unit below U+10000).

```ail tool=lsp rules=lsp.proto.1,lsp.proto.2,lsp.sync.1,lsp.sync.2,lsp.diag.1,lsp.hover.1,lsp.def.1,lsp.sym.1,lsp.complete.1,lsp.complete.2,lsp.rename.1,lsp.rename.2,lsp.action.1,lsp.action.2,lsp.pos.1
fn main()
    print(1)
```

## Sanitizers

`ail sanitize` finds results that depend on the task schedule
([ADR 0967](../decisions/0967-sanitizers.md)). Safe code cannot race (`conc.race.1`);
a schedule-dependent result points at `unsafe` or FFI code, or at a missing `wait`. The memory
checks of `unsafe.ptr.4-6` and `unsafe.mem.2` already trap, so they need no sanitizer mode.

**san.sched.1** — `ail sanitize [--seeds N] [--tests] [--timeout MS] [--json] FILE [ARGS...]` runs `ail run FILE` (with `--tests`, `ail test FILE`) once with `--schedule=fixed` and then with `--schedule=random:SEED` for each seed 1 to N (default 8). A run whose exit status or standard output differs from the fixed run is a finding: a warning with the rule `san.sched.1`, the seed and the command that replays it. With `--json` each finding is one line `{"api":1,"severity":"warning","rule":"san.sched.1","file":...,"seed":...,"message":...,"replay":...}`, then a summary line with `seeds` and `schedule_dependent`.

**san.sched.2** — The exit status is 0 when no run differs, 1 when one does, 2 for bad arguments. A run that can't start or exceeds `--timeout` (default 60000 ms) has status 124.

```ail tool=sanitize,FILE expect=schedules rules=san.sched.1,san.sched.2
fn main()
    print(1)
```

**san.race.1** — `ail sanitize --race [--tests] [--timeout MS] [--json] FILE [ARGS...]` runs the program built with `AIL_SANITIZE=race` (POSIX shells; the variable also works with `ail run` and `ail build`). Each non-volatile raw-pointer `read` and `write` in the program's own code is checked against the last write and last read of the same 8-byte granule. Two accesses by different tasks, at least one a write, with no happens-before edge between them are a conflict. The report is `error[san.race.1]` on standard error with both source positions and task numbers; with `--json` one line per report. Each pair of positions is reported once, up to 20. Safe code and `par` loops over safe code cannot conflict (`conc.race.1`), so only `unsafe` accesses are checked, including those in `spawn` bodies and `par` chunks.

**san.race.2** — The edges are: `spawn` (the child sees all the parent did before), `wait` and group waits (the waiter sees the child's whole run), `Mutex` lock and unlock, `Channel` send and receive, and `Atomic` operations (a store, `cas` or `add` publishes, a `load`, `cas` or `add` takes), each on its own object. A fresh `std.mem` block starts with no history. A run with a conflict exits 1 even if the program exited 0.

**san.race.3** — The detector may miss a conflict, never invent one: a granule keeps one last write and one last read, tasks past the 255th share an id, and a full shadow table stops tracking new addresses; the summary line says when the last two happened. A race-instrumented build is slower and uses about 15 MB more address space; a normal build is unchanged.

```ail tool=sanitize,--race,FILE expect=san.race.1 status=1 rules=san.race.1,san.race.2,san.race.3
use std.mem

fn bump(p *int)
    unsafe
        p.write(p.read() + 1)

fn main() !
    unsafe
        p := mem.alloc(1, 0)
        a := spawn bump(p)
        b := spawn bump(p)
        a.wait()?
        b.wait()?
        print(p.read())
        mem.free(p)
```

## Leak report

A garbage-collected program cannot lose memory it no longer reaches, but it can keep memory it
no longer needs: a cache that is never trimmed, a list that only grows, a global holding a
finished job. `ail leak` reports that growth and the roots that hold it
([ADR 0969](../decisions/0969-leak-report.md)). It reports memory the program still reaches, so
every number is a retained size, not a count of lost blocks.

**san.leak.1** — With the report on, each collection counts the live bytes and objects by kind. After at least four collections, when live bytes at the latest collection are at least 1.5 times those at the first plus 1 MiB, and no collection of the last 64 fell by more than a twentieth of the one before, the report says live memory grew, with the first and latest totals and each kind that grew by 256 KiB or more. The sizes are payload plus the 8-byte header, so they differ a little from `AIL_MAX_MEMORY_MB`, which counts blocks.

**san.leak.2** — The collection marks the roots one at a time, in the order: each global slot that points into the heap, the registers, the stack, the other threads. An object counts for the first root that reaches it, so the sizes add up to the live total. On growth the report lists the five roots that hold the most (at least 1 MiB each) with their bytes at the first and the latest collection, their object count and the kind with most bytes; `--all` lists the top five whenever they hold anything, growth or not. A root is `global slot N` (the slot's index; source names need the debug lane's positions, ADR 0947), `the stack`, `the registers` or `the other threads`. A conservative stack word can keep an object that no variable holds; the report cannot tell the two apart.

**san.leak.3** — `ail leak [--json] [--all] FILE [ARGS...]` builds FILE and runs it with the environment variable `AIL_LEAK` set to `1`, `json`, `all` or `all,json`; the same variable works for any native executable, on every target. The report goes to standard error after the program's own output and ends with a summary line (`leak: N collections, S live at exit, growing` or `no growth`). With `json`, each finding is one line `{"api":1,"severity":"warning","rule":"san.leak.1"|"san.leak.2",...}` and the last line is `{"summary":true,"collections":N,"live_bytes":B,"grew":BOOL}`. The program's exit status is unchanged.

**san.leak.4** — With the variable unset the runtime does nothing extra: the collector tests one word. With it set, collections run in single-thread mode, at least every 4 MiB allocated (or half the live bytes, if more), so a run is slower. A program that ends with status 0 after fewer than four collections gets one more collection at exit, when its locals are gone, so the report shows what the globals still hold. A run that traps keeps the collections it has.

```ail tool=leak,FILE expect=leak[san.leak.1] rules=san.leak.1,san.leak.2
fn main()
    mut keep := List[List[int]]()
    mut i := 0
    while i < 48
        mut row := List[int]()
        mut j := 0
        while j < 131072
            row.push(j)
            j += 1
        keep.push(row)
        i += 1
    print(keep.len())
```

```ail tool=leak,--json,FILE expect="collections":1, rules=san.leak.3,san.leak.4
fn main()
    print(1)
```

## New projects

**tool.new.1** — `ail new NAME` creates the directory `NAME` holding `ail.pkg`
(`package NAME`, [mod.pkg.1](12-modules.md)) and `main.ail`: a `greet`
function, a `main` that prints its result and a `@test` that checks it, so
`ail run NAME` and `ail test NAME/main.ail` work at once, and prints the two
commands. `NAME` starts with a lowercase letter and holds lowercase letters,
digits, `_` and `-`; any other name, and any `NAME` that already exists as a
file or directory, is refused with exit status 2, and nothing is created or
changed. *(ADR 0903)*

**tool.new.2** — The generated `main.ail` passes `ail check`, `ail fmt --check`
and its own `@test`. *(ADR 0903)*

```ail rules=tool.new.1,tool.new.2,tool.new.3,tool.new.4,tool.new.5
## demo: say what this program does.

## The greeting for `who`.
fn greet(who str) str
    "Hello, {who}!"

fn main()
    print(greet("world"))

@test fn greets()
    assert_eq(greet("ail"), "Hello, ail!")
```

**tool.new.3** — Unless `--no-agent` is given, `ail new NAME` also writes
`NAME/AGENTS.md` and `NAME/.mcp.json`. `AGENTS.md` is at most 350 tokens: the
core card (`ail card`) and the lookups beyond it (`ail card TOPIC`, `ail ask std
WORDS`), the loop (`ail task start`, `ail context FN`, edit, `ail check --fix`,
`ail task verify`, `ail task finish`, which writes the change certificate),
`ail try` for two candidate fixes, `ail explain RULE` for any diagnostic, and
`ail ask` instead of grep. It does not send the agent to the full card. `.mcp.json`
registers `ail mcp` on stdio under `mcpServers.ail`. The `agent_tools` suite
checks both files. *(ADR 1046)*

**tool.new.4** — `ail new --no-agent NAME` writes only `ail.pkg` and `main.ail`;
the flag may come before or after `NAME`. *(ADR 1046)*

**tool.new.5** — `ail new .` writes `AGENTS.md` and `.mcp.json` (the files of
`tool.new.3`, and nothing else) into the current directory when it holds a
`.ail` file or `ail.pkg`; any other directory is refused with exit status 2.
An existing file is never changed or appended to: for `AGENTS.md` the block to
add is printed instead, and for `.mcp.json` the registration to add. `--no-agent`
with `.` is a usage error (exit status 2). *(ADR 1046)*

## Help and the toolchain check

**tool.help.1** — `ail help`, `ail --help`, `ail -h` and `ail` with no arguments print every
user command, one line each, under the headings Build and run, Check and fix, Ask the compiler,
Agent loop and Project, and exit 0. Internal commands are not listed. A command word that is not
listed is refused with "unknown command X; run `ail help`" and exit status 2. *(ADR 1057)*

**tool.help.2** — `ail help CMD` and `ail CMD --help` (or `-h`, as the only argument after `CMD`)
print the command's usage line and, for each rule that governs it, `rule: ID (ail explain ID)`.
Longer argument lists keep `--help` for the command: `ail run FILE --help` passes it to the
program. *(ADR 1057)*

**tool.help.3** — `ail help --json` prints the table as one JSON array of objects with `name`,
`group`, `summary`, `usage` and `rules`; `ail help --json CMD` prints the object of one command.
`ail help --keywords` prints the language's reserved words (`lex.kw.1`), one per line, or with `--json`
one JSON array, so an editor reads them from the toolchain. The `cli` gate suite fails when a listed command has
no route in `ail` or in `ail.ps1`. *(ADR 1057)*

```ail tool=help expect=Agent rules=tool.help.1
fn main()
    print(1)
```

```ail tool=help,run expect=usage: rules=tool.help.2
fn main()
    print(1)
```

```ail tool=help,--json expect="group":"Agent rules=tool.help.3
fn main()
    print(1)
```

```ail tool=help,--keywords,--json expect="fn" rules=tool.help.3
fn main()
    print(1)
```

```ail tool=help,nosuchcommand status=2 expect=unknown rules=tool.help.1
fn main()
    print(1)
```

**tool.path.1** — `ail` finds its toolchain through symbolic links: the root is the directory of
the file the link names, so `ln -s /path/to/ail/ail ~/.local/bin/ail` runs the same toolchain as
`/path/to/ail/ail`. *(ADR 1058)*

```ail skip=bootstrap rules=tool.path.1
fn main()
    print(1)
```

**tool.doctor.1** — `ail doctor [--json]` prints one line per check: `ok NAME: DETAIL`, or
`warn`/`fail NAME: DETAIL; fix: WHAT TO RUN`. The checks are the toolchain root, the version and seed
tag, the first `ail` on PATH, `git`, `curl`, a hash tool, a writable build cache directory, and the
std.app and std.audio hosts. The exit status is 1 when a required check (root, git, cache) fails.
With `--json` it is one array of objects with `name`, `status`, `detail`, `fix` and `required`.
*(ADR 1058)*

**tool.doctor.2** — `ail doctor --path` prints one line to add to the shell profile that puts the
toolchain's directory on PATH (`export PATH="DIR:$PATH"`, or `$env:Path = "DIR;" + $env:Path` on
Windows) and changes nothing. `ail new` prints a hint on standard error when `ail` is not on PATH.
*(ADR 1058)*

```ail tool=doctor expect=toolchain rules=tool.doctor.1
fn main()
    print(1)
```

```ail tool=doctor,--path expect=PATH rules=tool.doctor.2
fn main()
    print(1)
```

## Performance remarks

**tool.perf.1** — `ail perf --explain FN FILE` compiles `FILE` natively and,
for each loop of the function `FN` (`name`, or `Type.method`), prints where the
loop is and what the optimizer did with it: whether its list bounds checks were
removed ([native.bce.1](23-native.md)) and, when they stayed, each access that
kept its check with the reason and, where one exists, a fix; whether the loop
runs in a vector kernel ([native.vec.1](23-native.md)); whether its list
headers are kept in registers ([native.bce.11](23-native.md)); whether the checker
finds its iterations independent ([conc.par.6](11-concurrency.md)); and each
call that stays in it with the reason it was not inlined. Every claim cites its
rule. `FILE` must pass `ail check`. The exit status is 0, or 2 for a bad call,
a file that does not check, or a `FN` the program does not have, which lists the
functions it has. The wording is non-normative. *(ADR 1021)*

**tool.perf.2** — The remarks are the decisions the optimizer's own passes
made, not a second analysis, so they cannot disagree with the executable
(`ail build` makes the same one). Asking for them changes no code and costs
nothing in a build that does not ask. With `AIL_OPT_OFF` naming a pass, the
remark of that pass says it was not analysed. *(ADR 1021)*

```ail tool=perf,--explain,total,FILE expect=removed rules=tool.perf.1,tool.perf.2
fn total(xs List[int]) int
    mut sum := 0
    for i in 0..xs.len()
        sum += xs[i]
    sum

fn main()
    print(total([1, 2, 3]))
```

## Examples
Conformance examples (ADR 0002). Each `tool=` block runs the named tool on the
example (`FILE` is the example, `DIR` its package directory) and checks that
its output contains the `expect=` text. The tools' output formats are
non-normative; these examples check the behaviour the rules describe.
`explain.cover.*` is also enforced by the `explain` gate suite
in `./ail selftest`, and `api.card.5` (the generated cards match `ail api --card`)
by the `api_card` suite; the harness cannot read the repository's cards.
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
```ail tool=api,--refs,FILE,9,7 expect=method:Shape.area rules=api.ref.2
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
```ail tool=api,--refs,FILE,11,5 expect=refs rules=api.ref.1,api.refs.1,api.refs.3
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
```ail tool=api,--card,--module,(root),FILE expect=add rules=api.card.6
fn add(a int, b int) int
    a + b
fn main()
    print(add(1, 2))
```
```ail tool=api,--card,--index,FILE expect=(root) rules=api.card.7
fn add(a int, b int) int
    a + b
fn main()
    print(add(1, 2))
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
## Structured edits
**edit.in.1** — `ail edit PATH` reads a JSON list of edits (or one edit
object) from standard input and applies them in order to the file or
package directory `PATH`. Each edit has an `"op"`: `replace_body`,
`replace`, `insert`, `delete`, `rename`, `import`, `change_sig`, `extract`
or `move`, with the keys of `edit.op.1–6` and `edit.op.8–10`. Input that isn't such JSON is an error (exit 2)
([ADR 0397](../decisions/0397-ail-edit-structured-edits-by-id.md)).
**edit.in.3** — Input that doesn't start with `[` or `{` is the compact
form: hunks (`edit.op.7`), each under a line holding an id, and
`rename OLD NEW` lines (`edit.op.5`). A hunk's lines start with `-` (a line
to remove), `+` (a line to add) or a space (a line to keep); empty lines are
ignored, and a lone `+` adds an empty line. Lines before any id line form a
hunk on the whole file
([ADR 0399](../decisions/0399-compact-edits-and-diff-hunks.md)).
**edit.in.2** — Edits apply all or nothing: when one cannot be applied, the
command names it (`edit K of N`), writes nothing and exits 2.
**edit.id.1** — An edit names a declaration by its id (`api.id.1`):
`fn:NAME`, `method:TYPE.NAME`, `type:NAME`, `trait:NAME`, `const:NAME`,
and for `rename` also `field:TYPE.NAME` and `variant:TYPE.NAME`. Only
declarations that start at column 0 are found. A declaration runs from its
doc comment and attributes to its last indented line.
**edit.id.3** — A bare name (`average`, `Point.norm`) stands for the one
declaration whose id is that name after its kind (`fn:average`,
`method:Point.norm`); a name that two kinds share is an error, as in
`edit.id.2`.
**edit.id.2** — In a package, an id may carry the module of its file
(`fn:geo.origin`, `api.pkg.1`). An id that names declarations in two files
is an error that asks for the module.
**edit.op.1** — `{"op":"replace_body","id":ID,"body":TEXT}` replaces the
lines after the declaration's signature with `TEXT`, re-indented to one
level. Leading `requires` and `ensures` lines are kept unless `TEXT` has
its own.
**edit.op.2** — `{"op":"replace","id":ID,"text":TEXT}` replaces the whole
declaration, doc comment included. `{"op":"delete","id":ID}` removes it.
**edit.op.3** — `{"op":"insert","text":TEXT}` adds a declaration after
(`"after":ID`) or before (`"before":ID`) another, or at the end of the file
(`"file"` picks it in a package; the default is the package's first root
file).
**edit.op.4** — The edits of `edit.op.1–3` are textual: their result is
checked with the rest (`edit.out.2`), not before.
**edit.op.5** — `{"op":"rename","id":ID,"to":NAME}` renames the declaration
and every reference to it that `ail api --symbols` reports (`api.ref.1`),
in every file of the program. The program as edited so far must check
without errors; otherwise the rename fails.
**edit.op.6** — `{"op":"import","module":M}` adds `use M` after the last
`use` line (or at the top) unless it is there.
**edit.op.7** — A hunk (`{"op":"hunk","id":ID,"lines":[...]}` in JSON, `ID`
optional) finds its `-` and context lines, compared without leading and
trailing spaces, at exactly one place in the declaration (or the file), and
replaces them with its context and `+` lines in order. `+` lines are
indented by the difference between the found line's indentation and the
hunk's first `-` or context line's, so lines written at column 0 land at
the found line's depth. A hunk of one `-` line and one `+` line whose `-`
text is no whole line replaces that text where it occurs exactly once in
the declaration. A hunk that matches nowhere, or in more than one place, is
an error.
**edit.op.8** — `{"op":"change_sig","id":ID,"params":[...],"args":{NAME:EXPR}}`
changes a function's or method's parameters and every call to it that
`ail api --symbols` reports. `params` is the new list in order, `self` left
out (it stays first): an old parameter's name keeps it, `NAME Type` with an
old name changes its type, and `NAME Type` with a new name adds one, whose
argument at every call is `args[NAME]`. An old parameter not listed is
removed, and so is its argument at every call. Each call's arguments are
written positionally in the new order, without labels (`expr.call.3`). The
program as edited so far must check, as for `edit.op.5`; a reference that is
not a call (a function value), or a call with a comment inside its
parentheses, fails the edit
([ADR 1054](../decisions/1054-ail-edit-refactors.md)).
**edit.op.9** — `{"op":"extract","id":ID,"from":LINE,"to":LINE,"name":NAME}`
moves body lines into a new function `NAME` after the declaration and puts a
call in their place. `from` is the trimmed text of exactly one line of the
body; `to` (default: `from`) is the first line at or after it with that
text. The lines must be whole statements of one block, must not `return`,
and must not `break` or `continue` a loop that starts before them. The
parameters are the locals the lines use that are in scope before them, with
the checker's types (`api.loc.1`), in order of first use; a `mut` local the
lines may change is a `mut` parameter, passed as `mut x`. A name the lines
declare at their own depth and that is used after them is returned (several
as a tuple) and bound by the call (`x := NAME(...)`). Lines that end the
body of a function with a result become its result. When the enclosing
function can fail and the lines use `?`, the new one can fail (`!T`) and the
call has `?`. In a method with `self`, the new function is a method with the
same receiver. The program as edited so far must check
([ADR 1054](../decisions/1054-ail-edit-refactors.md)).
**edit.op.10** — `{"op":"move","id":ID,"to":MODULE}`, for a package, moves a
function, constant or type (a type with its `fn Type.name` methods,
`mod.unit.4`) to the end of a file of module `MODULE` (`"file"` picks one;
the default is its first file). Every reference is requalified: inside
`MODULE` unqualified, elsewhere as `m.name` under the file's import of
`MODULE`, which is added when missing (`mod.use.1–2`). In the moved text,
names of the old module get its qualifier and the new file gets the `use`
lines the text needs. The moved declarations are made `pub` when a reference
is outside `MODULE`. A `use` of the old module that no longer qualifies
anything is removed. A type with an `impl`, a moved body that names a
declaration of its old module that isn't `pub` or of the root module, and a
`MODULE` with no file fail the edit; an import cycle it makes is reported by
the check (`mod.use.5`)
([ADR 1054](../decisions/1054-ail-edit-refactors.md)).
**edit.out.1** — Every changed file is rewritten in canonical layout
(`ail fmt`) when it parses.
**edit.out.2** — The program is then checked and its diagnostics are
reported as `ail check` reports them, followed by one summary line,
`edited N files; E errors` with `, W warnings` added when there are any
(with `--json`,
`{"summary":true,"edits","written","files","errors","warnings"}`). The exit
status is 2 when there are errors, 0 otherwise. The files are written even
when there are errors, unless `--if-clean` is given.
**edit.out.3** — `--dry-run` writes nothing and prints a unified diff of
each file that would change; `--diff` prints the diff and writes.
A single existing file is checked from memory, so a preview or a refused
edit never changes it on disk, not even for a moment. Known limitation: for a
package (a directory path) or an edit that adds a file, the new state is
written, checked and put back, so a program reading those files at that moment
can see the edited text. Writes go to a temporary file beside the target and
are renamed over it.
**edit.out.4** — `--test` runs `ail test PATH` after an edit that was
written without errors, and its report and exit status follow the summary.
For a package, `PATH` is its root file: `main.ail`, or else its first file at
the top.

**edit.out.5** — `--impact` adds the edit's impact (`api.impact.1-4`) after the
summary: the declarations it changed, the tests it can affect and the
effects it changed. It is computed from the program before and after the edit,
so it costs nothing when the flag is absent. With `--json` the summary line
has an `impact` member.

```ail pkg=edit_ops tool=edit,--dry-run,main.ail stdin=edits.json expect=file; rules=edit.in.1,edit.id.1,edit.op.1,edit.op.2,edit.op.3,edit.op.4,edit.op.5,edit.op.6,edit.out.1,edit.out.2,edit.out.3
```

```ail pkg=edit_ops tool=edit,main.ail stdin=bad.json status=2 expect=written rules=edit.in.2,edit.id.1,edit.id.2
```

```ail pkg=edit_ops tool=edit,main.ail stdin=text.txt status=2 expect=edit.in.3 rules=edit.in.3
```

```ail pkg=edit_ops tool=edit,--test,main.ail stdin=hunk.txt expect=passed, rules=edit.in.3,edit.id.3,edit.op.7,edit.out.4
```

```ail pkg=edit_refactor tool=edit,--test,main.ail stdin=sig.json expect=passed, rules=edit.op.8,edit.out.4
```

```ail pkg=edit_refactor tool=edit,--test,main.ail stdin=extract.json expect=passed, rules=edit.op.9
```

```ail pkg=edit_refactor tool=edit,--test,main.ail stdin=value.json expect=passed, rules=edit.op.9
```

```ail pkg=refactor_move tool=edit,--test,DIR stdin=move.json expect=passed, rules=edit.op.10,mod.use.1
```

```ail pkg=edit_impact tool=edit,--dry-run,--impact,main.ail stdin=edits.json expect=tests: rules=edit.out.5,api.impact.1,api.impact.2,api.impact.4
```

```ail pkg=edit_impact_sig tool=edit,--dry-run,--impact,main.ail stdin=edits.json status=2 expect=partial: rules=edit.out.5,api.impact.2,api.impact.5
```

```ail tool=test,--update,FILE expect=updated rules=snap.update.1,snap.update.2,snap.update.5
@test
fn adds()
    assert_eq(1 + 1, 3)
```

```ail tool=test,--update,--dry-run,FILE expect=would rules=snap.update.3
@test
fn repeated()
    for i in [1, 2]
        assert_eq(i, 1)
```

```ail tool=test,--update,--dry-run,FILE expect=would rules=snap.update.4
## Doubles a number.
##
## ```ail
## print(double(2))
## # => 5
## ```
fn double(n int) int
    n * 2

fn main()
    print(double(1))
```

```ail tool=test,--update,--dry-run,--json,FILE expect=dry_run rules=snap.update.6
@test
fn adds()
    assert_eq(1 + 1, 3)
```

```ail pkg=snap_out tool=test,--update,FILE expect=updated rules=snap.update.5
```
## Record and replay
**replay.rec.1** — `ail run --record R FILE` runs `FILE` as `ail run` does
and writes R, a recording of every captured call's result
(`replay.cap.1`). `ail test --record R FILE` keeps the calls of the tests
that failed. `--record` and `--replay` need `run` or `test`, a file name,
and not each other
([ADR 0398](../decisions/0398-record-and-replay.md)).
**replay.rec.2** — A recording is JSON lines. The first is
`{"ail_record":1,"program","args","seed","limit"}`: the program path as
given, its arguments, and the seed `std.random` starts from (the `--seed`
given, or one taken from the clock, `std.random.3`). Each captured call
then adds `{"call":"MODULE.FN","line","col",...}` in the order the calls
ran, with `"result"`, `"trap"` and `"message"`, or `"uncaptured":true`.
`ail test` writes `{"test":NAME}` before each failed test's calls.
**replay.rec.3** — A result is written as JSON: `unit` as `null`, `int`,
`bool` and `str` as themselves, and the rest as objects (`{"f":"1.5"}`,
`{"bytes":HEX}`, `{"char":"c"}`, `{"none":true}`,
`{"struct":NAME,"fields":[...]}`, `{"enum":NAME,"variant":N,"fields":[...]}`,
`{"map":[[k,v],...]}`, `{"set":[...]}`, `{"num":KIND,"bits":N}`,
`{"nested":N}`, and a failure as `{"fail":ERROR,"context":[...],"trace":[[line,col],...]}`).
A call that traps records the trap's rule and message.
**replay.rec.4** — A recording holds at most 64 MiB of calls. A call that
would pass the limit, and every later one, is left out, and the recording
ends with `{"truncated":true}`.
**replay.rec.5** — A recorded or replayed run keeps every loop on one
thread (`conc.par.9`), and each test of `ail test --record` or `--replay`
starts `std.random` from the recording's seed.
**replay.cap.1** — The captured calls are every `std.fs`, `std.env`,
`std.time`, `std.tz`, `std.net` and `std.proc` function, `io.read_line`,
`io.read_all` and `io.read_bytes`, and `ui.clipboard_get` and
`ui.clipboard_set`. Not captured: `print` and `io.eprint` (output, which
the replay writes again), `io.exit`, `std.random` (repeated by the seed),
`std.task`, `ui.open`, the pure modules, and methods on handles.
**replay.cap.2** — A captured call whose result holds a handle (a process
from `proc.start`, a socket, a listener) is recorded as
`"uncaptured":true`, and a replay that reaches it stops with a trap
`replay.cap.2`.
**replay.nat.1** — The native `ail run --record` and `--replay`
(`AIL_INSTRUMENT=record`, ADR 0873) capture every call of `replay.cap.1`.
A result that is a list of strings (`fs.list_dir`, `fs.walk`, `fs.glob`), a
struct (`fs.metadata`, `proc.run`, `net.get` and the other responses) or a map
(`env.vars`) is written as in `replay.rec.3`. A handle is written by number and
never used again by the replay: a connection, listener, server or body
answers each call on it from the recording. A child process (`proc.start`,
`proc.wait`, `proc.kill`) is written with the lines of its output streams,
`{"stream":N,"out"|"err":LINE}` and `{"stream":N,"end":"out"|"err"}`, after its
`proc.start`; a replay fills the same channels and drops what is sent to its
input. A call the build can't capture is written as `"uncaptured":true` and
traps `replay.cap.2` in a replay. `ail test --record` and `--replay` run each
test in its own process, record into `{"test":NAME}` sections, and replay only
the tests the recording holds (`replay.rec.5`: each starts from the recording's
seed).
**replay.run.1** — `ail run --replay R [FILE]` runs `FILE` (by default the
recording's program) with the recording's arguments and seed, and answers
each captured call from R instead of running it: nothing is read, written,
fetched, started or waited for. `ail test --replay R [FILE]` runs only the
tests R holds, each from its own calls.
**replay.run.2** — An unchanged program replays the recorded run: the same
output, the same failures, and a trap at the same position with the same
message. A recorded trap in a call traps again at that call.
**replay.run.3** — A call that is not the recording's next one (another
function, or none left) traps with `replay.run.3`, naming the call the
recording expected and where it was recorded; a recorded result that names
a type the program doesn't have traps the same way.

```ail pkg=replay_env tool=run,--record,out.json,main.ail expect=nobody rules=replay.rec.1,replay.rec.5
```

```ail pkg=replay_env tool=run,--replay,r.json expect=first rules=replay.rec.2,replay.rec.3,replay.cap.1,replay.nat.1,replay.run.1,replay.run.2
```

```ail pkg=replay_env tool=run,--replay,other.json,main.ail status=1 expect=replay.run.3 rules=replay.run.3
```

```ail pkg=replay_env tool=run,--replay,h.json status=1 expect=replay.cap.2 rules=replay.cap.2
```

```ail pkg=replay_env tool=run,--replay,cut.json status=1 expect=replay.rec.4 rules=replay.rec.4
```

## Failure bundles

**bundle.write.1** — `ail test FILE --bundle-failure[=DIR]` records the tests as
`--record` does (`replay.rec.1`) and, for each test that failed, writes
`DIR/NAME.json` (default `.ail/bundles`; `NAME` keeps letters, digits and `_`,
other characters become `-`). It writes nothing when every test passes and
prints `bundle: PATH` on standard error for each file
([ADR 0910](../decisions/0910-test-bundle-failure.md)).
**bundle.write.2** — A bundle is JSON lines. The first line is
`{"ail_bundle":1,"test","program","program_hash","compiler","message","cases","property_seed","input","schedule"}`:
the test, the program path as given, the FNV-1a hash of its text, the
`ail --version` text, the failure message, and, for a property test, its case
count, seed and shrunk input (`test.prop.3`), and the `AIL_SCHEDULE` it ran under
(`conc.sched.2`). The rest is the recording of `replay.rec.2`: its first line,
`{"test":NAME}`, and the test's calls.
**bundle.run.1** — `ail test --bundle PATH` runs the test of the bundle again
from its recording, as `--replay` does (`replay.run.1`), and reports as `ail test`
does. The program is the bundle's `program`.
**bundle.run.2** — When the program's text no longer has the bundle's
`program_hash`, `--bundle` says so on standard error before it runs; the result
is the run's own.

```ail pkg=bundle_env tool=test,--bundle-failure=out,t.ail status=1 expect=bundle: rules=bundle.write.1,bundle.write.2
```

```ail pkg=bundle_env tool=test,--bundle,bad.json status=1 expect=differs rules=bundle.run.1,bundle.run.2
```

## Verifying a change

**verify.run.1** — `ail verify --diff [--rev REV] [--json] FILE` compares `FILE`
with git revision REV (default `HEAD`) as `ail ask FILE --affected` does
(`api.impact.3`) and prints four parts: *proved*, *tested*, *unverified* and
*failures*. The exit status is 0 when there is no check error and no new
failure (`verify.run.5`), 1 otherwise, and 2 for a usage error or a program that cannot be compared
([ADR 0909](../decisions/0909-ail-verify-diff.md)).
**verify.run.2** — *Proved* says whether `ail check FILE` accepts the program
(a missing `main` is not an error) and lists the changed declarations
(`api.impact.1`). A program with errors is not tested.
**verify.run.3** — *Tested* is the affected tests (`api.impact.3`) that
`ail test FILE --affected` ran: the names that passed and the names that failed.
**verify.run.4** — *Unverified* lists what no result speaks for: that no test
ran for the changed declarations, each changed declaration that no affected
test reaches (when `ail ask --affected --json` gives `reaches`), a partial impact
(`"partial":true`), each function whose effects changed
(`api.impact.4`), and each skipped test. *Failures* lists the check errors and
the failed tests. `--json` prints the same parts as one object with
`"verify":1`.
**verify.run.5** — A failure is *new* unless the test also fails when the tree at
REV (`git archive REV`) is tested; it is then *old*. Check errors are always new.
When REV cannot be unpacked or run, every failed test is new.

```ail tool=verify,--diff,--rev,nosuch,FILE status=2 expect=api.impact.3 rules=verify.run.1,verify.run.2,verify.run.3,verify.run.4,verify.run.5
fn main()
    print("hi")
```

**verify.cert.1** — `ail verify --diff [--rev REV] [--mutate [--max N]] --attest OUT FILE` runs
`verify.run.1-5` and writes one JSON object to `OUT`, ending with a newline: `cert` (1), `base` (the
commit REV resolves to), `rev`, `file`, `compiler` (the output of `ail version`), `files` (every path
changed since REV and every untracked path, sorted, except `.ail-task` and paths under `.ail/` (`task.finish.3`), each `{"path","hash"}` with the `git hash-object`
hash or `"deleted"`), `proved`, `tested`, `unverified`, `failures` (as in `verify.run.2-5`),
`effects` (the effect changes, `api.impact.4`) and, with `--mutate` and a program that checks,
`mutation`: `max`, `score` (`mutants`, `killed`, `survived`, `invalid` of `test.mutate.5`) and
`survivors_in_changed`, the surviving mutants whose function is a changed declaration
([ADR 1052](../decisions/1052-change-certificate.md)).
**verify.cert.2** — The exit status of `--attest` is that of `verify.run.1`. `--attest` without
`--diff`, and `--mutate` without `--attest`, are usage errors (exit 2).
**verify.cert.3** — `ail verify --check CERT` re-runs the certificate: `ail verify --diff --rev BASE`
with `--mutate --max N` when `CERT` has `mutation`, on `file`. It compares `cert`, `base`, `file`,
`compiler`, `files`, `proved`, `tested`, `unverified`, `failures`, `effects` and `mutation`
member by member, as compact JSON text. `rev` is not compared.
**verify.cert.4** — `--check` prints `differs: KEY` with both values for each member that differs
and exits 1; with none it prints `certificate holds` and exits 0. A file that is missing or not a
certificate, or a program that cannot be re-run, exits 2. A certificate is evidence that
reproduces, not identity: nothing is signed.

```ail tool=verify,--check,nosuch.json status=2 expect=can't rules=verify.cert.1,verify.cert.2,verify.cert.3,verify.cert.4
fn main()
    print("hi")
```

## Shrinking a failure bundle

**shrink.run.1** — `ail replay --shrink [--time MS] [-o OUT] BUNDLE` takes a failure
bundle (`bundle.write.1-2`). A file that is not a bundle, or a bundle whose test does
not fail when replayed (`bundle.run.1`), is an error (exit 2) and nothing is written
([ADR 0920](../decisions/0920-replay-shrink.md)).
**shrink.run.2** — The texts the recorded calls returned (a file read, an
environment value, a process's output) are cut line by line, largest cuts first. A
cut stays when `ail test --bundle` still fails with the same `FAIL` line, test name
and message included, as the unshrunk bundle; otherwise it is undone. Numbers,
booleans and the order of calls are never changed.
**shrink.run.3** — The search stops after `--time` milliseconds (default 60000),
counted from the first cut, and keeps the cuts made so far. The limit is checked
between tries, so a run can overrun it by the try in progress and the first replay
that proves the bundle fails.
**shrink.run.4** — When anything was cut, the smaller bundle is written to OUT
(default: BUNDLE with `.min` added) and the command prints `shrunk`; when nothing
could be cut it writes nothing and prints `nothing to cut`. Both exit 0.

```ail tool=replay,--shrink,nosuch.json status=2 expect=shrink.run.1 rules=shrink.run.1,shrink.run.2,shrink.run.3,shrink.run.4
```

## Inferred contracts

`ail infer --contracts FN FILE` proposes `requires` and `ensures` lines for one
function from the calls the tests make
([ADR 1023](../decisions/1023-ail-infer-contracts.md), which builds
[ADR 0895](../decisions/0895-ail-infer.md)). The conditions are observed, not
proved; [contract.fuzz.1](20-contracts.md) fuzzes them afterwards.

**infer.cand.1** — `ail infer --contracts FN FILE [--apply] [--min N] [--cases N] [--json]`
runs the tests of FILE (the property tests too, `--cases N` of them) against a copy
of FILE in which FN records each call. A call is one row: the value of each `int`
argument, the length of each `str` and `List` argument, and the same for the result.
Other types are not recorded. FN is a top-level function of FILE whose signature is on
one line; a generic or fallible function, one with a `mut`, `move` or default
parameter, or none of that name, is an error with status 2 that cites this rule.

**infer.cand.2** — A condition is proposed only when at least N calls were seen
(default 20). With fewer, nothing is proposed and the answer says how many were seen.

**infer.cand.3** — A condition is kept only when every call satisfies it. The forms are,
for a recorded value `v`: `v > 0`, `v >= 0` (an `int`), `v != 0` (an `int` seen with
both signs); and for two recorded values `a` and `b`: `a < b`, `a <= b`, `a > b`,
`a >= b`. A value that took one value in every call is not proposed. The conditions
about parameters are `requires`; those about `result` are `ensures`, and a function
with a parameter named `result` gets none.

**infer.cand.4** — The conditions are ordered: `requires` on two parameters,
`requires` on one, `ensures` on `result`, `ensures` comparing `result` with a
parameter. A range on `result` that a comparison already gives, as `result > n` with
`n > 0` does, is left out.

**infer.cand.5** — At most 3 conditions are proposed: the first 3 of that order.

**infer.cand.6** — A condition already written as a `requires` or `ensures` line under
the signature is not proposed again.

**infer.out.1** — The patch goes to standard output in `ail edit` hunk form (`edit.in.3`):
the function's name, its signature line as context and a `+` line for each condition, so
`ail edit FILE < patch` applies it. Standard error has one line per condition with its
call count. With `--json` the answer is one object: `infer`, `fn`, `calls`, `min`,
`applied`, `note`, `proposals` (each with `kind`, `cond` and `calls`) and `patch`.

**infer.out.2** — Before anything is shown the patch is dry-run through
`ail edit --dry-run`. If the edited program does not check, nothing is proposed, the
errors are shown and the status is 2.

**infer.out.3** — With `--apply` the patch is given to `ail edit FILE`, which formats the
file, checks it and writes it; the answer is its answer. Without `--apply` nothing is
written.

**infer.out.4** — The answer says the conditions are observed in tests, not proved. The
status is 0 when the tests ran, whether or not anything was proposed, and 2 for a usage
error or an error in FILE.

```ail tool=infer,--contracts,pay,FILE,--json expect="kind":"requires" rules=infer.cand.1,infer.cand.3,infer.cand.4,infer.out.1
fn pay(n int) int
    n + 1
@test fn pays()
    for i in 1..30
        assert_eq(pay(i), i + 1)
fn main()
    print(pay(1))
```

```ail tool=infer,--contracts,pay,FILE,--json expect="cond":"result rules=infer.cand.3,infer.cand.4,infer.cand.5
fn pay(n int) int
    n + 1
@test fn pays()
    for i in 1..30
        assert_eq(pay(i), i + 1)
fn main()
    print(pay(1))
```

```ail tool=infer,--contracts,pay,FILE expect=proved rules=infer.out.2,infer.out.4
fn pay(n int) int
    n + 1
@test fn pays()
    for i in 1..30
        assert_eq(pay(i), i + 1)
fn main()
    print(pay(1))
```

```ail tool=infer,--contracts,pay,FILE,--apply expect=edited rules=infer.out.3
fn pay(n int) int
    n + 1
@test fn pays()
    for i in 1..30
        assert_eq(pay(i), i + 1)
fn main()
    print(pay(1))
```

```ail tool=infer,--contracts,pay,FILE,--min,100 expect=needed rules=infer.cand.2
fn pay(n int) int
    n + 1
@test fn pays()
    for i in 1..30
        assert_eq(pay(i), i + 1)
fn main()
    print(pay(1))
```

```ail tool=infer,--contracts,pay,FILE expect=holds rules=infer.cand.6
fn pay(n int) int
    requires n > 0
    ensures result > n
    n + 1
@test fn pays()
    for i in 1..30
        assert_eq(pay(i), i + 1)
fn main()
    print(pay(1))
```

```ail tool=infer,--contracts,pay,FILE status=2 expect=infer.cand.1 rules=infer.cand.1
fn pay[T](n T) T
    n
fn main()
    print(pay(1))
```

```ail tool=infer,--contracts,nosuch,FILE status=2 expect=infer.cand.1 rules=infer.cand.1
fn pay(n int) int
    n + 1
@test fn pays()
    for i in 1..30
        assert_eq(pay(i), i + 1)
fn main()
    print(pay(1))
```

## Proved contracts

`ail prove` lists the contract checks the checker proves and leaves out
([contract.prove.1-3](20-contracts.md),
[ADR 1055](../decisions/1055-contracts-proved-by-interval-facts.md)).

**prove.cmd.1** — `ail prove [--json] FILE|DIR` loads and checks the program as
`ail check` does; a program with errors prints them and exits 2, as does any
other flag or a missing operand. Otherwise it prints one line per check the
prover looked at, function by function in the order they are written: for a
`requires` at a call, ``FILE:LINE:COL requires `COND` of `F` in `G`: VERDICT``
at the call, where `G` is the calling function; then for each `ensures` of a
top-level function, ``FILE:LINE:COL ensures `COND` of `F`: VERDICT`` at the
condition. `VERDICT` is
`proved` or `checked` (still evaluated at run time). The last line is
`N of M contract checks proved`. The status is 0.

```ail tool=prove,FILE expect=proved rules=prove.cmd.1
fn at(xs List[int], i int) int
    requires i < xs.len()
    xs[i]

fn main()
    xs := [3, 1, 4]
    for i in 0..xs.len()
        print(at(xs, i))
```

**prove.cmd.2** — With `--json` the answer is one object: `ail_api` (1),
`prove` (1), `proved`, `total`, and `contracts`, a list with, for each line of
`prove.cmd.1`, `kind` (`requires` or `ensures`), `fn`, `cond`, `at` (the call
or the condition), `contract` (the condition), `in` (the calling function) and
`proved`.

```ail tool=prove,--json,FILE expect="proved":true rules=prove.cmd.2
fn digit(n int) int
    requires n >= 0
    ensures result < 10
    n % 10

fn main()
    print(digit(7))
```

## Trying patches

`ail try PROGRAM [--set FILE]... [--apply]` checks and tests patches without
touching the program ([ADR 0893](../decisions/0893-ail-try.md)). The first
slice works on a scratch copy; an in-memory overlay is a later speed-up with
the same output.

**try.in.1** — A patch set is `ail edit` input (`edit.in.1`, `edit.in.3`).
Each `--set FILE` is one set, in order; with no `--set` the one set is read from
standard input. No set, an unknown option or a missing `PROGRAM` is an error:
exit status 2.

**try.in.2** — `--sets-json` reads the sets from standard input as one JSON array, one set per element: a string is the set's text, any other value is sent as its JSON text. The labels are `1`, `2`, ... in the verdict lines. It takes no `--set` and no `--apply`, and an input that is not a non-empty array is an error: exit status 2. `ail mcp` uses it for `ail_try`.

**try.run.1** — Each set is applied to its own copy of `PROGRAM` (a file or a
directory) under `.ail/try/N` with `ail edit --if-clean` (`edit.out.2`), so the
check is the one `ail edit` runs. The copies are removed when the set is done,
and `PROGRAM` is never written.

**try.run.2** — A set that checks is followed by `ail test` on the copy, the whole
program's tests (the affected set comes with the overlay). Tests that touch the
disk run against the copy's files.

**try.verdict.1** — One line per set, `set LABEL: check ok|N errors; tests N passed, M failed; patch ~T tokens`,
with `LABEL` the set's file name (`stdin` for standard input) and `T` the estimate of
`api.card.3` over the set. A set that doesn't check says `tests skipped`.

**try.verdict.2** — A set that is not clean adds one indented line: its first
diagnostic or its first failing test.

**try.verdict.3** — A set whose edit can't be applied (`edit.in.2`) says
`patch does not apply; check and tests skipped`, with the edit's error as the
detail.

**try.verdict.4** — Exit status is 0 when every set is clean (checks, all tests
pass) and 1 otherwise.

**try.apply.1** — `--apply` takes exactly one set and writes it to `PROGRAM` with
`ail edit --if-clean` only when its verdict is clean. Otherwise nothing is
written, the output says `not applied`, and the exit status is 1. With more than
one `--set`, exit status 2.

**try.apply.2** — If the real write fails after a clean verdict, the error is
printed and the exit status is 2.

```ail skip=bootstrap tool=try,--set,FILE rules=try.in.1,try.in.2,try.run.1,try.run.2,try.verdict.1,try.verdict.2,try.verdict.3,try.verdict.4,try.apply.1,try.apply.2
fn main()
    print(1)
```
