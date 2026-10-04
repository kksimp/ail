# 04. Types

Chapter key: `type`

> **Status:** Drafted (partial). Only optional types are specified. Numeric
> types, structs, enums, collections, strings and the rest are not yet
> designed.

## Optional types

**type.opt.1** — AIL has no null value. *(ADR 0048)*

**type.opt.2** — `T?` denotes an optional value of type `T`. It is exactly
equivalent to `Option[T]`. *(ADR 0048)*

**type.opt.3** — The canonical form of an optional type is `T?`.
*(ADR 0048)*

```ail
fn find(id i64) User?
    ...
```

> **Note:** see also `plat.ptr.2` (the 64-bit size and index type) and
> `expr.arith.*` (integer arithmetic).
