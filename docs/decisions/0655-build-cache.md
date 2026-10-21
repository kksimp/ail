# 0655. A content-addressed cache of finished builds

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** native.build.5, native.build.3
- **Builds on:** [ADR 0393](0393-native-modules.md), [native-compile-speed.md](../design/native-compile-speed.md)
- **Amends:** ADR 0393 (the rejected "cached, precompiled runtime" stays rejected; this caches whole results, not the runtime)

## Context

A cold `ail build examples/hello.ail` took about 1.8 s on this head. Lane 182
found the program-dependent part is about 0.1 s; the rest (the Rust front end
loading the AIL compiler, then the AIL compiler parsing and lowering its
runtime on the interpreter) does not depend on the program. Agents build in a
loop, and many builds repeat an input: `ail test --native` over unchanged
files, `ail build` before `ail run`, a selftest run twice. Lane 182 rejected
caching the whole output because "every program is different"; a loop that
edits the program between builds does miss, but the repeats are free to serve.

## Decision

1. **`ail build` stores each finished executable** under a SHA-256 of its
   inputs (native.build.5): the path as given (traps embed it), the target, every `.ail`
   file of the backend and of the program's directory tree, and the `ail`
   binary's version, size and modification time. A hit writes the stored bytes
   and runs no backend. The key is conservative: any `.ail` file beside the
   program changes it, whether imported or not.
2. **Where.** `$AIL_CACHE_DIR`, else `$XDG_CACHE_HOME/ail/build`, else
   `~/.cache/ail/build`: outside the project, so no new ignore rule. Writes are
   atomic (temporary file, rename). The 64 most recently used entries stay.
3. **`--no-cache` and `AIL_NO_CACHE`** read and write nothing. Failed builds
   are never stored. `ail check` still runs first, so diagnostics are identical.
4. **Determinism.** `bootstrap/tests/build_cache.rs` builds one program cold,
   again (a hit), and with `--no-cache`: all three are the same bytes; a
   changed program or a changed sibling file is a new entry; `--no-cache`
   stores nothing.
5. **The Rust front end gets cheaper too** (separate from the cache, same
   lane): derive expansion ran the whole-program effect inference once per
   generator (84 times for `compiler/`) and scanned the token list once per
   `impl`. It now infers once and indexes the `impl` keywords. Output is
   unchanged. This takes `ail run compiler` from 0.57 s to 0.33 s, a saving on every
   cold build, `ail run compiler` and selftest invocation.

## Measurements

Median of five runs, Linux x86-64 (4 cores); bytes identical in every row.

| Program | Cold before | Cold after (`--no-cache`) | Warm (hit) |
|---|---|---|---|
| hello | 1.818 s | 1.470 s | 0.043 s |
| perfbench `collatz` | 1.744 s | 1.399 s | 0.012 s |
| perfbench `nbody` | 2.029 s | 1.665 s | 0.010 s |

The warm target (under 0.3 s) is met for an unchanged program. A program that
changed is a cold build, about 0.35 s faster than before. Profile of
`ail run compiler` (callgrind): 2.25 G instructions before, 1.35 G after;
the remaining front-end cost is parsing the compiler twice (the second time
with derived code) and one full check.

## Rejected alternatives

- **A per-module MIR cache.** The program's own modules cost about 0.1 s; the
  fixed cost is the runtime and the backend's front end, which lower into one
  module before inlining (ADR 0393). Caching MIR needs a MIR reader written in
  AIL, run on the interpreter, for a saving lane 182 could not measure.
- **A precompiled runtime.** Still rejected (ADR 0393): a cache format tied to
  code generators that change daily. This cache is keyed by the generators'
  own sources, so it never goes stale, and it stores nothing partial.
- **Parsed runtime as JSON.** Measured: a build that read the Rust parser's AST
  of the runtime took 2.2 s against 1.8 s, because `from_json` on the
  interpreter is slower than parsing the 800 lines.
- **A cache inside the project (`.ail-build/`).** Needs an ignore rule in
  every repository and is lost on a clean checkout; the user cache is shared.
- **Keying by modification time.** Faster, but a touched file or a restored
  backup would give a stale hit; content hashes of 14,000 lines cost under 1 ms.
