# 0966. `ail lsp`: the semantic API as a language server

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076), from the designer's goal that AIL replace C, C++ and Rust (2026-11-05); number allocated by the integrator
- **Builds on:** [ADR 0918](0918-ail-mcp-compiler-tools.md) (`ail mcp`, the same shell-out shape), ADRs 0258-0261 (`ail api --type|--refs`)
- **Topic:** 20 — AI semantic API

## Context

`ail-edit` is a custom editor. Every other editor (VS Code, Neovim, Helix, Zed, JetBrains) reaches a language through the Language Server Protocol, so without one AIL has no way into the editors C, C++ and Rust developers already use.

## Decision

- `ail lsp` runs an LSP server on stdio, written in AIL (`tools/ailsp/ailsp.ail`). Methods: `initialize`, `shutdown`, `exit`, `textDocument/didOpen|didChange|didSave|didClose`, `hover`, `definition`, `references`, `documentSymbol`.
- It adds no analysis. Diagnostics are `ail check --json`, hover `ail api --type`, definition and references `ail api --refs`, symbols `ail api --outline`. The checker stays the only checker (`compiler/check/`).
- A diagnostic's `code` is its stable rule id, so an agent or human runs `ail explain RULE` on it. Fixes are not offered as code actions in version 1.
- Unsaved buffers are overlaid on a scratch copy of the document's directory (lsp.sync.2), so hover and definition are right between saves.
- Framing: standard input has no read-exactly-N call, so a body is read as a line and cut by its byte count; the remainder is the next header. This relies on bodies without raw newlines, which every client sends (JSON escapes them). Output counts the newline `print` adds.
- Rule keys: `lsp.proto.*`, `lsp.sync.*`, `lsp.diag.*`, `lsp.hover.*`, `lsp.def.*`, `lsp.sym.*`, `lsp.pos.*`.

Cost when unused (ADR 0868): none. It is a separate tool run only by `ail lsp`.

## Consequences

Each request costs a process start (~tens of ms on the native CLI). Diagnostics are on open and save, not per keystroke, so the server never queues behind a slow check. Completion, rename, formatting and code actions are follow-ups; each is one more `ail` command behind one more dispatch line.

## Amendment (2026-11-06): completion, rename and code actions

Decided by: AI (delegated, ADR 0076). Approved by the designer as a follow-up.

- **Code actions** (lsp.action.1-2): the `fixes` of `check --json` become `quickfix` actions, plus one `source.fixAll.ail` that takes the first non-overlapping fix of each diagnostic. This supersedes "fixes are not offered as code actions in version 1".
- **Rename** (lsp.rename.1-2): the references of `api --refs` are the edit. No second resolver. The result is not re-checked.
- **Completion** (lsp.complete.1-2): `api --locals`, `--outline` and `--type` on a scratch copy with the word at the cursor taken out, because a buffer being typed does not check; the last good outline answers when the copy still fails. Members are those of types declared in the file; standard types give none until `api` exposes the standard cards by type.
- Each completion, rename or action request costs one to three `ail` processes, as the other requests do. The server also runs `api --outline` after each open or save to keep the fallback outline.
- Not done: formatting, workspace symbols, signature help.

Rejected: completing from a tokenizer scan of the buffer (a second analysis path, ADR 0918 reasoning); running `ail fix` for fix-all (it writes files; an edit list lets the editor undo).

## Rejected alternatives

- **Call the checker in process:** faster, but a second path to keep equal to the CLI (ADR 0918 reasoning); `load_with` already allows it later without a protocol change.
- **Diagnostics on every change:** a synchronous server would block hover behind a check; revisit with a task per request.
- **A Rust or C server:** the toolchain is in AIL (ADR 0022).
- **A `std.io.read_exact` first:** the right fix, but a runtime change; recorded in open questions for the std lane.
