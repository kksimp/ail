# 0029. Token-efficiency benchmarks: program corpus, multiple tokenizers, and AI task runs

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 28 — Testing strategy; 2 — Core syntax (decision input)
- **Spec rules:** none
- **Resolves:** OQ-007

## Context

Token efficiency is an explicit project goal (brief §4). It is the scoreboard
for syntax decisions, so the way it is measured needs to be decided before
those decisions are made. The [feasibility note](../design/feasibility.md)
shows that most of the gains come from context and repair loops, not from
syntax alone.

## Options considered

### Option A — Corpus + several tokenizers + AI task runs (chosen)
- **Pros:** Honest and repeatable. Measures the whole AI loop. Hard to game
  with cryptic syntax.
- **Cons:** Expensive to build and maintain. Task runs cost money and vary
  between runs, so each needs repeated trials.

### Option B — Corpus + tokenizers only
- **Pros:** Cheap, deterministic, and can run in CI.
- **Cons:** Misses the biggest wins. Encourages over-optimising syntax.

### Option C — Static counts now, tasks from Stage 2
- **Pros:** Matches what can be measured at each stage.
- **Cons:** Early decisions rest on the weaker metric.

## Decision

AIL's token-efficiency benchmark has three layers:

1. **Corpus.** A fixed, versioned set of equivalent programs (algorithms, CLI
   tools, network servers, data processing, concurrency patterns) written in
   AIL, C, Rust, Go, Python, TypeScript and Zig.
2. **Tokenizers.** Counts from several real model tokenizers, never just one.
   Results are reported per tokenizer and in aggregate.
3. **Tasks.** AI agents perform *write*, *modify* and *repair seeded errors*
   tasks on the corpus. Total tokens consumed (input and output) and success
   rate are measured, with repeated trials and the variance reported.

## Consequences

- **Layers 1–2 can start during Stage 1.** AIL syntax candidates can be
  written by hand and counted before a compiler exists. Syntax ADRs should
  cite these measurements.
- Layer 3 needs a working `ail check` (Stage 2+). Until then, syntax decisions
  rest on layers 1–2, with readability judged by the designer.
- **Measurement instruments are not product** (like comparison compilers under
  ADR 0010). Using third-party tokenizers and model APIs for measurement does
  not conflict with ADR 0009.
- **Constraint:** some vendors' tokenizers are only available through paid
  network APIs. Open tokenizers can run locally. The benchmark should record
  exactly which tokenizer versions it used.
- Roadmap refinement #2 is partly accepted: token benchmarks start in Stage 1.

## Rejected alternatives

- **Static counts only:** misses context and repair savings, and invites
  cryptic syntax.
- **Delaying the method choice:** the full method is adopted now, and each
  layer is applied as soon as it is feasible.
