# 1107. Labeled loops: `outer: for ...`, `break outer`, `continue outer`

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer put every item of the AI-first review into 0.1.0 on 2026-11-12 (package S23)
- **Topic:** 3 — Expressions and control flow
- **Spec rules:** [expr.loop.9, expr.loop.10](../spec/05-expressions.md); amends expr.loop.2, expr.loop.7
- **Builds on:** [ADR 0091](0091-control-flow.md), [ADR 0139](0139-unit-results-statement-else-and-divergence.md), [ADR 1088](1088-while-binding.md)
- **Amends:** the `gram.foreign.1` hint for `outer: for`, which said AIL has no loop labels

## Context

`break` and `continue` act on the innermost loop. Leaving two nested loops
needed a function and `return`, or a `mut done := false` flag tested in
every loop. Go and Rust both have labels, so agents write `outer: for` and
`break outer`; since the foreign-habit pass (S1) turned that into a hint,
the habit was answered with a refusal, not a feature.

## Options considered

### Option A — `name: for ...` with `break name` / `continue name` (chosen)
- **Pros:** Go's spelling, which agents and humans already know; the label
  sits at the start of the loop it names, so the target is local to read;
  one token more than the flag it replaces, and no extra binding.
- **Cons:** the first statement form that starts with `name:`.

### Option B — Rust's `'outer: for` and `break 'outer`
- **Cons:** a quote that is not a character literal; the lexer and every
  tool would need a lifetime-shaped token. Rejected.

### Option C — the existing `for x in e as name` (`as` is taken)
- **Cons:** `as` already names the loop's iterator elsewhere; reusing it for
  a label reads as a second binding. Rejected.

### Option D — `break 2` (levels)
- **Cons:** the number silently changes meaning when a loop is added or
  removed between the statement and its target. Fails AI understanding.

### Option E — keep refusing (flag or function)
- **Cons:** the flag form is the commonest source of a loop that keeps
  running after the exit; a feature the agents' habit expects. Rejected.

## Decision

Option A.

1. A label is `name:` before `for` or `while`, on the same line. Labels are a
   namespace of their own (no clash with bindings or items), visible only in
   the loop's header and block, and per function: a function literal or a
   local function doesn't see the labels around it. Reusing a label that an
   enclosing loop has is an error (expr.loop.9); a `par for` can't be labeled.
2. `break name` and `continue name` act on the labeled enclosing loop
   (expr.loop.10). The parser resolves the name, so the AST carries the
   target loop's position (`Break(pos, target)`, `Continue(pos, target)`),
   not a name, and no pass has to look labels up. Nothing changes for loops:
   there is no new statement variant and the loops' own nodes are unchanged.
3. Leaving several loops runs the `defer` statements of every block left and
   ends the task scopes opened inside the target (`defer_loop_exit_to`,
   `unwind_loop_to`): the same rules as `break`, from a deeper level.
4. Checks: the move checker sends the state of a labeled `break` or
   `continue` to the target loop, so a binding moved before `break outer`
   is seen after `outer`. A `while true` is diverging unless a `break` of
   its own, or a labeled `break` from a nested loop, leaves it
   (expr.loop.7). Loops that contain a labeled `break` or `continue`
   are not proved independent for `par` (conc.par.2): a `break outer` out of
   a `par for` is an error at parse time.
5. Diagnostics: an unknown label gets a fix that writes the nearest enclosing
   label, or, when no loop is labeled, labels the outermost enclosing loop
   with the name used. Outside any loop the error is expr.loop.5.
6. The `outer: for` entry of `gram.foreign.1` is removed: it is now valid.
7. The formatter works on tokens, so source is kept as written.
8. Native: `compiler/mir/lower_leave.ail` holds the jump; `lower.ail` gains no
   state field, only the loops' position in `Loop`.

## Consequences

- The sources of `Break` and `Continue` change arity (two fields): every
  match on them in the compiler was updated in this change.
- A labeled `while true` left only by `break outer` from a nested loop is not a
  diverging statement, so the function still needs its result after it.
- Loops that contain a labeled jump are kept sequential by the `par` analysis,
  a small speed cost for the rare loop that has one.
