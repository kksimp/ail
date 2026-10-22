# 0643. A check inlined from a callee keeps the callee's frame and the call that was inlined

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Native runtime diagnostics
- **Spec rules:** `trace.frame.2`

## Context

The inliner (ADR 0391) copies small leaf functions into their callers. A
trap in the copy used to report the callee's line and function (kept by its
`Site`) but the call, hence the caller's frame, was gone from the machine
stack: `pick` inlined into `middle` printed `pick`, then `middle`'s caller.

## Options considered

### Option A — link each copied check to the call it came from (chosen)
`Site.from` holds the site of the inlined call (a position and the caller's
name). The compiler writes the chain into the trap's excerpt text: the
callee's line, then the caller's line at the call, then the physical stack.
- **Pros:** the printed stack is the one the interpreter prints; no run-time
  cost; the chain is compile-time text.
- **Cons:** the inliner clones a site per copied check (the MIR dumps
  renumber).

### Option B — don't inline functions whose checks can trap
- **Pros:** simpler.
- **Cons:** loses the speed ADR 0391 measured on arithmetic helpers.

### Option C — leave the frame out
- **Pros:** nothing to build.
- **Cons:** a wrong stack is worse than none for an agent.

## Decision

Option A (speed and AI understanding both).

## Consequences

- A call position is stored in the `imm` of a user `call` instruction
  (`mir.call_pos`: file, line, column), so `Module.sites` and the dump
  numbers of ordinary checks don't change.
- Only one level is linked: a function that calls anything is not inlined
  (leaf functions only).

## Rejected alternatives

Options B and C above.
