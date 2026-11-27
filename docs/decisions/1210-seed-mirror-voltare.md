# 1210. Seeds are mirrored on seeds.voltare.us; the GitHub release stays the fallback

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** Designer (Kaleb, 2026-11-26: a seed mirror on seeds.voltare.us; standing approval to cut a seed after each green batch that adds language features, tagged releases still need his approval); detail by AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none (release tooling; `native.build.3` is unchanged)
- **Builds on:** [ADR 0855](0855-retire-the-rust-bootstrap.md) (seeds), [ADR 0976](0976-public-release-and-seed-trust.md) (seed trust)

## Context

`tools/release/fetch-seed.sh` and `fetch-seed.ps1` download the seed from the repository's GitHub release. That needs
GitHub to serve release assets to every checkout and every gate run, and a private or rate-limited repository falls back
to `gh` or a token. Seeds are now cut often (after each green batch that adds language features), so the download path
is on every first build.

## Decision

- `tools/release/seed-publish.sh TAG DIR` uploads the seeds built by `seed.sh` to a Cloudflare R2 bucket served as
  `https://seeds.voltare.us`, at `ail/TAG/<asset>`, with `ail/TAG/seeds.sha256` and `ail/TAG/manifest.json` (tag,
  commit, build time, name, hash and size of each seed). A published tag is immutable: the script refuses to overwrite
  one. It checks each asset as served against its hash before it moves `ail/latest.json`, a pointer for people that
  `./ail` never reads. `SKIP_LATEST=1` leaves `latest.json` alone, for uploading older tags.
- `seed.sh TAG --voltare` runs `seed-publish.sh` after the build; `--upload` (the GitHub release) is unchanged and the two
  combine.
- The fetch scripts try `$AIL_SEED_HOST/ail/TAG/<asset>` first (default `https://seeds.voltare.us` when unset; `none`, or
  empty where the shell keeps an empty variable, turns the mirror off). A copy that fails the hash pinned in `tools/release/seeds.sha256` is discarded, and the GitHub release,
  then `gh`, are tried as before. The pinned hashes stay the only trust root (ADR 0976); the mirror is a transport.
- Upload credentials live in the Mac Keychain (service `ail-seed-r2`, stored once by `tools/release/seed-credentials.sh`)
  or in `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`. Nothing is committed.
- Approval: seeds of `main` after a green batch are covered by the designer's standing approval; a tagged release still
  needs the designer's own approval.

## Rejected alternatives

- **Mirror only, no GitHub fallback.** A mirror outage or a 404 would stop every fresh checkout; the fallback costs one
  extra request in that case only.
- **Trust the mirror's own `seeds.sha256`.** Moves trust to whoever can write the bucket. The tree's pins already say
  which bytes are right.
- **Mutable tags on the mirror.** Old commits and the apps' `.ail-version` pins name old tags, so a tag's bytes must never
  change.

## Amendment 2026-11-27: index.json for the download page

`tools/release/seed-index.sh` writes `ail/index.json` to the bucket: the manifest of every published tag, newest first.
`seed-publish.sh` runs it after each publish (and after a `SKIP_LATEST` upload), so the download page on voltare.us lists a
new seed with no site redeploy. The page shows the pin from `tools/release/seeds.sha256` on main as the trust root and
marks the mirror's own hashes as unverified. Decided by: Designer (Kaleb, 2026-11-27: the download page updates itself
with every new seed); detail by AI. Rejected: regenerating and redeploying the static page per seed (a manual step that
would be forgotten); a Pages function that reads the bucket (more moving parts for the same result).

From 2026-11-27 the mirror carries only seeds from `seed-2026.11.14b` (the seed of v0.1.0) on, and `seed-index.sh` leaves older tags out of `index.json`
(`SEED_INDEX_FLOOR`). Decided by: Designer (Kaleb, 2026-11-27: drop all pre-0.1 seeds, keep the GitHub release copies). Older commits still
build through the GitHub fallback. Rejected: deleting the GitHub copies too (old commits would stop building from a fresh clone).
