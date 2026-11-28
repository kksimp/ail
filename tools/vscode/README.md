# AIL for VS Code

Syntax highlighting for `.ail` and a client for `ail lsp` ([ADR 0966](../../docs/decisions/0966-ail-lsp-language-server.md)):
diagnostics with rule IDs (the diagnostic code), hover, go to definition, references, document symbols,
completion, rename, quick fixes and a "fix all" action. Nothing here runs the compiler itself; the server is `ail lsp`.

## Use from a clone

```sh
cd tools/vscode && npm install
code --extensionDevelopmentPath="$PWD" ../../examples
```

Open `examples/json.ail`. The extension uses `ail.path` if set, else the `ail` script at the workspace
root, else `ail` on PATH, and passes the same path to the server as `AIL` (the server shells out to it).
Settings: `ail.path`, `ail.trace.server`, `ail.debug` (sets `AIL_LSP_DEBUG=1`). Commands: *AIL: Restart
Language Server*, *AIL: Show Server Log*.

## Check

```sh
npm test        # scope cases, every .ail file in the repository, and the keyword list in compiler/syntax/lex.ail
npm run test:sublime   # ail.sublime-syntax matches the grammar (needs python3 and pyyaml)
```

Smoke test in VS Code (examples/json.ail): break a line and see a squiggle whose code is a rule ID
(for example `lex.kw.1`); hover a call; F12 on a function; Shift+F12 for references; F2 rename; Ctrl+. for a quick fix.

## Package and install locally

```sh
cd tools/vscode && npm install
npx vsce package                              # writes ail-VERSION.vsix (VERSION from package.json)
code --install-extension ail-VERSION.vsix
```

Not published to the Marketplace (that is a publishing decision for the designer).

## Grammar and GitHub

`syntaxes/ail.tmLanguage.json` (scope `source.ail`) is a plain TextMate grammar, so it also works for
GitHub linguist. Linguist only accepts a language once it is in use in enough public repositories; the
grammar is ready for that pull request. It follows [lexical structure](../../docs/spec/02-lexical.md):
keywords `lex.kw.1`, comments `lex.comment.*`, strings, raw strings, bytes and interpolation `lex.str.*`,
numbers `lex.num.*`. Rule IDs written in comments highlight as `constant.other.rule-id.ail`. Format specs after `:` in
an interpolation (`{n:#>8x}`) are `constant.other.format-spec.ail`, not code.

`syntaxes/ail.sublime-syntax` (Sublime Text, bat, syntect) is generated from the JSON grammar by
`python3 gen-sublime.py`; edit only the JSON and regenerate. `test/known_bad.txt` lists the repository's
.ail files that exist to be rejected by the lexer; the grammar must flag exactly those.

Not done: formatting through `ail fmt` (the server has no formatting method yet), semantic tokens,
snippets, debugging.
