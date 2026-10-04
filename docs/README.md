# AIL documentation

Start here. Documentation is plain Markdown ([ADR 0003](decisions/0003-documentation-in-repo-markdown.md)).

| Document | What it's for |
|---|---|
| [vision.md](vision.md) | Identity, goals, and design philosophy (from the founding brief) |
| [status.md](status.md) | What is decided, specified, implemented, and tested, at a glance |
| [roadmap.md](roadmap.md) | Phased development plan |
| [open-questions.md](open-questions.md) | Unresolved design questions, with options, pros, cons, and recommendations |
| [decisions/](decisions/README.md) | Architecture Decision Records: one per decision, including rejected alternatives |
| [spec/](spec/README.md) | Normative language specification with stable rule IDs |
| [glossary.md](glossary.md) | Shared terminology |
| [design/](design/) | Non-normative design notes and analyses |

Design notes:
- [design/feasibility.md](design/feasibility.md): is AIL achievable, and what
  token-efficiency and performance levels are realistic?
- [design/performance-beyond-c.md](design/performance-beyond-c.md): research
  on matching or beating C for systems and allocation-heavy code.
- [design/toolchain-constraints.md](design/toolchain-constraints.md): hard
  external limits on a fully custom toolchain.
- [design/ai-author-wishlist.md](design/ai-author-wishlist.md): what would
  make AIL easiest and most efficient for an AI author (proposals).
- [design/syntax-measurements-r1.md](design/syntax-measurements-r1.md): first
  real tokenizer measurements of candidate syntax (block style, signatures,
  keywords).
- [design/indentation-examples.md](design/indentation-examples.md): the same
  code in each candidate indentation style (OQ-026).

Contributors, human or AI, should read [AGENTS.md](../AGENTS.md) for the
documentation rules. **A change is not complete until its documentation is.**
