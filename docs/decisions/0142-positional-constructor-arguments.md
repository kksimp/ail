# 0142. Struct and variant values can be built positionally when no swap could compile

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** Designer
- **Topic:** 4 — Type system
- **Spec rules:** [type.struct.3–4, type.struct.7–8](../spec/04-types.md), [type.enum.6](../spec/04-types.md)
- **Amends:** [ADR 0058](0058-struct-literal-syntax.md) (`field: value` was the only form), [ADR 0113](0113-struct-literal-field-order.md), [ADR 0123](0123-variants-with-data.md)
- **Evidence:** [program-measurements-r2.md](../design/program-measurements-r2.md) (Python parity round)

## Context

ADR 0058 made `Type(field: value)` the way to build a value, and left field
shorthand open. On the three example programs, the labels cost about 3% of
all tokens: `ParseError(line: line, col: col, msg: msg)` against Python's
`ParseError(line, col, msg)`. Labels protect against swapped arguments, but a
swap only goes unnoticed when the two fields have the same type; otherwise
the type checker rejects it.

## Options considered

Mean of five tokenizers on the three programs (14,802.8 tokens today):

| Option | Tokens | Change | Silent swaps possible |
|---|---|---|---|
| Names only (ADR 0058) | 14,802.8 | — | no |
| Shorthand only (`Pos(row, col)` when locals are named like the fields) | 14,641.8 | −1.1% | no |
| **Positional when the fields filled have different types, plus shorthand (chosen)** | 14,375.4 | **−2.9%** | no |
| Positional always (Python dataclasses, Kotlin) | 14,359.2 | −3.0% | yes |

## Decision

1. Each argument of a struct literal or variant constructor is one of:
   - **named**, `field: value`, as before;
   - **shorthand**, a bare local name equal to a field's name, which sets that
     field (`Member(key, value)`);
   - **positional**, any other unlabelled expression.
2. Named and shorthand arguments set their fields. Positional arguments fill
   the remaining fields in declaration order. Every field is set exactly once.
3. Positional arguments are allowed only when the fields they fill have
   pairwise different declared types, so that swapping two of them is always
   a type error. Two fields whose types are type parameters count as the
   same type. Otherwise it's a compile-time error whose help names the fields.

```ail
error struct ParseError
    line int
    col int
    msg str

struct Pos
    row int
    col int

ParseError(lines.len(), col, msg)   # col and msg: shorthand; line: positional
Str(text)                           # one field
Pos(row, col)                       # shorthand: the locals are row and col
Pos(r, c)                           # error: row and col are both int; name them
Pos(row: r, col: c)                 # named, as before
```

## Consequences

- One-field variants (`Str(text)`, `Int(n)`) need no label.
- A type whose fields share a type keeps its labels unless the arguments are
  locals named like the fields.

## Rejected alternatives

- **Positional always:** 0.1% fewer tokens, but `Pos(c, r)` would compile and
  swap silently.
- **Shorthand only:** safe, but saves less than half as much.
- **Names only (status quo):** the most tokens; labels repeat what the types
  already guarantee.
