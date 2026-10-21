# 0431. `extern c fn` declares a C function, loaded dynamically on first call

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 14 — FFI
- **Spec rules:** [ffi.decl.2](../spec/14-ffi.md), [ffi.load.1–3](../spec/14-ffi.md)
- **Builds:** [ADR 0103](0103-c-header-import.md) (the declaration form), [ADR 0105](0105-libc-only-with-c.md)

## Context

ADR 0103 chose `extern c fn name(params) Result` for one C function and a
header import for many. The header parser is a large piece of work with its
own C-subset grammar; the declaration form alone is enough to reach libc,
libm, SQLite and any other C library, and it is what the header importer would
generate. The interpreter has no linker, so it needs a way to find the symbol
at run time.

## Decision

1. `extern c fn name(params) Result` and `extern c "lib" fn name(params)
   Result` declare a C function with no body, at the top level. `c` names the
   language so `extern` can later take others without a new keyword.
2. The interpreter resolves the symbol on the first call with `dlopen` and
   `dlsym` (Linux, macOS) and caches it. A declaration without a library looks
   in the running program, which has the C library. A name is tried with and
   without `lib`, with the platform's suffixes (spec `ffi.load.1`), because a
   run-time machine has `libm.so.6` where a development machine has `libm.so`.
3. Failures are run-time traps (`ffi.load.1`, `ffi.load.2`), not
   check-time errors: `ail check` and the self-hosted checker never touch the
   file system or the loader, so checking stays deterministic.
4. Windows reports `ffi.load.3` (not supported yet) rather than guessing;
   it is stubbed with the spec citation (ADR 0004: Windows doesn't gate).
5. `use c "header.h"` (`ffi.decl.1`) stays unbuilt.

## Options considered

### Option A — declaration form only, dynamic loading (chosen)
- **Pros:** small, testable on both gate platforms; the declaration is what a
  header importer would emit; no new crate.
- **Cons:** the program writes each signature by hand, and a wrong signature
  is undefined behaviour (spec `ffi.call.2`).

### Option B — the header importer first
- **Cons:** weeks of work before the first call works; ADR 0103 itself leaves
  the C subset open.

### Option C — link C libraries at build time
- **Cons:** there is no `ail build` and no linker yet; dynamic loading is
  also what the native backend does for libc (ADR 0105, `native-abi.md`).

### Option D — Rust's spelling (`extern "C" fn f(a: f64) -> f64`) or an `extern c` block
Measured with `tools/tokbench` on four declarations (hypot, abs, strlen, atoi),
mean of five tokenizers:

| Spelling | Mean tokens |
|---|---|
| `extern c "libm" fn hypot(a float, b float) float` (chosen) | 54.4 |
| `extern "C" fn hypot(a: f64) -> f64` with Rust types | 77.8 |
| one `extern c "lib"` block of indented `fn` lines | 54.6 |

The Rust-like form costs 43% more (ADR 0011: resemblance to Rust is not a
goal). The block form ties the one-line form (under 2%), so the one-line form
wins on locality: a declaration is understood without its block header.

## Consequences
- `FnDecl` gains `ext`; the syntax tree JSON, `compiler/syntax/` and
  `compiler/ast/` carry it.
- The loader code is toolchain code (`bootstrap/src/interp/ffi.rs`); the code a
  native build links for these calls is Runtime Code under the licence
  exception (ADR 0510) and is described in `docs/design/native-abi.md`.

## Rejected alternatives

- The header importer first (option B): rejected because it would take weeks before the first call worked, and ADR 0103 leaves the C subset open.
- Linking C libraries at build time (option C): rejected because there was no `ail build` or linker yet, and dynamic loading is also what the native backend does for libc (ADR 0105).
- Rust's spelling `extern "C" fn f(a: f64) -> f64` (option D): costs 77.8 tokens against 54.4 for the chosen form, 43% more. ADR 0011 says resemblance to Rust is not a goal.
- A block form `extern c "lib"` of indented `fn` lines: ties with the one-line form (54.6 against 54.4, under 2%), and loses on locality, since a declaration is understood without its block header.
