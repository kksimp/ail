# 1054. `ail edit` refactors: change a signature, extract, move

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 22 — Tools; AI semantic API
- **Spec rules:** `edit.op.8`, `edit.op.9`, `edit.op.10`

## Context

Slice E10 of the AI-first plan of 2026-11-10. `ail edit` (ADRs 0397, 0399)
renames across a program, but every other refactor costs an agent one edit
per call site: adding a parameter to a function with twenty callers is twenty
hunks, each read first. The checker already knows every reference
(`api.ref.1`) and the type of every local (`api.loc.1`), so the tool can do
the per-site work.

## Options considered

### Option A — three ops in `ail edit`, driven by the checker's references (chosen)
- `change_sig`: the new parameter list (kept names, retyped `n int`, added
  `name Type` with an argument expression); every call is rewritten.
- `extract`: body lines, named by their text, become a function; parameters,
  `mut` and results come from the checker's locals.
- `move`: a function, constant or type (with its methods) goes to another
  module; qualifiers and `use` lines follow.
- **Pros:** one short JSON object per refactor; same all-or-nothing,
  format-then-check pipeline as the other ops; no new command, so no launcher
  arms; reachable through MCP's `edit` tool.
- **Cons:** text rewriting at reported positions, so odd layouts (a comment
  inside an argument list) are refused rather than handled.

### Option B — a separate `ail refactor` command
- **Pros:** room for more options.
- **Cons:** a new command needs arms in `ail` and `ail.ps1`, MCP and docs,
  and duplicates the edit pipeline (load, format, check, diff, `--test`).

### Option C — rewrite through the AST and print it back
- **Pros:** exact for any layout.
- **Cons:** the formatter works on text, not trees; printing a tree back
  loses comments. Much larger.

## Decision

`ail edit` gains `change_sig`, `extract` and `move` (`edit.op.8-10`), in
`compiler/edit/refactor.ail`. Like `rename`, each needs the program as edited
so far to check, rewrites only at places the checker reports, and fails the
whole batch with a message when a place can't be rewritten.

## Rationale

Token use (criterion 1) decides: a parameter added across a package is one
edit instead of one per caller, and the agent never reads the call sites.
AI understanding (2): the result goes through `ail check` like every edit, so
a refactor that breaks a caller is reported, not silent.

## Consequences

- `change_sig` writes every argument positionally in the new order; labels
  are dropped. A removed parameter's argument is dropped with it, side
  effects included. Calls through a function value are refused.
- `extract` refuses lines that `return`, or `break`/`continue` a loop outside
  them, and lines that don't form whole statements. A `mut` local the lines
  may change becomes a `mut` parameter (an over-approximation: any method
  call on it counts). Inside a method with `self` the new function is a
  method with the same receiver.
- `move` refuses a type with an `impl`, a body that names a non-`pub`
  declaration of its old module, and a body that names the root module's
  declarations. Import cycles it creates are reported by the check.
- Measuring it is an agentbench `fix` task that adds a parameter across a
  package (E3).

## Rejected alternatives

- **`ail refactor` (B):** a second edit pipeline and launcher arms for no gain.
- **AST printing (C):** loses comments and needs a printer the formatter doesn't have.
