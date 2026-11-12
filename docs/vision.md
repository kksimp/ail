# Vision

A condensed version of the founding brief (2026-10-04). Where this summary and
an ADR disagree, the ADR wins.

## Identity

- **Name:** AIL, short for *AI-First Language*. Pronounced "ale". Never expand
  it to "Artificial Intelligence First Programming Language".
- **Organisation:** Voltare. Styled as "AIL by Voltare" or simply "AIL".
- **Files:** `.ail`. **CLI:** `ail` (`build`, `run`, `test`, `fmt`, `check`).
  **Repository:** `ail`.
- **Personality:** *ail* is French for garlic. That is a light cultural joke,
  never the technical focus. AIL is a serious, production-quality systems
  language.
- The name is permanent unless there is a significant technical or legal reason
  to change it.

## Mission

A modern, native, AI-first programming language and toolchain, designed from
the ground up for humans **and** AI agents. It is not an existing language with
an AI assistant bolted on: the language, compiler, runtime, standard library
and tooling all treat AI as a first-class user.

## Core goals

1. **Performance:** native code, low runtime overhead, aggressive optimisation,
   and speed equal to optimised C as the target (ADR 1065).
   Measured, never just claimed.
2. **Memory:** safe by default, with first-class GC inside a hybrid model:
   stack allocation, escape analysis, arenas, a GC heap, pinning, and
   unsafe manual allocation. The compiler picks the cheapest safe strategy.
3. **AI-first:** features for AI agents better than any other language's,
   while staying readable and unambiguous ([ADR 1065](decisions/1065-goals-c-parity-ai-first.md)).
4. **Token efficiency:** measured, never pursued through cryptic syntax. Using
   more tokens than other languages is acceptable when it buys speed or a
   better AI-first feature (ADR 1065).
5. **Human readability:** pleasant, reviewable, and canonically formatted.
6. **Strong static types:** inference, generics, traits, Option/Result.
7. **Explicit, concise errors:** value-based errors with `?`-style propagation.
8. **Concurrency:** tasks, threads, async, channels, and strong data-race
   protection.
9. **Cross-platform:** Linux, Windows and macOS are all first-class.
10. **Custom toolchain:** own as much of the stack as practical. Existing
    components are allowed only as temporary bootstrap.
11. **Rich compiler representation**, plus an **AI semantic API**, **AI-aware
    diagnostics**, and **structural editing**.
12. **Systems capability:** explicit `unsafe`, first-class C interoperability,
    built-in build and package system, debugging and profiling, and
    performance transparency reports.

## Design philosophy

1. **Write less. Mean more.** No boilerplate, and no cryptic syntax either.
2. **The compiler should do the work.** Infer, optimise and generate whatever
   can be done safely.
3. **Safety by default.** Unsafe operations are explicit.
4. **Performance must be measurable.**
5. **AI and humans are both first-class users.**
6. **The AST is a first-class interface.**
7. **Canonical syntax matters.**
8. **Cross-platform support is fundamental.**
9. **Minimise dependencies.** Own critical infrastructure where it pays off.
10. **Don't prematurely optimise the design.** Semantics first, implementation
    second.

Interpretations made during Stage 1:

- "Predictable data layouts" means **reported and controllable**, not
  "declaration order" ([ADR 0013](decisions/0013-compiler-chosen-layout.md)).
- Canonical syntax (#7) has one documented exception: `int`/`i64` and
  `float`/`f64` are both canonical ([ADR 0059](decisions/0059-default-numeric-spelling.md)).

Added during Stage 1 (ADR 0002):

11. **Correctness must be checkable.** Spec rules are citable and their examples
    are tested.
