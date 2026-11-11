# Native compile speed (lane 182)

> **History.** Measured in October 2026 on the Rust front end, which is deleted ([ADR 0904](../decisions/0904-rust-bootstrap-deleted.md)). The numbers are not current; see [native-performance.md](native-performance.md) and [benchmarks.md](../benchmarks.md).

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

Options, after measuring the first one:

1. **Lazy lowering of runtime functions: tried, no measurable gain.** The
   worklist lowered a runtime function only when a lowered function calls
   it, plus `rt.init`, `rt.exit`, and `rt.trap` when the program has checks.
   It reached 35 of the 50 runtime functions for collatz, because the
   reachable set is large: `rt.trap` and the output routines pull in the
   float printing and bignum code. Full `ail build collatz`, five runs each,
   median 1178 ms eager against 1172 ms lazy. The output was byte-identical
   to the eager build. The change was reverted, not merged, because it adds
   a lowering pass to lane 120's file for no gain.
2. **Per-function machine code with fixups.** Generate each runtime function
   once, keyed by a hash of `compiler/rt/native/*.ail` and the target, and
   store the bytes with symbolic relocations for `rt.*` calls and string
   constants. The link step patches them. This removes the parse, the
   lowering and most of the code generation, but needs per-function emission
   from the x86-64 and a64 generators. Lane 120 has said it is not planning
   that, so it is not started.
3. **Cache the lowered runtime MIR** keyed by the same hash, read back by a
   MIR text reader. Saves the lowering only, not the 200 ms parse, and the
   reader is a new parser on the interpreter. Not started.
4. **Cache the parsed runtime as JSON.** `ail api --ast --program` and
   `ast.Program.from_json` already exist. This saves only the 200 ms parse,
   and the JSON parse on the interpreter is unmeasured. Not started.

The measured lazy result shows the runtime's lowering is a smaller share than
the table suggests. Before choosing between options 2 to 4, measure the
parse and the post-lowering passes (`inline_module`, `verify`) on the full
module separately.

## Not in this lane

The 0.47 s front end belongs to the Rust bootstrap (`77592931:bootstrap/src/`). The
fix is to cache the checked compiler closure per source hash under
`$XDG_CACHE_HOME/ail`, which needs a serialiser for `Program` and
`Resolved`. That changes the bootstrap's data model and needs the designer's
view, so it is flagged here rather than started.

## After lane 200 (ADR 0655)

Profiling `ail run compiler` showed 77% of the Rust front end in derive
expansion (84 generators, each re-running the whole-program effect inference,
plus a token scan per `impl`). Fixing those took the front end from 0.57 s to
0.33 s and a cold hello from 1.82 s to 1.47 s. A content-addressed cache of
finished executables (native.build.5) makes an unchanged rebuild 0.01-0.04 s.
The "caching the whole output" rejection below stands for a build loop that
edits the program every time: that case is the cold build. The remaining cold
cost is the interpreted backend parsing and lowering the runtime (about 1.1 s).

### Hot paths (ADR 0656)

`AIL_BYTECODE_STATS=3 ail run compiler build FILE RT TARGET` lists the AIL
functions by the bytecode instructions they ran. It found `mir.dominators` at
43% of a hello build, the runtime's first three files parsed twice, and the
lexer's symbol match; fixing them took the AIL side of a hello build from
1.45 s to 0.85 s with identical output, and a cold `ail build` to about 1.0 s.
What is left is spread thin: runtime lex and parse 0.29 s, lowering 0.14 s,
code generation 0.12 s.

## Rejected alternatives

- **Caching the whole `ail build` output by program hash.** Every program
  is different, so the hit rate is near zero in a build loop.
- **Caching the Rust front end in a `build.rs`-generated file.** The
  compiler source changes during development, and `build.rs` would rerun on
  every edit to `compiler/`.
