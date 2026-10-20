# 0373. The self-hosted front end reports a rule and a position, and runs the corpus in one process

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** `api.tok.2`, `api.ast.2`

## Context

The lexer and parser written in AIL (`compiler/syntax/`, ADR 0371) are
compared with the bootstrap's through `ail api --tokens` and `ail api
--ast`, whose diagnostic lines carry a rule and a position only. The Rust
diagnostics also carry a message, help and fixes, about a third of the
parser's source. Running the AIL front end once per file took 0.2 s each,
about 2.5 minutes for the corpus, mostly checking `compiler/syntax/` again.

## Options considered

### Option A — rule and position now, one run over the corpus (chosen)
`Diag` holds the rule, the position, and, for a diagnostic with a fix, the
message, which `gram.rec.1` needs to merge a repeated mistake on one line.
`compiler/selfhost.ail tokens|parse FILE...` takes every file, with a line
`== FILE` before each output.
- **Pros:** the comparison is exact where the dumps are; the selftest
  runs both stages over 770 files in about 20 s.
- **Cons:** the AIL front end can't yet print the messages users see.

### Option B — port messages, help and fixes with the grammar
- **Cons:** about 1,000 more lines that the dumps can't check, before the
  grammar itself agrees.

## Decision

Option A.

## Rationale

AI understanding (ADR 0076 criterion 2): every ported line is checked
against the oracle (ADR 0032). Messages and fixes come with the driver that
prints them, when `ail check` runs on the AIL front end, and the dumps then
gain the message text.

## Consequences

- The AIL front end's diagnostics are complete for `ail api`, not for `ail
  check`'s rendered output.
- The selftest compares both stages on the whole corpus in one process
  each.

## Rejected alternatives

- **B:** unchecked code ahead of the parts that can be checked.
