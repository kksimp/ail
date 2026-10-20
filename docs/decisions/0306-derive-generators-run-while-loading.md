# 0306. The bootstrap runs derive generators while it loads the program

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 17 — Metaprogramming; 16 — Reflection; 18 — Compile-time execution
- **Spec rules:** [ct.derive.1–4](../spec/15-compile-time.md), [refl.info.1–3](../spec/16-reflection.md) (amended: `is_error`)
- **Implements:** [ADR 0227](0227-derive-generators.md), [ADR 0229](0229-compile-time-reflection-only.md)

## Context

ADR 0227 lets a trait name a generator, a compile-time function from
`TypeInfo` to AIL text, and ADR 0229 defines `TypeInfo`. The bootstrap has
no query compiler (ADR 0224) to run a generator in the middle of checking,
but it has an interpreter (ADR 0032) and a loader that every command shares
(`run`, `check`, `test`, `api`, `effects`, `fix`). **AI understanding
decides**: generated code must behave exactly like written code in every
command; speed of programs without derives must not change.

## Options considered

### Option A — expand in the loader, before checking (chosen)
The loader parses the program; if an empty `impl` names a trait declared
`@derive(f)`, it adds one wrapper function per such `impl` that calls `f`
with a `TypeInfo` literal, checks that program, runs the wrappers in the
interpreter, splices each text into its `impl` as a file
`gen:impl:T.Trait` in the `impl`'s module, and parses again.
- **Pros:** every command sees the generated methods with no special case;
  diagnostics, effects, `ail api`, traces and tests work on them; programs
  without `@derive` or a prelude generator trait pay one token scan.
- **Cons:** a program with derives is parsed twice and checked once more;
  a generator can't use code that another derive generates; a problem in the
  program's own code doesn't stop generation, so errors there are reported by
  the full check after it.

### Option B — run generators inside the checker
- **Pros:** one parse.
- **Cons:** the checker would have to run the interpreter on a half-checked
  program and splice items into the tables it is building; a large change to
  the shared `check.rs`.

### Option C — a separate `ail expand` step that writes files
- **Pros:** simplest bootstrap.
- **Cons:** hidden build step and stale files; contradicts ADR 0228.

## Decision

1. The loader expands derives as in Option A (`bootstrap/src/derive.rs`).
   A generator's effect set must be compile-time-safe (`ct.derive.3`, the
   diagnostic names the chain); its failure or trap is `ct.derive.4`; a
   generator that isn't declared is `ct.derive.1`; a target that isn't a
   struct or enum of the program is `ct.derive.2`.
2. `TypeInfo`'s error flag is `is_error`: `error` is a keyword
   (`lex.kw.1`), so ADR 0229's field name can't be declared.
3. `TypeInfo`, `TypeKind`, `FieldInfo`, `VariantInfo` and
   `DeriveError(message str)` are an optional prelude module, read when the
   program writes `@derive`, names one of them, or uses `std.json` (as
   `std.json` itself is, ADR 0162).
4. Step and memory limits (`ct.eval.3`) aren't counted yet; the run-time
   limits apply.

## Consequences

- `ToJson` and `FromJson` become prelude derives (ADR 0308).
- Generation errors stop the load, so the full check of the rest of the
  program waits for them to be fixed: one more round for an agent in that
  case only.
- The wrapper names (`derive_job_N`) never reach a diagnostic: effect
  chains drop them.

## Rejected alternatives

- **In the checker (B):** too invasive for tonight's shared files, and the
  query compiler will replace this anyway.
- **A separate expand step (C):** hidden, stale code.
