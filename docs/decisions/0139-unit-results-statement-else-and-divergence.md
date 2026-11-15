# 0139. Unit results can fail (`! E`); `else` works on them as a statement; divergence is a rule, not a type

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 8 — Error handling; 2 — Core syntax (expressions)
- **Spec rules:** [err.type.7](../spec/08-errors.md) (amends err.type.1–3), [err.else.3](../spec/08-errors.md), [err.else.5](../spec/08-errors.md), [err.model.2](../spec/08-errors.md), [expr.block.3](../spec/05-expressions.md)
- **Amended by:** [ADR 1119](1119-never-return-type.md) (a `never` result type for declared functions)
- **Amends:** [ADR 0125](0125-if-binding.md) (its rejected alternative "`else` on an expression statement" is revisited for results with no success value)
- **Extends:** [ADR 0046](0046-inferred-error-sets.md) (`err.type`), [ADR 0049](0049-else-for-fallback-and-early-exit.md) (`err.else`), [ADR 0088](0088-entry-point-and-print.md) (`mod.main.1` already allows `!` and `! E` for `main`), [ADR 0117](0117-typing-rules.md) (`expr.block.3`)
- **Evidence:** `examples/interp.ail`, `examples/sheet.ail`, `examples/json.ail`; tokbench, mean of five tokenizers

## Context

Three programs (an interpreter, a JSON tool and a spreadsheet engine; 2,036
lines of AIL) hit three gaps:

1. **Fallible functions with nothing to return.** `mod.main.1` allows
   `fn main() !` and `fn main() ! E`, but `err.type.2` knows only `!T`. The
   drafts wrote `fn run_script(source str) !` (interp) and
   `fn Parser.expect(mut self, sym char) ! ParseError` (sheet) on ordinary
   functions. The bootstrap reads `! ParseError` as `!T` with
   `T = ParseError`, which `err.decl.3` rejects.
2. **Handling a failure, then carrying on.** interp's `main` runs sixteen
   scripts; a failing one prints its error and the loop moves on:
   `run_script(src) else err` plus a block that only prints. ADR 0125 rejected
   `else` on an expression statement because it "discards the success value";
   a result with no success value discards nothing. `err.else.2` also demands
   a block that leaves or produces a `T`, and the spec doesn't say what
   `else name` binds for an inferred set (the block reads `err.pos.line`).
3. **Diverging code.** `_ => unreachable("...")` and a function that ends in a
   loop returning from inside must type-check. `expr.block.3` lists only
   `return`, `break` and `continue`, and `unreachable` isn't defined (the
   interp drafter first wrote `panic(...)`).

## Options considered

The interp `main` loop, mean of five tokenizers:

| Option | `main` | with `run_script` |
|---|---|---|
| **`run_script(src) else err` + block (chosen)** | **53.0** | **109.0** |
| `if done := run_script(src)`, `continue` as a filler, `else err` | 60.0 (+13%) | 116.0 |
| `ok := run_script(src) else err ... continue`; `run_script` returns `bool` | 57.8 | 117.8 |
| `run_script` returns `MiniError?`; every `?` becomes `else err` / `return err` | — | 129.6 (+19%) |

A save-warn-continue snippet (`save(path, lines) else err` and a `print`, then
more code) is 33.6 tokens against 46.8 for the best `if ok :=` form (−28%).

## Deciding criterion

Token usage **ties**: the statement form is 11.7% smaller at its site, but it
appears once in three programs, a rare construct (ADR 0076). **AI
understanding** decides:
- it mirrors `x := e else ...` (ADR 0049), and the interp drafter wrote it
  unprompted;
- the workarounds are fragile: the `if` form binds a meaningless name and
  needs a filler branch, which works only because the call ends the loop;
- each return-type spelling has one reading, which the formatter shows, and
  divergence adds no type and no prelude name beyond `unreachable` ([ADR 0084](0084-no-shadowing-module-names.md)).

