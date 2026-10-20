# 0325. `ail api` on package directories

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Semantic API (topic 12); modules (topic 5); testing
- **Spec rules:** `api.pkg.1–4`

## Context

`ail api FILE` outlines one file plus what it imports. A package is a directory
(ADR 0097), and the editor and agents ask about whole packages: what does the
package export, and where is this function used? A single-file query misses
uses in sibling root files and in the modules that import the symbol. Module
renaming (modules.rs) already gives every item a module-qualified name, so ids
are `fn:geo.origin` without new work.

## Options considered

### Option A — `DIR` in place of `FILE`, one extra positional for queries (chosen)
- **Pros:** one new idea (a directory is a package); existing JSON shapes stay;
  position queries load the whole package, so `--refs` is complete.
- **Cons:** an item's `line` is file-local, so a reader needs `file` with it.

### Option B — a new `ail package` command
- **Pros:** no overloading of `ail api`.
- **Cons:** a second surface to learn and document; agents already use `ail api`.

### Option C — global line numbers across the package
- **Pros:** no `file` key needed.
- **Cons:** meaningless in any editor; breaks every consumer of `line`.

## Decision

`ail api [--outline] [--symbols] DIR` outlines the package, with module-prefixed
ids and `module` and `file` keys per item. `--locals|--type|--refs DIR FILE LINE COL`
answers in the context of the whole package. The cursor may sit on the name
after `mod.`. Single-file forms are unchanged.

## Rationale

AI understanding decided (ADR 0076, criterion 2): one query that sees every use
removes a class of missed references. Tokens are a tie: the extra keys are two
short strings per item, and only package mode emits them.

## Consequences

- The editor can rename across a package with `--refs DIR`.
- Locals are tracked for root files only (the first file's lines are the ones
  the scope rules index); a symbol in any module file works.
- A package outline repeats `file` per item; a later change could group items
  by file if tokbench says it matters.

## Rejected alternatives

- New command: duplicates `ail api`'s flags and output machinery.
- Global line numbers: unusable outside the loader.
