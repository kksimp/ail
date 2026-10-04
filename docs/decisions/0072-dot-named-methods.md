# 0072. Methods are declared as dot-named functions: `fn Type.name(self, ...)`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 10 — Traits/interfaces; 2 — Core syntax
- **Spec rules:** [trait.method.1–2](../spec/10-traits.md)
- **Evidence:** [traits-generics-options.md §1](../design/traits-generics-options.md#1-how-a-type-gets-methods) (mean tokens: dot-named 49.4, `impl` block 50.8, Go receiver 50.8, free functions 48.4)

## Options considered

- **Dot-named functions (chosen):** each method is an independent top-level
  item (one-place edits for AI and structural tools), greppable, and needs no
  extra indentation. Con: methods aren't visually grouped.
- **`impl` blocks:** grouped, but an extra indentation level and +3% tokens.
- **Free functions called with dot syntax (UFCS):** fewest concepts, but
  `x.f()` depends on imports, which hurts local reasoning.
- **Go-style receivers:** receiver names vary per method.

## Decision

1. A method is declared `fn Type.name(self, params...) Result`. The receiver
   is written `self` (read-only) or `mut self` (mutable).
2. A method is called as `value.name(args...)`.

```
fn Circle.area(self) f64
    pi * self.r * self.r

fn Circle.scale(mut self, k f64)
    self.r = self.r * k
```

## Consequences

- Open (OQ-036): functions without `self` declared as `fn Type.name(...)`
  (e.g. constructors called as `Circle.new(...)`), which modules may declare
  methods for a type, and the method-resolution rules.
- Trait implementations use `impl` blocks (ADR 0073). Inherent methods and
  trait methods therefore look different. That's intentional, because they
  mean different things.

## Rejected alternatives

- **`impl` blocks for inherent methods:** indentation and edit-locality cost.
- **UFCS:** import-dependent meaning.
- **Go receivers:** inconsistent receiver naming.
