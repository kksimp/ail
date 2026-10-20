# 0237. `par for` asserts independence; provable conflicts are errors

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 11 — Concurrency; 2 — Lexical structure
- **Spec rules:** [conc.par.5](../spec/11-concurrency.md), [lex.kw.1](../spec/02-lexical.md) (amended: `par`)
- **Relates to:** [ADR 0236](0236-automatic-parallel-loops.md)

## Context

Some loops are independent for reasons the checker can't see: writes through
a permutation (`out[perm[i]] = ...`), reads of other elements known not to
be written, an accumulator whose type isn't visible. The designer chose a
`par` keyword for these, with an error when the loop is provably unsafe.

## Options considered

### Option A — `par for`, `par` reserved (chosen)
- **Pros:** One token. A reserved word has one meaning everywhere. No example or test used `par` as a name.
- **Cons:** Takes a short word from user names.

### Option B — contextual `par` (a name elsewhere)
- **Pros:** No reservation.
- **Cons:** Two meanings for one word; lex.kw.2's `c` is the only precedent and exists for FFI.

### Option C — `@par for` or `@parallel` on the loop
- **Pros:** Reuses attributes.
- **Cons:** Attributes are declaration metadata (ADR 0083); `@` costs a token (+2.0 vs `for`, +1.0 vs `par for`).

### Option D — `parallel for`, or `for x in par(xs)`
- **Pros:** `parallel` is plain English and also one token.
- **Cons:** The designer named `par`; `parallel` ties on tokens, so it doesn't override that. `par(xs)` costs more (+1.6) and looks like a call that changes the collection.

## Decision

`par for x in e` has exactly the meaning of `for x in e` plus the assertion
that its iterations are independent. The checker still applies every
obligation of conc.par.2–4 it can decide; a `par for` that fails one is a
`conc.par.5` error naming the first blocking write or effect, with a help.
What it can't decide (element distinctness, cross-element reads, an
accumulator's type) the assertion covers. A checked build MAY verify the
assertion at run time and trap on overlapping writes.

## Rationale

Criterion 1 (tokens), measured with `tools/tokbench/run.sh` on a five-line
loop (mean of five tokenizers, `for` = 59.2): `par for` 60.2, `parallel for`
60.2, `for i in par(0..4)` 60.8, `@par for` 61.2. `par` and `parallel` tie;
the designer's word stands.
Criterion 2 (AI understanding): the keyword is visible at the loop and never
changes results; a wrong assertion can only make a parallel run differ from
the sequential one, which a checked build catches.

## Consequences

- `par` is reserved; `par` without `for` is a parse error citing conc.par.5.
- `ail effects --parallel` reports `asserted` for loops that pass only by assertion.
- Clauses (`schedule`, `chunk`) are not offered: the runtime chooses (ADR 0238).

## Rejected alternatives

- **Contextual keyword:** two meanings for one word.
- **Attribute:** attributes are for declarations and cost a token.
- **OpenMP-style clauses:** a program could come to depend on scheduling.