**Why this extends ADRs 0046 and 0049 instead of changing them** (AGENTS.md
escalates changes to a designer's ADR). ADR 0046 defines `!T` with `T` a
success type, and `err.decl.3` ([ADR 0120](0120-error-keyword.md)) says `T` is never an error type. So
every `!X` with `X` a declared error type was already a compile-time error;
the new reading applies only to those programs, and every valid `!X` keeps
its meaning. A bare `!` and `! E` were specified only for `main` (ADR 0088).
ADR 0049 covers `T?` and `T ! E` with a success value, where `err.else.2`
still governs; the new form applies only where there is no `T`.

## Decision

1. **Unit results (`err.type.7`, amends `err.type.1–3`).** In a return type,
   `! E` and `! A | B` (nothing before `!`) mean "no success value; can fail
   with `E` (or `A | B`)". A bare `!` means no success value and an inferred
   set, for any function (it generalises `mod.main.1`). `err.type.3` lists
   `! E` among the explicit forms.
2. **Reading `!X`.** `!X`, with `X` a type name, is read by name resolution:
   if `X` is a declared error type it is `! X`, otherwise `X ! S` with `S`
   inferred, as before. A type parameter is never an error type here, so a
   generic signature reads the same for every instantiation (cf.
   `gen.impl.1`). Any other type after `!` (`!List[X]`, `!X?`) is `T ! S`.
   The formatter writes `! E` with a space for an explicit set and `!T`
   without one for an inferred set.
3. **Body.** The body of a unit-result function is a statement block. It
   succeeds when the block completes normally or at a bare `return`, and
   fails at `return e` and at `?`.
4. **Statement `else` (`err.else.5`).** For `e` whose type is a result with no
   success value (`!`, `! E`), the expression statements `e else BLOCK`,
   `e else name BLOCK` (binds the error as `err.else.3`) and
   `e else STATEMENT` (`save() else return`) run the else part only on
   failure. The block MAY complete normally; execution continues after the
   statement. A value fallback (`save() else 0`) is an error. On `T?`, or on
   `T ! E` with a success value, a statement `else` is an error with the help
   "bind the value: `x := ... else ...`".
5. **The bound error (`err.else.3`, amended).** The name bound by `else name`
   (also in `if ... else name`) has the error part's type. A set, inferred or
   explicit, with one member gives that member's type, so field access works.
   With two or more members, field access waits for OQ-029.
6. **Divergence (`expr.block.3`, amended).** A statement diverges when it is
   `return`, `break`, `continue`, a call to `unreachable(msg)`, `while true`
   (the literal `true`) whose body has no `break` for that loop, or an
   `if`/`else` or `match` all of whose branches diverge. A `break` inside a
   nested loop belongs to that loop. A block whose last statement diverges is
   accepted wherever any type is expected, including a one-line arm and an
   else part, so a function can end with `while true`. No type is nameable
   for divergence: there is no `Never`.
7. **`unreachable` (`err.model.2`, amended).** `unreachable(message str)` is a
   prelude function that traps with the message. `unreachable()` gets the help
   "add a message"; `panic`, `assert`, `fatalError` and `loop` get helps
   ([ADR 0140](0140-diagnostics-for-other-languages.md)).

```ail excerpt
fn run_script(source str) !
    tokens := tokenize(source)?
    mut interp := Interp.new()
    interp.run(parse(tokens)?)?

fn main()
    for script in SCRIPTS
        print("== {script.name} ==")
        run_script(script.source) else err
            print("error at {err.pos.line}: {err.msg}")

fn Ids.next_free(mut self) int
    while true
        id := self.counter
        self.counter += 1
        if not self.used.contains(id)
            return id
```

## Consequences

- A fallible function with nothing to return needs no dummy value or `bool`.
  The checker and formatter need name resolution to read `!X` (as `type.enum.4`
  already does for qualification). The bootstrap parser reads `! ParseError`
  as `!T` today and must change.
- `unreachable` is a prelude name, so locals can't reuse it (ADR 0084).
- The quick reference says `if x := ...` with `else err` is how to "handle
  success and failure and then carry on"; it needs a line for the statement
  form and the four return-type spellings.

## Rejected alternatives

- **Keep the `if name := e` workaround, or return `bool` or `MiniError?`:**
  60.0, 57.8 and 129.6 tokens; a meaningless binding and a filler branch, a
  dropped error value, or three lines per `?`.
- **Statement `else` on `T?` or on results with a success value:** it
  discards the value, ADR 0125's reason for rejecting it, which still holds
  there.
- **`else` as an expression** (`nums.min() else 0` inside an expression):
  wanted (sheet's Min/Max arms are 20.8 tokens against 33.6, −38%), but it
  needs a precedence decision that belongs with the expression grammar. Left
  to [open-questions.md](../open-questions.md), not part of this batch.
- **A nameable `Never` type** (Rust's `!`, Swift's `Never`, TypeScript's
  `never`): a new type, a prelude name and coercion rules, and none of the
  three programs needed a user-written diverging function.
- **Treating a type parameter as an error type, or reading `!X` per
  instantiation:** `fn wrap[T](x T) !T` would mean different things per call.
