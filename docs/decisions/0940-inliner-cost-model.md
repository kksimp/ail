# 0940. Native: inlining functions that call others, by a call-graph cost model

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076); 2026-11-12 round by the designer ("ship F7, the compiler gaining size or building a little bit slower isnt an issue to me")
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.inline.1](../spec/23-native.md), [trace.frame.2](../spec/21-trace.md)
- **Builds on:** [ADR 0391](0391-native-backend-optimisations.md) (leaf inlining), [ADR 0890](0890-optimizer-roadmap-to-match-c.md) (optimizer roadmap), [ADR 0923](0923-struct-scalar-replacement.md) (late inlining)

## Context

ADR 0391 inlines only leaves (`INLINE_LIMIT` 40 instructions, no calls);
ADR 0923's late inlining takes up to 100 for a callee given a fresh object.
`find` in the lzr drag-race program, `advance` in nbody and method chains
stay calls, and the facts a caller knows (`n == data.len()` in `compress`)
never reach the callee's bounds checks.

## Decision

1. **Candidates** (`compiler/mir/inline_wide.ail`): a function that is not
   on a cycle of the call graph (Tarjan, `recursive_names`), not `main` or a
   runtime function, with no indirect call, no primitive that reads its frame
   or starts a thread, no jump to its entry and no loop. One called from a single
   place (and whose address is not taken) qualifies up to `SINGLE_LIMIT` 200
   instructions; any other up to `WIDE_LIMIT` 12.
2. **Budget.** Copies of non-single candidates may grow one caller by at
   most `CALLER_GROWTH` 200 instructions. Only a call inside a loop of the
   caller is copied (fifth round). The single call of a single-call function
   goes inline there (the function is then dropped, so it costs no size); a copy of that call brought in by
   another copy counts as any other call. Leaves are not counted. There is no
   program-wide cap: by the designer's rule (2026-11-05), size is not traded
   for speed, and every tighter budget measured lost the gain (below).
3. **Frames.** A call copied in keeps its site with a link to the call it was
   inlined at; `mark_recursion` turns such a site into a `frame` site instead
   of clearing it, which the generators give call-table rows and no stack
   check. A call inlined at a copied call links to that call's frames. A trap
   below a copied body lists every source frame (trace.frame.2).
4. **Dead functions.** A candidate that nothing calls or names after all
   passes is dropped from the module.
5. **Runtime functions take leaves only.** `rt.dispatch` calls each
   `spawn` site's `task.K` once, so without this the spawned function would
   be copied into it; the passes after inlining leave runtime functions
   alone, so a copied call there missed the result object `sret_convert`
   passes (native.stack.2) and wrote through a stray pointer
   (`tests/cases/std_app_inbox.ail` crashed; `tests/native/run/inline_wide_task.ail`).
6. Off under `AIL_INSTRUMENT` and with `AIL_OPT_OFF=wide` (or `inline`).

## Measured, first run (ail-linux, 870d0571 against main d40e5092)

With loops allowed in the callee, `matmul` went 5 → 10 ms and `spectralnorm`
3 → 5 ms (both back to main with `AIL_OPT_OFF=wide`), `lzr` 4191 → 4115 ms,
the rest flat. A function with a loop is therefore never a candidate: the call
costs little beside its loop, and the copy lost the loop's facts in the caller.

Second run (6eb13dff against main d555d685, limits 30 and 400, no loops):
`lzr` 4187 → 3838 ms (gcc -O2 2257), the regressions gone, the rest flat; but
the stage-3 compiler grew 8.89 → 16.0 MB (9.18 MB with `AIL_OPT_OFF=wide`):
small callers-of-others copied at hundreds of sites. Hence `WIDE_LIMIT` 12
(about the cost of the call and its moves) and `CALLER_GROWTH` 200.

