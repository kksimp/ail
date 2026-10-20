# 0215. `str` methods `slice`, `char_at`, `find`, `replace`, `to_lower`, `to_upper` (resolves OQ-048) (*delegated*)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Strings
- **Spec rules:** `type.str.13`, `type.str.14`

## Context

The editor edited text with `s.chars()`, list slices and `join("")`: 19
`chars()` and 10 `join("")` calls, each an O(n) round trip. OQ-048 offered
`s.slice(a, b)`, `s[a..b]` (tied to OQ-043), or only case methods. OQ-043 is
now resolved (ADR 0208: `len()` counts characters).

## Options considered

### Option A — named methods counting characters (chosen)
- **Pros:** consistent with `len()` and `pad_left`; no indexing syntax; explicit.
- **Cons:** O(n) per call, as `len()`.

### Option B — `s[a..b]`
- **Pros:** shorter.
- **Cons:** looks O(1) and invites byte-offset confusion; reopens `type.str.2`.

### Option C — only `to_lower` and `to_upper`
- **Pros:** minimal.
- **Cons:** leaves the `chars()` round trips.

## Decision

Add `slice(a, b)`, `char_at(i) char?`, `find(sub) int?` (character index),
`replace(from, to)` (traps on empty `from`), `to_lower()` and `to_upper()`.
`slice` traps out of range, never clamping. `sub`, `from` and `to` take
`str`.

## Rationale

Measured with `tools/tokbench/run.sh` on `tools/edit/edit.ail` after rewriting
its `splice` and `chars()`/`join("")` round trips: mean 12,289.6 to 12,183.6
tokens (-0.9%), a tie under ADR 0076's 2% threshold. AI understanding decided.
The editor's 18 headless tests pass unchanged.

## Rejected alternatives

- Option B: contradicts `type.str.2` and hides cost.
- Option C: doesn't remove the round trips.
- `char` arguments for `find` and `replace`: a one-character `str` does the same.
