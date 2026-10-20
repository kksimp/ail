# 0222. The compiler has four representations: AST, HIR, MIR and machine IR

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 19 — AST/IR architecture; 23 — Compiler architecture
- **Spec rules:** none (compiler architecture)
- **Design note:** [ir-architecture.md](../design/ir-architecture.md)

## Context

Topic 19 was not started. ADR 0015 needs a query compiler, ADR 0016 needs
recovery records in every pass, ADR 0018 needs an SSA backend, and ADR 0024
needs a semantic API that is stable while all of that changes. One
structure can't serve an API that must stay stable and an optimiser that
must keep changing. Tokens are unaffected (nothing is written in source), so
**AI understanding decides**, then speed.

## Options considered

### Option A — AST → HIR → MIR → machine IR (chosen)
- **Pros:** the API serves a resolved, typed, source-shaped HIR that maps
  every node to a span; the optimiser works on an explicit SSA MIR; each
  tier can change without moving the others.
- **Cons:** two lowerings to keep correct; mitigated by the interpreter
  running HIR as the oracle (ADR 0032) and a MIR verifier after every pass.

### Option B — one rich tree all the way down
- **Pros:** one structure.
- **Cons:** control flow and allocation stay implicit, and every backend
  change is an API change.

### Option C — a lossless syntax tree as the API (Roslyn-style)
- **Pros:** perfect round-tripping.
- **Cons:** much larger answers for agents (criterion 1 on API output); the
  formatter already owns layout.

## Decision

1. **AST:** per file, from the parser; internal; no trivia (`ail fmt` works
   from tokens).
2. **HIR:** the resolved, typed, *expanded* program: items named by stable
   id (ADR 0151), every expression typed, source constructs kept, generic
   code not monomorphised, every node with an origin (a span, or a
   generated origin: ADR 0228). The API, the checker's facts and the
   interpreter work on it.
3. **MIR:** per body instance; CFG with SSA values (block parameters);
   explicit places, allocations, copies, trap checks, dispatch and effect
   operations; optimisation decisions recorded as data with their inputs
   (ADR 0015); recovery records for every source variable (ADR 0016);
   verified after every pass. Written in AIL by the backend (ADR 0022).
4. **Machine IR:** per ISA, after instruction selection; carries recovery
   records into debug info.

## Consequences

- **Designer, look first at:** the HIR/MIR split of point 2–3 (the
  interpreter runs HIR, the compiler runs MIR), since it fixes where the
  oracle and the backend diverge.
- The bootstrap needs no restructuring: `Resolved` already holds HIR facts
  as position-keyed side tables; they become HIR fields when the query
  compiler starts (Stage 3).
- Lane 39's parallelisation decisions are MIR decision records.
- The serialised HIR handed from the Rust front end to the AIL backend is a
  new internal format, to be designed with the backend.

## Rejected alternatives

- **One structure (B):** couples the API to the optimiser.
- **Lossless tree as API (C):** too large for agents; layout belongs to `ail fmt`.
- **Sea of nodes for MIR:** harder to keep per-statement recovery records
  and decision explanations; can be revisited inside the backend.
- **Monomorphised HIR:** many copies of one function in the API; ADR 0080
  makes instantiation a compiler choice.
