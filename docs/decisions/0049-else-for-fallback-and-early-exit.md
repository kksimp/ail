# 0049. `else` handles failure: fallback values and early exit

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 8 — Error handling; 2 — Core syntax
- **Spec rules:** [err.else.1–4](../spec/08-errors.md)
- **Partly resolves:** OQ-029
- **Evidence:** cl100k tokens: `x := f() else 8080` 12 (same as `??` and `.or()`; `.or()` costs 1–2 more elsewhere). Early exit: `else return` 10, `match` form 20, `if let` form 21.

## Context

[ADR 0045](0045-error-model.md) and [ADR 0048](0048-optional-shorthand.md)
define propagation with `?`. Code also needs to supply a fallback value, or to
exit early while binding the success value.

## Options considered

### Fallback
- **`else` (chosen):** reads like English, and the same keyword handles early
  exit. Con: `else` also appears in `if`/`else`, so position tells them apart.
- **`??`:** compact, but a symbol to memorise, and early exit would need a
  second concept.
- **`.or()`:** a library convention, eager evaluation unless a lazy variant
  is used, and more tokens on some tokenizers.

### Early exit
- **`else` block (chosen):** half the tokens of `match` or `if let`. Keeps the
  happy path unindented.
- **`match` only:** double the tokens, and nested happy paths.
- **`if let`:** the most tokens, and the success value lives only inside the branch.

## Decision

For an expression `e` of type `T?` or `T ! E`:

1. **Fallback:** `x := e else d` binds the success value of `e`, or `d` when
   `e` is absent or failed. `d` is evaluated only in that case.
2. **Early exit:** the `else` part may be a statement that leaves the
   enclosing block (`return`, `break`, `continue`, or a trapping call), or
   an indented block ending in one.
3. **Error access:** for `T ! E`, `else name` binds the error to `name`
   inside the else block.
4. After the statement, the bound name always holds a success value.

```
port := env("PORT") else 8080
user := find(id) else return NotFound
data := read(path) else err
    log.warn("read failed", err)
    return Defaults
```

## Consequences

- The type checker verifies that an `else` block either yields a value of
  type `T` or definitely leaves the enclosing block.
- Still open in OQ-029: safe member access (`a?.b`) and matching on specific
  errors. Those depend on the `match` design.

## Rejected alternatives

- **`??`, `.or()`:** second concepts with no token benefit.
- **`match` and `if let` as the only forms:** about double the tokens, and
  more nesting.
