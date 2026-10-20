# 0376. The syntax tree in AIL, `compiler/ast/`, and its JSON is the derived shape

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), agreed between lanes 113 and 114
- **Topic:** 23 — Compiler architecture (Stage 3, ADR 0022)
- **Spec rules:** `json.derive.1–2`, `ct.derive.2`, `expr.bind.5` (fixes below); `api.ast.*` (lane 113)

## Context

Stage 3 writes the lexer, parser (lane 113) and checker (lane 114) in AIL,
run by the bootstrap and compared with the Rust ones (ADR 0032). Both need
one syntax tree, and the Rust parser must print the same tree
(`ail api --ast`) so each AIL stage can be tested alone. Files directly in
`compiler/` are the package's root module, which no module can import.

## Options considered

### Option A — the AST types in their own module, JSON derived from them (chosen)
`compiler/ast/ast.ail` mirrors `bootstrap/src/ast.rs` with `impl ToJson`
and `impl FromJson` on every type. The JSON is whatever the derives write:
structs are objects, a variant with fields is `{"kind": "Name", ...}`, a
field-less variant is its name, `none` is `null`, positions are
`{"line", "col"}` in the bootstrap's numbering.
- **Pros:** no hand-written reader or writer in AIL; `Program.from_json`
  loads the Rust dump, and the parser's tree compares by `to_json()`; one
  definition to keep in step with `ast.rs`.
- **Cons:** derived integers go through `float`; names must avoid keywords
  and the `kind` member.

### Option B — a hand-designed JSON schema read with `Json` accessors
- **Cons:** two readers and a writer to write and keep equal, each larger
  than the types themselves.

### Option C — the AST types in the root module beside the parser
- **Cons:** the checker's module can't import the root module.

## Decision

Option A. Integer literals keep their value as decimal text
(`Expr.Int(value str)`, `Literal.IntLit(value str)`), so a JSON number never
holds a value above 2^53; positions and indexes are far below it. A char is
a one-character `str`. Pairs of a name and position are `Ident`. Names
that are keywords or clash in AIL are renamed: `is_unique`, `is_ensures`,
`is_par`; `TypeRef.shape` (`Named`, `Optional`, `Fallible`, `AnyTrait`,
`RefTo`); `Stmt.Eval`; `Else` is `Fallback`, `Leave`, `Handler`; `Expr`
uses `NoneLit`, `ListLit`, `MapLit`, `SetLit`, `Member`, `OrElse`,
`NewRef`; `Within` is `TopLevel`, `InTrait`, `InImpl`; `Pattern.Alt`;
`InterpPart.Hole`; `CompKind` is `ListComp`, `SetComp`, `MapComp`.

Writing it found three bootstrap faults, fixed with it:

- A derived `from_json` built variants positionally, which `type.struct.7`
  rejects when two fields share a type (`Add(lhs Expr, rhs Expr)`); it now
  names each field (json.derive.2).
- `impl ToJson for T` in a module failed with `ct.derive.2`: the generator
  looked for `T` under its bare name, and wrote the field types qualified
  (`ast.Pos`), which the module itself can't name. It now matches the last
  segment and writes the module's own types bare.
- A prelude local (`depth` in the JSON derive) clashed with a program
  function of that name (`expr.bind.5`); the prelude's locals no longer
  meet the program's items.

## Rationale

Token use and AI understanding both favour one set of types over a schema
plus readers (ADR 0076 criteria 1–2). Exact integers by text avoid a
`std.json` change that would break exhaustive matches on `Json`.

## Consequences

- Lane 113 writes `ail api --ast` (one file, raw) and `ail api --ast
  --program` (the assembled program with prelude, files, public lines and
  derive output) in this shape; its selftest round-trips every dump through
  `Program.from_json`.
- `ast.rs` and `ast.ail` change together; the round-trip test catches drift.
- Derived JSON prints integers as floats (`1.0`); a byte comparison with
  the Rust dump either prints numbers the same way or compares parsed values.

## Rejected alternatives

Options B and C, above. A `Json.Int` variant: it would break every
exhaustive `match` on `Json` in programs, for a precision the AST doesn't
need.
