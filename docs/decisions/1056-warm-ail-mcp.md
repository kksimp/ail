# 1056. Keep `ail mcp` warm: direct native calls and a memo of unchanged reads

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076). Slice E14 of `lanes/ai-first-edge-2026-11-10.md`; in 0.1.0 by the designer's decision of 2026-11-10.
- **Topic:** 20 — AI semantic API
- **Spec rules:** mcp.warm.1, mcp.warm.2, mcp.warm.3, mcp.warm.4
- **Amends:** [0918](0918-ail-mcp-compiler-tools.md)

## Context

Every `ail mcp` tool call runs `./ail CMD`: a shell (PowerShell 5.1 on Windows) that scans
`compiler/` and `std/` for files newer than the native CLI, then execs it. The native CLI then
loads and checks the program. An agent's loop repeats calls: it checks, asks two questions,
checks again before editing. Tokens and rounds don't change; wall time does.

ADR 0918 rejected calling the checker in process ("a second code path to keep equal to the
CLI"). The checker is whole-program (prelude, std and the program are one token stream), so a
kept graph would be rebuilt after any edit anyway.

## Options considered

### Option A — skip the launcher and memoize unchanged reads (chosen)
- **Pros:** no second code path: the answer is the native CLI's own. Removes a shell start and a
  tree scan per call (a PowerShell start on Windows). An unchanged read-only call costs only
  reading the program's files. Lives in `tools/ailmcp/`; the launchers gain one variable each.
- **Cons:** a check after an edit still checks the whole program.

### Option B — a resident compiler (`ail serve`) keeping the loaded program
- **Pros:** could keep the parsed prelude and std between calls.
- **Cons:** every command prints and exits; capturing that in process means reworking each
  command, and it is the second code path ADR 0918 rejected. Unmeasured gain.

### Option C — speculative checks after each edit
- **Pros:** the next `ail_check` would often be ready.
- **Cons:** spends CPU on checks nobody asks for, and misses edits made outside the server.

## Decision

`ail mcp` runs the native CLI directly for the commands the launcher only passes on, and answers
`ail_check`, `ail_ask`, `ail_context`, `ail_api`, `ail_explain` and a dry-run `ail_fix` from a
memo when the call, the toolchain and the program's files are unchanged. `AIL_MCP_COLD` turns
both off; `_meta["ail/warm"]` says which path answered.

## Rationale

Speed is the only criterion that moves (ADR 0076, criterion 4). Option A takes the part of the
cost that is pure overhead without touching what a call answers. The fingerprint hashes file
contents, not times, so an edit within one clock tick is still seen.

## Consequences

- `tools/ailmcp/bench.sh` measures cold against warm on the gate hosts.
- A package that names a path above its root, or a root of more than 4000 files, is never
  memoized; a toolchain newer than the native CLI goes through the launcher, which rebuilds it.
- A resident compiler (Option B) stays open until the bench shows the check itself dominates.

## Rejected alternatives

- **Option B, resident compiler:** a second code path (ADR 0918) for a gain nobody measured.
- **Option C, speculative checks:** CPU for unasked work, and blind to edits made elsewhere.
- **Fingerprint by modification time:** cheaper, but misses a same-size edit within one tick.
- **Memoize `ail_test`, `ail_verify`, `ail_impact`, `ail_review`:** tests can depend on time and
  the system, and the others on git state the fingerprint doesn't cover.
