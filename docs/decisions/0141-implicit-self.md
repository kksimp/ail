# 0141. Inside a method, the receiver's fields and methods are in scope by bare name

- **Status:** Accepted; rule 2's formatter clause superseded by [1081](1081-formatter-writes-self.md); a local may hide a field since [1121](1121-locals-may-shadow-functions-and-modules.md)
- **Date:** 2026-10-05
- **Decided by:** Designer
- **Topic:** 10 — Traits; 2 — Core syntax
- **Spec rules:** [trait.method.6–7](../spec/10-traits.md)
- **Extends:** [ADR 0072](0072-dot-named-methods.md) (methods with an explicit `self`), [ADR 0044](0044-no-shadowing.md) (no shadowing)
- **Evidence:** [program-measurements-r2.md](../design/program-measurements-r2.md) (Python parity round); `tools/tokbench/corpus/programs/`

## Context

On the three example programs (ADR 0130–0140), AIL is 9.9% above Python.
The receiver prefix `self.` appears 394 times in them, mostly before method
calls (`self.peek()`). Python writes `self.` too, so removing it is one of the
few ways to save more than Python does.

## Options considered

Measured by rewriting the three programs (mean of five tokenizers; 14,802.8
tokens today):

| Option | Tokens | Change |
|---|---|---|
| `self.x` everywhere (Python, Rust) | 14,802.8 | — |
| Bare field names only | 14,618.4 | −1.2% |
| **Bare field and method names (Swift, Kotlin, Java, C#) (chosen)** | 14,205.6 | **−4.0%** |

## Decision

1. Inside a method declared with `self` or `mut self`, every field and method
   of the receiver is in scope by its bare name: `x` means `self.x`, and
   `f(args)` means `self.f(args)`. The methods are the type's own and those of
   the traits it implements. In a trait's default method, the trait's methods
   are in scope the same way.
2. `self.x` and `self.f()` stay valid; the formatter writes the bare form.
   `self` alone still names the whole receiver. *(Superseded in part by
   [ADR 1081](1081-formatter-writes-self.md): the formatter writes `self.`.)*
3. To keep a bare name unambiguous, a parameter or local of a method can't
   have the name of a field of its receiver (an extension of ADR 0044). A
   local may share a method's name (`pos := pos()`), since a method is only
   reached through a call. A bare call whose name is both a method of the
   receiver and a top-level function is a compile-time error that names both
   and suggests `self.f()` or renaming.
4. Associated functions (no `self`) see no fields or methods by bare name.

```ail excerpt
fn Parser.parse_value(mut self) !Json
    skip_whitespace()
    c := peek() else return unexpected()
    pos += 1
```

## Consequences

- A method body reads like a function body; whether a bare name is a local or
  a member is decided by whether the function declares it.
- Adding a field to a type can make an existing method's local of that name
  an error. That is the price of rule 3, and the error names the
  new member.

## Rejected alternatives

- **Explicit `self.` only** (Python, Rust): clearest at the use site, but the
  most tokens, and Python already writes it.
- **Fields only:** saves 1.2%, and leaves the larger saving (method calls)
  on the table, with two rules for one receiver.
- **Let locals shadow fields** (Kotlin, Swift, with `self.x` to reach the
  field): a bare name would change meaning when a local is added; AIL has no
  shadowing anywhere else (ADR 0044).
