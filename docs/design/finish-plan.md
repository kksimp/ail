# Plan: finish AIL (batch 2, 2026-10-20)

Non-normative. The designer asked (2026-10-20 16:53 UTC) for one plan that
finishes AIL the fastest, most efficient way: the most token-efficient
language, beating C and Python where it matters, with AI-first features the
AI itself wants. Phases: **build out, test, fix, test again, drag race**
against C and Python. Lane budget: up to 2 large, 10 medium and 15 small lanes
lanes at once. The integrator is the Fable thread; this document is its
plan, and the briefs are in the project files (`lanes/120-…`).

## 1. Where we are (main at ae1ca19)

- Front end in AIL: lexer, parser (byte-identical to Rust on 770 files,
  29% fewer tokens, 8x slower under the walker), checker declarations
  merged and the rest (522/522 programs, every message identical) in
  PR #102. Still Rust: interpreter and bytecode tier, runtime, standard
  library, the `ail` command, module loading and derive expansion.
- Speed: bytecode tier complete (ADR 0359). loops 0.52x of CPython,
  parsum 0.22x, interp 0.81x; `std.tensor` CPU slice 26x Python. No
  native code yet, so nothing beats C.
- Tokens: agent corpus 5.7% under Python; `compiler/syntax` 29% under
  the Rust equivalent.
- Gaps (from the status answer of 2026-10-20 15:56 UTC): no native
  binaries; client-only networking without headers or sockets; text-only
  files, processes and bodies; blocking `proc.run`; no FFI; no regex,
  dates, compression or hashing modules; text-cell GUI only; function
  values, match guards, range patterns, bit operations, named arguments,
  `select` on channels; OQ-114 to OQ-117.

## 2. What "finish" means here

1. **Beat C where it matters:** a native backend **written in AIL**
   (ADR 0022, ADR 0010: no LLVM, own encoders and linker) run by the
   bootstrap, for x86-64-v3 and ARMv8.2 in lockstep (ADR 0008). First
   target: the pure subset (integers, floats, loops, functions, lists of
   numbers) as `ail build`, drag-raced against `clang -O2` and `gcc -O2`
   on the perfbench programs. Allocation-heavy code follows the ideas in
   [performance-beyond-c.md](performance-beyond-c.md) later.
2. **Beat Python everywhere:** the bytecode tier already does on loops;
   the native backend does the rest. Token-wise, every language change in
   this batch is measured with tokbench and must not lose.
3. **Close the gaps** above as standard modules and language features,
   each with spec rules, ADRs, tests, examples and cards.
4. **AI-first features the AI wants** (lane 121, delegated ADRs): a
   semantic diff of two program versions, structured edits by symbol,
   record and replay of effectful runs, snapshot updating for tests, and
   token-budgeted output on every command.
5. **Test, fix, test, race:** fuzz and differential tests for every new
   module and for native vs interpreter; a conformance pass; then the
   drag-race harness publishes AIL vs C vs Python with before/after tables.

## 3. Rounds

| Round | What | Who |
|---|---|---|
| 1 Build out | Every lane lands its slices as PRs against the batch branch; the integrator merges continuously (Linux gate per merge). | all |
| 2 Test | Fuzz, differential and conformance lanes run over everything merged; each lane adds property tests for its own module. | 132, 150, 151, 146 |
| 3 Fix | Lanes fix what round 2 found in their area; the integrator files the rest as cases. | all |
| 4 Test again | Round 2 repeated; macOS gate on the whole branch; final PR to `main`. | integrator |
| 5 Drag race | perfbench vs C and Python on both platforms; report with before/after tokens and speed. | 132, 143, 149 |

Rounds overlap: a lane that finishes its build-out moves to its own
tests and fixes without waiting.

## 4. Lanes

Numbers continue the daytime lanes. ADR blocks are fixed per lane.

| Lane | Size | Title | ADRs |
|---|---|---|---|
| 120 | large | Native backend in AIL: MIR, x86-64 and ARM64 encoders, ELF and Mach-O, `ail build` for the pure subset | 0384–0393 |
| 121 | large | AI-first features: `ail diff`, `ail edit`, record and replay, snapshot updating, token budgets | 0394–0403 |
| 122 | medium | Native runtime and ABI: value layout, strings, lists, syscalls, entry point, with lane 120 | 0404–0411 |
| 123 | medium | Binary data: `bytes`, binary files, bodies and process pipes, base64 and hex | 0412–0417 |
| 124 | medium | Networking: HTTP server, headers and methods, raw TCP sockets, `net.serve` capability | 0418–0423 |
| 125 | medium | Processes and channels: streaming `proc.spawn`, kill, `select` | 0424–0429 |
| 126 | medium | FFI: `extern` C functions, `unsafe` blocks, raw pointers, dynamic loading in the interpreter | 0430–0437 |
| 127 | medium | Regex: a linear-time engine as `std.regex` | 0438–0441 |
| 128 | medium | Hashing and compression: `std.hash` (SHA-2, HMAC, CRC32) and `std.compress` (deflate, gzip, zip) | 0442–0446 |
| 129 | medium | Dates and time: calendar, ISO 8601, durations, zones | 0447–0451 |
| 130 | medium | Language gaps A: function values, named arguments, match guards and range patterns | 0452–0459 |
| 132 | medium | Test, fuzz and drag race: differential native vs interpreter, module fuzzers, `tools/perfbench` vs C | 0460–0464 |
| 140 | small | Bit operations and shifts | 0465–0467 |
| 141 | small | OQ-114 to OQ-117: typed `none` locals, `else none`, shadowing, `if` as operand | 0468–0470 |
| 142 | small | `--watch` on the bytecode tier | 0471–0473 |
| 143 | small | Drag-race programs: AIL, C and Python twins of the benchmark set | 0474–0476 |
| 144 | small | Agent corpus round 3: ten more agent programs | 0477–0479 |
| 145 | small | Examples, quick reference and cards for the new modules | 0480–0482 |
| 146 | small | Spec rule-id and `ail explain` audit for every new behaviour | 0483–0485 |
| 147 | small | `ail fix` rules for the new diagnostics | 0486–0488 |
| 148 | small | Editor: find and replace, go to definition, problems from the new checks | 0489–0491 |
| 149 | small | Bench automation: one script for tokbench and perfbench before/after tables | 0492–0494 |
| 150 | small | Property and vector tests for bytes, regex, hash, compress and dates | 0495–0497 |
| 151 | small | Conformance: every spec example runs; new chapters covered | 0498–0500 |
| 152 | small | Diagnostics quality pass over the corpus | 0501–0503 |
| 153 | small | Windows awareness of the new modules (paths, pipes, sockets) | 0504–0506 |
| 154 | small | `std.tensor` CPU gaps: `conv2d`, activations, `f32`, indexing, broadcasting | 0507–0509 |

Lane 114's PR #102 (the checker in AIL) merges to `main` first through the
daytime gate and is then in the batch branch.

## 5. Assumptions recorded

- The native backend is written in AIL under `compiler/` (ADR 0022), run
  by the bootstrap interpreter and bytecode tier; a Rust backend would be
  thrown away and contradict a designer decision.
- Lanes PR against the batch branch `claude/project-thread-ipafhh`, not
  `main`, so one macOS run gates a round rather than every slice
  (ADR 0085 still holds for the final PR).
- The batch runs until it is done (designer's decision, 2026-10-20
  17:02 UTC): no deadline. The integrator replans and refills lanes as
  they finish, merges the batch branch to `main` at the end of each day
  after the macOS gate, and posts a daily report.
