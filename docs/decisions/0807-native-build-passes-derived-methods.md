# 0807. `ail build` passes derived methods to the native backend

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; approved by the integrator for the native self-compile lanes)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), ct.gen.1, trait.self.1
- **Amended by:** [ADR 0850](0850-native-compiler-runs-builtin-derive-generators.md) (the native compiler runs the built-in generators)
- **Builds on:** [ADR 0536](0536-native-generics-by-monomorphisation.md) (`impl` methods in native code), ADRs 0227-0229 and 0306-0309 (derive generators), ADR 0774 (no implicit derive)

## Context

An `impl ToJson for T`, `impl FromJson for T` or `impl Default for T`
without methods is filled in by a derive generator that the bootstrap
runs in the interpreter before checking (`bootstrap/src/derive.rs`). The
native backend parses the program's files itself, so it saw the empty
`impl` and nothing else: `P.default()` was "not in the native subset", and
the compiler's own `ast.Program.to_json`/`from_json` blocked the
self-compile.

## Decision

1. **`ail build` (and `ail api --mir`) write the generated text to a file**
   named by the environment variable `AIL_DERIVED`: for each deriving
   `impl`, a line `@@derive MODULE TYPE.TRAIT` (`-` for the root module)
   and the text the generator wrote, as `ail api --expanded` shows it.
2. **The native backend adds each text as a source of its module** named
   `gen:impl:TYPE.TRAIT`, with each `fn m` read as `fn TYPE.m`; lines stay
   where the generator put them, so a problem in generated code names the
   same file and line as the interpreter's (ct.gen.3). The methods then
   lower like written `impl` methods (ADR 0536).
3. **`Self` in a method's signature** is the method's type (trait.self.1),
   as generated methods write it.

## Rejected alternatives

- **Running the generators in the native compiler.** That needs
  compile-time evaluation in the AIL front end, which belongs with the
  self-hosted toolchain; the bootstrap already runs them and checks the
  result.
- **Passing the whole expanded program as JSON.** The native backend
  parses source today and keeps file paths and positions from it; one
  side file of generated text changes nothing else.
- **Splicing the text into the original `impl`.** It would move every
  later line of the user's file, so positions in problems and traps would
  no longer match the source.

## Consequences

- `tests/native/run/derive_default.ail` (derived `Default`) matches the
  interpreter on x86-64 and aarch64.
- Derived `ToJson`/`FromJson` methods now reach the backend; they compile
  once the prelude's `json` module is native (ADR 0808).
