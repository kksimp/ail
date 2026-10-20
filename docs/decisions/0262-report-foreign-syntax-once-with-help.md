# 0262. Report foreign syntax once, give every error a help, and measure it with a mistakes corpus

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI semantic API (diagnostics)
- **Spec rules:** `gram.foreign.1`, `gram.help.1`, `gram.help.2` ([03-grammar.md](../spec/03-grammar.md))

## Context

An AI learns AIL from its diagnostics while it writes (plan principle "read the result"). The mistakes it makes are mostly habits from Python, Rust, Go and TypeScript. An audit of 109 such programs ([diagnostics-audit.md](../design/diagnostics-audit.md)) found that only 55 of 109 first diagnostics had a `help`, 9 programs produced a cascade of several reports for one slip, three were `bootstrap.unsupported` for a construct AIL deliberately lacks, and `ail fix` resolved 9% of them. Criteria: AI understanding (#2: no ambiguity, good diagnostics), which also saves tokens, since each wasted round trip costs a read and a rewrite.

## Options considered

### Option A — name each foreign construct once, with the AIL spelling (chosen)
- **Pros:** the first report is the only one an agent needs; one rule, `gram.foreign.1`, covers the family, so the tables in `foreign.rs` can grow without new rules; fixes are exact where the source shows enough.
- **Cons:** a table to maintain; a guess in the table can mislead (kept to words whose AIL meaning is unambiguous).

### Option B — accept the foreign spelling as a synonym
- **Pros:** fewer errors.
- **Cons:** two spellings of everything, against the one-way-to-write-it goal and the token measurements behind the current syntax.

### Option C — leave the generic parser errors and improve the docs
- **Pros:** nothing to build.
- **Cons:** "expected end of line after the statement, found `:`" doesn't tell an agent what to write; the language card is not in front of it at that moment.

## Decision

1. **gram.foreign.1:** a construct other languages have and AIL doesn't is reported once, at its first token, naming the AIL spelling, with a fix where it is mechanical.
2. **gram.help.1:** every error diagnostic carries a `help`; a rule without a specific one gets a default (`diag::default_help`). A mistake is never reported as `bootstrap.unsupported`: `Option[T]` is `type.opt.3` (the canonical form is `T?`, with a fix) and `str + str` is `trait.op.1` (`+` is not defined for `str`; the card and ADR 0214 already say so).
3. **gram.help.2:** `bootstrap/tests/mistakes/` is the standing corpus. Each program records its rules, every error needs a help, and a `.fixed` file records what `ail fix` produces. The test requires at least 60 programs and that `ail fix` resolves at least 60% of them.
4. `//` is offered as a comment (`#`) only at the start of a line; elsewhere it may be Python's integer division, so the help names both and nothing is fixed.

## Rationale

The corpus turns "diagnostics are good" into a number a later change can't quietly worsen. Defaulting the help means a new rule can't ship without one, which the corpus test enforces on the programs that matter.

## Consequences

- Before/after on the corpus: first diagnostic has help 50% to 100%; programs with a cascade 9 to 2 (the two left hold two separate mistakes each); `ail fix` resolves 9% to 68%.
- New foreign words are one line in `src/foreign.rs`.
- Programs that exist only to be rejected live in `tests/mistakes/`; they are not formatted.

## Rejected alternatives

- **Accept foreign spellings** (Option B): see above.
- **A `--lenient` flag that rewrites foreign syntax silently:** `ail fix` already does this explicitly and shows what it changed.
- **Fix `//` anywhere on the line:** turns `a // b` into a comment and silently drops the rest of the line.
- **Keep `Option[T]` as `bootstrap.unsupported`:** it is not a missing feature; `T?` is the specified spelling.
