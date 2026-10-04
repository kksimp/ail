# Traits, methods and generics: options explained

> **Non-normative design aid,** 2026-10-04. Background for the traits and
> generics questions (status.md topics 9 and 10). Every example is a file in
> `tools/tokbench/corpus/r2/`, and the numbers come from `tools/tokbench/run.sh`
> (mean of five tokenizers).

## 1. How a type gets methods

| Form | Example | Tokens |
|---|---|---|
| **Dot-named functions** | `fn Circle.area(self) f64` | 49.4 |
| `impl` block | `impl Circle` + indented `fn area(self) f64` | 50.8 |
| Go-style receiver | `fn (c Circle) area() f64` | 50.8 |
| Plain functions called with dot syntax (UFCS) | `fn area(c Circle) f64`, called as `c.area()` | 48.4 |

```
fn Circle.area(self) f64
    pi * self.r * self.r

fn Circle.scale(mut self, k f64)      # mutating method: `mut self`
    self.r = self.r * k
```

- **Dot-named functions:** each method is an independent top-level item, so
  adding, moving or deleting one is a one-place edit (good for AI structural
  edits), and "Circle.area" is greppable.
- **`impl` blocks:** group related methods, but add an indentation level.
- **Go receivers:** the receiver name varies per method (`c`, `circle`, …).
- **Plain functions with dot syntax:** the fewest concepts, but `x.f()` then
  depends on which modules are imported, so the same line can mean different
  things in different files.

## 2. How a type declares that it satisfies a trait

| Form | Example | Tokens |
|---|---|---|
| **Explicit `impl Trait for Type`** | `impl Shape for Circle` + its methods | 44.4 |
| Structural (implicit, Go-style) | Nothing. Having `area` is enough. | 40.0 |
| Declared on the type | `struct Circle is Shape` | 42.2 |

```
trait Shape
    fn area(self) f64

impl Shape for Circle
    fn area(self) f64
        pi * self.r * self.r
```

- **Explicit:** intent is visible, the compiler checks completeness in one
  place, there's no accidental conformance, and it can be added later for
  types you don't own (retroactive).
- **Structural:** the fewest tokens, but a type can satisfy a trait by
  accident, and errors appear at the use site instead of the definition.
- **Declared on the type:** visible at the type, but you can't make an
  existing (library) type implement your trait.

## 3. Generic constraints

| Form | Example | Tokens |
|---|---|---|
| **Inline** | `fn largest[T Shape](items List[T]) T` | 44.0 |
| `where` clause | `fn largest[T](items List[T]) T where T Shape` | 46.0 |
| Implicit (template-style) | `fn largest[T](items List[T]) T` | 42.8 |

- **Inline** matches the `name Type` rule (ADR 0037).
- **Implicit** constraints can't type-check the generic function on its own.
  Errors appear only when it's used, and they're notoriously hard to read
  (C++ templates). That's bad for AI repair loops.

## 4. Mutating a method's receiver: mark the call?

[ADR 0061](../decisions/0061-call-site-mut.md) requires `mut` at the call site
for `mut` arguments. Should the receiver of a mutating method be marked too?

| Form | Example | Tokens (4 lines) |
|---|---|---|
| **Unmarked** | `tokens.push(x)` | 28.8 |
| Marked | `mut tokens.push(x)` | 31.8 (+10%) |

Unmarked is how Swift and Rust do it: the receiver must be a `mut` binding
(visible where it's declared), and the method's signature says `mut self`.
Marked is fully consistent with ADR 0061, but mutation-heavy code pays about 10%.
