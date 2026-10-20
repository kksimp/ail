# Garlic's JavaScript engine

> **Non-normative.** Design notes for the JavaScript engine of Garlic, the
> web browser written in AIL. The code is `tools/garlic/js/`. It is a tool,
> not part of the AIL language, so its choices are recorded here, not in
> ADRs. Designer's decision (2026-10-20): write an in-house engine now,
> in parallel with the browser's text-mode milestone.

## Goal

Run the scripts of real pages inside Garlic, written entirely in AIL and from
the ECMAScript specification (ECMA-262), with every change a script makes to
the page visible in Garlic's page model, so that an agent can see what scripts
did.

## Slices

1. **Lexer and parser** (done): scripts to an ESTree-shaped tree.
2. **Interpreter**: values (undefined, null, boolean, number as `float`,
   string, object with a prototype chain, function with closures), property
   lookup, `this`, `new`, exceptions, and the core built-ins (`Object`,
   `Function`, `Array`, `String`, `Number`, `Math`, `JSON`, the `Error` types).
   It has run limits like `ail run --max-steps`.
3. **DOM bindings** to Garlic's document tree: `getElementById`,
   `querySelector` with simple selectors, `textContent`, `createElement`,
   `appendChild`, `setAttribute`, the `click` and `DOMContentLoaded` events,
   and `setTimeout` on a deterministic event loop. Each script mutation is
   recorded against the node it changed.
4. **Later**: generators, async functions and promises, modules, BigInt, and
   a bytecode compiler.

## Layout

```
tools/garlic/js/
  main.ail          command line: --tokens, --ast [--loc], --check, --test, --test262
  suite.ail         runs the packed test262 parser tests
  syntax/           module `syntax`: lexer, tree, parser, dumps
    lex.ail         ECMA-262 §12 (lexical grammar)
    unicode.ail     ID_Start and ID_Continue ranges (Unicode 16.0)
    ast.ail         the tree
    parse.ail       §13-§16 (expressions, statements, functions, classes, scripts)
    dump.ail        ESTree JSON
    text.ail        Number::toString and JSON string quoting
  testdata/
    syntax/         NAME.js with NAME.ast (tree) or NAME.err (SyntaxError)
    tokens/         a lexer sample and its tokens
    test262-parser/ the tc39 parser tests, packed, with their licences
```

Garlic itself imports the parser as `use js.syntax` from `tools/garlic/`.

```sh
./ail run tools/garlic/js --ast page.js          # ESTree JSON, one line
./ail run tools/garlic/js --ast --loc page.js    # with "loc":"LINE:COL" on each node
./ail run tools/garlic/js --check a.js b.js      # "ok" or the SyntaxError, per file
./ail run tools/garlic/js --tokens page.js       # LINE:COL KIND TEXT per token
```

## Design choices

**The parser drives the lexer.** Whether `/` starts a regular expression and
whether `}` resumes a template depend on the grammar (§12, goal symbols). The
parser pulls tokens one at a time, with one token of lookahead (rarely two),
and rescans a `/`, `/=` or `}` when it is at that point in the grammar. This is
exact, unlike tokenising first with the "previous token" heuristic. The
standalone `--tokens` dump still uses that heuristic, with a paren stack so
that `if (a) /re/` is read correctly. It is for inspection only.

**Covers become patterns.** `[a, b] = c`, `({a = 1} = o)` and `(a, {b}) => a`
are parsed as expressions and converted, following the spec's cover grammars
(§13.2.5, §13.15.1, §15.3). A shorthand `{a = 1}` is valid only once its object
becomes a pattern. Pending ones are kept on a list and reported as "Invalid
shorthand property initializer" if no conversion claims them.

**Parentheses are kept internally** as a `Paren` node, so `(a) = 1` (valid)
and `({a}) = 1` (invalid) can be told apart. The ESTree dump drops them.

**The first error stops the parse.** Every JavaScript engine reports one
`SyntaxError` per script (§16.1.6), and pages expect that. Messages follow
V8's wording, so agents and people see the text they already know.

**Early errors that need scopes are checked while parsing.** Each block,
function, `for` head, `switch` and `catch` gets a scope of `var`, lexical and
function names. A clash such as `let a; var a` is reported where it happens,
with the Annex B allowances: sloppy duplicate block functions,
`catch (e) { var e }`, and `for (var x = 1 in o)`.

