# 0868. AI-first tooling after Rust removal: eight native features, zero cost when unused

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** Designer (2026-10-23, approved all eight items of the AI-first wishlist)
- **Topic:** 22 — Tooling
- **Builds on:** [ADR 0147](0147-typed-holes-todo.md), [ADR 0394](0394-token-budgets-on-every-command.md), [ADR 0398](0398-record-and-replay.md), [ADR 0016](0016-optimized-and-observable-debugging.md)

## Context

AIL's goal is to be the language agents write best, while staying as fast or almost as fast
as C. Several agent tools exist today only in the Rust interpreter (record and replay,
capability denial) or only in part (typed holes list names, not candidates). Once the Rust
bootstrap is gone ([plan](../roadmap.md)), each tool needs a native form.

## Decision

These eight items start after Rust removal, in this order of priority. Each must cost
nothing in a program that doesn't use it: no runtime check, no extra code, no slower build.

1. **`ail ask`: semantic queries.** Questions such as "who writes field `x`?", "what calls
   `f`?" and "which effects reach `main`?" are answered from the compiler's own graph of the
   program, with no grep and no file reading.
2. **`uhwhatnow?`: the hole that asks** (name chosen by the designer, 2026-10-23).
   `uhwhatnow?` is a typed hole like `todo` (ADR 0147): it has whatever type its place
   needs, and evaluating it traps. Its warning lists the in-scope values and calls whose
   type fits, ranked by fit. The lexer reads `uhwhatnow?` as **one keyword token**, and
   `uhwhatnow` without the `?` is a reserved word that does not compile, with a fix that
   adds the `?`. So it never meets the try operator: `x?` keeps its meaning, and no
   identifier can be named `uhwhatnow`. The try operator on a hole would mean nothing
   anyway, since a hole already has any type. **`todo` stays** as the quiet placeholder
   for unfinished code: its warning is one line. `uhwhatnow?` is the question form and
   gets the expanded, ranked answer. They differ only in how much the compiler says (as
   item 6 does), and existing code keeps compiling.
3. **Edit impact.** `ail edit` reports which tests and effects an edit touches, and
   `ail test --affected` runs only those.
4. **Native time travel.** A native build records a run (inputs, scheduling, syscalls) when
   asked to. Replay answers questions such as "when did `x` become 0?" without adding prints
   (native form of ADR 0398). One native instrumentation layer (line, call and value
   events, compiled in only when a flag asks) serves replay, `ail test --trace`
   (trace.test.1-2, the trap snapshots) and `ail test --coverage`. These two test flags
   were interpreter-only; they come first, because Rust removal (step D) needs them.
5. **Contracts checked by tools** (lower priority). `requires` and `ensures` are fuzzed or
   checked statically, so agents get counterexamples rather than traces.
6. **Token-priced diagnostics.** Every diagnostic has a one-line form and an expanded form,
   and `--budget` (ADR 0394) chooses between them.
7. **Native agent sandbox.** Running generated code with capability denial (`eff.cap.1`) plus
   CPU, memory and wall-time caps, enforced by the native runtime.
8. **Live attach.** A running native binary answers state, metrics and heap-summary queries
   over a small local protocol, so `@observe` works for every program. Only programs built
   with attach enabled pay for it.

## Consequences

Each item gets its own ADR, spec rules and owner when work starts. The roadmap lists the
items after the Rust-removal milestone.

## Rejected alternatives

- **A bare `?` as the hole token:** it clashes with the try operator (`x?`).
- **`uhwhatnow` without the `?`:** this parses too, but the `?` is what makes it read as
  a question, which was the designer's point. As one token it costs nothing in parsing.
- **Replacing `todo`:** this would break existing code and lose the quiet form.
- **Build these on the interpreter first:** the interpreter is being removed; work done there
  would be thrown away.
- **Always-on recording or attach:** this would cost speed in every program, against the
  near-C goal.
