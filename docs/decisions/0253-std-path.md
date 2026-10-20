# 0253. `std.path`: `dir`, `base`, `ext` and `join` on path text

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Standard modules; token efficiency
- **Spec rules:** std.path.1–4

## Context

Every agent-corpus program found its own directory with
`parts := env.program().split("/")` and `parts[..parts.len() - 1].join("/")`
(two lines, three times). Python uses `pathlib`.

## Options considered

### Option A — a `std.path` module (chosen)
- **Pros:** −31.8 tokens (−1.8% of the corpus); `dir`, `base`, `ext`,
  `join` cover what agent code does with paths; pure text, so no effect or
  capability; correct on Windows separators.
- **Cons:** one more `use` line in files that need it.

### Option B — `env.program_dir()`
- **Pros:** shortest for this one case.
- **Cons:** solves only the program's directory; `base` and `ext` stay
  hand-written.

### Option C — path methods on `str`
- **Cons:** puts file-system vocabulary on every string; no module name at the
  call to say what the text is.

## Decision

Add `std.path` with `dir`, `base`, `ext` (without the dot) and `join`,
working on text only, with `/` (and `\\` on Windows) as separators.

## Rationale

Tokens (−1.8%) are under the threshold; **AI understanding** decided: the
names match `dirname`/`basename`/`join` closely, edge cases are specified
(`dir("x")` is `.`, never `""`, so `"{dir}/data"` stays relative), and a
module keeps the vocabulary off `str`.

## Consequences

- `ail effects` reports nothing for `std.path` calls.
- Normalisation (`..`, `.`) and absolute paths are left out until a program needs them.

## Rejected alternatives

- `env.program_dir()`: one case only.
- `str` methods: wrong home for the vocabulary.
