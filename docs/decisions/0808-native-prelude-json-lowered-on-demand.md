# 0808. The prelude's JSON and `Default` modules in native builds, lowered on demand

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; approved by the integrator for the native self-compile lanes)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), json.prelude.1
- **Builds on:** [ADR 0807](0807-native-build-passes-derived-methods.md) (derived methods), [ADR 0536](0536-native-generics-by-monomorphisation.md) (generics), ADR 0393 (no cached runtime)

## Context

`Json`, `parse_json`, the `ToJson`/`FromJson` impls on builtin types and
`Default` live in the bootstrap's prelude (`bootstrap/src/prelude.ail`,
modules `json`, `jsonderive` and `default`). The native backend never saw
them, so any program naming `Json`, and every derived `to_json`, failed
the build. Much of the module (the parser's error values, `match` on
`char`) is not native yet, and a program that only builds a `Json` should
not fail on the parser it never calls.

## Decision

1. **`ail build` passes the prelude** as a last block `@@prelude` of the
   `AIL_DERIVED` file when a file of the program contains `json`, `Json` or
   `Default`. The backend keeps the three modules' lines and blanks the
   rest, so positions are the prelude's own, and adds the text as a source
   `<prelude>` of the root module.
2. **Its names are visible from every module.** A type, enum or function
   name not found in the current module falls back to the prelude's.
3. **Lowered on demand.** Problems found while declaring the prelude are
   held, not reported. A function's body is lowered only when lowered code
   calls it, until no new function is reached (the same fixpoint as
   generic instances). A call to a function whose declaration had a
   problem reports that held problem, at the prelude's line, and fails the
   build.

## Rejected alternatives

- **Lowering the whole module.** Every JSON program would fail until all of
  the parser is native, and every build would pay for code it never runs.
- **A native copy of the JSON module in `compiler/rt/native`.** Two copies
  to keep in step; the runtime is for primitives, not library code
  (ADR 0393).
- **Reading `bootstrap/src/prelude.ail` from disk.** The installed `ail`
  carries the prelude in its binary; passing that text keeps one source.
- **Choosing by the bootstrap's token-level rule** (`json.prelude.1`).
  The backend lowers on demand anyway, so a coarse text test costs only a
  parse when it is wrong.

## Consequences

- `tests/native/run/json_prelude.ail` (building `Json`, `kind`, `get` and
  a derived `to_json`) matches the interpreter on x86-64 and aarch64.
- `parse_json` and `to_text` report the parser's error values and `match`
  on `char` until those are native (#392, #399, #397).
