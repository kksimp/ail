# AGENTS.md — Guide for AI agents working on AIL

This file is the operating manual for any AI agent (Claude, Codex, Copilot, etc.)
contributing to AIL. Humans are welcome to read it too. Keep it short, accurate,
and current. If something here is wrong, fix it in the same change that
discovers it.

## What this project is

**AIL** (AI-First Language, pronounced "ale", by **Voltare**) is a native,
garbage-collected, memory-safe systems programming language designed so that
humans and AI agents are both first-class programmers. Source files use `.ail`.
The CLI is `ail` (`ail build`, `ail run`, `ail test`, `ail fmt`, `ail check`).

The project is in **Stage 1: language specification and design**.
**Do not write compiler code** until the designer explicitly starts Stage 2.

## Git workflow

Per [ADR 0028](docs/decisions/0028-git-workflow.md):
- **`main` is the only long-lived branch.** Always reduce back to `main`.
- **Docs and spec changes:** commit directly to `main`.
- **Code changes (Stage 2+):** use a short-lived branch and a PR that must
  pass the five-target CI gate. Merge and **delete the branch** as soon as
  it passes.
- Keep open branches to the minimum (one per change in flight). Track every
  branch you create and report it until it's merged and deleted.
- Commit with clear messages, and push when a change is complete.

## Roles

- **The human is the systems designer.** They make every design decision.
- **The AI is the implementer and lead architect-advisor.** It analyses,
  recommends, challenges assumptions, and implements. It never makes a
  language design decision itself.

### How to raise design questions

1. Ask about **every** open design question. Don't silently choose a default
   for anything that affects the language, toolchain, runtime, or project
   policy.
2. Every option must list its **pros and cons**. Put the recommended option
   first and mark it "(Recommended)".
3. Prefer structured multiple-choice questions, and batch related questions
   together.
4. Challenge the designer's assumptions when a choice conflicts with the
   project's goals (performance, simplicity, AI-first, safety, cross-platform).
5. If the designer dismisses a question, **stop and wait**. Do not proceed on
   assumptions.
6. Put code examples and comparisons **inside the question previews, or in a
   file sent to the designer**. Text written alongside a question prompt may
   not be visible to the designer while they answer.
7. Back syntax questions with **measurements** (ADR 0029) whenever possible.

## Documentation rules (mandatory)

Documentation is a first-class deliverable. A change is **not complete** until
its documentation is complete.

| When you… | You must… |
|---|---|
| Get a design decision from the designer | Write an ADR in `docs/decisions/` (see the template), add it to the index, and update `docs/status.md` |
| Identify an unresolved question | Add it to `docs/open-questions.md` with options, pros and cons, and a recommendation |
| Resolve an open question | Move it out of `open-questions.md` into an ADR, and link the ADR from where the question used to be |
| Add or change normative language behavior | Edit `docs/spec/` using rule IDs (see `docs/spec/README.md`) and cite the ADR that justifies it |
| Introduce a new term | Add it to `docs/glossary.md` |
| Finish a roadmap item or change its scope | Update `docs/roadmap.md` and `docs/status.md` |
| Supersede a decision | Never delete the old ADR. Set its status to `Superseded by NNNN` and link both ways |

Style:
- Use plain Markdown, with no generated or site-specific syntax (ADR 0003).
- Write dates as ISO 8601: `YYYY-MM-DD`.
- Use relative links between documents.
- Every topic records its **rejected alternatives** and why they were rejected.
- Write precisely and concisely. Docs are loaded into AI context, so every
  sentence should earn its tokens.
- Never put a model name or model identifier in any committed file.

## Repository layout

```
AGENTS.md                 this file
CLAUDE.md                 imports AGENTS.md for Claude Code
README.md                 project overview for humans
docs/
  README.md               documentation index (start here)
  vision.md               goals and design philosophy (from the founding brief)
  roadmap.md              phased development plan
  status.md               what is decided, specified, implemented, tested
  open-questions.md       unresolved design questions with pros and cons
  glossary.md             shared terminology
  decisions/              Architecture Decision Records (ADRs), one per decision
  spec/                   normative language specification with rule IDs
  design/                 non-normative design notes, research, and constraints
```

## Settled constraints (summary; the ADRs are authoritative)

- Targets: Linux, Windows, and macOS are all first-class. macOS is ARM64 only (ADR 0004).
- All targets are 64-bit and little-endian (ADR 0007).
- x86-64 baseline is x86-64-v3, with runtime dispatch for higher levels (ADR 0005).
- ARM64 baseline is ARMv8.2-A (ADR 0006).
- x86-64 and ARM64 are developed in **strict lockstep**. Nothing lands unless
  it works and is optimised on both (ADR 0008). All five targets gate CI (ADR 0027).
- The memory model is abstract and weakly ordered. Safe code is data-race-free.
  Atomics default to SeqCst (ADR 0025).
- **Custom tooling by default.** An existing tool is acceptable only if it meets
  *every* requirement. Prefer R&D over losing a feature (ADR 0009).
- **No LLVM or any external backend, ever.** Clang, gcc and rustc are allowed
  only for benchmarks and differential testing (ADR 0010).
- Resemblance to existing languages is not a design goal. Tokenizer cost is
  still measured (ADR 0011).
- When a constraint blocks something the designer wants, **explain the
  constraint plainly** so the designer can design around it.
- **Mutable value semantics:** values are independent, `mut` is marked at
  call sites, borrows are second-class (no lifetimes), and shared mutation
  only goes through `ref T` (ADRs 0060–0063).
- Bootstrap front end and interpreter are in **Rust**. The optimising backend
  is written **in AIL** (ADR 0022, ADR 0023).
- See `docs/decisions/README.md` for the full list.
