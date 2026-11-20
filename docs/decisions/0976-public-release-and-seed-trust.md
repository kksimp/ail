# 0976. Public release and seed trust: who builds the seeds, how a user checks one

- **Status:** Accepted (the designer approved the public flip; the repository is public and 0.1.0 shipped 2026-11-15)
- **Date:** 2026-11-07
- **Decided by:** Designer (Kaleb, 2026-11-07: remove the README token section; security contact by the AI's judgement; wait for the polish list before going public, target 2026-11-14 to 2026-11-17); detail by AI (delegated, ADR 0076). The public flip itself needs the designer's own approval.
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none (release policy; `native.build.3` and `native.build.1` describe the mechanics)
- **Builds on:** [ADR 0855](0855-retire-the-rust-bootstrap.md) (seeds), [ADR 0610](0610-self-host-bootstrap-harness.md) (fixed point), [ADR 0925](0925-windows-release-gate.md) (Windows release gate), [ADR 0904](0904-rust-bootstrap-deleted.md) (no Rust)
- **Will supersede:** the "A public release" line of ADR 0855's rejected alternatives ("the project is private for now; a private release needs no new policy")

## Context

Since ADR 0904 the only way to get a first `ail` is a binary seed: a release
asset that `./ail` downloads, checks against `tools/release/seeds.sha256` and
uses to build `compiler/main.ail` (ADR 0855). That was safe enough while every
user was the designer. A public user runs a binary they did not build, so they
need to know who built it, from which source, and how to check it without
trusting the download.

Facts today: seeds are built by `tools/release/seed.sh` on the designer's Mac
only after the self-host fixed point holds (stage 2 and stage 3 byte-identical,
ADR 0610); the hashes are committed in the same tree that pins `SEED_TAG`; the
download script refuses a hash mismatch; four hosts have seeds (aarch64-macos,
x86_64-linux, aarch64-linux, x86_64-windows), aarch64-windows has none
(ADR 0968). Ten seed releases exist; old commits, and the app repositories'
`.ail-version` pins, name older ones.

## Options considered

### Option A — designer-built seeds, hash-pinned in the tree, self-reproducible (chosen)
- **Pros:** no new infrastructure; the hash travels with the source that uses it, so a swapped asset fails closed; anyone with a seed can rebuild the seed at the same commit and compare hashes; one publisher keeps the chain short.
- **Cons:** trust starts at one person's machine; self-reproduction proves consistency, not the absence of a compiler-level backdoor (the "trusting trust" problem).

### Option B — CI-built seeds with signed provenance
- **Pros:** builds on clean, logged machines; signatures tie a binary to a workflow run.
- **Cons:** there is no hosted CI (ADR 0085, 0842); it needs accounts and paid or third-party services the project avoids (ADR 0009); it moves trust to the CI vendor rather than removing it.

### Option C — diverse double-compiling from the deleted Rust bootstrap
- **Pros:** the strongest answer to trusting trust: an independent compiler path reaches the same bytes.
- **Cons:** the Rust bootstrap stopped matching the language long before ADR 0904; reviving it is weeks of work for a check nobody asked for yet. Kept as a possible later step.

### Option D — signed seeds (GPG or Sigstore) on top of A
- **Pros:** proves the publisher even if the GitHub account is compromised.
- **Cons:** key management and an external service or key the designer must hold; the in-tree hash already ties the asset to a reviewed commit. Revisit if the project gains a second publisher.

## Decision

When the repository is public:

1. **Who builds and publishes.** Only the designer publishes seeds, from the
   Mac, with `tools/release/seed.sh TAG --upload`. Agents and automation never
   publish (as ADR 0855). A seed release is made only from a commit on `main`,
   and its tag points at that commit.
2. **Before a publish.** The self-host fixed point holds (`seed.sh` checks it),
   the Mac gate is green, and the Windows `native`, `examples` and `run` suites
   are green on that tree (ADR 0925).
3. **Hosts.** Seeds ship for aarch64-macos, x86_64-linux, aarch64-linux and
   x86_64-windows. aarch64-windows gets none while ADR 0968 is experimental.
4. **What a user trusts.** The SHA-256 in `tools/release/seeds.sha256` at the
   commit they cloned. `fetch-seed` installs nothing that does not match. The
   download needs no login (`releases/download/TAG/ASSET`).
5. **How a user checks a seed.** With the seed at commit `C`, run
   `tools/self-host/run.sh`: stages 2 and 3 must be byte-identical. Building
   the seed form with stage 2 (the `seed.sh` loop, without `--upload`) must
   give the hashes in `tools/release/seeds.sha256`. A seed that reproduces itself
   is consistent with the source at `C`; it is not an independent proof
   (Option C).
6. **Old releases.** A seed release that any commit on `main` named in
   `SEED_TAG` stays published, so old commits and pinned app versions still
   build. Only releases never pinned on `main` may be deleted.
7. **The README** links this ADR and states that seeds are binaries built by
   the designer and checked by hash.

## Rationale

Option A keeps the trust chain short and inspectable with no service, account
or money (ADR 0076 item 4 rules those out without the designer). Pinning the
hash in the tree means the only thing a user trusts beyond the source is one
person's build, and the self-reproduction check lets anyone confirm that build
matches the source. It is honest about what it does not prove.

## Consequences

- `tools/release/seed.sh` must stop refusing to upload when the repository is
  public, and its "PRIVATE release" wording must go (follow-up in the scripts
  lane that makes `fetch-seed` work without a login).
- A one-command `tools/release/verify-seed.sh` that runs check 5 is a
  follow-up; until then the steps above are the procedure.
- Deleting a pinned seed release breaks every commit that pins it; release
  cleanup removes only unpinned ones.
- ADR 0855's "a public release needs no new policy" line is superseded by
  this ADR once it is Accepted.

## Rejected alternatives

- **CI-built seeds (B):** no hosted CI, and it adds an outside party to trust.
- **Diverse double-compiling now (C):** weeks of work reviving a deleted compiler; possible later.
- **Signatures now (D):** key management for a single publisher whose hash is already pinned in reviewed source.
- **Committing seeds to git:** rejected in ADR 0855 (megabytes per update in history, forever).
