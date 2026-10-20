# 0167. Test functions are `@test` functions run by `ail test`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 28 Testing strategy
- **Spec rules:** `test.attr.1–3`, `test.run.1–4`, `gram.attr.1–2` (implemented)

## Context

Tests should sit next to the code with minimal syntax, and agents need
structured results ([plan](../design/ai-first-plan.md) section 6). ADR 0083
already reserves `@name` attributes before `fn`.

## Options considered

### Option A — `@test fn name()` run by `ail test FILE` (chosen)
- **Pros:** one extra token per test; uses the attribute syntax already
  decided; ordinary functions, so the checker, formatter and API need no
  special form; `--json` gives agents a stable result.
- **Cons:** needs generic attribute parsing in the front end.

### Option B — a `test "name"` block keyword
- **Pros:** names can be free text.
- **Cons:** a new construct and keyword; the name isn't a symbol the API can
  refer to.

### Option C — naming convention (`fn test_x()`)
- **Pros:** no syntax.
- **Cons:** implicit, collides with ordinary names, and gives no place to put
  other attributes.

## Decision

`@test fn name()` marks a test (no parameters, returns nothing or `!`, not
callable). `ail test FILE [--json]` runs them in source order, each isolated,
and prints a text or JSON report; exit 0 if all pass, 1 otherwise. Attribute
parsing is generic; the checker rejects names other than `@test` and `@pure`.

## Rationale

AI understanding decides: one declared form, a machine-readable result, and
tests that are ordinary symbols. Tokens favour A over B (no string name, no
keyword) by a small margin.

## Consequences

- Lane 6's `@pure` uses the same parsing.
- `ail run` ignores tests; the bootstrap parses attributes only on top-level
  functions and discards their arguments for now.
- The case harness gains `NAME.test` (expected `ail test --json` output).

## Rejected alternatives

- Test block keyword: new construct, name not a symbol.
- Naming convention: implicit and collision-prone.
