# 0153. A package is rooted at the directory run; imports load lazily and can't form cycles

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 12 — Modules
- **Spec rules:** [mod.unit.3](../spec/12-modules.md), [mod.unit.5](../spec/12-modules.md), [mod.use.4–5](../spec/12-modules.md)

## Context

ADRs 0097–0099 define modules as directories, but not where the package root
is, what `ail run` loads, or whether imports may be cyclic. Packages and
manifests are still undecided (build-system topic), so the bootstrap needs a
rule that works without a manifest and keeps ADR 0101's single-file programs.

## Options considered

### Option A — `ail run DIR` is a package; `ail run FILE` is the file plus its imports (chosen)
- **Pros:** the argument means exactly what it says; every file in
  `examples/` keeps running alone even though its siblings each declare
  `main`; no magic file names.
- **Cons:** `ail run pkg/main.ail` doesn't see a sibling `pkg/util.ail`;
  the program needs `ail run pkg`.

### Option B — a file named `main.ail` makes its directory a package
- **Pros:** `ail run pkg/main.ail` sees its siblings.
- **Cons:** a magic name; the same command means different things depending
  on the file's name.

### Option C — any file makes its directory the root module
- **Cons:** breaks `examples/` and ADR 0101.

### Cycles: forbidden (chosen) or allowed
Allowing cycles costs nothing in the interpreter, but forbidding them keeps
dependencies layered, lets a future compiler check modules in dependency
order, and gives an AI a reading order. Go forbids them for the same reasons.

## Decision

`ail run DIR` (and `ail check DIR`) makes the `.ail` files directly in
`DIR` the root module, with `DIR` as the package root. `ail run FILE` makes
`FILE` alone the root module, with its directory as the package root.
`use a.b` loads every `.ail` file directly in `ROOT/a/b`, only when
imported, transitively. A missing module (`mod.use.4`) and an import cycle
(`mod.use.5`, naming the cycle) are compile-time errors.

## Rationale

AI understanding decided: the command line has one meaning, there are no
magic names, and dependencies form a DAG an agent can read bottom-up. Token
cost is unaffected.

## Consequences

- Diagnostics, traps and error traces name the file each position is in.
- A manifest, when designed, can name the root explicitly; this rule is its
  default.

## Rejected alternatives

- **B:** a magic file name makes the same command mean two things.
- **C:** breaks single-file programs (ADR 0101).
- **Allowing cycles:** no layering, and it rules out modular checking later.
