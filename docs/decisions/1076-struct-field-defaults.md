# 1076. Struct field defaults: `name Type = value`

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076)

## Context

Spec `type.struct.4` made every struct literal name every field, and the
spec noted that field defaults were undecided. Config-like structs grow
fields and force every call site to change. A language-review item for the
0.1.0 release candidate asked for them, with the condition that a reader
of a literal is never misled.

## Decision

A struct field may be declared `name Type = value` (`type.struct.8`). A
literal that leaves the field out gets the value. The value is a **plain
constant**: literals, `none`, `[]`, `{}`, `List[T]()`, `Map[K, V]()`,
`Set[T]()` and lists, sets, maps, tuples and a leading `-` of them. It names
no constant, function or type. A missing field's error (`type.struct.4`)
says which fields may be left out. Variant fields have no defaults. Each
literal evaluates the value afresh, so no two values share a default list.

## Why this shape (ADR 0076 order)

1. AI understanding: a default that cannot call or name anything has no
   effect and no scope, so a model reading `Config(name: "a")` can find the
   value on the struct's declaration line and nowhere else. The same text
   means the same in every module.
2. Tokens: no cost for code that does not use the feature.

## Rejected alternatives

- **Any `mod.const.2` constant expression.** It can name constants and types
  of the declaring module, which are not in scope in an importer; resolving
  them there needs a separate pass. Possible later without breaking this rule.
- **A default that may call functions.** Hidden effects at every literal.
- **`..Default` spread or a builder.** More tokens, and a second way.
- **Defaults on variant fields.** Variants are matched by position (ADR 0123).

## Consequences

- A derived `impl Default` leaves fields with a declared default out of the literal it writes, so it keeps them (the built-in generators only; reflection `FieldInfo` is unchanged).
- `ail fmt` is token based and needs no change. `FromJson` still requires
  every key.
- The compiler's own source does not use the feature in this release.
