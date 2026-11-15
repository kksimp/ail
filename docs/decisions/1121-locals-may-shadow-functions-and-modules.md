# 1121. A local may hide a function, an imported module or a field until its block ends

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** Designer (Kaleb approved the direction on 2026-11-15, E13 follow-up slice S13); the field case by AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 5 — Expressions (names and bindings); 10 — Traits (implicit `self`)
- **Spec rules:** [expr.bind.5](../spec/05-expressions.md), [trait.method.7](../spec/10-traits.md), [std.use.3](../spec/17-std.md)
- **Supersedes:** [ADR 0469](0469-locals-may-shadow-functions.md)
- **Amends:** [ADR 0084](0084-no-shadowing-module-names.md) (imported names), [ADR 0141](0141-implicit-self.md) (locals named like a field)

## Context

In the E13 agent benchmark 24 AIL trials lost a compile round to the local-name
rules. Three shapes, all in code split into modules:

- `qty := qty(f[2]) else return BadLine` in a module that declares `fn qty`
  (40 reports). ADR 0469 had let a local shadow a function, but only in the
  root module: the loader renames a module's items to qualified names and still
  reported every local named like one of them, functions included.
- `mut out := List[str]()` in a file with `use out` (6 reports).
- `mut d := self.d + self.c / 100` in a method of a type with a field `d`
  (12 reports, `trait.method.7`).

Each time the agent's meaning was clear and the fix was a rename.

## Decision

A local (or parameter, loop variable, pattern binding) may take the name of a
function, an imported module (user or standard) or, in a method, a field of the
receiver. From its declaration to the end of its block the bare name means the
local; the initializer of a `:=` is checked before the name is bound, so
`qty := qty(s)` and `path := path.stem(f)` call the function and the module.
Types, traits and constants stay errors (`expr.bind.5`), for the reason ADR
0469 gave: `Point(...)` would become ambiguous in a way a function call is not.

What stays an error, with a message that names the hidden thing:

- a call of a hidden function in the local's block: `expr.call.1` (unchanged);
- `m.f` where the local `m` hides a module that has an item `f`:
  `expr.bind.5` for a user module ("the local `out` hides the module `out`"),
  `std.use.4` for a standard module (unchanged).

### Fields (trait.method.7)

Options:

- **A (chosen): a local may hide a field.** `ail fmt` already writes every
  bare member as `self.x` (fmt.self, ADR 1081), so in formatted code a bare
  name in a method is always a local and a field is always `self.x`. The
  shadow cannot change what a formatted line means; it only changes what a
  bare name means in unformatted code after the local is declared, which is
  the same rule as every other shadow here.
- **B: keep the error, print the rename.** Rejected: 12 trials wrote this
  shape on purpose, reading the field through `self.d` on the same line; the
  error costs a round and the rename adds nothing a reader needs.

## Implementation

- `compiler/load/resolve.ail`: the resolver keeps the locals in scope (pushed
  at each binding, dropped at the end of each block, branch, arm, loop,
  closure and comprehension). A local in scope is never renamed to a module
  item or read as an import. Only a module's non-function items are reported.
- `compiler/syntax/implicit.ail`: the implicit-`self` pass keeps the locals
  that hide a field the same way and leaves those names bare, so `ail fmt`
  writes no `self.` there (`compiler/fmt/explicit.ail` uses the pass's sites).
- `compiler/check/body.ail`: imported modules no longer make a root-module
  local an error. `compiler/check/shadow.ail`: the `expr.bind.5` report for
  `m.f` through a local that hides a user module.

Old code is unaffected: every program that compiled before still means the
same thing, since the change only makes more declarations legal and a bare
name could not mean a local before.

## Rejected alternatives

- Allowing types, traits and constants too: a constructor call `T(...)` and a
  constant read would become ambiguous with no error at the use.
- Allowing the shadow only in the root module (ADR 0469 as it was): moving
  code into a module would make it stop compiling.
- Keeping the module rule and printing the rename (for `use out` plus
  `mut out`): the error stops code whose meaning is plain; the one confusing
  use, `out.report(...)` after the local, is still an error that names the
  module.

## Consequences

- Tests: `tests/cases/packages/module_local_shadow` (function, module and
  field shadows in a package), `tests/cases/packages/module_local_hides`
  (`expr.bind.5` on `out.report`), `tests/cases/local_hides_std_module.ail`;
  `implicit_self_errors`, `std_use_errors` and `packages/imports` lose the
  reports this removes.
- The language card and `docs/for-agents.md` say what may and may not be
  shadowed.
