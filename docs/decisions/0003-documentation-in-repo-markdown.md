# 0003. Keep documentation as Markdown in the repository; build an AIL docs tool later

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** Project process / documentation
- **Spec rules:** none

## Context

The designer requires excellent documentation. Docs must stay in sync with the
code, be versioned with releases, be easy for AI agents to read, and avoid
dependencies the project doesn't control (brief principle 9).

## Options considered

### Option A — Plain Markdown in `/docs`, replaced later by a docs tool written in AIL (chosen)
- **Pros:** No dependencies. GitHub renders it. It is versioned alongside the
  code and AI-readable. The later tool will exercise AIL on a real project.
- **Cons:** No search or navigation site until the custom tool exists.

### Option B — Markdown plus an existing static-site generator (mdBook etc.)
- **Pros:** A polished, searchable site early on.
- **Cons:** An external dependency and site config to maintain, which runs
  against the "own the stack" goal.

### Option C — Separate wiki or docs repository
- **Pros:** Non-coders can edit, and the code repo stays lean.
- **Cons:** Drifts from the code, isn't versioned with releases, and is worse
  for AI agents working in the repo.

## Decision

All project documentation is plain Markdown under `docs/`, plus `README.md`
and `AGENTS.md` at the root. A documentation generator written in AIL is a
future roadmap item.

The following documents are maintained from day one:

| Document | Purpose |
|---|---|
| `docs/spec/` | Normative language specification (ADR 0002) |
| `docs/decisions/` | ADRs, including rejected alternatives (ADR 0001) |
| `docs/glossary.md` | Shared terminology |
| `docs/open-questions.md` | Unresolved questions with options, pros and cons |
| `docs/roadmap.md` | Phased plan |
| `docs/status.md` | What is decided, specified, implemented, and tested |
| `AGENTS.md` (+ `CLAUDE.md`) | Rules for AI contributors, including the doc-update rules |

## Rationale

Having no dependencies and versioning docs with the code serves the
"minimise dependencies" and AI-first goals. The AI-agent guide is how
"amazing documentation" is enforced across sessions and contributors.

## Consequences

- Only portable Markdown: no generator-specific extensions.
- Documentation updates are part of every change (see `AGENTS.md`).
- Roadmap: an `ail doc` tool written in AIL (dogfooding).

## Rejected alternatives

- **Static-site generator now:** an avoidable external dependency, and its
  benefits can wait.
- **Wiki or separate repo:** drift and versioning problems, and it is invisible
  to agents working in the repo.
