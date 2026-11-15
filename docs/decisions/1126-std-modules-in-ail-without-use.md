# 1126. Standard modules written in AIL are in scope without `use` too

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Modules; AI understanding
- **Spec rules:** std.use.1, std.use.4
- **Extends:** [ADR 0277](0277-standard-modules-without-use.md)

## Context

[ADR 0277](0277-standard-modules-without-use.md) put every standard module in
scope without `use`, but only the modules built into the compiler (`fs`, `io`,
`env`, ...) followed it. The modules written in AIL (`csv`, `bigint`, `args`,
`date`, ...) still needed `use std.NAME`, and the cards said both "standard
modules need no `use`" and "these need it". In the E13 agent benchmark a
missing `use std.bigint` or `use std.csv` was one of the commonest
first-build errors of a from-scratch program: two rules where one would do.

## Options considered

### Option A: load a module written in AIL when a file names it (chosen)
A file that writes `csv.MEMBER` gets an implicit `use std.csv`, made by the
loader (`compiler/load/stduse.ail`) for both `ail check` and the native build.
A file that also binds `csv` as a local, parameter or field gets it only when
`MEMBER` is one of the module's `pub` items, so `args.len()` on a local
`args` loads nothing. A local hides the module until its block ends, as it
hides any import ([ADR 1121](1121-locals-may-shadow-functions-and-modules.md)).
- **Pros:** one rule for every module of the index; the qualified call still
  names its module at each use; nothing is loaded that the program doesn't
  name, so compile time only changes for programs that drop a `use` line.
- **Cons:** the loader guesses a local from the tokens; a file that both
  binds `args` and misspells `args.parse` gets "no local `args` is in scope
  here, and `std.args` has no item ..." instead of the item list.

### Option B: load every module written in AIL into every program
- **Pros:** no guessing.
- **Cons:** about 20,000 lines of std checked and lowered for every program;
  `ail check` and every build pay for modules they never use (ADR 1065).

### Option C: keep `use` for the modules written in AIL and say so better
- **Pros:** no loader change.
- **Cons:** two rules stay; an agent still fails its first build once.

## Decision

Every module of std.use.1's index is in scope without `use`, those written in
AIL under their last name (`csv`, `bigint`, `date`, `http`, `webm`). The parts
of another module that are not in the index (`std.screen.view`,
`std.audio.device`, `std.gfx.host`, `std.net.jar`, `std.ui.widgets`) keep
their `use`. The standard library's own files keep their written imports.

## Rationale

AI understanding (criterion 1): one rule replaces two, and the commonest
first-build error goes away. Readability (criterion 2) is unchanged: calls are
qualified as before. Speed (criterion 3): a module is loaded only when named,
which is what `use` loaded before. Tokens (criterion 4): one line less per
module a program uses.

## Consequences

- `tests/mistakes/bigint_no_use` no longer fails and was removed;
  `tests/cases/std_implicit_ail` and `std_implicit_ail_bound` cover the rule.
- The cards' "`use std.bigint`" lines are replaced in the card sync (S17).
- A new std module whose name a program already uses as an item is hidden by
  the program's own item, as ADR 0277 said for the built-in ones.

## Rejected alternatives

- Option B: every program would pay to check and lower all of std.
- Option C: keeps the two rules this decision exists to remove.
