# 1309. How-tos for the new std surface; help for `Json.Int` and `@derive`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), from the 2026-11-27 state review
- **Topic:** Agent documentation and diagnostics
- **Spec rules:** [json.type.1](../spec/std-json.md), [gram.attr.2](../spec/03-grammar.md), [trait.derive.1](../spec/10-traits.md)

## Context

The review found that `ail ask how "http server"` and `"derive json"` found nothing, that `Json.Int(3)` got the fix
`Json.int(3)` (a nearest-name guess at a different function), and that `@derive(ToJson)` got a generic attribute
error. With the card's std detail moved to one index line ([ADR 1306](1306-card-std-index-line.md)), how-tos are
the lookup that returns code to paste.

## Decision

- Six how-tos in `docs/how-to.md`: a web server, HTTP routes with parameters and JSON, testing a router in a unit
  test, JSON for your own struct, a database file, and sorting a map's keys or values; `check-doc-snippets` checks
  their code.
- `Json.Int`, `Json.String` and the other variant names of other JSON libraries get help naming `Json.Num`,
  `Json.Str`, `Json.Bool`, `Json.Arr` or `Json.Obj`, and no fix: there is no one meaning to write.
- `@derive(...)` gets help that names the empty `impl` form (`impl ToJson for T`).
- Mistakes pairs `json_int_variant` and `derive_attr_json` check both.

## Rejected alternatives

- **A certain fix `Json.Int(3)` to `Json.Num(3.0)`.** The literal's type changes; an agent may have meant
  `n.to_json()`. Help is enough.
- **Accept `@derive` as syntax.** No syntax changes to mimic other languages (the designer's standing rule).
