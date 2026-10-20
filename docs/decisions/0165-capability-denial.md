# 0165. `--deny` rejects at compile time a program that reaches a denied capability

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API; 13 — Package management (capabilities only)
- **Spec rules:** [eff.cap.1–3](../spec/18-effects.md)
- **Plan:** [ai-first-plan.md](../design/ai-first-plan.md) §3.2

## Context

An AI-written tool should be sandboxable by what it can reach, not by a
container (wishlist 5.1, 6.6). With inferred capabilities (ADR 0163), the
toolchain can refuse a program before it runs. The deciding criterion is
**AI understanding** (and safety): a refusal with the call chain is an
actionable answer; tokens are unaffected.

## Options considered

### Option A — compile-time denial with `--deny LIST` on `run` and `check` (chosen)
- **Pros:** nothing runs; the error names the chain; `check --deny` lets an
  agent test a program against a policy without running it.
- **Cons:** conservative where dispatch is dynamic: a program can be refused
  for an implementation it never calls.

### Option B — runtime denial (trap when a denied call happens)
- **Pros:** exact.
- **Cons:** the program may have done half its work first; a path not taken
  in testing fails later.

### Option C — allow-list (`--allow fs`)
- **Pros:** safer default for untrusted code.
- **Cons:** every existing program would need flags; the plan asks for
  "nothing changes for a program that uses no capabilities". An allow-list
  belongs in the package manifest, which waits for Topic 13.

## Decision

`ail run --deny LIST FILE` and `ail check --deny LIST FILE` report
`eff.cap.1` at `main` for each denied capability that `main` has, naming the
chain (`main -> save -> fs.write_text`), and exit 2. `LIST` is
comma-separated capabilities or `all`; an unknown name is a usage error
(`eff.cap.2`), and `io` and the other effects can't be denied (`eff.cap.3`).
`net` is accepted though nothing gives it yet.

## Rationale

Compile-time refusal is the strongest guarantee available without a
sandbox, and it composes with the conservative inference: anything the
program *might* reach is refused. Printing stays allowed because it is how
the program reports to the agent that ran it.

## Consequences

- Deterministic mode (§3.3) can be defined as denying `clock` (and a future
  `random`) plus seeding.
- The package manifest can later carry a default deny or allow list.
- `ail effects` doesn't take `--deny`.

## Rejected alternatives

- **Runtime denial (B):** partial effects before the trap; failures depend on
  inputs.
- **Allow-list by default (C):** breaks every existing program; deferred to
  manifests.
- **Denying `io`:** a program that can't print can't report; `io.*` reads
  and `exit` don't reach outside resources beyond the process's own streams.
