# 1105. Scoped capabilities in `ail.pkg`, and in `ail review`

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved the item for 0.1 (AI-first review S14)
- **Topic:** 13 — Package management (capabilities); 20 — AI semantic API
- **Spec rules:** [eff.allow.10–11](../spec/18-effects.md), [api.rev.12](../spec/22-tools.md)
- **Builds on:** [ADR 1072](1072-scoped-runtime-capabilities.md), [ADR 0232](0232-capabilities-per-dependency.md)

## Context

`--allow` (ADR 1072) scopes a run from the command line. A package that always needs
the same limits (`fs.read=./data`, `net=api.example.com`) should say so in `ail.pkg`,
where `caps` already limits it by capability kind (ADR 0232), and a reviewer should see
a widened scope as a capability change.

## Options considered

### Option A — the `--allow` grammar in `caps`; the manifests give the run's policy (chosen)
- **Pros:** one grammar to learn; the compile-time check is unchanged in kind; no flag is needed to run a package safely.
- **Cons:** scopes are process-wide at run time; per-package scopes cannot be enforced at the call.

### Option B — a separate `scope` key on the line
- **Cons:** two places for one idea; a second grammar.

### Option C — per-package run-time enforcement
- **Cons:** needs the caller's package at every checked call; a cost on every call for a case the compile-time capability check mostly covers.

## Decision

`caps` takes the entries of `--allow` (`eff.allow.10`). Compile time: `fs.read` and `fs.write`
count as `fs`, `net.serve` as `net`. Run time: when the root `package` line has `caps` and any list in the graph has a scope, `ail run`
with no `--allow` sets `AIL_ALLOW` to every entry of every list (`ail check --policy-out FILE` writes it for the launchers). `--allow` replaces it.
`ail review` reports a gained entry or widened scope as `NEW CAPABILITY grant PACKAGE` (`api.rev.12`).

## Consequences

- A dependency's scope is process-wide at run time; the root's own `caps` still bound what the root's code may reach by kind.
- A root with no `caps`, or with unscoped lists only, behaves as before.
- Entries have no spaces or commas inside a scope, as with `--allow`.

## Rejected alternatives

- **A separate key (B)** and **per-package enforcement (C)**, above.
- **Merging the manifest policy with `--allow` by intersection:** two limits to reason about; the person running the program decides, so `--allow` wins.
