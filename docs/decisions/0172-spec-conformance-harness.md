# 0172. Run every `ail` block in the spec as a conformance test

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Testing and conformance
- **Spec rules:** none (see [spec README](../spec/README.md))
- **Implements:** [ADR 0002](0002-normative-spec-with-rule-ids.md)

## Context

ADR 0002 promised that examples in the spec become conformance tests. Nothing
checked them, so spec and bootstrap could silently disagree.

## Options considered

### Option A — fence annotations checked by `bootstrap/tests/spec.rs` (chosen)
- **Pros:** the expectation sits next to the example; runs in `./ail selftest`;
  failures name chapter and line. **AI understanding** decides: an agent
  reading a block sees whether it is meant to pass.
- **Cons:** blocks that need unshown declarations must be skipped or completed.

### Option B — a separate directory of conformance programs
- **Pros:** every program is whole.
- **Cons:** a second copy of the examples that drifts from the prose.

## Decision

Annotations: `ail` (must check clean), `ail fragment` (column-0 declarations
stay top level, other lines become the body of a generated `fn main()`),
`ail error=RULE`, `ail skip=bootstrap` (feature not implemented) and
`ail skip=context` (illustration relying on unshown declarations). Skips are
counted. An unknown tag fails the suite. The first run checks 27 blocks and
skips 37; making skipped illustrations self-contained is follow-up work.

## Rejected alternatives

- **Wrapping fragments by appending an empty `main`:** statement fragments
  are the common case and need to be inside `main`.
- **Failing on skips:** would block while features such as `pub` and `ref`
  are unimplemented.
