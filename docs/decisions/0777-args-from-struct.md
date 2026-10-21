# 0777. `args.read[Opts]`: command-line options derived from a struct

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076); builds on [ADR 0776](0776-fieldinfo-doc.md)
- **Topic:** 17 — Standard library, 15 — Compile-time
- **Spec rules:** std.args.7-8 ([17-std](../spec/17-std.md)), ct.derive.6 ([15-compile-time](../spec/15-compile-time.md))
- **Amends:** [ADR 0627](0627-std-args.md) (its "derive from a struct" rejection)

## Context

ADR 0627 rejected a derive until generators could do it. They can now:
`FieldInfo.doc` (ADR 0776) gives each field's help text. Twin t05 declared
every option twice, once as a spec and once as a read of `Args`.

## Decision

`impl FromArgs for Opts` derives `Opts.from_args(name, about, words)` from the
fields, and `args.read[Opts](name, about, words)` calls it. `bool` is a switch,
`str?`/`int?`/`float?` an option (`none` when absent, so a default is
`a.out else "a.txt"`), `List[str]` the rest, and `str`/`int`/`float` a
required positional. The `##` comment is the help; a switch or option gets its
first letter as the short name unless taken (std.args.7). `Args` gains `has`,
`opt`, `int`, `float`, `opt_int` and `opt_float` for the derived code
(std.args.8). `FromArgs` and `ArgsError` move to the prelude's `argsderive`
part, read when a program names either, like `ToJson`; `args.ArgsError` is
now plain `ArgsError`.

Generated code names `args.flag`, so generated files now see the `use` lines
of their module's other files (ct.derive.6).

## Token measurement

Twin t05, mean of five tokenizers: the spec-list form 95.4, derived 87.4
(-8.0, -8.4%), Python 71.0. The `impl FromArgs for Opts` line and
`str?` plus `else "a.txt"` cost what the sketch's 70.4 left out.

Quick reference: +46 tokens (12,818.8 to 12,865.2 on this branch's base, before the trim in
the card-trim lane); language card unchanged (6,949.2).

## Rejected alternatives

- **`args.parse[Opts]`:** AIL has no overloads, and `parse` is taken by the
  spec form; renaming it would break every caller for one token.
- **Defaults as field values or `@default`:** no field defaults exist; `T?`
  with `else` at the use costs about three tokens and needs no new syntax.
- **No `impl` line (implicit derive):** rejected for `ToJson` in ADR 0774, for
  the same reason: the type's behaviour should be visible where it is declared.
- **Guessing positional versus option from the name:** `T?` is the one
  signal, so the rule fits a sentence.
