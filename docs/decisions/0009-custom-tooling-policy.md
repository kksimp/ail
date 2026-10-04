# 0009. Build custom tooling unless an existing tool matches every requirement

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** Project process; 23 — Compiler architecture; 27 — Build system (all tooling)
- **Spec rules:** none

## Context

The founding brief prefers owning critical infrastructure (principle 9) but
left room for existing components "where it dramatically accelerates
progress". The designer has now made that preference concrete.

## Options considered

### Option A — Custom unless an existing tool matches exactly (chosen)
An existing tool is acceptable only if it does **everything** AIL needs, the
way AIL needs it. If it fails even one requirement, AIL builds its own.
Research and development is preferred over losing a feature that custom
tooling could provide.
- **Pros:** No feature is ever lost to a third-party tool's limits. Full
  control over AI-facing data such as diagnostics, IR, debug info and reports.
  No inherited assumptions.
- **Cons:** Much more work and a longer time to first usable release. AIL must
  maintain its own linker, debug-info writers, object writers and so on.

### Option B — Reuse wherever "good enough"
- **Pros:** Fastest progress.
- **Cons:** Features end up bounded by other tools. Contradicts the project's
  identity.

### Option C — Custom only for core differentiators
- **Pros:** Balanced effort.
- **Cons:** "Differentiator" is subjective and erodes over time. The designer
  rejected it.

## Decision

AIL builds its own tooling by default. An existing component may be used only
if it matches **every** AIL requirement. Missing a single required capability
disqualifies it. When in doubt, choose R&D.

## Rationale

The designer would rather invest in research than accept capability limits.
The AI-first goals (structured data flowing through every layer) are
especially hard to retrofit onto tools AIL doesn't own.

## Consequences

- Every proposal to use an existing component must list AIL's requirements and
  show that the component meets all of them. Otherwise the proposal is rejected.
- Some external dependencies are **unavoidable** and fall outside this policy
  because no custom alternative is possible: OS system libraries on macOS
  and Windows, Apple notarization, and OS loaders. They are listed in
  [design/toolchain-constraints.md](../design/toolchain-constraints.md).
- Comparison and validation tools (other compilers used only for benchmarking
  or differential testing) are allowed, because they are not part of AIL's
  product.

## Rejected alternatives

- **Reuse when good enough:** caps AIL's capabilities at other projects' limits.
- **Custom for differentiators only:** a subjective line that would erode.
