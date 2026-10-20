# Native compile speed (lane 182)

Where the time goes in `ail build FILE`, measured on Linux x86-64 (4 cores,
16 GB), and what the runtime cache should do about it. Numbers are the
median of three runs for `ail build tools/perfbench/programs/collatz.ail`
and single runs of each stage, so they are good to about ±10%.

## Baseline

| Stage | Wall time | Share | Where it runs |
|---|---|---|---|
| `ail check FILE` (the driver's pre-check) | 16 ms | 1% | Rust |
| Front end: load and check `compiler/` (34 files, 13.7k lines) | 470 ms | 42% | Rust, inside `ail run compiler` |
| Parse the runtime (`compiler/rt/native/`, 800 lines) | 200 ms | 18% | AIL on the interpreter |
| Lower the runtime to MIR (about 65 functions, inline, fold) | 330 ms | 29% | AIL on the interpreter |
| Lower the program, generate x86-64 code, hex-encode, write | 110 ms | 10% | AIL on the interpreter |
| **Total** | **1.13 s** | | |

The stage rows are cumulative timings of `ail run compiler parse` and
`ail run compiler mir` with and without the runtime, and of
`ail run compiler build`, subtracted from one another.

Fixed cost per build is therefore about 0.47 s of Rust front end plus 0.2 s
of runtime parsing and 0.33 s of runtime lowering: about 1.0 s of the 1.13 s
does not depend on the user's program. The program-dependent part is about
0.1 s.

Two other observations:

- `ail check compiler/main.ail` prints 72 `expr.match.3` warnings on every
  run. The driver filters them out, but the checker still computes them.
  Silencing them for the compiler's own sources would not change the
  front-end time by much; it is listed for completeness.
- The Rust front end's cost is roughly linear in source size (35 ms for
  `mir/lower.ail` alone). Caching the front end is the only way to cut it.

## What the runtime cache should cache

The program-independent work is the runtime's parse, lowering and code
generation. Lowering a runtime function does not depend on the user's
program, but the current pipeline lowers the runtime and the program into
one `Module`, runs `inline_module` over the whole thing, and only then
generates code. So the cache can't be a plain copy of the final bytes.

Options, in order of expected gain for the work:

1. **Lazy lowering of runtime functions.** Lower a runtime function only
   when the program (or another reached runtime function) calls it. Most
   programs use a few of the 65. This removes most of the 330 ms with no
   new file format. It is a change to `compiler/mir/lower.ail` (lane 120's
   file), so it needs lane 120's agreement.
2. **Per-function machine code with fixups.** Generate each runtime function
   once, keyed by a hash of `compiler/rt/native/` and the target, and store
   the bytes with symbolic relocations for `rt.*` calls and string
   constants. The link step patches them. This removes the parse, the
   lowering and most of the code generation (about 0.5 s) but needs the
   x86-64 and a64 generators to emit per function, which is lane 120's code.
3. **Cache the parsed runtime as JSON.** `ail api --ast --program` and
   `ast.Program.from_json` already exist. This saves only the 200 ms parse,
   and the JSON parse on the interpreter is itself unmeasured. Not
   recommended on its own.

Option 1 first, then option 2 when lane 120's per-function emission lands.
The cache key for option 2 is the SHA-256 of `compiler/rt/native/*.ail`
(`compiler/obj/sha256.ail` already exists), the target name, and the
compiler version string.

## Not in this lane

The 0.47 s front end belongs to the Rust bootstrap (`bootstrap/src/`). The
fix is to cache the checked compiler closure per source hash under
`$XDG_CACHE_HOME/ail`, which needs a serialiser for `Program` and
`Resolved`. That changes the bootstrap's data model and needs the designer's
view, so it is flagged here rather than started.

## Rejected alternatives

- **Caching the whole `ail build` output by program hash.** Every program
  is different, so the hit rate is near zero in a build loop.
- **Caching the Rust front end in a `build.rs`-generated file.** The
  compiler source changes during development, and `build.rs` would rerun on
  every edit to `compiler/`.
