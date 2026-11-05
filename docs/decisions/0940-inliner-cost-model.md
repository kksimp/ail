# 0940. Native: inlining functions that call others, by a call-graph cost model

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
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
2. **Budget** (after review: the compiler may grow to 9.8 MB at most). A
   non-single candidate goes only at a hot site, a call inside a loop of
   the caller (`find_loops`, computed only for a function that calls one).
   Those copies may grow one caller by at most its own size and
   `CALLER_GROWTH` 200 instructions, and the whole program by at most
   `PROGRAM_GROWTH` 10% of its instructions. The single call of a
   single-call function goes inline wherever it is written (the function is
   then dropped, so it costs no size); a copy of that call brought in by
   another copy counts as any other call. Leaves are not counted.
3. **Frames.** A call copied in keeps its site with a link to the call it was
   inlined at; `mark_recursion` turns such a site into a `frame` site instead
   of clearing it, which the generators give call-table rows and no stack
   check. A call inlined at a copied call links to that call's frames. A trap
   below a copied body lists every source frame (trace.frame.2).
4. **Dead functions.** A candidate that nothing calls or names after all
   passes is dropped from the module.
5. Off under `AIL_INSTRUMENT` and with `AIL_OPT_OFF=wide` (or `inline`).

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
- **Dropping the inlined call's frames**: a trap would name the wrong
  function, against trace.frame.2.
