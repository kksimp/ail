# 0151. Symbols have path ids; `--symbols` lists every reference by id

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.id.1-3, api.ref.1-2 ([semantic-api.md](../design/semantic-api.md#stable-ids))

## Context

Rename, "where is this used" and the later query compiler (ADR 0015) need to
name a declaration in a way that survives edits (plan §2.4).

## Options considered

### Option A — path ids such as `method:Point.shift` (chosen)
- **Pros:** readable, predictable without running the tool, stable when
  lines move, and unique per file.
- **Cons:** a rename changes the id (which is what a rename means).

### Option B — position ids such as `fn@12:4`
- **Pros:** trivially unique.
- **Cons:** change on every edit above the declaration; meaningless to read.

### Option C — numeric ids
- **Cons:** unstable across runs and unreadable.

## Decision

An id is `kind:path`: `fn:add`, `type:Point`, `trait:Shape`,
`method:Point.shift` (also for trait and `impl` methods), `field:Point.x`,
`variant:Color.Red`, `field:Shape.Rect.w`, `impl:Circle.Shape`,
`const:MAX_DEPTH`. An `impl` method whose plain id is taken also names its
trait (`method:Circle.Shape.area`). `--symbols` adds `references`: every use
of a declared name as `{id, line, col}`, sorted by position. A call
dispatched through a trait refers to the trait's method.

The checker records what each use resolves to (`Resolved::symbols`), so
references come from the checker's own name resolution, not a second
resolver; type mentions in signatures are matched by name in `api.rs`.

## Rationale

AI understanding (criterion 2) decides: an agent can write `method:Point.shift`
from the source without asking the tool, and the id says what kind of thing
it names. Taking resolution from the checker means references can't disagree
with what the program does.

## Consequences

- Rename and find-references become a lookup on `references`.
- With modules, ids gain a module prefix (`geo.type:Point` or similar),
  decided with lane 3's modules.

## Rejected alternatives

- Position ids: change on unrelated edits.
- Numeric ids: unreadable and unstable.
- A separate resolver in `api.rs`: would drift from the checker's rules
  (implicit `self`, bare variants, trait dispatch).
