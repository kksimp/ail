# 0383. No package registry: dependencies are git URLs, with tags as versions

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** Designer
- **Topic:** 13 — Package management
- **Spec rules:** [mod.pkg.3–6](../spec/12-modules.md#packages) stay as they are
- **Confirms:** [ADR 0231](0231-path-and-git-dependencies.md),
  [ADR 0355](0355-fetch-git-dependencies-with-system-git.md),
  [ADR 0357](0357-ail-pkg-update.md)
- **Resolves:** OQ-049

## Context

Dependencies come from a path or a git URL pinned by `rev` or `tag`
(ADR 0231), fetched with the system git (ADR 0355), and updated with
`ail pkg update` (ADR 0357). A central registry with names and semver
ranges was deferred to the designer because it needs hosting, accounts, a
naming policy and a solver.

## Options considered

### Option A — a central registry with semver ranges
- **Pros:** short `dep` lines, discovery, a lockfile and a solver.
- **Cons:** a service to host and operate, accounts, a naming policy.

### Option B — git URLs only, tags as versions (chosen)
- **Pros:** nothing to host or run; the mechanism already exists and is
  tested; a tag is a version; the URL is the name, so no naming policy.
- **Cons:** longer `dep` lines in every manifest; no version ranges, so
  updates are explicit (`ail pkg update`).

### Option C — a name-to-git index without a solver
- **Pros:** short names.
- **Cons:** still a service to run, for a token saving confined to the
  manifest.

### Option D — decide later
- **Cons:** leaves tooling questions (ADR 0357's subcommands) open.

## Decision

Option B. AIL has no package registry. A dependency is a path or a git
URL with a `tag` (a version) or a `rev` (an exact commit). Nothing in the
manifest, the lockfile or the tooling assumes a registry.

## Consequences

- OQ-049 is closed; ADRs 0231, 0355 and 0357 stand without their
  registry caveats. Anything that was designed to leave room for a registry
  (a `name` separate from the URL, a solver) is not needed.
- Discovery for agents is a documentation matter: a curated list of
  packages in the docs, not a service.
