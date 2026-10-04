# 0068. Ownership-taking parameters and arguments are marked `move`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership; 2 — Core syntax
- **Spec rules:** [own.move.1–2](../spec/07-ownership.md)
- **Partly resolves:** OQ-035
- **Evidence:** cl100k tokens for signature plus call: `move` 12; `take`, `own`, `sink`, `consume` 14 (`move` is 2 cheaper on 3 of 5 tokenizers)

## Options considered

- **`move` (chosen):** cheapest on 3 of 5 tokenizers, describes what
  happens, and widely recognised. Con: C++'s `move` leaves a usable husk.
  AIL's is stricter (the source becomes unusable, enforced by the compiler).
- **`take`:** reads naturally, but costs more.
- **`own`:** names the concept, but reads oddly at the call site.

Call-site marker:
- **Required (chosen):** consistent with `mut` (ADR 0061). A reader sees at
  the call that the value is consumed.
- **Not required:** one token less, but consumption is hidden in the callee's signature.

## Decision

1. A parameter written `move p T` takes ownership of its argument.
2. The argument must be written with a leading `move` at the call site:
   `close(move f)`. The moved-from binding can't be used afterwards.

```
fn close(move f File)
    ...

close(move f)
```

## Consequences

- Parameter modes are now: read-only (default), `mut`, and `move`.
- Whether `move` may also be used with copyable values (to hand over storage
  explicitly, enabling in-place reuse without relying on last-use analysis)
  is still to be decided.
- Assignment and return of unique values move implicitly (`own.uniq.2`). Only
  calls carry the explicit marker. Revisit if that inconsistency causes
  confusion.

## Rejected alternatives

- **`take`, `own`:** more tokens, no clarity gain.
- **No call-site marker:** hides consumption.
