# 0850. The native compiler runs the prelude's derive generators itself

- **Status:** Accepted
- **Date:** 2026-10-22
- **Decided by:** AI (delegated, ADR 0076; approved by the integrator for the self-host lanes)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), ct.derive.2, ct.gen.1, json.derive.1-2
- **Amends:** [ADR 0807](0807-native-build-passes-derived-methods.md) (the Rust-written `AIL_DERIVED` file), builds on [ADR 0808](0808-native-prelude-json-lowered-on-demand.md)

## Context

A natively built compiler is run without the file `ail build` writes for it,
so `ast.Program.to_json` and `from_json` of the compiler's own sources
stopped stage 2 of the self-host (ADR 0807 point 1). ADR 0807 rejected running
generators in the native compiler because that needs compile-time evaluation.
That holds for a generator the user writes. The four the prelude provides
(`derive_to_json`, `derive_from_json`, `derive_default`, `derive_from_args`)
are ordinary AIL functions of a `TypeInfo`, so the compiler can contain and call them.

## Decision

1. `compiler/derive/gens.ail` holds the prelude's `TypeInfo` types and the four
   generators with their helpers, copied from `bootstrap/src/prelude.ail` by
   `tools/native/gen_derive.ail` (then `ail fmt`); a native test fails when it is stale.
2. `compiler/derive/info.ail` finds each `impl` of `ToJson`, `FromJson`, `Default` or
   `FromArgs` with no methods, builds the `TypeInfo` of its type from the syntax tree
   (as `bootstrap/src/derive.rs` does: field types as written less the module's own
   prefix, `pub` fields, `##` docs) and runs the generator.
3. `compiler/main.ail` adds each text as the source `gen:impl:TYPE.TRAIT`, as
   ADR 0807 does for the file's texts. A text `AIL_DERIVED` supplies wins. An `impl` of a
   trait the program declares itself is left alone. A generator's refusal prints
   `ct.derive.4` and stops the build.
4. User-written `@derive` generators still need the Rust front end and its file.

## Rejected alternatives

- **Making the stage harness export `AIL_DERIVED` for stage 2.** It keeps the
  self-built compiler dependent on the Rust one for every program with a derive.
- **A compile-time evaluator in the compiler.** Far larger than the four
  generators need; user generators can wait for it.
- **Porting the generators to hand-written `compiler/` code.** Two copies would drift;
  the copy is generated and tested instead.

## Consequences

- Executables built without the file are byte-identical to `ail build`'s for the
  derive fixtures (`derives_build_without_the_derived_file`).
- Cross-module field types written through an alias keep the alias in the text
  (the bootstrap writes the module's full path); the generated code reads the same inside the module.
- Changing a generator in `prelude.ail` needs `gen_derive.ail` rerun.
