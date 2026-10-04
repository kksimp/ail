# 0002. Write a normative specification with stable rule IDs and tested examples

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** Project process / specification
- **Spec rules:** defines the rule-ID scheme in [spec/README.md](../spec/README.md)

## Context

AIL's compiler is meant to give structured, machine-readable diagnostics, and
AI agents are meant to query program semantics. For both, the language
definition has to be precise enough to cite. A spec written as prose drifts
away from the implementation over time.

## Options considered

### Option A — Normative rules with stable IDs, plus tested examples (chosen)
- **Pros:** Unambiguous MUST/SHOULD wording. Diagnostics and the AI semantic
  API can cite the exact rule (e.g. `mem.atomic.2`). Examples double as
  conformance tests, so the spec cannot silently drift from the compiler. The
  grammar is machine-checkable.
- **Cons:** Slower to write. Rule IDs need maintenance. The test harness only
  becomes possible once a compiler exists.

### Option B — Readable prose with rationale
- **Pros:** Fast to write and pleasant to read.
- **Cons:** Ambiguous at the edges, can't be cited precisely, and drifts.

### Option C — Formal (mathematical) semantics
- **Pros:** Maximum precision, and soundness becomes provable.
- **Cons:** Very slow to write, few people can maintain it, and it is premature
  before the language stabilises.

## Decision

The specification lives in `docs/spec/` and consists of normative rules with
stable, citable IDs, using RFC 2119 keywords. Each rule may carry examples
that will become conformance tests. The grammar will be kept in a
machine-checkable form. Explanatory prose and rationale are allowed but are
clearly marked non-normative.

## Rationale

Being able to cite rules is what makes the AI-aware diagnostics (brief §13) and
the AI semantic interface (brief §12) precise. Tested examples serve the
"performance must be measurable" philosophy's sibling: *correctness must be
checkable*.

## Consequences

- The rule-ID scheme is defined in [spec/README.md](../spec/README.md).
- Once a rule ID is published it is never reused. Removed rules are marked
  `[removed]`, not renumbered.
- Stage 2+ must include a conformance harness that extracts spec examples.
- Formal semantics for a core subset may be added later. This does not rule it out.

## Rejected alternatives

- **Prose only:** can't be cited by tools and drifts from the implementation.
- **Formal semantics now:** premature and too costly at this stage. Revisit
  for the core calculus after the language stabilises.
