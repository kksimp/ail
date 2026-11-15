# 1123. `ail ask std` answers in one step

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 22 — Tooling
- **Spec rules:** api.find.1, api.find.2, api.find.3, api.find.6, api.find.8, tool.card.2
- **Amends:** [ADR 1051](1051-core-card-and-std-lookup.md)

## Context

In the E13 agent benchmark (ADR 0916) from-scratch AIL trials spent about five lookup calls each. Three causes were in
`ail ask std` itself:

- A line was a bare signature, so the agent ran `ail explain RULE` to learn what the function does.
- The index held only functions whose `std.*` rule opens with a backticked signature: 262 of the 416 top-level
  `pub fn`s of the std modules written in AIL were missing, among them `BigInt.add`, `sub`, `mul`, `cmp`, `abs`,
  `negate` and `as_int`.
- The words agents use are not the spec's words: `ail ask std stderr`, `argv`, `exit code` and `stdin` found nothing,
  because the rules say "standard error", "arguments", "exit status" and "standard input".

`ail ask how` also answered on one shared verb: `read program arguments` printed "read a file by lines".

## Options considered

### Option A: description lines, std sources in the index, a synonym table, a stricter how-to match (chosen)
- **Pros:** one call answers both "which function" and "what it does"; the index lists what `ail api` lists, so a
  std function cannot be missing; the synonym table is data, reviewed like `not-in-std.txt`; a test keeps both honest.
- **Cons:** up to twice as many output lines (ten answers, each with a description of at most 120 characters).

### Option B: rewrite every std rule to open with each signature
- **Pros:** one source of truth.
- **Cons:** hundreds of spec edits across chapters other lanes own; the sources would still drift from the spec
  with nothing to catch it.

### Option C: fuzzy matching (edit distance, stemming) instead of a synonym table
- **Cons:** `stderr` is not near "standard error" by any spelling measure; fuzzy hits add noise that costs AI
  understanding (rank 1) more than the table costs to keep.

## Decision

- Each answer of `ail ask std` and `ail card std.MODULE` is followed, when it has one, by a line indented four spaces:
  the first sentence of its rule that names it, without its opening signatures and ADR notes, else the first sentence
  of its `##` doc (api.find.1).
- The index adds every top-level `pub fn` of `std/DIR/FILE.ail` that no rule gave, used as `FILE`, without `self`,
  with `[mut self]` for a `mut` receiver; its rule is the first `std.M.N` its doc cites, else the first rule of its
  module that names it (api.find.2). A method's receiver is the first parameter of its shape (api.find.3).
- `docs/design/std-synonyms.txt` (`WORDS|WORDS -> MODULE.NAME ...`) puts the named functions first when one of a row's
  alternatives is in the query (api.find.8).
- `ail ask how` answers only when at least two thirds of the query's words hit a section (api.find.6).
- `tests/std_index/check.ail`, run by the gate's `agent_tools` suite, fails when a top-level `pub fn` that
  `ail api --text` lists is not in `ail card std.MODULE`, or a synonym row's first alternative does not answer with its
  first function.
- `ail card std.ui.widgets` also finds the module by the name it is used under.

## Rationale

AI understanding ranks first (ADR 1077): the description says what a function does without a second call, and an
index built from the sources cannot omit a function. Tokens rank last: a description costs about 25 tokens per answer,
less than one `ail explain` round trip.

## Consequences

ADR 1051's caveat (a function whose rule does not open with a signature is not found) no longer holds for the std
modules written in AIL; for the built-in modules it still does, and the spec suite's examples cover them. A new std
module needs no registration here. A new synonym row is checked by the gate.

## Rejected alternatives

- **Rewrite the std rules** (Option B): many files owned elsewhere, and still no check against the sources.
- **Fuzzy matching** (Option C): misses the real cases and adds noise.
- **Descriptions only with `--long`:** agents do not pass options they do not know; the default is what they see.
