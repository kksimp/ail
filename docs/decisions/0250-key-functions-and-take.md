# 0250. Sort and pick by key with `by: |x| key`, and `xs.take(n)`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Expressions; collections; token efficiency
- **Spec rules:** [expr.key.1–4](../spec/05-expressions.md#key-functions), type.coll.11 (amended), type.coll.15

## Context

The agent-tool corpus (ADR 0217) measured AIL 28% heavier than Python
([program-measurements-r3.md](../design/program-measurements-r3.md)). The
largest single cause was ranking: `logscan` needed a `Ranked` struct, an
`impl Ord`, a negated count and a loop where Python writes
`sorted(..., key=...)[:n]`. Closures are not designed (ADR 0182), and a full
closure design (captures, `mut` captures, effects, escaping) is far larger
than this lane.

## Options considered

Each measured on the round-4 corpus (`logscan`, `tasks`, `build`, mean of
five tokenizers, [ADR 0255](0255-round-4-token-measurements.md)); the whole
change is −83.4 tokens (−4.6% of the corpus, −15.7% of `logscan`) against
the `Ranked` version.

| Form | logscan |
|---|---|
| `sorted(by: \|m\| -counts[m])` (chosen) | 532.6 |
| `sorted(by: fn(m) -counts[m])` | 533.0 |
| `sorted(by: m => -counts[m])` | 531.6 |
| `sorted(by: -counts[it])` (implicit `it`) | 529.8 |
| `sorted_by(\|m\| -counts[m])` | 531.8 |
| `sorted(key: \|m\| ...)` | 532.6 |

### Option A — second-class key functions, `by: |x| key` (chosen)
- **Pros:** −4.6% of the corpus; the parameter is named, so the key reads
  locally; the key function can't escape, so it needs no capture rules (it
  sees the enclosing locals like a comprehension does); keys are computed once
  per element, so the cost is predictable.
- **Cons:** only four methods take it; `|` gains a second meaning next to
  match patterns (the parser and `ail fmt` tell them apart by position).

### Option B — `fn(x) key`
- **Pros:** reuses the declaration keyword.
- **Cons:** a tie on tokens; a line that starts with `fn` would no longer
  always be a declaration (ADR 0184's grep and recovery anchor).

### Option C — implicit `it`
- **Pros:** the fewest tokens (−2.8 on `logscan`).
- **Cons:** a name nobody declared; ambiguous once keys nest; under the 2%
  threshold, so AI understanding decides against it.

### Option D — `xs.by(field)` / `sorted_by(.field)`
- **Cons:** covers only a field; `logscan`'s key is a map lookup.

### Option E — full closures with `map`/`filter`
- **Cons:** the closure design ADR 0182 declined; comprehensions already
  cover map and filter.

## Decision

A call argument `by: |x| e` is a key function, allowed only as the single
argument of `sorted`, `sort`, `min` and `max` on a list (and `sorted` on a
set). `e` must be `Ord`, is evaluated once per element in order, and sorting
stays stable, so `xs.sorted().sorted(by: |x| -n[x])` sorts by count and then
by name. `xs.take(n)` returns the first `n` elements, or all of them.

## Rationale

Tokens decided that a key form is needed (−4.6%). Among the spellings the
difference is under 0.5% of `logscan`, a tie, so **AI understanding**
decided: `|x|` names its parameter (unlike `it`), is the anonymous-function
form models know from Rust, and keeps `fn` meaning a declaration.
`take(n)` replaces `[..n.min(xs.len())]` (−5.8 tokens, −0.3%): a tie on
tokens, chosen because the clamped slice is a common bug (`xs[..n]` traps on
a short list) and Python's `[:n]` clamps.

## Consequences

- `Arg` gains a `param`; the checker binds it like a comprehension
  variable; `ail fmt` writes `by: |x| key`.
- No tuples yet, so a two-part key is two stable sorts.
- A later closure design can generalise `|x| e` without breaking this.

## Rejected alternatives

- `fn(x)`: same tokens, weakens `fn` as a declaration anchor.
- `it`: an undeclared name.
- `.by(field)`: too narrow.
- Full closures: far larger, and comprehensions already cover map/filter.
- Clamping slices instead of `take`: changes ADR 0136's trap rule for every slice.
