# 0570. Constant lists are built once, on first use

- **Status:** Superseded by [0604](0604-native-computed-constants.md)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** mod.const.1, native.subset.1, own.val.1
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (constants are inlined at each use)

## Context

Constants whose value is a number or string literal are inlined at each use.
A constant list such as `WORDS := ["the", "of", ...]` (the `wordfreq`
program) has no such form: inlining it would build a new list at every use,
twice per iteration of a million-iteration loop, with no collector yet.

## Decision

A constant whose value is a list literal of constant expressions is built
the first time a use is reached and kept in a global slot. A use is:

```text
p := load64(global(SLOT))
if p == 0
    p = <build the list>
    store64(global(SLOT), p)
```

- Slots 16–63 of the runtime's globals (the image has 64; the runtime uses
  0–7) are handed out in order of first use; past 48 constant lists a use
  builds the list inline.
- The list is never changed (constants are immutable), and a binding or
  assignment from the constant's name copies it like any list place
  (own.val.1).
- These slots hold heap pointers: **the collector must scan them as roots**
  (lane 171).

## Consequences

- A use costs a load and a branch; the list is built once per program.
- The first use may be in any function, so the build sits behind a branch
  there, which the optimiser sees as ordinary blocks.

## Rejected alternatives

- **Build every constant list at start-up.** Simple, but builds lists a run
  never uses, and needs an initialisation function ordered before `main`.
- **Place the list in read-only data.** A string element is an object in
  read-only data already; a list would need its header and data laid out by
  the object writers of both formats, and a `List[str]` of literals is the
  only case that benefits. Possible later, with the collector's header.
- **Inline at each use.** Correct and what the first revision did for
  literals; wrong for a list in a loop.
