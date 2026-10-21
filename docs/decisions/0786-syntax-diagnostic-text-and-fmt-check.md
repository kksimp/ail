# 0786. The self-hosted front end prints diagnostic help and fixes; `fmt --check` fails on broken sources

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** tooling, diagnostics
- **Spec rules:** gram.foreign.1, gram.rec.1, fmt.safe

## Context

The self-hosted syntax front end (`compiler/syntax`) reported a diagnostic as a
rule, a position and a message. The bootstrap also prints the source line with
a caret, a help line and a fix line. The two texts differed for almost every
syntax error, so the self-hosted `ail` could not stand in for the bootstrap's
output. `ail fmt --check` also skipped any file in a directory that didn't
parse, so a broken source file could pass the format gate unnoticed (a stray
`&` in `compiler/rt/native/limits.ail` did).

## Decision

1. The AIL `Diag` carries `help` and `fix` (the fix title), and the self-hosted
   report path prints them as the bootstrap's text form does
   (`bootstrap/src/diag.rs` `to_text`). Fix edits are not carried: only the title
   is printed.
2. The report path keeps one syntax diagnostic per line (gram.rec.1, ADR 0263),
   as the bootstrap's load does. The api dumps (`--tokens`, `--ast`) keep every
   diagnostic, as they always did.
3. `ail fmt --check` fails on a file that doesn't parse, unless it is a negative
   test: it has an expectation file (`NAME.err`, `NAME.rules` or `NAME.trap`)
   beside it, or an `expected.*` file in its package directory or one of the two
   above. Outside check mode the skip stays silent, as before.
4. A test compares the self-hosted text with the bootstrap's on every mistake
   corpus program. Programs that still differ are listed in
   `bootstrap/tests/diag_text_known.txt`. The list may only shrink: a new
   divergence fails, and a listed program that now matches must be removed.

## Measurements

On the mistakes corpus (250 programs), 85 have syntax errors. Before this
change, 1 of those 85 matched the bootstrap's text. Most of the 84 that still
differ are bare `diag(rule, pos)` sites with no message, help or fix; those are
the next work.

## Rejected alternatives

- **Print the message only and leave help and fix to the bootstrap.** Rejected:
  the AI reads the self-hosted output, and the help line is the part it acts on.
- **Fail the whole parity test on any divergence.** Rejected: 84 divergences
  would make the selftest fail before the texts are written. The known list
  keeps the gate honest without blocking.
- **Treat every file that fails to parse as a failure in `fmt --check`.**
  Rejected: negative tests live in the same directories, and they must stay
  skipped.
- **Name negative tests by directory only.** Rejected: a file-level expectation
  is precise and needs no list.
