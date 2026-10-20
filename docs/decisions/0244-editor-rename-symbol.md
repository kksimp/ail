# 0244. Editor F2 renames through `ail api` references, else by whole word, after a confirming prompt

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling — the editor ([ADR 0180](0180-editor-written-in-ail.md))
- **Spec rules:** none (tool behaviour); uses the `ail api --symbols` output (`api.out.*`, [semantic-api.md](../design/semantic-api.md))
- **Extends:** [ADR 0241](0241-editor-tabs-and-running.md)

## Context

The editor needs rename-symbol. `ail api --symbols` already reports every
declaration with its id and position, and every reference by id: struct
literal labels, `p.x`, implicit `self` fields and method calls included.
It doesn't report locals or parameters. Deciding criterion: **AI
understanding** (a rename must not silently change the wrong things or miss
places), then token cost of the editor's code.

## Options considered

### Option A — semantic through `ail api` when it covers the name, else whole word, always shown before applying (chosen)
- **Pros:** exact for declarations (renaming field `x` leaves a local `x`
  alone); still works for locals; the prompt says how many places and how
  they were found, which is the confirmation.
- **Cons:** the whole-word fallback can touch an unrelated name in another
  function.

### Option B — whole word only, until a `--refs` mode reports locals
- **Pros:** simplest.
- **Cons:** renames unrelated fields, methods and locals that share a name.

### Option C — wait for a dedicated `ail api --refs`
- **Cons:** nothing tonight; the existing output already carries references.

## Decision

F2 on a name opens a prompt on the tab bar: ``rename `x` (4 places, found
by ail api) to: x``. The semantic set is the declaration plus its
references, used only when the cursor is on one of them and every place
still holds the name (an index left stale by errors falls back). Otherwise
it is every whole-word match in the buffer, labelled "every whole word in
this file". Enter applies it as one undo step; Esc cancels; a keyword or a
non-word is refused. Renames are within the open buffer.

## Consequences

- When `ail api` reports locals and cross-file references, the fallback
  shrinks and rename can span files, with no change to the prompt.

## Rejected alternatives

- **Whole word only:** wrong for fields and methods that share a name.
- **Waiting for `--refs`:** the data is already there.
