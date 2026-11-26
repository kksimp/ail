# 1195. New projects run deny-by-default

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** the designer (approved building edge idea 9 on 2026-11-25; the capability list is the AI's, delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Package management (capabilities); 21 — Tooling
- **Spec rules:** [tool.new.6](../spec/22-tools.md), [eff.allow.12](../spec/18-effects.md)
- **Builds on:** [ADR 1072](1072-scoped-runtime-capabilities.md), [ADR 1105](1105-scoped-caps-in-manifests-and-review.md), [ADR 0903](0903-ail-new.md)

## Context

Scoped capabilities are built: `--allow` (ADR 1072) and scoped `caps` in `ail.pkg`
(ADR 1105) reject at compile time and trap at run time. Both are opt-in, so no
project has them. Code an agent wrote runs with the user's full authority unless
someone types a flag. `ail test` already runs each test in a sandbox
(`test.sandbox.1`); `ail run` does not.

## Options considered

### Option A — `ail new` writes a scoped `caps` line (chosen)
The new project's manifest is `package NAME caps fs=.,env,clock,random`.
- **Pros:** uses only what is built; no existing program changes; the limit is one
  visible, versioned line that `ail review` reports when it widens (`eff.allow.11`);
  removing the list removes the limit.
- **Cons:** only projects made by `ail new` get it.

### Option B — every `ail run` denies unless a manifest allows
- **Pros:** covers single files and old projects.
- **Cons:** breaks every existing program that reads a file or the network, the
  examples, the corpus and the gate suites; a single file has no manifest to hold
  the list.

### Option C — a `--sandbox` flag on `ail run`
- **Pros:** no default changes.
- **Cons:** it is `--allow` again: agents with a shell use `check`, `run` and `test`
  with no flags, so nobody gets it.

### Option D — an operating-system sandbox
Rejected in ADR 1072 (three unrelated mechanisms, not at parity). It can be added
under the same list later.

## Decision

1. `ail new NAME` writes `package NAME caps fs=.,env,clock,random` (`tool.new.6`),
   with a two-line comment saying what it does, and prints one line about it.
   `ail new .` writes no manifest and is unchanged.
2. A path scope in a manifest that is not absolute is taken from the manifest's
   directory (`eff.allow.12`). Before, it was taken from the working directory, so
   `ail run NAME` from the parent directory scoped the wrong tree.
3. A policy that came from a manifest carries `manifest=PATH`, an entry that allows
   nothing. The denial then reads `add ENTRY to the caps of the package line in PATH`
   in place of the `--allow` advice (`eff.allow.12`).

The list: `fs=.` is the project directory. `env` is granted because `env.args()` is
in it and nearly every command-line program reads its arguments; a variable can't
leave the machine without `net`. `clock` and `random` carry no authority. `net`,
`net.serve`, `proc`, `ui` and `app` are left out, so reaching one is error `eff.cap.5`
at the `package` line, whose help names the word to add.

## Rationale

AI understanding first: the refusal arrives in the output of `check` and `run`, names
one line of one file, and the fix is an edit a reviewer sees. Nothing has to be
discovered. Readability: the limit is in the manifest, not in a flag the next
person may not type. Speed: a policy check is one scan of the environment per
standard-library call that reaches the system, as in ADR 1072; a project without
`caps` pays nothing.

## Consequences

- Existing projects, single files, the examples and the apps are not affected: a
  manifest without a scoped `caps` list sets no policy.
- An existing manifest with a relative path scope, run from a directory other than
  its own, now scopes the package's tree (the reading its author meant).
- A new project that calls the network, another program or a window fails `ail check`
  until its `caps` gains the word. An agent can add it; the gain shows in `ail review`
  as `NEW CAPABILITY`.
- A program built with `ail build` and started by hand is checked at compile time
  only (`eff.allow.8`).
- It is not a sandbox (`eff.allow.9`): unsafe code and C calls need their own
  manifest line and are then unrestricted.
- The Windows fast path of `ail run` inside a gate (`AIL_FAST_RUN`) does not read a
  manifest policy; unchanged here.
