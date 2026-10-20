# 0355. Fetch git dependencies with the system `git` into a read-only cache

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 13 — Package management; 27 — Build system
- **Spec rules:** `mod.pkg.6`, `mod.pkg.7`

## Context

ADRs 0231 and 0233 designed `dep NAME git URL rev|tag PIN` but the
bootstrap rejected it. Agents add dependencies by URL; making them clone by
hand and switch to `path` costs steps and loses the pin. Constraint: no
hosted registry (OQ-049 is the designer's), and ADR 0009 prefers custom
tooling unless an existing tool meets every requirement.

## Options considered

### Option A — run the system `git` (chosen)
- **Pros:** handles every transport, credential helper and proxy the user
  already set up; no new dependency in the toolchain; about 200 lines.
- **Cons:** needs `git` installed; its error text varies by version.

### Option B — a git client in the toolchain (Rust crate or own code)
- **Pros:** no external program. **Cons:** a large crate (gix, libgit2) or
  months of protocol work; credentials and proxies to reimplement.

### Option C — download archives (`.tar.gz` per tag)
- **Pros:** no git. **Cons:** host-specific URLs; no commit hash to lock.

## Decision

The toolchain clones `URL` with `git clone --no-checkout`, resolves the pin
(`refs/tags/TAG` or `REV`) to a full commit, checks it out, removes `.git`,
and moves the tree to `$AIL_HOME/deps/NAME-COMMIT` (`~/.ail/deps` by
default) with read-only files. A cached commit is reused and never updated.
`GIT_TERMINAL_PROMPT=0` stops password prompts. Failures are `mod.pkg.7`
errors at the `dep` line naming the dependency. The program needs no grant
for the fetch; `--deny net` makes the toolchain fetch nothing.

## Rationale

ADR 0076 criteria: tokens are equal (no syntax change). AI understanding
decided: an agent writes one manifest line and gets a diagnostic naming the
dependency on any failure. Option A meets every requirement ADR 0009 asks of
an existing tool for this job (it is the reference implementation of the
protocol), and B would be custom work with nothing gained. A git dependency is
then an ordinary package, so grants (`eff.cap.4`) apply unchanged.

## Consequences

- The cache is per user and shared by every project; clearing it is
  deleting the directory. Files are read-only so an edit can't silently
  diverge from the lock.
- The cache key is name plus commit. The design's content hash (`sum`) is
  dropped, see ADR 0356.

## Rejected alternatives

- **B:** too large for the bootstrap; revisit when the toolchain is in AIL.
- **C:** no stable commit to lock and host-specific.
- **Fetching through the program's `std.proc` capability:** the fetch is the
  toolchain's, before the program exists; the program's grants don't apply.
