# 1050. Drop checks a dominating check already passed; field stores forget one offset

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 25 — Optimization strategy
- **Spec rules:** native.fwd.1, native.fwd.2, native.fwd.3
- **Builds on:** [ADR 0816](0816-native-codegen-entry-addressing-constants.md) (load forwarding), [ADR 0578](0578-dominated-list-accesses.md) (native.bce.2)

## Context

The slow-spots study of 2026-11-10 (slice F5) found two costs in the range
coder of lzr (`Dec.bit`, `Enc.bit`, 23% of its time) and in every `mut self`
method: `self.probs` and `self.rng` reloaded after each field store, and
`probs[idx]` bounds-checked twice with the same list and index. Load
forwarding (ADR 0816) forgot list headers and every slot on any slot store,
and knew nothing at the start of a block with two predecessors. The
dominated-access rule native.bce.2 holds only while no operation in the
whole program shortens a list, so it is off in any program that calls `pop`,
`clear` or `remove` (the compiler, Clove, most real code).

## Options considered

### Option A — extend forward.ail (chosen)

One reverse-postorder walk per function carries, besides the loads, the set
of `get`/`set` checks (list, index) and `bounds` checks (index, length) that
passed. A block starts with what all its predecessors agree on when each was
visited first; a loop header and the entry start empty.

- **Pros:** one pass, no new file in the pipeline; the fact that makes the
  second load go (the same list value) is the one that makes the second check
  go. Linear in the function with small maps.
- **Cons:** facts never enter a loop from outside it (no fixed point), so a
  check before a loop does not vouch for the same check in its body.

### Option B — a separate dominator-tree check pass after forwarding

- **Pros:** reaches the facts of any dominating block.
- **Cons:** needs the same "nothing shortened a list on any path between" test
  that A gets for free from the walk, and a second walk; dominance alone is
  not enough (a `pop` on one branch).

### Option C — full GVN with memory SSA

- **Pros:** the general answer; loops included.
- **Cons:** a new IR layer before 0.1.0; much more code and risk.

## Decision

1. A `get` or `set` with a site loses its check when a check of the same list
   value and index value passed on every path to it, and nothing on those
   paths can shorten a list: only element stores, stores to object slots,
   arithmetic, loads, pushes, `rt.list_new`, `rt.list_reserve` and
   `rt.obj_new` come between. A repeated `bounds index, len` (same values)
   is dropped (native.fwd.1).
2. A store to a slot of a struct, tuple or enum object forgets the slots at
   that byte offset (in every object) and raw memory, nothing else: objects
   never overlap and always start where their pointer points, and a list
   header is its own object. A store through any other value (a list, an
   address) forgets everything, as a call does (native.fwd.2).
3. A block entered from several predecessors keeps the loads and checks they
   all end with, with the same value (native.fwd.3).

## Soundness

- A check is removed only where it cannot fire: on every path, the same
  index was found in `0 <= i < len` for the same list object, and a list's
  length only falls through a call that is not a push or allocation, a store
  to the list itself, a raw store or a primitive with effects, each of which
  empties the set. A `bounds` check reads two immutable values, so it holds
  wherever its first copy dominates.
- What a block starts with was true at the end of every predecessor, and each
  predecessor was finished first (no back edge), so it is true on entry. A
  value in a fact is defined in a block that dominates each predecessor, so
  it dominates the block and is available there.
- Traps are unchanged where a check is not dominated: the kept check fires at
  the same access with the same message (native.build.1). Tests
  `trap_fwd_pop`, `trap_fwd_join` and `trap_fwd_call` cover a `pop` on one
  branch, a check on only one branch, and a call that shortens the list.

## Rationale

ADR 0076 order: no source change, so tokens tie; AI understanding is
unaffected (no new rule a program must follow); speed is the criterion, and
Option A buys the measured cases at the least risk before RC4.

## Consequences

- `Dec.bit`/`Enc.bit` keep `self.probs`, its header and the passed check in
  registers across `self.rng` stores.
- `AIL_OPT_OFF=forward` turns the loads and the checks off together.
- Follow-up: carrying facts into loops (a fixed point, or hoisting) and a CSE
  rerun after forwarding are left to F9 and later GVN work.
- Follow-up (F4 extension, after F3's registers): hoist `self.f` and its
  list header out of a loop that stores no slot at that offset and makes no
  call; struct-of-lists code (Clove's context mixing) still pays the first
  load of each iteration.
- Measurements (perfbench, x86-64 and arm64): see the PR; recorded here when
  it lands.

## Rejected alternatives

- **Option B:** repeats the walk A needs anyway for the "nothing shortened"
  test.
- **Option C:** right long-term, too large for 0.1.0.
- **A type rule** ("an `int` store can't change a pointer slot"): an enum
  object reuses one offset for payloads of different types, and the offset
  rule already separates the fields of one struct.
- **Keeping native.bce.2 program-wide only:** it stays, but a single `pop`
  anywhere turns it off, which is the common case.
