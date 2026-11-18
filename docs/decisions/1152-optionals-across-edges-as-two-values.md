# 1152. Optionals of numbers that cross block edges are two values

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the open slowdowns, runtime speed only, no syntax change
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.stack.4](../spec/23-native.md)
- **Builds on:** [ADR 1146](1146-optional-results-in-the-callers-frame.md) (native.stack.3), [ADR 0574](0574-native-optional-layout.md)

## Context

ADR 1146 removed the box of an optional result that its caller only tests
and unwraps. A result that becomes an edge argument was left boxed, so an
optional variable assigned in a loop still allocates:

```text
mut best := none
for i in 0..n
    c := pick(i)
    if better(i)
        best = c
```

Measured on the Mac after #1348 (`optret-lane/optloop.ail`): 0.559 s, with
the pass off 0.563 s, the C twin 0.151 s.

## Options considered

### Option A — split the block parameter in two SSA values (chosen)

A pass before the call conversion (`compiler/mir/optphi.ail`) replaces a
block parameter of type `int?`, `float?`, `bool?`, `char?` or a sized
integer's optional by a `bool` (present) and a number, when every use is a
test against `none`, an unwrap or an edge argument for another such
parameter. Each edge passes `(false, 0)` for `none`, `(true, x)` for a box
of `x`, the pair of another split parameter, or, for a call, `(r != 0, the
word in the call's cell)`.

- **Pros:** plain SSA values; an earlier value held while a later one is
  made (`prev = cur; cur = next()`) needs no reasoning, because nothing is
  overwritten. No backend change.
- **Cons:** two registers per variable instead of one.

### Option B — a frame cell per variable

Each assignment stores the number in the variable's cell; the parameter is 0
or the cell's address.

- **Cons:** the cell is overwritten by the next assignment while an older
  copy of the variable may still be read (the lost-copy problem), so it
  needs a liveness and interference check per variable. More code and more
  risk than Option A for a slower result (a store and a load per iteration).

### Option C — leave it

- **Cons:** 3.7 times the C twin on an ordinary best-so-far loop.

## Decision

1. Candidates are the parameters of every block but the entry whose type is
   an optional of `int`, `bool`, `char`, a sized integer (a word) or `float`,
   `f32` (a float). Optional enums are left out: the pass can't tell a tag
   from an object.
2. A call may feed a split parameter only when its callee is strict: every
   `return` is `none` or a result that goes to the cell (a fresh box, or a
   strict function's result returned at once), so its `$out` entry returns 0
   or the cell and the number can be read from the cell right after the
   call. The call's other uses must be tests, unwraps or such edges.
3. Candidates and calls start as qualifying and are dropped until none
   fails: a use that is not a test, an unwrap, a copy or an edge to a
   qualifying parameter of the same class; an edge value that is not `none`,
   a box of the same class, a qualifying parameter or a qualifying call.
4. Copies of a split parameter go; a box nothing reads any more is removed.
5. Runs inside `optret_convert`, before the `$out` entries are made, on user
   functions. `AIL_OPT_OFF=optphi` (or `optret`, `stack`) turns it off.

## Soundness

- A present value's number is the word the box held: boxes are never written
  after they are made (ADR 1146), so reading it where the value is made
  gives what reading it at the unwrap gave.
- A strict callee's cell is read immediately after the call, before any
  other code runs, and never again.
- A test `x == none` is `not present`; an unwrap is the number. Lowering
  unwraps only after a test, and the unwrap of an absent pair would read a
  constant 0 at worst, not memory.
- Nothing is added that can trap; removing an unread box removes only an
  allocation.
- Test: `tests/native/run/opt_loop_carried.ail` (`int?`, `float?`, `bool?`;
  conditional and unconditional assignment; `none` and plain numbers
  assigned; two variables rotating; a variable pushed to a list and one fed
  by a function that hands a box on, which both stay boxed; a function that
  returns its loop variable).

## Rationale

ADR 0076 order: no source change; speed decides, then risk. Option A is
both the faster code and the simpler proof.

## Consequences

- To be measured on the Mac and ail-linux before any number is quoted.
- Not covered: optional enums; a variable fed by a function that can return
  a box it was given; optional fields and list elements, which stay boxes.

## Rejected alternatives

- **Option B:** needs an interference check and is slower.
- **Option C:** leaves a measured 3.7x.
