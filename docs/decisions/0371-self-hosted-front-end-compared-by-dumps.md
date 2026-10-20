# 0371. Compare the self-hosted lexer and parser with the bootstrap through text dumps

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture (Stage 3, ADR 0022)
- **Spec rules:** `api.tok.1–3`, `api.ast.1–3`

## Context

Stage 3 writes the front end in AIL (`compiler/`), run by the bootstrap,
with the Rust front end as the oracle (ADR 0032). The lexer and parser in
AIL must be shown equal to the Rust ones on every program, and each stage
must be testable alone, before the checker in AIL (lane 114) consumes the
parser's output. ADR 0376 fixes the syntax tree's types and JSON.

## Options considered

### Option A — the bootstrap prints each stage's output as text; compare bytes (chosen)
`ail api --tokens FILE` lists the Rust lexer's tokens one per line;
`ail api --ast FILE` prints the Rust parser's tree of that one file as the
derived JSON of `compiler/ast/`. The AIL stages print the same text, and
the selftest compares the two on every `.ail` file in the repository.
- **Pros:** a mismatch is a line diff; each stage is tested alone; the
  dumps are useful to agents and tools on their own; no Rust code reads AIL
  values.
- **Cons:** the dump formats are one more interface to keep stable.

### Option B — run the AIL stage inside the bootstrap and compare values in Rust
- **Cons:** needs a bridge from interpreter values to Rust's AST; couples
  the oracle to the thing it checks.

### Option C — compare only end results (the program's output)
- **Cons:** misses positions, recovery and diagnostics; a wrong tree can
  still run.

## Decision

Option A. The token listing is one line per token, `LINE:COL KIND
[VALUE]`, then the comments, then the diagnostics as `LINE:COL diag RULE`.
`ail api --ast FILE` is the parse of the file alone: no prelude, no imported
modules, none of the passes after parsing (implicit `self`, inherited
contracts, derive output, the standard declarations), so the AIL parser
can match it exactly. `ail api --ast --program FILE|DIR` is the assembled
program the checker sees. Diagnostics are compared by rule and position,
not by message, before the one-per-line filter (`gram.rec.1`).

## Rationale

AI understanding (ADR 0076 criterion 2): a byte diff names the first wrong
token or node, with its position, and needs no tooling to read. Option A
also costs the least code.

## Consequences

- `compiler/syntax/` holds the lexer and parser, a module (not the root),
  so the checker's module can import them.
- The selftest runs the AIL lexer and parser over the repository; a change
  to the Rust front end that the AIL one doesn't follow fails it.
- Messages and fixes of syntax diagnostics stay in Rust for now; matching
  them is later Stage 3 work.

## Rejected alternatives

- **B:** couples the oracle to the implementation under test, and needs a
  value bridge that would be thrown away.
- **C:** too weak: positions and error recovery would go unchecked.