**ESTree for the tree's JSON.** Most JavaScript tools share ESTree, so an
agent can read `--ast` output without learning a new schema. That is also what
let the parser be checked node for node against a reference parser.

**Source is Unicode scalars, not UTF-16.** AIL strings are UTF-8 and `char` is
a Unicode scalar value. A lone surrogate written as an escape (`"\uD800"`)
becomes U+FFFD, and a surrogate pair written as two escapes becomes one code
point. JavaScript's `length` and indexing count UTF-16 code units, so the
interpreter will need a string representation that can hold lone surrogates
and count in code units. That choice is open for slice 2. Pages almost never
contain lone surrogates.

**Identifiers use generated Unicode tables.** AIL's `is_alpha` doesn't cover
ID_Continue (combining marks, connector punctuation). `unicode.ail` holds the
ranges of ID_Start and ID_Continue above ASCII, generated from the Unicode
Character Database (Unicode licence, compatible with Apache 2.0), and looks
them up by binary search.

**Numbers print as JavaScript prints them.** `number_to_string` implements
Number::toString (§6.1.6.1.20) on top of AIL's shortest round-trip float
printing. The interpreter will use it too.

## Not yet

- Generators, `async` functions and `await`, `for await`: a clear
  "not supported yet" SyntaxError (slice 4).
- Modules (`import`, `export`): scripts only, as in a classic `<script>`.
- BigInt literals (`10n`).
- Regular expression pattern syntax (§22.2.1): the literal's body and flags
  are checked, and the pattern is checked when the regex engine compiles it.
- Private names are not checked against their class's declarations.
- Class `accessor` fields and decorators (not in ES2022).

## Testing

- `testdata/syntax`: 16 hand-written scripts, one per area (literals,
  templates, operators, ASI, declarations, functions, arrows, classes,
  objects, patterns, control flow, optional chaining, `new`, Annex B,
  Unicode identifiers, regex context), each with its ESTree tree, and 56
  invalid scripts with their exact SyntaxError.
- `testdata/test262-parser`: the tc39 [test262-parser-tests](https://github.com/tc39/test262-parser-tests)
  (commit `0e808c7`, 2021-05-27), scripts only, packed one test per line.
  Licences: Apache 2.0, MIT and BSD, listed per file in its `licenses.md`.
  Every `pass` script must parse to the tree whose fingerprint is in
  `trees.txt`, and to the same tree as its `pass-explicit` twin. Every
  `fail` and `early` script must be rejected. `known.txt` lists each
  exception and why: an unsupported feature, a twin that is not the same
  script, a test made stale by a later edition, or a gap. A listed test that
  starts to behave is reported, so the list only shrinks.
- `bootstrap/tests/tools.rs` (`garlic_js_parses_its_suites`) runs all of these
  in `./ail selftest`.

Before `trees.txt` was written, every tree was compared node for node with
acorn 8.14 (ECMAScript 2022, script goal), outside the repository. Results on
2026-10-20:

| Corpus | Result |
|---|---|
| test262 `pass` (1,907 scripts) | 1,822 identical trees; 85 use generators or async |
| test262 `fail` (682) | 623 rejected; 54 use unsupported features; 4 regex patterns accepted; `new super()` rejected where acorn accepts and the suite agrees with us |
| test262 `early` (605) | 513 rejected; 88 unsupported; 1 regex pattern; 3 rejected where acorn accepts and the suite agrees with us |
| npm's own modules (1,210 files) | 935 identical trees; 255 use async; 19 rejected by acorn too |

## Speed

On the bootstrap interpreter (bytecode tier), Linux x86-64: acorn's own
238 KB `acorn.js` parses in 1.7 s, about 140 KB/s. Acorn on V8 takes 0.1 s.
The interpreter work (slice 2) and the native backend will move this. The
parser does no I/O and allocates mostly in lists, so it should compile well.

## Notes on writing it in AIL

What a 3,700-line AIL program ran into, for the language's designers:

- A variant whose fields share a type can't be built by position
  (`Binary(op, l, r, p)`), so tree constructors are written with names.
- A `match` arm can't be empty, and a `|` pattern can't bind names. "Is this
  node an X?" is written as `expr_kind(e) == "Ident"`, from one exhaustive
  `match` per enum, to avoid `_` arms that the checker warns about.
- `use` is per file, and a local named like a standard module (`path`) hides
  it.
- The rest was direct: `T ! E` with `?` makes "first error stops the parse"
  a one-character cost per call.
