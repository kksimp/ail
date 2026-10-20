# 0200. Run doc-comment examples as tests

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Testing, [ADR 0167](0167-test-functions-and-ail-test.md)
- **Spec rules:** `test.doc.1`, `test.doc.2`, `test.doc.3`

## Context

Doc examples rot when nothing runs them. Lane 2 attaches `##` doc comments to declarations, so the examples are already found next to what they document.

## Options considered

### Option A — `ail test` compiles each block as a test in the file's scope (chosen)
- **Pros:** the example can call the documented function and anything else in the file; failures point into the doc comment; examples and `ail api` docs stay one text.
- **Cons:** examples can't import from other files except by `use` lines they contain.

### Option B — extract examples to files and run `ail run`
- **Pros:** examples are standalone programs.
- **Cons:** can't see the file's declarations; much more ceremony per example.

## Decision

Each block tagged exactly ` ```ail ` in a `##` doc comment on a declaration is a test named `doc:NAME` (`doc:Type.method`, and `doc:NAME#2` for a second block). It runs after the file's own tests. Blocks tagged `ail fragment`, `text` or anything else are not run.

## Rationale

ADR 0076 criterion 2 (AI understanding): the example the reader sees is the example that is verified. It also uses the same checker and interpreter as every other test.

## Consequences

- A compile error in an example is reported against the doc comment's line and no test runs (exit 2).
- Doc examples run after ordinary tests, so test output order has the file's tests first.

## Rejected alternatives

- Option B: ceremony and no access to the file's scope.
