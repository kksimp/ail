# 0363. The bytecode tier compiles method calls, data construction, `match`, interpolation and `?`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), within the designer's ADR 0359
- **Topic:** interpreter performance; 23 — Compiler architecture
- **Spec rules:** none (the tier is unobservable; it changes no output, exit code or trap)

## Context

Slice 1 ([ADR 0361](0361-bytecode-tier-design.md)) compiled locals,
arithmetic, control flow and plain calls, and handed everything else to the
tree walker. `interp`, `json` and `sheet` gained almost nothing (1.00–1.06x),
because their hot code is method calls, struct and variant construction,
`match`, interpolation and `?`. Lane 111's slice 3 widens the subset to those.
It also reports how much of `interp`, `sheet`, `json` and the agent corpus
runs on the tier.

A method call is the hard case. Its meaning depends on the receiver's value
at run time: a built-in method of a list, map or string; a user method of a
struct or enum; a method of a `ref` cell's contents (own.ref.3); a handle or
atomic method. And the walker evaluates the receiver before the arguments,
or keeps it as a place when the method mutates it.

## Options considered

### Method calls

#### Option A — a guard on the receiver's value, then a direct call (chosen)
`MethodGuard` looks at the receiver and jumps to the walker's path unless the
tier can make the call. `MethodRun` then makes it:
- for a struct or enum, the type's user method, called as `invoke` calls it,
  with a `mut self` receiver moved in and back (trait.method.3);
- for any other value, the built-in method, through the walker's own
  `builtin_with` once the arguments are evaluated.

The guard keeps the user method it resolved, by the receiver's type, so a
call site that sees one type looks it up once. A `ref`, handle or atomic
receiver, or a user method with `mut` parameters, takes the walker's path.

- **Pros:** the common cases (`xs.push(x)`, `s.len()`, `node.eval()`)
  dispatch without the walker; everything else is unchanged.
- **Cons:** two instructions per call; a guard miss costs a little more than
  the walker alone.

#### Option B — specialise each call site on its first receiver type
- **Pros:** one instruction on a hit.
- **Cons:** needs a deoptimisation path when the type changes, which the guard
  gives already at the cost of one compare.

#### Option C — keep method calls on the walker
- **Cons:** about half of `interp`'s hand-offs; the measured gain stayed 1.06x.

### Receivers and evaluation order

- **A local or a field path of one** (`xs`, `self.items`) is read in place
  when the call runs, after the arguments. That is the walker's order only
  if no argument changes that local. Values are independent (own.val.1), so a
  call changes only its receiver's local and its `mut` and `move` arguments.
  The compiler's write analysis now names the local it asks about, so
  `stmts.push(statement()?)` compiles though `statement` is a `mut self`
  method of a different local.
- **Any other receiver** of a method that doesn't mutate it (`peek().kind`,
  `scopes[k].get(name)`, a constant such as `KEYWORDS.contains(w)`) is
  evaluated first into a register, as the walker evaluates it first. On a
  guard miss, `MethodOn` hands the walker that value, through the walker's own
  `method_with_value`, so the receiver is never evaluated twice.

A first version re-ran the whole call on the walker after a guard miss. That
was correct only for receivers without effects, and it limited the tier to
those, so it was replaced.

### Other constructs

- **Struct and variant construction**, `Enum.Variant(...)`, list literals and
  map and set literals are instructions. A struct with invariants stays on the
  walker, which checks them (contract.inv.1).
- **`match`**: the scrutinee goes in a register. Each arm is an `Arm`
  instruction that tests the walker's own `matches` and pushes the bindings,
  and the arm ends by truncating them.
- **Interpolation**: one `Text` instruction per part right after the part's
  value, so `to_str` implementations run in the walker's order. One `Interp`
  instruction then joins the parts.
- **`?` and `else`**: `Try` propagates a failure, or `none` from an optional,
  as the walker does (err.prop.1, err.ctx.2), and leaves through the exits
  table. `ElseTest` branches to the fallback (err.else.1–6). A fallback block
  that binds the error stays on the walker.
- **`local.field = v` and `local.field op= v`** are instructions.
- **String `==` and `!=`** compare the bytes directly when the program has no
  `Eq` implementation that applies to `str` (expr.cmp.1). Otherwise they take
  the walker's `binop`, which dispatches.

### Measuring coverage

`AIL_BYTECODE_STATS=1` now also counts the instructions the tier ran,
hand-offs included, and prints the share that ran natively. The count is in a
second copy of the interpreter loop, chosen when the variable is set, so a
normal run pays nothing for it. `=2` lists hand-offs by kind and method name.

## Decision

Adopt the method-call design (Option A), the receiver rules, and the other
constructs as described above. Trait-qualified calls (`x.cmp(y)` through a
trait, `Trait.name(x)`), associated functions, standard-module calls, calls
with labelled or key-function arguments, and `mut` receivers that aren't a
local's place stay on the walker.

## Rationale

Speed is ADR 0076's fourth criterion and ADR 0359 is the designer's call for
it. The first criteria are untouched: no syntax, diagnostics or output
changes. The guard keeps the walker as the definition of every case the tier
doesn't make itself, and the both-tier test ([ADR 0362](0362-differential-harness-for-the-bytecode-tier.md))
checks every program in the repository each selftest.

## Consequences

- Results (Linux x86-64, [interpreter-performance.md](../design/interpreter-performance.md)):
  `interp` 1.37x fewer instructions than the walker (slice 1: 1.06x), and
  0.86x of CPython's time; `json` 1.14x and `sheet` 1.09x (both start-up
  bound); `loops` 2.10x, 0.54x of CPython's time.
- On the tier: `interp` runs all 50 of its called functions, and 99.3% of the
  instructions the tier runs are native; `json` 99.7%; `sheet` 97.2%; the ten
  agent-corpus programs 99.5% together. The rest are hand-offs: field and
  element assignments deeper than one step, trait-qualified calls,
  comprehensions and slices.
- Splitting the walker's `method_call` and `builtin` so that the tier can enter
  after the receiver or the arguments costs the walker 0.9% on `interp`.
- `interp`'s remaining cost is spread out: value clones and drops, the
  environment's name pushes, allocation, and calls. A further 2x needs a frame
  without names (ADR 0361's Option B), which is out of the lane's scope.

## Rejected alternatives

- Option B (per-site specialisation): the guard's cached compare gets the same
  hit path without a deoptimisation mechanism.
- Option C (methods on the walker): measured at 1.06x on `interp`.
- Re-evaluating the receiver on a guard miss: wrong for receivers with
  effects; replaced by `MethodOn`.
- An instruction counter in the normal loop: a load and a branch on every
  instruction, for a number only `AIL_BYTECODE_STATS` prints.
