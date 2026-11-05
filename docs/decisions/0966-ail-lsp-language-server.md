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

## Rejected alternatives

- **Call the checker in process:** faster, but a second path to keep equal to the CLI (ADR 0918 reasoning); `load_with` already allows it later without a protocol change.
- **Diagnostics on every change:** a synchronous server would block hover behind a check; revisit with a task per request.
- **A Rust or C server:** the toolchain is in AIL (ADR 0022).
- **A `std.io.read_exact` first:** the right fix, but a runtime change; recorded in open questions for the std lane.
