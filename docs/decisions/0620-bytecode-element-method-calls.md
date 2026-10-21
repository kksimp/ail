# 0620. Bytecode tier: `mut self` method calls on list elements

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — Interpreter
- **Spec rules:** type.coll.4, own.param.2
- **Builds on:** [ADR 0619](0619-bytecode-calls-loops-and-fusion.md), [ADR 0363](0363-bytecode-tier-methods-and-data.md)

## Context

After ADR 0619 the struct-heavy loop `ps[i].step(dt)` (a `mut self` method on
a list element) was the last common hand-off: the whole call ran on the tree
walker, 200,000 hand-offs in a 100-particle, 2,000-step benchmark. Related
hand-offs found on the way: `self.x += v` in a `mut self` method, and an `else`
nested inside a receiver such as `(s.strip_prefix(p) else "0").to_int() else 0`.

## Decision

- One instruction, `ElemCall`, for `xs[i].name(args)` when `xs` is a local,
  `name` is a `mut self` method name, and neither the index nor an argument
  writes a local. At run time, if `xs` is a list, `i` is in range and the
  element is a struct whose method takes `mut self` with plain parameters, the
  element moves out of the list, runs as `self`, and moves back. Any other case
  evaluates the expression on the walker, so traps and messages are unchanged.
- Field `op=` on `self` compiles when the value cannot write locals.
- `has_try` looks through a nested `else`, so it no longer forces a hand-off.

- A `mut self` call on a field path of a local (`o.inner.grow(i)`) runs on the
  tier: the field moves out of the struct (copy on write, as the walker's place
  path does) into the call and back. Register receivers still can't take `mut self`.
- A call through a local holding a function value (`f(i)`, type.fn.3) is
  `FnGuard` + `FnCall`: the guard checks the value and its arity before the
  arguments run, else the walker's call.

## Results (callgrind-stable counts; wall time on Linux x86-64, min of 3)

| Program | Before | After |
|---|---:|---:|
| struct loop (`ps[i].step`, 800k calls) | 0.178 s (base), 0.132 s (0619) | 0.096 s (CPython 0.075 s) |
| hand-offs in that loop | 600,000 | 0 |
| nested `else` loop | 1 hand-off per use | 0 |
| `f(i)` and `o.inner.grow(i)` loop (1,000 turns) | 2,001 hand-offs | 1 |

## Rejected alternatives

- **Generalise to any place (`a.b[i].m()`, nested lists).** More paths to keep
  byte-identical for no measured gain; those calls stay on the walker.
- **Copy the element in and write it back on success only.** Costs a clone of
  the struct per call, which defeats the point.
