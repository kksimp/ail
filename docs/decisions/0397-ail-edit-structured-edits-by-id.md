# 0397. `ail edit`: structured edits by declaration id, formatted and checked

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 22 — Tools; AI semantic API
- **Amended by:** [ADR 0399](0399-compact-edits-and-diff-hunks.md)
- **Spec rules:** [edit.in.1–2, edit.id.1–2, edit.op.1–6, edit.out.1–3](../spec/22-tools.md#structured-edits)

## Context

An agent changes code through a text-editing tool: it quotes the exact old
text (with enough context to be unique) and the new text, once per site.
A rename across a package is one such edit per reference, after a search
to find them; replacing a function body quotes the whole old body. Each
edit can fail on whitespace, and none is checked until the agent runs
`ail check`. The plan (finish-plan.md §2.4) asks for structured edits by
symbol whose answer is the new state's diagnostics.

## Options considered

### Option A — edits on the syntax tree, printed back by the formatter
- **Pros:** exact.
- **Cons:** the formatter prints from tokens, not trees; comments would need
  re-attaching; a large new subsystem in shared files.

### Option B — edits on declaration spans found in the text, then `fmt` and `check` (chosen)
- **Pros:** works on files that don't parse or check yet (except `rename`);
  a declaration is a column-0 line plus its indented block, which AIL's
  layout makes unambiguous; the formatter normalises whatever the agent
  wrote; small: one module, `edit.rs`, and a `references` helper in `api.rs`.
- **Cons:** declarations nested in `trait` and `impl` blocks can't be
  targeted yet (top-level `fn Type.method` can); a column-0 line inside a
  multi-line string would end a span early (rare; the check then reports it).

### Option C — line-and-column edits (as `ail fix` applies)
- **Cons:** positions go stale after the first edit; the agent has to read
  the file to know them, which is the cost this feature removes.

Details decided with Option B:

- **Rename** uses the checker's references (`ail api --symbols`), so it
  renames uses, not text: a comment or a local with the same name is left
  alone. It needs the program to check first. Qualified references
  (`geo.origin`) point at their qualifier; the name is found after it.
- **All or nothing:** a failing edit writes nothing (`edit.in.2`), so an
  agent never sees a half-applied list.
- **Writing with errors:** the default writes and reports, because the next
  step is usually to fix them; `--if-clean` keeps the old files when the
  result has errors, for agents that want a transaction.
- **Contracts survive `replace_body`** unless the new body states its own:
  the common edit changes the implementation, not the contract.
- **Input** is JSON (a list, or one object), parsed by a small reader in
  `edit.rs`. Rejected: an AIL-like edit language (another syntax to learn
  and measure).

## Measurements

Estimated tokens (ADR 0311) the agent writes and reads, real code from
`tools/edit/edit.ail` (4,700 lines):

| Task | Old way | `ail edit` | Saved |
|---|---|---|---|
| Rename `shown_path` (6 sites) | 6 text edits quoting each line: 610 | one op and the answer: 63 | 90% |
| Replace a 12-line body | quote old body and new: ~2 × body + 30 | new body + id + answer: ~1 × body + 25 | ~45% |
| Add a function after another | quote an anchor (3 lines) + new text | new text + `after` id | ~20% |

The rename also needs no search for the sites, which the old way does
before its edits, and every result is formatted and checked in the same
call.

## Decision

Option B with the details above.

## Consequences

- Agents can make whole-program renames in one call and get the
  diagnostics back.
- Declarations inside `trait` and `impl` blocks, and local variables, are
  not edit targets yet; a later version can add them by the same spans.
- Windows: the command reads standard input and writes files through the
  standard library (ADR 0004).
