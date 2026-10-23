# 0862. Native `mut` parameters and `mut self` of value types use cells; `for mut` over scalars writes through

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** AI (delegated, ADR 0076; burst lane B4)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** own.param.2, own.excl.5, [native.subset.1](../spec/23-native.md)

## Context

A native `mut` parameter of a struct, list, map or set is a pointer to the
caller's object, so writes reach the caller. A scalar, `str`, enum or optional
is a value held in a register or a shared object, so `mut n int` and
`self = match self ...` on an enum built natively but changed nothing in the
caller. `for mut x in xs` over such elements was refused (native.subset.1).

## Decision

1. A `mut` parameter (or `mut self` of an enum) whose type is `int`, `bool`,
   `char`, `str`, an enum or an optional takes a **cell**: the caller allocates
   one word holding the argument, passes it, and after the call assigns the
   word back to the argument (a local, field, element or map entry, through the
   ordinary assignment lowering). The callee reads the cell once into the name
   and stores every assignment to the name through to the cell.
2. In the signature the cell is the type `*(T)`, a pointer to a one-element tuple.
   No program can write that type, so a call path that doesn't know cells
   (closures, tasks, generics, traits) reports `expected *(int), found int`
   instead of passing a scalar as a pointer.
3. `self = e` in a `mut self` method of a struct stores the new fields into the
   caller's object, as for a `mut` parameter.
4. `for mut x in xs` over `int`, `bool`, `char`, `float` or `str` elements binds
   `x` to the element and sets the element at each assignment to `x`, so every
   way out of the loop leaves the list changed. The list is not copied.
5. Float cells and `mut` arguments that aren't a variable, field or element stay
   outside the subset.

## Rejected alternatives

- **Return the final value from the callee.** It changes every call path's
  ABI and needs multiple results for several `mut` parameters.
- **Write back once at the end of `for mut`.** `break`, `continue`, `return`
  and `?` would each need the write-back.
- **Box every `mut` scalar local.** It slows every program for the few that
  pass `mut` scalars.
- **Report these as not in the subset.** Interim state (RR-5 found the silent
  wrong result); the coverage report counted 37 problems from these two constructs.
