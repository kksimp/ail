# 0232. The manifest grants each dependency its capabilities, checked at compile time (resolves OQ-045)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 13 — Package management; 20 — AI semantic API
- **Spec rules:** [eff.cap.2](../spec/18-effects.md#denying-capabilities), [eff.cap.4–5](../spec/18-effects.md#denying-capabilities)
- **Resolves:** [OQ-045](../open-questions.md)
- **Design:** [packages.md](../design/packages.md)

## Context

AI agents add dependencies quickly; a careless or malicious one shouldn't
reach the filesystem or network unless the project says so (wishlist 5.1,
6.6). Effects and capabilities are inferred per function (ADR 0163), and
`--deny` refuses a program at compile time (ADR 0165), which deferred
allow-lists to the manifest.

## Options considered

### Option A — `caps LIST` per `dep` line, default none, checked by inference (chosen)
- **Pros:** the grant is one word on the line that adds the dependency;
  reuses the inference and chains of `--deny`; nothing runs before refusal.
- **Cons:** conservative where dispatch is dynamic (`eff.infer.5`).

### Option B — capabilities as values passed from `main`
- **Pros:** the most explicit. **Cons:** a parameter on every call chain into
  a dependency, a token cost on every call for what inference already knows.

### Option C — nothing at the package level
- **Cons:** an AI-added dependency reaches anything `main` can.

### Default for a `dep` without `caps`: none (chosen) vs. all
None is the safe default; granting is one word, and an omitted grant fails
loudly at the line to fix.

## Decision

A `dep` line's `caps LIST` grants that dependency the listed capabilities,
and only those; without `caps` it gets none. A `package` line's `caps`
limits that package's own code; without it the package is unlimited. A
package's capabilities include those it reaches through its dependencies.
After checking, every function of a package must reach only capabilities
each of its grants includes; otherwise one error per grant and capability,
at the manifest line, naming a chain from a function of the package (a
public one if any): `eff.cap.4` for a `dep` line, `eff.cap.5` for a
`package` line. `LIST` takes the capability names of `--deny`, `all`
or `none` (`eff.cap.2`).

## Rationale

AI understanding and safety decided: an agent reads every dependency's reach
in one file, and a violation names the exact line and the call chain. Tokens:
zero in source code; a grant costs 2–4 tokens per line.

## Consequences

- The root `package` line's `caps` is the allow-list ADR 0165 deferred.
- Every function of a package counts, not only those `main` reaches.
- `unsafe` and FFI bypass inference; they will need their own grant.

## Rejected alternatives

- **B:** token cost on every call chain.
- **C:** no supply-chain protection.
- **Default `all`:** defeats the purpose.
- **Checking only what `main` reaches:** a grant would then depend on one
  program's use of the package instead of on the package.
- **Runtime denial:** partial effects before the trap (ADR 0165).
