# 1061. Native: lists built to a known count check against that count

- **Status:** Accepted
- **Date:** 2026-11-11
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 23 — Implementation (native optimizer)
- **Spec rules:** [native.bce.15](../spec/23-native.md)
- **Builds on:** [ADR 0890](0890-optimizer-roadmap-to-match-c.md), [ADR 0577](0577-bounds-checks-in-counted-loops.md)

## Context

The slow-spots study (`lanes/slow-spots-2026-11-10.md`, slice F4) found that
vm's fetch path pays for list headers: `st := [0 for _ in 0..64]` is read and
written in a dispatch loop too large for header hoisting, and every `st[sp]`
loads the length from the list header to compare with. The length is 64, known
where the list is made.

## Options considered

### Option A — a pass before bce.ail (chosen)
`compiler/mir/constlen.ail` finds each list made by `rt.list_new` whose pushes
are all in its making block (a literal), or all in one counted loop with
constant bounds that leaves only from its header (each push in a block every
pass goes through), and whose every other use is `get`, `set` or `len` after
the pushes. Any other use (a call, a store, a block or jump argument, a
return, a push of it into itself) rules the list out. `len` becomes the
constant; a `get` or `set` outside a counted loop becomes `bounds idx, K`,
`data` and `ldx`/`stx`. Both back ends compare against an immediate when the
length is a constant. `hoist.ail` accepts `data` uses.
- **Pros:** one file and one line in `inline.ail`; trap site, index and length
  are unchanged; lists the optimizer already hoists lose the length register.
- **Cons:** accesses inside counted loops keep the header length (bce.ail
  guards those once).

### Option B — after bce.ail
Versioned push loops are two loop copies, so a list's pushes are in two places
and a use scan can mistake a guard's `len` before the loop for the length after it.

### Option C — carry the length in `get.imm`
Hides it from bce.ail and hoist.ail, which would then not see a plain `get`.

## Decision

Option A, rule native.bce.15, opt-out `AIL_OPT_OFF=constlen`. Runtime (`rt.`)
functions are skipped. There is no `List.filled` in std; a comprehension
`[x for _ in 0..C]` and a literal are the forms covered.

## Consequences

- Tests: `tests/native/run/constlen.ail`, `constlen_refused.ail` (a later push
  keeps the checks) and `trap_constlen.ail` (the trap names the same site,
  index and length).
- Amendment (2026-11-12): `MAX_BLOCKS` (12) in hoist.ail is dropped. It kept
  header hoisting out of dispatch loops because the two registers held per list
  spilled on x86-64 (vm +1.4% to +3.0%). With F3's RDX and R11 allocatable
  (ADR 1049) and lists of known length needing one register (the element
  pointer; the check is against an immediate), a loop may now hold up to
  `MAX_REGS` (6) registers of hoisted headers, busiest lists first, at most
  `MAX_LISTS` (4) lists; a list that does not fit stays as it was. vm's `st`
  (one register), `code` and `vars` (two each) fit exactly. The cap was a
  register-pressure guess; the vm timing in the F4 job report decides whether
  `MAX_REGS` stays at 6.

## Rejected alternatives

- **Pass after bce.ail** (Option B) and **length in `get.imm`** (Option C), above.
- **Allow calls that only read the list:** a callee can be handed the list by a
  holder that pushes; escape analysis is not worth it for the common case.
