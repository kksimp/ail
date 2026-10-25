# 0908. The apps live in their own repositories

- **Status:** Accepted
- **Date:** 2026-10-25
- **Decided by:** Designer ("we need to break up the project. I want AIL on the main repo", 2026-10-25)
- **Topic:** 22 — Tooling

## Context

`kksimp/ail` held the language and four applications written in it: the Garlic browser (`tools/garlic`), the PDF viewer (`tools/pdf`), the code editor (`tools/edit`) and the Clove compressor. The apps made up a large part of the tree and of the gate, and every language change had to keep them green.

## Decision

The repository holds the language only: compiler, std, runtime, tools and docs. Each app has a private repository under AGPL-3.0, with its history kept (`git filter-repo`):

| App | Repository |
|---|---|
| Garlic browser (with its JavaScript engine) | `kksimp/garlic` |
| PDF viewer, named Garlic Press | `kksimp/garlic-press` |
| Code editor | `kksimp/ail-edit` |
| Clove compressor | `kksimp/clove` |

- The `ail` repository stays Apache 2.0 with the Runtime Library Exception (ADR 0510). The apps are separate works.
- Each app pins the ail commit it is checked against in `.ail-version` and runs `tools/check.sh` (check, fmt, tests) with ail checked out at `../ail`. `ail` does not test the apps; the gate suites for them are deleted.
- Garlic depended on `tools/pdf/image` by path. A git dependency is a whole repository (ADR 0231), so Garlic carries a copy in `vendor/image`. Moving the decoders into std (ADR 0513) removes the copy.
- Design notes and cards for the apps moved with them. ADRs stay here as history.

## Rejected alternatives

- **Git submodules:** tie the ail checkout to the apps again and break scripts that list tracked files.
- **Keep the monorepo with a separate app gate:** the repository stays large and every language change still touches the apps.
- **Depend on the apps by git dependency from ail:** ail must not depend on programs written in it.
- **A plain subtree split without history:** loses blame and the record of why each piece exists.
