# 0573. Function values in native code: the address of a declared function

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** type.fn.1-3 (ADR 0455), [native.subset.1](../spec/23-native.md)
- **Builds on:** ADR 0455 (function types and declared functions as values), [ADR 0571](0571-key-functions-inline.md)

## Context

ADR 0455 makes a function value a declared top-level function that is not
generic, takes no `mut` or `move` parameter and cannot fail. It has no
captures, and closures are a later decision. Lane 120 suggested a kind-6
closure object (code word at +0, captures after it) with a hidden closure
argument on every callee.

## Decision

A function value is the **address of the function** itself, in one integer
register. Two MIR ops: `fnaddr` (symbol `name`; rematerialised at each use
like `str`) and `icall` (args: the value, then the arguments; `dst` typed by
the function type's result, none for a unit result). `Ty.Fn(params, ret)` is
the value's MIR type, so a mismatch is caught as for any type. Both code
generators reuse `call`'s argument moves and put the callee in the scratch
register that no argument or move uses (`r10`, `x8`): `call r10` and `blr x8`. An
`icall` clobbers the caller-saved registers like `call` and stops a function
being inlined. Lists, structs and fields of function type stay outside the
subset until lists of pointers and structs land.

When the language gets closures, a value becomes a pointer to a kind-6
object (code address at +0, captures after it) and `icall` loads the code
word; declared functions would get a one-word static object. That change
touches only `fn_value` and the two generators' `icall`.

## Rejected alternatives

- **A closure object and a hidden first argument now.** Every function whose
  address is taken would need a second entry that drops the hidden argument,
  and each call would pay a load, all for captures the language lacks.
- **Monomorphising callbacks away.** Doesn't cover values returned from
  functions or chosen at run time.

## Addendum: `unreachable(message)`

The self-compile survey found `unreachable("...")` (err.model.2) unsupported
as a call to an unknown name. It is a MIR op `trap` (the message string; a
site with kind `unreachable`) that ends its block with the `unreachable`
terminator. Both generators jump to a trap stub with kind 7, and `rt.trap`
prints `unreachable: MESSAGE` after the usual head line, as the interpreter
does. The message may be any `str` the subset can build; interpolation
outside `print` waits for lane 172. Not allowed in the runtime itself.
