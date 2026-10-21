# Decisions digest, 2026-10-21

The delegated decisions (ADR 0076) from the night run, ADRs 0618 to 0743, as they
stand on the integration branch `claude/project-thread-ipafhh`. Each line was checked
against the ADR text. Where an ADR gives no criterion or measurement, the line says
"not stated in the ADR" rather than guessing. ADRs from lanes not yet on the
integration branch (for example 0720s and 0730s from open PRs) are not listed here.

Criteria, in ADR 0076 order: (1) tokens, (2) AI understanding, (3) human readability,
(4) speed.

## Bytecode tier

- [0618](../decisions/0618-bytecode-typed-fast-paths.md) Typed fast paths: floats and in-bounds list writes skip the generic path. Criterion: not stated in the ADR. Measured: the tier's geometric mean on the perfbench programs falls from 1.85x to 1.33x of CPython.
- [0619](../decisions/0619-bytecode-calls-loops-and-fusion.md) Lean plain calls and fused loop instructions. Criterion: not stated. Measured: 2 to 4% fewer instructions on the benchmarks.

## Native backend and compiler

- [0622](../decisions/0622-loop-starts-steps-and-index-spans.md) Loops with computed starts and steps, and index arithmetic bounded by a constant. Criterion: not stated. Measured: nbody wall time 0.176 s to 0.117 s, 2.08x to 1.37x of gcc -O2.
- [0623](../decisions/0623-branch-and-codegen-peepholes.md) Branch and codegen peepholes (flipped `not` branches, bit operations into the result register, direct spill moves). Criterion: not stated. Measured: no perfbench program got faster.
- [0624](../decisions/0624-loop-weighted-spill-choice.md) Register spill choice weighted by loop depth. Criterion: not stated. Measured: not stated in the ADR.
- [0625](../decisions/0625-address-folding.md) Address folding for `y + k` indexes; callee-saved registers saved where calls start. Criterion: not stated. Cost: `ail build` hello about 5% slower (a dominator pass per function).
- [0656](../decisions/0656-compiler-hot-paths.md) Cold-build hot paths in the AIL compiler, found by a per-function profile; adds `AIL_BYTECODE_STATS=3`. Criterion: not stated (the measured effect is build time). Measured: `mir.dominators` ran 43% of instructions in a hello build; removing it cut instructions by 7% but slowed the build.
- [0700](../decisions/0700-native-exit-sleep-range-traps.md) Native `io.exit` and `time.sleep_ms` range checks as traps (kinds 9 and 10). Criterion: not stated (behaviour parity with the interpreter).
- [0701](../decisions/0701-native-error-context.md) Native `?` carries context messages, printed as `context:` lines. Criterion: not stated (parity with the interpreter). Uses a new trace-list element, not a new global slot.

## Build and tools

- [0655](../decisions/0655-build-cache.md) Content-addressed cache of finished executables, on by default; `--no-cache` and `AIL_NO_CACHE` bypass it. Criterion: not stated (the measured effect is build time). Measured: cold build about 0.35 s faster.
- [0679](../decisions/0679-native-test-runner.md) `ail test --native`: a differential runner that compares native and interpreter verdicts. Criterion: not stated.
- [0693](../decisions/0693-test-captures-stderr.md) `ail test` captures a test's standard error and shows it under a failing test. Criterion: not stated.
- [0695](../decisions/0695-fix-bitwise-not.md) `ail fix` rewrites `~name` to `name.bit_not()`; other operands get help only. Criterion: not stated (the fix rule forbids guessed values; see 0648).
- [0698](../decisions/0698-card-spec-audit-for-std.md) Audit: every std function must be named in `docs/spec/` (check 5 of `tools/rules-audit.sh`). Criterion: AI understanding. Measured: language card 7,044 tokens against the 8,000 budget.
- [0703](../decisions/0703-card-middle-trim-level.md) Middle trim level 4 for `ail api --card` (amends 0311). Criterion: tokens. Measured: garlic card 3,026 to 5,750 tokens at 8,000 budget; compiler 2,639 to 4,756.
- [0711](../decisions/0711-tool-driving-agent-programs.md) Agent programs that drive AIL's own tools; the twin calls the same `ail` binary. Criterion: not stated (measurement method). The trimmed twins are the reported numbers.

## Standard library

