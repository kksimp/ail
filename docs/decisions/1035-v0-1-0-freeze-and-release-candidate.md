# 1035. v0.1.0: freeze after the lanes in flight, one release candidate gated on three hosts

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** Designer (2026-11-09 20:45 UTC, chose "Freeze after current lanes finish" on a decision card); the procedure is the AI's (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 28 — Testing strategy
- **Spec rules:** none (release policy)
- **Builds on:** [ADR 0842](0842-mac-first-gate.md), [ADR 0925](0925-windows-release-gate.md), [ADR 0976](0976-public-release-and-seed-trust.md), [ADR 1043](1043-versions-tags-changelog.md)

## Context

On 2026-11-09 `main` had 27 open pull requests and moved several times a day. The gate
([ADR 0842](0842-mac-first-gate.md)) checks each chain on the Mac; no single commit had been
checked on macOS, Linux and Windows together, and nobody had installed AIL from a clean machine.
A first public tag needs one commit that all of that is true of.

## Options considered

### Option A — freeze at once
- **Pros:** earliest release candidate.
- **Cons:** drops work that was hours from landing.

### Option B — freeze after the lanes in flight finish (chosen)
- **Pros:** nothing in flight is thrown away; the cut is still a single point.
- **Cons:** the tag comes later.

### Option C — no freeze
- **Pros:** no waiting.
- **Cons:** the tagged commit is never the commit that was tested.

## Decision

1. **Freeze.** Pull requests and lanes open on 2026-11-09 20:45 UTC may land. Nothing started
   after that targets 0.1.0; it targets 0.2. The freeze commit is `main` when the last of them
   has merged or moved to 0.2.
2. **Release candidate.** At the freeze commit, on the designer's machines: the full
   `tools/gate/mac-gate.sh` on the Mac; the same gate and the self-host fixed point on Linux
   x86-64; the `native`, `examples` and `run` suites on Windows x86-64 built with the tree's own
   stage-2 compiler ([ADR 0925](0925-windows-release-gate.md)); a clean install on each (shallow
   clone, no GitHub login, the README's commands as written); the Linux ARM64 seed run once under
   emulation; a scan of the history for secrets.
3. **After the candidate, docs only.** The tagged commit may differ from the tested commit only
   in `docs/`, `*.md` files and `tools/release/` version data (`VERSION`, `SEED_TAG`,
   `seeds.sha256`). Any other change makes a new candidate, and the red host is run again.
4. **Tag.** The seed is built from the release commit and `v0.1.0` is tagged as
   [ADR 1043](1043-versions-tags-changelog.md) says, with the designer's own OK.

## Rationale

Testing ranks under the designer's criteria only as a means, but a public tag is a promise
about one commit. Option B keeps that promise and loses no finished work; the cost is days.

## Consequences

- Later releases repeat steps 2 to 4; step 1 becomes "a freeze commit is named".
- Features marked Deferred in the ADR index are not in 0.1.0.
- The seed pinned on `main` between releases may be older than the release seed.

## Rejected alternatives

- **A release branch:** [ADR 0028](0028-git-workflow.md) keeps `main` the only long-lived branch; a freeze of days does not need one.
- **Hosted CI for the candidate:** [ADR 0842](0842-mac-first-gate.md); builds run only on the designer's machines.
