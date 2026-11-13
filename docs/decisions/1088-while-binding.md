# 1088. `while name := e` loops while a value is present

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076); the designer approved every AI-first review item for 0.1.0 on 2026-11-12.
- **Topic:** Loops ([expr.loop.8](../spec/05-expressions.md))
- **Spec rules:** expr.loop.8
- **Amends:** [ADR 0125](0125-if-binding.md) (binding conditions were `if` and `elif` only)

## Context

`if v := e` binds the value of a `T?` or `T ! E`, but a loop that drains one
(`stack.pop()`, a line reader) had to be written `while true` plus
`v := e else break`. Agents reach for Rust's `while let`.

## Decision

`while name := e` runs while `e` has a value, binding `name` per iteration.
The parser rewrites it to `while true` whose block starts with
`name := e else break`, so the checker, the ownership and effect passes and
every lowering see an ordinary loop and need no new code (the native
lowering rule of ADR 0823 is met by having none). `continue` re-evaluates
`e`. There is no `else err` part. `while let x = e` is an error whose fix
writes `while x := e`. The formatter works on tokens, so source is kept as written.

## Options considered

- **A new `While` condition kind with its own lowering (rejected):** touches about 25
  exhaustive matches and `lower.ail` for no behaviour the rewrite lacks.
- **`while let` spelling (rejected):** AIL has no `let`; `:=` is the one binding form.
- **An `else err` part (rejected):** a loop has no single place to report an error; use `else break` with a handler.
