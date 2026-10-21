# 0776. `FieldInfo.doc` carries a field's `##` comment

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** Integrator, from lane 192's options (option 1 of 2)
- **Topic:** 16 — Reflection and 15 — Compile-time
- **Spec rules:** ct.derive.5 ([15-compile-time](../spec/15-compile-time.md)), refl.info.1 ([16-reflection](../spec/16-reflection.md))
- **Amends:** [ADR 0229](0229-compile-time-reflection-only.md)

## Context

A derived `args.parse[Opts]` (lane 192, twin t05) needs one help line per
option. Python's `argparse` takes it from `help=` arguments; the derive can
take it from the `##` comment above each field, which agents already write.
`FieldInfo` had only `name`, `type_text` and `public`, so a generator could not
see the comment.

## Decision

`FieldInfo` gains `doc str`: the field's `##` comment lines joined by `\n`
without the `##` marks, or `""` when there is none (`ct.derive.5`). The
bootstrap fills it from the source text while it builds `TypeInfo`. The
self-hosted front end has no `TypeInfo` (derive generators run in the Rust
bootstrap only), so it needs no change. Constructing a `FieldInfo` by hand now
takes a `doc` argument.

## Rejected alternatives

- **No help text** (option 2): the derived parser could print only option
  names, and t05 would keep a hand-written usage string.
- **A `@help("...")` attribute per field:** a second place for the same
  sentence, and more tokens than a comment agents write anyway.
