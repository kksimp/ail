# 0224. Incremental queries are keyed by stable ids and fingerprinted without spans

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 19 — AST/IR architecture; 23 — Compiler architecture; 27 — Build system
- **Spec rules:** none (compiler architecture)
- **Design note:** [ir-architecture.md](../design/ir-architecture.md) §5

## Context

ADR 0015 makes every build a whole-program optimised build, kept fast by a
fine-grained, deterministic, shareable cache. What the cache keys on decides
how much an edit invalidates. **Speed decides** (compile time on the agent's
edit loop, wishlist 2.5); tokens and readability are unaffected.

## Options considered

### Option A — stable-id keys, span-free fingerprints, red-green reuse (chosen)
- **Pros:** moving code or editing a comment re-runs only span-dependent
  queries (diagnostics, debug info); a recomputed result that hashes the
  same stops invalidation there; ids are already the API's identities
  (ADR 0151).
- **Cons:** a second table for spans; the hash function must be stable
  across compiler builds and machines.

### Option B — keys by file and position
- **Pros:** simple.
- **Cons:** any edit above an item invalidates it.

### Option C — whole-file granularity
- **Pros:** simplest cache.
- **Cons:** an edit to one function recompiles every function in the file
  and everything inlined from it.

## Decision

Every compiler step is a query keyed by stable ids (plus type arguments for
body instances and the ISA for machine code). Each result is fingerprinted
from its inputs' **span-free** content (spans replaced by offsets relative
to the item's start, kept in a side table read only by diagnostic and
debug-info queries). When a recomputed result's fingerprint is unchanged,
its dependants are reused. Queries are deterministic, so cache entries may
be shared between machines. Compile-time evaluation records the functions
it actually entered as its inputs (ADR 0226).

## Consequences

- **Designer, look first at:** nothing serious; this is the standard
  approach (rustc, salsa) applied to AIL's existing ids.
- The build can explain a large rebuild from the recorded dependencies
  (ADR 0015's "report why").
- Renaming an item changes its id, so it is a new item to the cache; rename
  is rare enough that this is fine.

## Rejected alternatives

- **Position keys (B):** invalidates on unrelated edits.
- **File granularity (C):** too coarse for whole-program optimisation.
