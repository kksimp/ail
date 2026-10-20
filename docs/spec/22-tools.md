# 22. Tools: api, explain, fix, budgets, diff, edit, replay, snapshots

Chapter keys: `api`, `explain`, `fix`, `budget`

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