- [0626](../decisions/0626-std-csv.md) `std.csv` (RFC 4180 parse, records, write). Criterion: tokens. Measured: 236.4 to 69.6 tokens (−71%), five tokenizers.
- [0627](../decisions/0627-std-args.md) `std.args`: flags, options, positionals, generated help. Criterion: tokens. Measured: 240.0 to 113.0 tokens (−53%) on the argument-parsing program.
- [0628](../decisions/0628-std-log.md) `std.log`: four levels to standard error, `LOG_LEVEL` picks the threshold. Criterion: not stated.
- [0629](../decisions/0629-list-chunks-windows-counts-str-center.md) List `chunks`, `windows`, `counts`, `flatten`; string `center`, `words`, `trim_start`, `trim_end`, `count`, `rfind`. Criterion: tokens (estimated in the ADR, not measured).
- [0637](../decisions/0637-http-persistent-connections-and-timeouts.md) HTTP/1.1 client connection pool (4 idle per origin, 32 total, 30 s), chunked requests, timeouts. Criterion: not stated. Opt-out: a program can send `Connection: close`.
- [0638](../decisions/0638-cookie-jar.md) Cookie jar `std.net.jar` with RFC 6265 rules. Criterion: not stated.
- [0639](../decisions/0639-http-server-keep-alive.md) HTTP server keeps connections alive for HTTP/1.1 requests. Criterion: not stated.
- [0640](../decisions/0640-samesite-and-graceful-shutdown.md) SameSite in the jar (lax by default; `none` needs `Secure`), and graceful server shutdown. Criterion: not stated.
- [0667](../decisions/0667-regex-group-all.md) `regex.group_all(re, text, group)`. Criterion: tokens. Measured: `regex_codes` 127.0 to 106.2 tokens (−16%); gap to Python +35% to +13%.
- [0716](../decisions/0716-streaming-response-bodies.md) Streaming response bodies: `net.open` and `net.Body`. Criterion: not stated.
- [0717](../decisions/0717-transparent-content-encoding.md) `std.net` asks for gzip or deflate and decodes it. Criterion: not stated. Opt-out: a program that sets `Accept-Encoding` keeps the raw bytes.

## Docs, spec and tests

- [0646](../decisions/0646-mistakes-corpus-round-two.md) Mistakes corpus round two: 96 new programs with `.rules`. Criterion: not stated.
- [0647](../decisions/0647-round-two-diagnostics-help-and-anchors.md) Help texts and fix anchors for round-two mistakes; three rule changes. Criterion: not stated. Rule IDs are unchanged except the three named in the ADR.
- [0648](../decisions/0648-round-two-fix-rules-and-the-corpus-target.md) Fix rules for round-two mistakes; sets the 60% corpus target (`src/fixable.txt` now 48 rules). Criterion: not stated.
- [0670](../decisions/0670-explain-examples-from-tagged-blocks.md) `ail explain` examples count worked examples; tagged blocks fill the gaps. Criterion: tokens (criterion 1, per the ADR).
- [0673](../decisions/0673-definitional-rule-marker.md) Language-only spec rules carry a `[definitional]` marker and need no citation. Criterion: not stated (audit accuracy).
- [0675](../decisions/0675-round-4-syntax-candidates.md) Round-4 syntax candidates: no new syntax; chunked `par` already works. Criterion: tokens (AGENTS.md treats under about 2% as a tie).

## Decisions the designer may want to revisit

These touch syntax, the card budget, defaults an agent sees, or runtime behaviour a user cannot easily turn off (ADR 0076 escalation criteria 1 to 3).

- [0675](../decisions/0675-round-4-syntax-candidates.md): no new syntax for `par` over chunks. Reason: a syntax choice; the designer may want a dedicated form.
- [0703](../decisions/0703-card-middle-trim-level.md): a new trim level changes what `ail api --card` gives an agent at each budget. Reason: the card budget and the defaults an agent sees; amends ADR 0311.
- [0629](../decisions/0629-list-chunks-windows-counts-str-center.md): eight new list and string methods appear on the card. Reason: new names an agent will see by default.
- [0655](../decisions/0655-build-cache.md): the build cache is on by default. Reason: runtime default; opt-out exists (`--no-cache`, `AIL_NO_CACHE`), but the default is the one most users get.
- [0717](../decisions/0717-transparent-content-encoding.md): gzip and deflate are requested and decoded by default. Reason: changes what a program sees by default; opt-out is setting `Accept-Encoding`.
- [0637](../decisions/0637-http-persistent-connections-and-timeouts.md) and [0639](../decisions/0639-http-server-keep-alive.md): connection pooling and keep-alive are on by default. Reason: runtime behaviour; the ADR gives no global switch.
- [0700](../decisions/0700-native-exit-sleep-range-traps.md) and [0701](../decisions/0701-native-error-context.md): native range traps and error context are always on. Reason: runtime behaviour with no opt-out in the ADR.
- [0618](../decisions/0618-bytecode-typed-fast-paths.md) and [0619](../decisions/0619-bytecode-calls-loops-and-fusion.md): the bytecode tier is the default run path. Reason: runtime behaviour; `ail run --no-bytecode` is the way out.

Open cards from the night, each with the recommendation from its lane:

- OQ-120: keep ADR 0066 (the lane's recommendation).
- OQ-124: no tuples for now.
- f32 tensors (lane 154): defer.
- `unsafe` in the interpreter hot loop (lane 190): no `unsafe`.
