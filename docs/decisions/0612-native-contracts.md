# 0612. Contracts in native code: checks at the call and at each return

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** contract.req.1, contract.ens.1-3, contract.run.1-2, [native.subset.1](../spec/23-native.md)
- **Builds on:** ADR 0211-0212 (contracts at run time), [ADR 0573](0573-native-function-values.md)

## Context

`ail build` refused every function with `requires` or `ensures`. Ten corpus
programs use them, eight of them nothing else outside the subset. The
interpreter's trap for a failed contract names the function, the condition
as written, the arguments (or the returned value) and the position of the
contract, and the native tests compare the first stderr line with it.

## Decision

- **`requires`** is checked **in the caller**, right before the call, because
  the trap is at the call (contract.req.1). The condition is lowered there
  with the parameters bound to the argument values (and `self`, with its
  fields by bare name, for a method). The position is where the interpreter
  puts it: the start of the callee name, or the method name.
- **`ensures`** is checked **in the function**, at each `return` and at its
  end, with `result` bound (not for a unit function) and each `old(e)`
  taken on entry (contract.ens.3, copied if it is a list or struct). The
  parameters read are their values at the return (contract.ens.1). The trap
  is at the `ensures` line and names where the function returned (the
  `return`, or its last statement).
- **A message is written in pieces.** The failure path calls
  `rt.trap_begin(head)`, then the `out_*` functions of `print` for each
  value (`out_quoted` for strings, `Name(f: v, ...)` for structs, so the text
  form is the interpreter's), then `rt.trap_end(excerpt)`, which exits 1. The head and
  excerpt are built at lowering time from the file's path and lines
  (`mir.excerpt`, shared with the other trap stubs). No new MIR op, no
  new stub kind.
- **Struct invariants** (contract.inv.2-3): a value built with the struct's
  literal form is checked right after it is built, at the invariant's line,
  naming the construction; a `mut self` method checks them at each return,
  after its `ensures`. The condition sees `self` and the fields by bare name.
- **A condition checks nothing** (contract.run.2). Calls in a condition, and
  in `old(e)`, go to a `.nc` copy of the callee: the same body lowered with
  no checks, whose own calls go to `.nc` copies in turn. Only functions a
  condition reaches are copied, once. This is exact, costs nothing at run
  time and needs no global flag, which tasks would have to make thread-local.
- A function value of a function with contracts (type.fn.2) is outside the
  subset: an indirect call would skip its `requires`. So are the contracts
  of a function in another module, and the `invariant` lines of a struct
  built from another module, until later slices. The `  values:` snapshot lines of the interpreter's trap
  are not produced (trace.snap.1); the tests drop them.
- Update 2026-10-21: the `requires` of a function in another module are now
  checked at the call. The condition is lowered in the callee's module, so
  its names mean what they do there, and the trap names the contract's file.
  A struct's `invariant` lines from another module are still outside the subset.

## Rejected alternatives

- **A checking wrapper function** called by every caller. The `requires`
  trap must be at the call site, which is static per call; passing the
  position as arguments breaks the six-argument limit.
- **A runtime "checking" flag** for contract.run.2: a load and branch on
  every contracted call, and a race between tasks.
- **Checking `requires` in the callee** with the caller's position passed
  in: the same limit, and the position could not be a constant.
- **Dropping the condition text and values**: the native tests compare the
  first line with the interpreter's, and the values are what an agent reads.

## Consequences

- Code grows by one branch and a cold failure block per condition, per call
  of a function with `requires`.
- Fallible results (lane 175): an `ensures` is skipped for a returned
  error (contract.ens.1) when they land.
