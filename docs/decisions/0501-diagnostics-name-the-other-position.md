# 0501. Diagnostics point at the other position; the rule ID stays the stable symbol

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 2 — Diagnostics
- **Spec rules:** none changed; the diagnostics use the existing rules `expr.bind.3` and `expr.block.2`
- **Criterion that decided:** token usage (ADR 0076, criterion 1), with the
  designer's request for notes as the requirement

## Context

The batch brief asks for, per commonly hit diagnostic: a concrete help, a
`notes` entry at the other position, a stable `symbol`, and an `ail explain`
entry. Measurement on the mistakes corpus (`bootstrap/tests/mistakes/`, 140
programs, `tools/tokbench/run.sh --json`, JSON diagnostics of `ail check`)
found that every corpus program's first diagnostic already has a `help`
(ADR 0262, `gram.help.1`), so the help part was done.

## Decision

1. **Notes on two checker diagnostics.** `expr.bind.3` (an immutable binding
   is assigned or mutated) adds a note at the binding's declaration, "`n` is
   declared here". `expr.block.2` (a body ends with the wrong type) adds a
   note at the declared return type, "the return type is declared here".
2. **No `symbol` field.** A diagnostic's rule ID (`expr.bind.3`) is already
   stable, spec-cited and unique per kind (ADR 0002). A second key per
   diagnostic would add tokens to every diagnostic and carry no new
   information. The explain entry (`ail explain RULE`) is keyed by the rule.

## Measured cost (cl100k, the same 140 programs)

| | Diagnostics JSON tokens | Bytes |
|---|---|---|
| Before (this branch, before the notes) | 15 537 | 49 343 |
| After the two notes | 15 637 (+100, +0.6%) | 49 666 |

The notes are the only increase: about one line per diagnostic that gains a
note. Under criterion 1 this is a small loss, accepted because an agent
otherwise has to search for the declaration.

## Rejected alternatives

- **A `symbol` field with a second name per diagnostic.** Rejected: +tokens
  on every diagnostic for no new information (see decision 2).
- **Notes on every diagnostic that has a second position.** Rejected for this
  slice: most second positions need the parser to keep positions it drops
  today. Left for later slices, one rule family at a time.
- **Rewriting the `expr.name.1` messages to name the fix.** Rejected: the
  `help` already names the fix, and repeating it in the message adds tokens to
  every such diagnostic.

## Ports

`expr.bind.3` and `expr.block.2` are not in `compiler/check/coverage.ail`
(`COVERED_RULES`), so the AIL checker does not emit them yet and the
differential test does not compare their messages. The port happens with the
ownership and body passes (self-hosted checker, ADR 0380).

## Consequences

- `ail explain` entries for the 30 most-hit diagnostics (the brief's remaining
  item) are not in this slice.
- The language card and the quick reference are unchanged.
