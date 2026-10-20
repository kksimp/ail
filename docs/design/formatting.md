# Canonical formatting

Non-normative. This is the layout `ail fmt` writes ([ADR 0169](../decisions/0169-canonical-formatting-style.md),
[ADR 0170](../decisions/0170-ail-fmt-command.md)). A program's meaning never
depends on it beyond what the lexical rules already require (`lex.block.*`,
`lex.stmt.*`). Every example and doc sample should be in this form, so an AI
never spends tokens choosing a layout. The `fmt.*` IDs below are cited by the
formatter's code and messages; they are style rules, not spec rules.

## Rules

**fmt.indent**: a block is indented by 4 spaces (ADR 0040). A line continuing
a statement inside brackets is indented one level per open bracket; a line
starting with a closing bracket is indented one level less.

**fmt.stmt**: one statement per line, as the grammar already requires (`lex.stmt.3`).

**fmt.space**: one space between tokens, except:
- none after `(` `[` `{` `.` `@`, and none before `)` `]` `}` `,` `.` `:` `?`;
- none around `..` and `..=` (`xs[1..]`, `0..n`);
- none before `(` or `[` that follows an operand: a call, index, or type
  arguments (`f(x)`, `xs[0]`, `List[int]()`, `largest[T Ord](...)`);
- none after a prefix `-` (`-x`, `f(-1)`) or a prefix `!` in a type (`!Json`).
  `T ! E` keeps its spaces. Since the two `!`s differ only by where they sit
  in a type, the formatter keeps the source's choice of "no space after `!`".

So `x := -a + b * 2`, `Point(x: 1, y: 2)`, `fn f(a int, b int) !int`,
`'"' | '\\' => c`, `{"a": 1}`.

**fmt.wrap**: the line-length limit is 100 characters, indentation included.
A statement that fits is written on one line, whatever its source layout.
One that doesn't is split at the longest top-level list (call arguments,
parameters, list, map or set literal), one element per line. If that list's
last element ends in a list holding at least half its text, the split moves
into that inner list ("hugging"), so `Edit("name", [` ... `])` keeps its
head on one line. Elements still too long split the same way. Parenthesised
expressions, indexes and type arguments aren't split; a statement with
nothing to split stays long.

```ail
print(count(
    [Name(text: "x", id: 1), Name(text: "y", id: 2), Name(text: "x", id: 3)],
    Name(text: "x", id: 0),
))
```

**fmt.comma**: a list split over several lines has a trailing comma after its
last element; a list on one line has none (ADR 0118).

**fmt.blank**: one blank line separates two top-level declarations when
either spans more than one line. Between single-line declarations (constants,
`use`, empty `impl`s), and between statements in a block, a blank line from
the source is kept, and runs of blank lines collapse to one. No blank line
opens a block or the file, and the file ends with exactly one newline.

**fmt.comment**: every comment stays on the line it was on. A trailing comment
follows its statement after one space. A comment line takes the depth it was
written at, clamped between the depth of the statement after it and the
deepest block it could close; at the next statement's depth it belongs to that
statement, and a blank line before the statement goes before its comments. A
statement with a comment inside it keeps its source line breaks, indented
per fmt.indent, with trailing commas per fmt.comma. Comment text is not
changed.

**fmt.attr**: attributes stay on their declaration's line (ADR 0083). The
formatter doesn't yet move a long line's attributes onto their own line.

**fmt.newline**: lines end in `\n`; trailing whitespace is removed. A
statement containing a multi-line string (`lex.str.3`) is written exactly as
in the source, because its lines are the string's content.

**fmt.safe**: the formatter changes only whitespace, line breaks and trailing
commas. It formats only files that lex and parse, and refuses to write output
whose token stream (ignoring trailing commas) or comments differ from the
input's.

**fmt.off**: a file whose first line before any code is a comment starting
`# ail fmt: off` is left as written. It exists for lexer tests whose layout is
the point (CRLF line endings, continuation lines); programs shouldn't use it.

## Measurement

Formatting the seven examples (ADR 0169, `tools/tokbench/run.sh`, five
tokenizers, mean): 14,988.8 tokens before, 15,050.0 after, **+0.41%**. Almost
all of it is fmt.blank's blank lines in `json.ail` and `sheet.ail` (+0.5%
and +0.6%), which have none today; `interp.ail` gains 0.3% from three
statements over 100 characters being split. The other four examples were
already canonical.

## Not yet done

- Splitting a long line with no list in it (a long binary expression or
  string) at an operator.
- Moving attributes onto their own line when a line is too long (ADR 0083).
- Formatting files that don't parse.
- Normalising the spacing inside `{...}` in interpolated strings.
