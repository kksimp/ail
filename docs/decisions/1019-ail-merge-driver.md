# 1019. `ail merge`: a git merge driver that merges by declaration

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 20 — AI semantic API
- **Spec rules:** merge.split.1, merge.split.2, merge.decl.1, merge.decl.2, merge.order.1, merge.line.1, merge.conflict.1, merge.cli.1-3, merge.install.1
- **Builds on:** [ADR 0396](0396-ail-diff-semantic-diff.md) (declarations as the unit of change), [ADR 0917](0917-ail-task-protocol.md) (a tool in `tools/`, no cost when unused)

## Context

Parallel lanes edit the same files. Git merges by line, so two branches that add a function at the same spot, or change neighbouring functions, conflict, and an integrator resolves each one by hand. That is the largest recurring cost of parallel work (the integrator's batches). In AIL a declaration starts at column 0 and its body is indented, so a declaration is easy to find without a parser.

## Decision

- `ail merge BASE OURS THEIRS` is a three-way merge by declaration. A file is cut at column-0 lines into declarations, each keyed by kind and name (`merge.split.*`). A declaration changed on one side takes that side; the same change on both sides appears once; the order of OURS is kept (`merge.decl.*`, `merge.order.1`).
- A declaration changed on both sides is first merged line by line with `git merge-file`; only if that conflicts is it one hunk with OURS, the base and THEIRS (`merge.line.1`, `merge.conflict.1`). Every other declaration is merged, so a conflict is local and the report names the declaration.
- It follows the merge-driver call convention (`%O %A %B %L %P`): the result goes to OURS, exit 1 on conflict (`merge.cli.1`). `--stdout` and `--json` serve agents (`merge.cli.3`).
- `ail merge --install` writes `*.ail merge=ail` to `.gitattributes` and sets `merge.ail.driver` (`merge.install.1`). Nothing is installed by default.
- It is a separate tool, `tools/ailmerge/ailmerge.ail`, run by `ail merge`, like `ail task`. It cuts the file by text and does not parse, so it merges files that do not parse yet, and a merge cannot be broken by a syntax error on either side. The merged file may still fail `ail check`; the merge does not judge that.

Cost when unused (ADR 0868): none.

## Consequences

Two lanes that add or change different functions stop conflicting, including two lanes that each add a function at the end of a file (the usual lane conflict). Where both lanes add the same key with different bodies, or change one function in overlapping lines, the conflict is one hunk for that declaration with the base shown, so an agent resolves it from one block.

The token saving per merge is to be measured with `tools/tokbench` on a replay of the integrator's recent conflicts, and recorded here. The merge itself prints one line per conflicting declaration.

## Rejected alternatives

- **Parse both sides and merge the trees:** exact, but it fails when a side does not parse, needs the checker's state for every file, and costs far more code. The column-0 rule already finds the same units.
- **Text-only merge with no line fallback:** simpler, but two edits in different parts of one long function would conflict, worse than git's own merge.
- **Resolving a true conflict automatically (choosing a side):** loses a change silently. A hunk with the base is cheap to resolve and impossible to miss.
- **Installing the driver on first use:** it changes the user's git configuration. `--install` is explicit.
- **A new `ail` subcommand in the compiler:** it would add code to the compiler for a tool run only during a merge.
