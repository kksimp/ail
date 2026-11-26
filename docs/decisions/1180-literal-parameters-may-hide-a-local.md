# 1180. A function literal's parameters and a comprehension's variables may hide a local

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** Designer (chose it on the 2026-11-26 decision card, "All three"; proposal by the checker rough-edges lane)
- **Topic:** 2 — Core syntax; 4 — Type system (name resolution)
- **Spec rules:** [expr.bind.16](../spec/05-expressions.md), [expr.bind.4](../spec/05-expressions.md), [expr.comp.2](../spec/05-expressions.md), [expr.key.2](../spec/05-expressions.md), [api.refs.3](../spec/22-tools.md)
- **Amends:** [ADR 0044](0044-no-shadowing.md)

## Context

ADR 0044 forbids every local name that repeats a visible one, so that a name means one thing in a function. For a
statement that holds: the reader sees `x := ...` and every later `x` is that binding. For the one-expression binders
it costs more than it gives. In the probe run of first drafts (`rough-edges-probes-main.txt`), `xs.map(|x| x * 2)`
and `[item.name for item in items]` failed with expr.bind.4 whenever the function already had an `x` or an `item`,
which a long function usually does. The author's fix is a made-up name (`x2`, `it`), which reads worse, and the same
line is accepted or rejected depending on code far above it.

## Decision

- **expr.bind.16:** a parameter of a function literal, the parameter of a key function, the `|x|` of
  `tensor.from_fn` and `Tensor.map`, and a comprehension's variables (the unpacked names of `for a, b in xs`
  included) may reuse the name of a local visible where they are written. The new name hides the local to the end of
  the literal's body or the comprehension. Afterwards the name is the outer local again, unchanged.
- **Unchanged (expr.bind.4):** `:=`, `for` statements, `match` and `if x := e` bindings, `else err`, parameters of a
  local `fn`, two parameters of one literal, two variables of one comprehension. A local whose type is still open
  (`mut xs := []`) can't be hidden, because its first use fixes its type.
- The hidden region is one expression that starts at the binder, so the reader who sees the use also sees the
  binding. That is the property ADR 0044 protects, and why statements keep the rule.
- A function literal doesn't copy a local its own parameter hides, and the same holds for a literal or comprehension
  nested in its body (type.fn.5; `compiler/ast/names.ail`).
- `ail api --refs` on a local leaves out the uses that belong to a hiding parameter or variable, and on such a
  parameter lists only its own (api.refs.3).
- Native: each binder already declared its name in a scope of its own (the literal's function, the inlined
  element loop, the comprehension's `for`), so the lowering needs no change beyond the capture list.

## Options considered

- **Keep ADR 0044 as it is:** rejected by the designer; the error teaches nothing and the workaround names read worse.
- **Allow shadowing everywhere:** rejected. `x := f(x)` chains hide which value a later line reads, the case ADR 0044
  was written against.
- **Allow it for `for` statements and `match` bindings too:** rejected. Their bodies are blocks of any length, so the
  binder can be far from the use.
- **A lint instead of an error:** rejected; AIL has one answer per construct, not a style setting.

## Consequences

- `compiler/check/{body,call,closure,builtins,tensor,tuples,moves}.ail`, `compiler/ast/names.ail`,
  `compiler/api/{refs,query}.ail`. No file in `compiler/mir/` changes.
- Case `literal_name_hides`; `comprehension_errors`, `key_function_errors` and `closures_errors` keep the errors that
  remain. The explain, fmt and card outputs need regenerating.
- Programs accepted before mean the same; more programs are accepted.
- Not built in the cloud (ADR 0842); gated on the Mac.
