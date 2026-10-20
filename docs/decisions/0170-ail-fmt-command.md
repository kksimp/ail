# 0170. `ail fmt` reprints the token stream, and refuses any change but layout

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax (layout tooling)
- **Spec rules:** none (non-normative); `fmt.safe` and `fmt.off` in [formatting.md](../design/formatting.md)

## Context

The formatter must be idempotent and must never change a program's meaning.
The lexer dropped comments, and the AST keeps neither comments nor the
source spelling of literals (`1e3`, escapes, raw strings).

## Deciding criterion

Token usage doesn't apply (no syntax). **AI understanding decides**: a
formatter an agent can run without reviewing its output must be provably
layout-only, and its `--check` output must show exactly what would change.

## Options considered

### Option A — reprint the lexer's tokens with their source text (chosen)
The lexer gains `lex_with_comments`, a side table of comments; the token
stream the parser sees is unchanged. Each token's text is its exact source
slice, so literals keep their spelling. The parser is run only to reject
files with syntax errors.
- **Pros:** literal spelling is preserved by construction; a cheap, complete
  safety check is possible (re-lex the output and compare tokens); small,
  additive change to shared files.
- **Cons:** layout decisions use token context, not the syntax tree (for
  example prefix `!` in types, fmt.space).

### Option B — pretty-print the AST
- **Pros:** full syntactic context for every decision.
- **Cons:** the AST would need comments, literal spellings and source spans
  on every node, a large change to `ast.rs` and `parser.rs` while nine other
  lanes edit them.

## Decision

1. `ail fmt PATH...` rewrites each file (directories: every `.ail` file
   below, skipping hidden and `target` directories) in canonical layout
   ([ADR 0169](0169-canonical-formatting-style.md)).
   `--check` writes nothing, prints a unified diff for each file that would
   change and exits 1; `--stdout` prints the formatted text.
2. Exit status: 0 done or already canonical; 1 `--check` found a change;
   2 a file couldn't be read or formatted.
3. **fmt.safe:** only files that lex and parse are formatted. A file named
   on the command line that doesn't is reported with its diagnostics (exit
   2); one found in a directory is skipped, with a count, so negative test
   cases don't fail a repository-wide check. Before writing, the output is
   re-lexed: its tokens (ignoring trailing commas, positions removed) and
   comments must equal the input's, or the file is left unchanged and an
   internal error is reported.
4. **fmt.off:** a comment starting `# ail fmt: off` before the first statement
   keeps the file as written, for lexer tests whose layout is under test.

## Consequences

- Idempotence and the safety check are tested over every example and test
  case (`src/fmt.rs`), and formatting every example is tested not to change
  its `ail run` output (`tests/fmt.rs`).
- A later semantic API (lane 2) can expose `fmt` as a library call; the
  diff format is the conventional unified one, which agents already read.

## Rejected alternatives

- **AST pretty-printer (B):** too invasive for tonight's parallel lanes, and
  loses literal spelling unless every node carries its source text.
- **Formatting files with errors:** the safety check needs a reference token
  stream with no recovery placeholders; an agent fixing errors gets
  diagnostics, not a reformatted file.
- **A per-statement `fmt: skip` comment:** invites layout choices back; the
  file-level `off` is limited to lexer tests.
