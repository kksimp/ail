# 0231. Dependencies come from a path or a pinned git commit, under one global name; a registry waits for the designer

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 13 — Package management; 12 — Modules
- **Spec rules:** [mod.pkg.3–6](../spec/12-modules.md#packages)
- **Design:** [packages.md](../design/packages.md)

## Context

ADR 0153 defines a package root but no dependencies. A dependency needs a
source, a name in `use`, and rules for what it can see. A registry with
versions involves hosting, accounts and publishing (AGENTS.md escalation
criterion 4), so tonight's design must work without one.

## Options considered

### Sources: `path` and pinned `git` (chosen) vs. also a registry now
A path covers vendored and monorepo code; a git URL with an exact `rev` or
`tag` covers everything else reproducibly. A registry is deferred.

### Names: the `dep` name, global to the program (chosen)
- **Pros:** one name per package everywhere: `use`, diagnostics, text forms
  (`geo.Point(...)`), effect chains. Fits the bootstrap's qualified names
  (ADR 0156).
- **Cons:** two packages can't share a name, and the root package can't have a
  module directory named like a package; two versions of one package can't
  coexist.

### Names: per-package namespaces
- **Pros:** any two packages can share a name.
- **Cons:** the same text names different code in different files; needs
  name mangling.

### Visibility: a package imports only what its own manifest declares (chosen) vs. everything in the graph
Declaring transitive imports keeps each package's dependencies readable from
its own manifest and stops code from relying on an accident of the graph.

## Decision

`dep NAME path DIR` makes the package rooted at `DIR` (relative to the
manifest) available as `use NAME` (its root module) and `use NAME.a.b`.
Each package resolves its imports against its own root and its own manifest;
importing a package it doesn't declare is `mod.pkg.3`. Package names are
global: one name, one directory; a dependency's own `package` name must
match; `std` is reserved; the root can't have a module directory with a
package's name (`mod.pkg.4`). Package dependencies can't form a cycle
(`mod.pkg.5`). `git URL` needs exactly one of `rev` and `tag`; the
bootstrap doesn't fetch and rejects it (`mod.pkg.6`). Version ranges and a
registry are a later decision for the designer.

## Rationale

AI understanding decided: one name per package and per-package visibility
make every import resolvable from two files (the importing file and its
package's manifest). Tokens are unaffected: `use` lines are unchanged.

## Consequences

- Exact pins mean no solver: a diamond with two pins for one name is an
  error naming both lines.
- Revisit per-package namespaces with the registry, if two versions of one
  package in a program become necessary.

## Rejected alternatives

- **A registry now:** a serious, hard-to-reverse decision with accounts and
  hosting; escalated rather than decided tonight.
- **Version ranges and a solver:** nothing to solve without a registry.
- **Per-package namespaces:** ambiguous names in diagnostics; mangling.
- **Seeing the whole graph:** hidden transitive dependencies.
- **Package cycles:** no layering; the reasons of ADR 0153.
