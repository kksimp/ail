# 0347. `--watch` shows writes to a collection's elements

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling for AI agents (semantic API)
- **Spec rules:** `trace.watch.2`, `trace.watch.4`

## Context

`--watch NAME` ([ADR 0268](0268-watch-and-no-debugger-protocol.md)) skipped builtin mutating methods, so a loop of `xs.push(x)` was invisible until the next assignment. An agent asking "who put this in the list" had no answer.

## Decision

Every builtin mutating method on the watched collection and every `NAME[k] = e` or `NAME[k] op= e` is a watch event: the collection's whole new value, plus what was written and where. Text ends `[push 3 at index 2]`; JSON adds `op`, `element` and `index` or `key`. Calls that change nothing (`pop` on an empty list, `remove` of a missing key, `add` of a present element) are not shown. Sets now print their members in a watch (they printed `{}`).

## Rationale

**AI understanding decides**: the element and index answer the question directly, with no diffing of two whole values. Tokens: the extra text is one bracketed suffix, only under `--watch`. Speed: the checks run only when tracing is on.

## Rejected alternatives

- *Only the element, not the whole value.* Rejected: `trace.watch.1` events always carry the whole value.
- *Separate event kind.* Rejected: consumers already parse `"trace":"watch"`.
