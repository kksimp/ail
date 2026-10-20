# 0308. `ToJson` and `FromJson` are prelude derives written in AIL

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 17 — Metaprogramming; 22 — Standard library (`std.json`)
- **Spec rules:** [json.derive.1–2](../spec/std-json.md), [json.query.4](../spec/std-json.md), [json.prelude.1](../spec/std-json.md) (amended)

## Context

JSON-heavy agent programs spend most of their tokens reading members one by
one and building error values (`examples/tasks.ail` was 1.02x its Python
twin). ADR 0227 promised `ToJson` and `FromJson` as library derives.
**Tokens decide** (measured with `tools/tokbench`, mean of five tokenizers):
`tasks.ail` drops from 688.8 to 597.2 tokens (−13.3%), 0.88x its Python
twin (675.2), with byte-identical output. Then **AI understanding**: the
generated code is plain AIL an agent can read with `ail api --expanded`.

## Options considered

### Option A — generators in AIL in the prelude, plus `need_*` helpers (chosen)
- **Pros:** dogfoods ADR 0227; the generators are ordinary AIL, tested like
  any other; the generated code is short because `j.need("k")?.need_str()?`
  carries the error message.
- **Cons:** slower to generate than native code (milliseconds per type).

### Option B — native Rust derives like `Eq` and `Ord`
- **Pros:** fastest.
- **Cons:** no generated text to show; a second mechanism (OQ-096 c).

## Decision

1. `trait ToJson` (`to_json(self) Json`) with generator `derive_to_json`;
   `str`, `int`, `float`, `bool`, `Json` and `List[T ToJson]` implement
   it in the prelude. A struct becomes an object of its fields in order (`T?`
   that is `none` gives `null`); a variant without fields its name, one with
   fields `{"Name": {fields}}`.
2. `trait FromJson` has no methods; `derive_from_json` adds
   `T.from_json(j Json) T ! JsonError` (ADR 0307). Fields are read by name
   with `Json.need` and `need_str`/`need_num`/`need_int`/`need_bool`/
   `need_arr`; another type `U` calls `U.from_json`; `T?` of a scalar or
   `Json` may be missing. Unit-only enums read a name. Generic types, maps,
   other optionals and variants with fields are a `ct.derive.4` error.
3. `need` fails with `missing KEY`, the others with `expected a KIND, found
   KIND`, at offset -1.
4. Member names are field names; per-field renaming waits for OQ-091.

## Consequences

- `examples/tasks.ail` uses both derives; its `TaskError`, `load_task` and
  `with_status` are gone.
- `ToJson` and `FromJson` join the names that load `std.json`.

## Rejected alternatives

- **Native derives (B):** invisible code, and a second mechanism.
- **Run-time reflection (`json.encode(any)`):** ruled out by ADR 0229.
