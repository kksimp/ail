# 0856. The native compiler runs user derive generators as a native program

- **Status:** Accepted
- **Date:** 2026-10-22
- **Decided by:** AI (delegated, ADR 0076; RR-2 of the Rust retirement plan)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** ct.derive.1-4, ct.gen.1, refl.info.1, [native.subset.1](../spec/23-native.md)
- **Builds on:** [ADR 0850](0850-native-compiler-runs-builtin-derive-generators.md) (the built-in generators), [ADR 0807](0807-native-build-passes-derived-methods.md), [ADR 0808](0808-native-prelude-json-lowered-on-demand.md) (quiet sources)

## Context

ADR 0850 covers the prelude's four generators. A trait the program declares
with `@derive(f)` has a generator that is the program's own code, which only the
Rust interpreter ran (ct.eval.2). Removing the Rust bootstrap needs it native.

## Decision

1. For each `impl` with no methods of a trait with `@derive(f)` (and no
   `AIL_DERIVED` text for it), `compiler/derive/info.ail` builds the
   `TypeInfo(...)` expression of the type, as `bootstrap/src/derive.rs` does.
2. `compiler/main.ail` builds a **generator program**: a `main` calling
   `derive_job_K()` for each job (each `f(TypeInfo(...))?`, written in the trait's
   module), then the program's sources and the prelude as quiet sources
   (ADR 0808), so only what a generator reaches is lowered, and the program may
   call methods that don't exist yet. The program's own root `main` is renamed.
3. The program is built for the host, written to a temporary file, run with
   `proc.run`, and removed. Each generator's text (or `DeriveError` message)
   comes back on its standard output between `@@job K` and `@@end` lines.
4. Each text becomes the source `gen:impl:TYPE.TRAIT`, as in ADR 0850. A
   refusal prints ``error[ct.derive.4]: `Trait` can't be derived for `Type`: MESSAGE``
   at the `impl` and stops the build.
5. The prelude's `derive` module (`TypeInfo` and friends) is kept in the embedded
   prelude, and a source naming `@derive` or `TypeInfo` loads the prelude.

## Rejected alternatives

- **An interpreter for generators in the native compiler.** AIL has no interpreter
  port (Kaleb's earlier decision); a generator is native code like any other.
- **Compiling the whole program and calling the generators in it.** The program's
  code can call methods the generators haven't written yet, so it would not lower.
- **Reading the generator's text through a pipe instead of a file.** `proc.run`
  already captures standard output; no new API is needed.

## Consequences

- Bytes of an executable with a user-derived `impl` equal `ail build`'s
  (`derives_build_without_the_derived_file`, `derive_user`).
- Derived `impl`s of generic types (`impl T for Pair[T]`) still stop at
  `native.subset.1`, as they do for the built-in generators.
- ct.derive.3 (a generator has no effects) stays the checker's job.
