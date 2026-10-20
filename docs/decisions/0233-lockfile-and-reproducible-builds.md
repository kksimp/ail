# 0233. `ail.lock` pins every git dependency by commit and content hash; cache keys include grants

- **Status:** Accepted; amended by [0356](0356-ail-lock-pins-commits.md) (no `sum`; implemented)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 13 — Package management; 27 — Build system
- **Spec rules:** `mod.pkg.8` (with ADR 0356)
- **Design:** [packages.md](../design/packages.md#versions-git-dependencies-and-the-lockfile-design-not-implemented)

## Context

A `tag` can move and a repository can be rewritten, so a manifest alone
doesn't make builds reproducible. ADR 0015's whole-program cache needs
stable keys for each package.

## Options considered

### Option A — a lockfile in the manifest's line format, with commit and content hash (chosen)
- **Pros:** reproducible and verifiable; same format as `ail.pkg` (ADR 0230);
  works offline once the cache holds the tree.
- **Cons:** one more file to commit.

### Option B — require `rev` (commit) in the manifest, no lockfile
- **Pros:** one file. **Cons:** no tags; no check that the fetched tree is
  the one reviewed.

### Option C — vendoring only (`path` dependencies)
- **Pros:** nothing to fetch. **Cons:** large repositories; updates by hand.

## Decision

`ail.lock`, next to `ail.pkg`, holds one line per git dependency of the
whole graph: `NAME git URL commit HASH sum sha256:HASH`, where `sum`
hashes the package's files in sorted path order. A build with a lockfile
uses only locked commits, fails on a `sum` mismatch, and needs no network
when the content-addressed cache holds the trees. Path dependencies aren't
locked. A package's cache key is the compiler version, its content hash, its
grants, and its dependencies' keys.

## Rationale

AI understanding and speed: an agent can verify exactly what it builds, and
cached analyses are shared across projects by content. Tokens: the lockfile
is generated and never needs reading.

## Consequences

- Needs `ail fetch` and a cache directory; not in the bootstrap.
- Grants are part of the key because they change which programs are accepted.

## Rejected alternatives

- **B:** loses tags and verification.
- **C:** vendoring stays possible through `path`, but not as the only way.
