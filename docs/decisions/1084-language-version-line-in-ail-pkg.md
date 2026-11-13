# 1084. A language version line in `ail.pkg` and a `fix --migrate` hook

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved every item of the AI-first review for 0.1.0 on 2026-11-12
- **Topic:** 13 — Package management; 4 — Versioning
- **Spec rules:** `mod.pkg.10`, `mod.pkg.11`, `mod.pkg.12`

## Context

Code written against an older language version should be migrated, not left stale, and an agent
must be able to tell which language a tree was written for. Nothing in a tree said so.

## Options considered

### Option A — an `ail MAJOR.MINOR` line in `ail.pkg` (chosen)
- **Pros:** one place, same line format as the rest of the manifest (ADR 0230); an agent reads it
  with one lookup; a newer tree is refused with a clear message.
- **Cons:** one more line per manifest.

### Option B — a `version` key on the `package` line
- **Pros:** no new line kind. **Cons:** `package` lines carry capabilities and unsafe settings, so
  a language version would read as the package's own release version.

### Option C — a comment or pragma at the top of each source file
- **Pros:** works for single files. **Cons:** repeated per file and easy to leave stale; single
  files already take the compiler's own version.

## Decision

A manifest may hold one `ail MAJOR.MINOR` line, before or after `package`. `ail new` writes
`ail 0.1`. No line, or no manifest, means the compiler's own version. A higher version is the error
`mod.pkg.11`; a lower one prints nothing in 0.1. `ail fix --migrate [DIR]` reads the version and
prints `nothing to migrate from 0.1`; later releases add one step per version. `ail doctor` shows
the version of the project in the current directory.

## Rationale

ADR 0076: AI understanding decided. A declared version makes a refusal or a migration explicit
instead of a pile of unrelated errors.

## Consequences

- Every release that changes the language adds a migration step and a spec note.
- The tree's `ail.pkg` files written by `ail new` before this change have no line, so they read
  as the compiler's own version.

## Rejected alternatives

- **B:** confuses the language version with a package release.
- **C:** repeated and stale-prone.
- **Failing when the line is missing:** breaks every existing manifest for no gain in 0.1.