Third run (358d4894 against main 84416e3b, limits 12 and 200): `lzr` 4193 →
3860 ms (-8%), vm and the perfbench ten flat; the stage-3 compiler 8.89 →
12.05 MB (+36%, with ADR 0939's tables), against 9.18 MB with
`AIL_OPT_OFF=wide`. The size is the cost of copies at many call sites and
their call-table rows; it buys the lzr gain and is reported for review.

Fourth round (2026-11-05; all variants
built on one machine and timed interleaved, best of 11; main 8d480dc, which
has ADR 0939). Variants: **full** (this decision), **p8c** (every wide copy
only at a call inside a loop, callers grown by at most 2x, program by at most
10%), **p8d** (full, but single-call functions only at calls inside loops).

| ail-linux (x86-64) | main | p8c | p8d | full |
|---|---|---|---|---|
| stage-3 compiler | 9.27 MB | 10.00 MB | 12.59 MB | 12.21 MB (+32%) |
| self-build, `--no-cache` | 28.7 s | 31.8 s | 34.5 s | 36.8 s (+28%) |
| lzr | 4182 ms | 4072 | 3832 | 3833 (-8.3%) |
| wordfreq | 24 ms | 24 | 23 | 20 (-17%) |
| matmul | 4 ms | 4 | 3 | 3 |
| sieve | 13 ms | 13 | 14 | 14 |
| collatz | 26 ms | 26 | 27 | 27 |

The vm, nbody, fannkuch, binarytrees, fasta, spectralnorm, mandelbrot and
regex_scan programs are flat in every variant (within 1%). On the Mac (ARM64)
every program is flat in every variant (lzr 1743 / 1735 / 1742 / 1737 ms);
the stage-3 compiler is 9.94 → 13.19 MB and the self-build 12.8 → 16.2 s with
full. With `AIL_OPT_OFF=wide`, the compiler is the size of main's.

Full is kept: it is the fastest on every program that moves, against p8d's
slightly larger compiler and smaller wordfreq gain. sieve and collatz are 1 ms
slower (13 and 26 ms runs) with full and with p8d alike, and not with p8c;
hot-only single-call copies (p8d) did not remove it, so it comes from small
copies at cold calls. It is reported for review and left to the next
inliner change to chase.

Fifth round (2026-11-12; F7 of `lanes/slow-spots-2026-11-10.md`, rebuilt on
main 3896938b; ail-linux, interleaved, best of 11). Variants: **p8e** (a wide
copy at every call; in `main` only inside its loops), **p8l** (a wide copy only
at a call inside a loop of its caller), **p8s** (single-call copies as p8e,
small copies as p8l), **p8t** (p8e with `WIDE_LIMIT` 8, `CALLER_GROWTH` 60).

| ail-linux (x86-64) | main | p8e | p8l | p8s | p8t |
|---|---|---|---|---|---|
| stage-3 compiler | 9.80 MB | +30% | +10% | +10% | +8% |
| self-build | 31.8 s | 41.2 s | 35.8 s | 37.3 s | 37.6 s |
| lzr | 3804 ms | 3715 | 3713 | 3716 | 3810 |

vm, regex_scan, wordfreq (at 10 M), nbody, fannkuch, binarytrees, sieve and
collatz are flat in every variant. The full table is in PR #1210.

**Decision of this round (decided by: designer, 2026-11-12):** wide inlining
ships in 0.1.0 as **p8l**. The designer ruled that the compiler growing or
building somewhat slower is not a concern, so the size cost no longer weighs
against the pass. p8l is chosen over p8e on the measurements alone: it has
all of p8e's lzr gain at a third of the size and build-time cost. This
replaces the fourth round's "full is kept"; with the earlier wordfreq gain no
longer measurable at a 10 M size, copies at calls outside loops buy nothing.
The fourth round's rejection of "hot sites only" is withdrawn for that reason.
`AIL_OPT_OFF=wide` still turns the pass off.

## Deferred

Cloning a function for a constant argument (`tree(base, 6, v)`) is left for a
follow-up, measured on its own: it needs a specialisation cache and a size
cap that this change does not.

## Rejected alternatives

- **Raising `INLINE_LIMIT`** for leaves only: `find` and `advance` call
  others, so no limit reaches them.
- **Inlining everything below a size, recursion included**: unbounded copies
  of a cycle, and the stage-2 compile-time cost that killed ADR 0890 pass 1a.
- **Callees with loops** (the first run): slower where measured, see above.
- **A program-wide cap** (p8c, the 9.8 MB target of review): lost the lzr
  and wordfreq gains, see the fourth round. (Copying only at calls inside
  loops, rejected with it then, was taken in the fifth round.)
- **A wide copy at every call** (p8e, the fourth round's choice): the same
  lzr time as p8l at three times the size and build-time cost (fifth round).
- **Smaller limits** (p8t, `WIDE_LIMIT` 8, `CALLER_GROWTH` 60): lost the lzr
  gain (fifth round).
- **Single-call functions only at hot sites** (p8d): no gain over full, and
  wordfreq 23 against 20 ms.
- **Dropping the inlined call's frames**: a trap would name the wrong
  function, against trace.frame.2.
