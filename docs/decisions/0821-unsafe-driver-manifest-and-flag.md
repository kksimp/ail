# 0821. `unsafe driver` in `ail.pkg`, and `--unsafe driver` for a program without one

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved [ADR 0820](0820-driver-level-unsafe.md)
- **Topic:** 15 — Unsafe semantics; 12 — Modules
- **Spec rules:** [unsafe.hw.1](../spec/13-unsafe.md), [mod.pkg.2](../spec/12-modules.md)
- **Builds:** [ADR 0820](0820-driver-level-unsafe.md), [ADR 0430](0430-unsafe-blocks-and-functions-built.md)

## Context

ADR 0820 adds the manifest level `unsafe driver`. This ADR fixes where it is
read, how a program with no `ail.pkg` gets it, and how it relates to
`unsafe allow`.

## Options considered

### Option A — the value `driver` of the existing `unsafe` key, plus a command-line flag (chosen)
- **Pros:** no new manifest key; a typo is caught by the existing
  "not an unsafe setting" error; `driver` implies `allow`, so one word is
  enough; the flag mirrors ADR 0430's rule that a manifest-less program opts
  in by being run.
- **Cons:** the flag is one more word on the command line.

### Option B — no flag; a program without `ail.pkg` may always use `std.hw`
- **Cons:** a pasted snippet could reach device memory, which ADR 0820 means to prevent.

### Option C — a separate key `hw allow`
- **Cons:** two keys to keep in step, and a package could have `hw` without `unsafe`.

## Decision

1. `unsafe driver` on the `package` line or a `dep` line gives the package
   `unsafe allow` and the right to use `std.hw` and the volatile methods.
   `unsafe allow` alone does not.
2. A program with no `ail.pkg` needs `--unsafe driver` on `ail run`, `ail check`
   and the other commands. Without it, `std.hw` is `unsafe.hw.1` at the use.
3. A package that lacks `unsafe allow` is reported once, at its `unsafe`
   block (`unsafe.block.3`), not again for each `std.hw` use.
4. `ail api --ast` carries the list as `driver_allowed`, so the checker in
   `compiler/check/` reads the same facts (ADR 0740).

## Rejected alternatives

- Option B: rejected because it lets snippets reach hardware without a visible grant.
- Option C: rejected because two keys can disagree.
