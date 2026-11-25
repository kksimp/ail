# 1176. The cards teach the form to write, and the edge behaviour a lookup would find

- **Status:** Accepted
- **Date:** 2026-11-25
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule of no syntax change, and the card budgets of ADR 1110
- **Topic:** AI-first tooling
- **Spec rules:** none changed
- **Amends:** [ADR 1110](1110-language-card-budget-16000.md) (what a card line is for; the budgets stay)

## Context

Three measurements say what the cards still fail to teach.

- The 80-trial rerun (first drafts green 56 of 80, C 70 of 80): the misses were forms, not names. 25 builds
  rewrote the source, which costs a re-read, and the card listed the rewritten spellings without the form to write.
  `unreachable` was named and never shown with its message; a `mut` parameter was shown far from its call; a method on
  a `T?`, `?` in a function with a plain result and `\"` inside an interpolation had no card line.
- The app-build test of 2026-11-25 (11 turns in AIL against 3 in Python): the extra turns were lookups of edge
  behaviour the card did not state (whether padding cuts, whether `is_alpha` is ASCII, what `words()` splits on, the
  name `to_str`, a digit's value).
- The mistakes corpus holds forms with no fix and no card line: a reversed or stepped range, `xs + ys`,
  `xs.count(v)`.

## Decision

1. **Pairs, not a list.** Where a card names a spelling the build rewrites, it gives the AIL form after `->`, and says
   to write that form the first time. The core card carries the pairs seen most in the runs.
2. **Forms with no fix get a line in the part read first** ("Habits from other languages"): a `mut` parameter with
   its call, a method on a `T?`, where `?` is allowed, plain code inside `{ }`, counting down or by a step, joining
   lists, `unreachable("why")`.
3. **Edge behaviour goes in "Compiles, but is not what you meant"**, each from a spec rule: the sign of `%`
   (expr.arith.5), the text of a float and what `round` and `to_int` give (type.num.7, type.num.9, type.str.11),
   padding never cuts (type.str.9, lex.str.7), `split` keeps empty pieces and `words()` does not (type.str.4,
   type.str.17), Unicode and ASCII character tests and a digit's value (type.char.2 to 4), the text of an enum value
   and of a `str` inside a collection (trait.auto.2), `count` taking a function (type.coll.28).
4. **The core card is chosen by measured cause.** To stay inside 3,000 tokens it drops the distinct-type line, the
   default-parameter and default-field sentences and the unwaited-task item, none of which appeared in a measured
   first-draft miss; the full card keeps or gains each (type.distinct.1, expr.call.4, type.struct.7, type.struct.8,
   conc.task.5).
5. **How-tos for the same forms**, so `ail ask how` finds them: count down or by a step, digit value of a
   character, join two lists, print and round a float, fixed-width columns, text with braces and quotes; "leave
   nested loops" shows the label form.

## Options considered

- **Leave the rewritten spellings as a bare list**: rejected. A rewrite is not free: the agent must re-read its
  file, and the run counted 25 such builds.
- **Spend no card tokens on forms the card already states** (literal braces, `+` on text): kept. Those are for the
  build to accept or rewrite; this ADR adds only what the card did not say.
- **Raise the core card's budget**: rejected here; the budget is the designer's (ADR 1110).
- **Put edge behaviour only in `ail ask std`**: rejected. A lookup costs a turn, and agents look up a form only when
  they doubt it; the card arrives before the first draft.

## Consequences

- `docs/language-card.md` (about 12,600 to 14,000 tokens of 16,000), `docs/language-card-core.md` (2,999 of 3,000),
  `docs/how-to.md`, `docs/for-agents.md`, `docs/quick-reference.md`, `llms-full.txt` (generated).
- `docs/design/card-coverage-exceptions.txt` loses `family expr.loop`, now cited by a how-to.
- Not measured: no paid run. The claim to test on the next run is fewer rewriting builds and fewer lookups before
  the first draft.
