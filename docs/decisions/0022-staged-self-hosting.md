# 0022. Staged self-hosting: the optimising backend is written in AIL from the start

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 23 — Compiler architecture; roadmap
- **Spec rules:** none
- **Resolves:** [OQ-011](../open-questions.md) (self-hosting part)

## Context

The designer leaned toward self-hosting after Stage 4 but liked self-hosting
from the start. Writing the optimising backend (ADR 0018), the largest
component, in a temporary language and then rewriting it in AIL would double
the largest single piece of work.

## Options considered

### Option A — Staged self-hosting (chosen)
The front end (lexer, parser, type checker) and the interpreter are written in
the bootstrap language (ADR 0023). The **optimising backend is written in AIL
from the start**, run by the interpreter until it can compile itself. The
front end is then ported to AIL.
- **Pros:** The backend is written once. AIL is tested on a large real
  program while the design can still change.
- **Cons:** The backend runs slowly on the interpreter (possibly 30–100x)
  until it compiles itself. Language changes ripple into backend code.

### Option B — Self-host after Stage 4
- **Pros:** A stable host language during the hardest work.
- **Cons:** The backend gets written twice.

### Option C — Self-host from the very start
- **Pros:** Maximum dogfooding.
- **Cons:** The type checker would be written in a language whose type system
  doesn't exist yet. Very slow start.

## Decision

Self-hosting is staged as described in Option A. The bootstrap sequence is:

1. The bootstrap front end and interpreter (Rust) run the AIL backend
   (written in AIL).
2. The AIL backend compiles itself to native code (the first self-compile).
3. The front end is ported to AIL. The compiler is now fully AIL.
4. The bootstrap chain is kept reproducible, so any released compiler can be
   rebuilt from source.

## Consequences

- **Interpreter performance matters more**: it has to run a large optimising
  compiler. It needs a reasonably fast design (e.g. compiling to a compact
  intermediate form rather than walking the tree directly). This is an input
  to Stage 3.
- **Language stability pressure**: the language features the backend relies
  on must stabilise early. A "backend-ready core subset" milestone is needed
  in Stage 1/2.
- The roadmap is reordered: work on the backend (Stage 4) can begin as soon
  as the interpreter (Stage 3) runs the core subset.
- Whether the Rust interpreter is retained permanently as an independent
  oracle after self-hosting is OQ-022.

## Rejected alternatives

- **After Stage 4:** the largest component would be written twice.
- **From the very start:** the type checker would be built on an undesigned
  type system.
