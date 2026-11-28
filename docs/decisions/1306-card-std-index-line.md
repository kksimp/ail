# 1306. The full language card names the newer standard modules on one index line

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer set the goal ("the full card needs to have whatever the agent needs to understand and write ail") and left the particulars to the AI
- **Topic:** Agent documentation
- **Amends:** [ADR 1110](1110-language-card-budget-16000.md) (what counts as covered)

## Context

The full card's budget is 16,000 tokens (`tools/tokbench/run.sh --check-budgets`). Main was at 16,036. The std wave
added a paragraph per module, and a union merge left three near-copies of the std paragraph: the batch card was 19,610
tokens. The card's measured benefit comes from an agent reading all of it, so it must stay within the budget.

## Decision

- The card keeps the language and the core std it had before the wave (io, fs, file, env, proc, net, `std.net.http`
  routing and client helpers, json, db, regex, csv, args, log, date, random, time, tensor, bigint).
- The modules first released after 0.2.0 ([ADR 1305](1305-experimental-std-modules.md)) get one index line in
  "Standard library": each name, a few words where a name misleads (`sqlite` only reads; a `db` file is not SQLite; a
  local named `url` hides the module), and the paths that need `use`. `ail card std.NAME` prints a module's
  signatures and `ail ask std "words"` finds a function.
- Their rule families are listed in `docs/design/card-coverage-exceptions.txt` with that reason, so
  `check-card-coverage` stays a gate.
- Small cuts elsewhere bring the card to 15,984 tokens: a duplicated `else`-block note, two long asides, and the
  `sorted` method the card named on maps, which maps do not have.

## Rejected alternatives

- **Raise the budget.** The designer set it; the card's value is that an agent reads all of it.
- **Link to GitHub for std.** An agent with a shell already has `ail card` and `ail ask`, which answer offline and
  match the installed compiler.
- **Drop the new modules from the card entirely.** An agent would not know `validate` or `template` exist and would
  write its own.
