# 1082. A call whose value is thrown away and that does nothing else is an error

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** Designer (Kaleb), 2026-11-12, approving item 2 of the AI-first review for 0.1.0. Design details by AI (delegated, ADR 0076).
- **Topic:** Effects ([18-effects.md](../spec/18-effects.md))
- **Spec rules:** `eff.use.1`

## Context

A statement such as `xs.sorted()`, `s.trim()` or `compute()` computes a value
and drops it. When the call has no effect, the statement does nothing, and it
is nearly always a bug: the writer meant `xs.sort()`, or meant to use the
value. Agents make this mistake with copying methods taken from other
languages. `err.use.1` already rejects a dropped `T ! E`; a dropped plain
value was silent. The checker already infers every function's effects
(ADR 0163), so it can tell a call that does something from one that doesn't.

## Options considered

### Option A: error when the dropped call has no effect (chosen)
- **Pros:** uses the inferred effects, so a call that prints, writes a file,
  mutates or spawns stays legal with no annotation; the error names the
  likely intent and `ail fix` rewrites it.
- **Cons:** a call through a function value, or a function without a body,
  has effects the checker can't see at the call; those are never reported.

### Option B: error on every dropped value (Rust's `#[must_use]` everywhere)
- **Pros:** simplest rule.
- **Cons:** `xs.pop()`, `set.add(x)` and logging helpers that return a value
  would all need `_ :=`; noise without finding bugs.

### Option C: a `@must_use` attribute on chosen functions
- **Pros:** explicit.
- **Cons:** an agent must know to write it; the built-in copying methods
  would need a hand-kept list; misses user functions by default.

### Option D: a warning
- **Cons:** agents and `ail check` users act on errors; a warning is easily
  left in place (decision order: AI understanding first, ADR 1077).

## Decision

`eff.use.1`: an expression statement whose value is not its block's value,
that is a call or method call, whose type is not unit, `Never` or `T ! E`,
is an error when its inferred effects have none of `io`, `unsafe`, `hw`,
`spawn`, `mutates` or a capability and typing it changed no place (a `mut`
argument or receiver). Never reported: calls through a function value,
methods of channels, tasks, mutexes and atomics, functions without a body,
standard functions outside the effects table, `task.*`, and calls inside an
`unsafe` block. The fixes are the in-place twin of a copying built-in
(`sorted` → `sort`, `reversed` → `reverse`) when there is one, then
`_ := expr`, which throws a value away on purpose.

The checker notes each candidate while typing a block (`Checker.discards`,
with a counter of mutations and function-value calls) and reports after
typing in `compiler/check/discard.ail`, from the same effect inference as
`@pure`. The tree is swept in the same change.

## Consequences

- Programs that drop pure values stop compiling; `ail fix` repairs them.
- Dropped results of calls through function values stay silent.
- The rule is part of 0.1.0, so no program written for 0.1 depends on the
  old behaviour.
