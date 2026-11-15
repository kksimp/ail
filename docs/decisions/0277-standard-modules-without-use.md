# 0277. Standard modules are in scope without `use`

- **Status:** Accepted (extended by [ADR 1126](1126-std-modules-in-ail-without-use.md): the modules written in AIL follow it too)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Modules; token efficiency
- **Spec rules:** std.use.2–4

## Context

Every agent-style program starts with one `use std.x` line per standard module
it touches: three to five lines, 54.4 tokens on the three-program agent corpus
(3.1%, the largest single item after round 4), 77.6 on the five-program corpus.
Standard modules are always reached qualified (`fs.read_text`), so the line
repeats what each use already says. OQ-071 recommended dropping it once module
name resolution existed (lane 3, ADRs 0097–0099). Measurements are in
[program-measurements-r5.md](../design/program-measurements-r5.md).

## Options considered

### Option A — in scope unless something else has the name (chosen)
- **Pros:** −3.0% on the agent corpus, −0.4% on the examples; small tools,
  the case AIL targets, gain most (−4.3% on `tool`). A qualified call still
  names its module at every use, and `ail effects` lists what a program
  touches, so nothing an agent reads is lost.
- **Cons:** a module's name can be hidden; a program can't see at the top which
  modules it uses.

### Option B — one line, `use std.env, fs, proc`
- **Pros:** keeps a list at the top.
- **Cons:** −0.03% of the corpus (ADR 0184); a second import syntax.

### Option C — keep one line per module
- **Pros:** explicit.
- **Cons:** the largest remaining token cost on agent code.

## Decision

Every standard module is in scope under its own name without `use`, unless the
program declares a type, trait, function or constant with that name, or imports
something else under it (`std.use.4`). A local with the name hides the module
in its scope, and calling a module function through such a local says so.
`use std.NAME` and `use std.NAME as d` keep working; only a module imported
with `use` reserves its name against locals (`std.use.3`, `expr.bind.5`).

## Rationale

Tokens decide (criterion 1): 3.0% of the agent corpus clears the ~2% threshold.
AI understanding (criterion 2) is unchanged or better: names like `path`,
`time` and `env` stay usable for locals (they were errors next to an import),
the qualified call is as explicit as before, and the shadowing diagnostic
explains the one surprise.

## Consequences

- The examples and the agent corpus drop their `use std.` lines.
- `std.use.2`'s "not imported" error and the `ail fix` that added the line no
  longer apply; its fix fixture was removed.
- A future standard module whose name a program already uses is hidden by the
  program's own item, so adding modules doesn't break programs.

## Rejected alternatives

- One-line `use` (B): saves almost nothing (ADR 0184).
- Keeping the lines (C): the largest remaining token cost on agent programs.
- Standard modules reserved everywhere (no shadowing): would break any local
  called `path`, `time` or `env`, which are common.
