# 1043. Pre-1.0 versions, release tags and the changelog

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076). Tagging `v0.1.0` needs the designer's own OK.
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none (release policy)
- **Builds on:** [ADR 0976](0976-public-release-and-seed-trust.md), [ADR 0855](0855-retire-the-rust-bootstrap.md) (seeds)

## Context

Seeds carry dated tags (`seed-2026.11.08g`), `ail --version` printed a fixed `0.1.0`, and
there is no changelog. Users cannot say which AIL they run or what changed.

## Decision

1. **Release tags** are `v0.MINOR.PATCH` on `main`. Until 1.0, MINOR may change the language,
   std or CLI; PATCH only fixes. Seed tags stay as they are and are not releases.
2. **A release is cut with a seed**: when `tools/release/seed.sh` publishes a seed for a
   release, the same commit gets the `v*` tag. Not every seed is a release.
3. **`ail --version`** prints `ail VERSION (git-describe, native)`. VERSION is the first line of
   `tools/release/VERSION`, bumped in the release commit; git-describe gives the tag and commit.
4. **`CHANGELOG.md`** has one section per release, produced by `tools/release/changelog.sh VERSION`
   from the merged pull-request titles on `main` since the previous `v*` tag. It is prepended by
   hand in the release commit. Titles are therefore user-facing text. Amended 2026-11-11: each
   section opens with a hand-written highlights block between `<!-- highlights:begin -->` and
   `<!-- highlights:end -->`; the script copies that block from the existing section and
   regenerates only the list under `### Changes`. The same text is the body of the GitHub Release.
5. **`v0.1.0`** is tagged on the public-flip commit, after the designer approves the flip.

## Consequences

One source of truth per fact: VERSION file for the number, git for the commit, PR titles for
notes. No release automation runs without the designer.

## Rejected alternatives

- **CalVer using the seed tag:** says when, not what changed; no signal for breaking changes.
- **Jumping to 1.0:** promises stability the language does not yet have.
- **Hand-written changelog only:** drifts from what merged; the script plus a hand-written intro is cheap.
- **Conventional-commit parsing:** history is squash and merge commits with free-form titles.
