# 1134. A changed copy of an entry that is never stored back is a warning

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Values; AI understanding
- **Spec rules:** expr.bind.15, own.val.1

## Context

Values are independent (ADR 0060): `mut c := m[k]` binds a copy, so
`c.n += 1` changes the copy and leaves the map as it was. An agent used to
references writes exactly that and the program runs and prints the old
number. The error is silent, so it costs an iteration of running and
reading output (verdict item 17).

## Options considered

### Option A: warn when such a copy is changed and never read (chosen)
`mut x := m[k]`, `xs[i]` or `m.get(k) else d` marks `x` a copy of an entry.
Assigning to `x`, `x.f` or `x[i]` is not counted as a read; if nothing reads
`x` afterwards, the warning `expr.bind.15` says to store it back or change the
entry in place. Reading `x` anywhere, such as `m[k] = x`, silences it.
- **Pros:** no false warning on the normal pattern (copy, change, store); the
  warning is the shape of the bug; it is a warning, so no program stops.
- **Cons:** a copy that is read first, then changed, then dropped is not
  reported; method calls that change the copy (`x.items.push(e)`) are not
  seen. Both leave the program as it is today.

### Option B: make `m[k].f = e` the only way and reject copies
- **Cons:** `c := m[k]` is how values are read; rejecting changes the design.

## Decision

Option A, only for assignments, to keep false positives at zero.
