# 0644. The trap stack is always built, with no flag: cost measured

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Native runtime diagnostics
- **Spec rules:** `trace.frame.1`

## Context

The brief allowed making the stack opt-in if it cost more than 2% on the
perfbench programs.

## Measurements (Linux x86-64, perfbench programs, `ail build`)

| Program | Before (bytes) | After (bytes) | Change |
|---|---|---|---|
| binarytrees | 19,560 | 21,696 | +10.9% |
| collatz | 17,688 | 20,128 | +13.8% |
| fannkuch | 22,448 | 24,712 | +10.1% |
| fasta | 21,008 | 22,904 | +9.0% |
| mandelbrot | 19,072 | 20,864 | +9.4% |
| matmul | 21,912 | 23,944 | +9.3% |
| nbody | 41,408 | 47,840 | +15.5% |
| spectralnorm | 23,976 | 28,560 | +19.1% |
| sieve | 19,168 | 21,048 | +9.8% |

Run time: no instruction is added to any call or check, so the generated
code is the same; wall-clock runs of nbody, fannkuch, spectralnorm and
binarytrees (two runs each) were within noise (1–3 ms spread on 7–85 ms).

## Options considered

### Option A — always on (chosen)
- **Pros:** an agent never has to rebuild to read a failure; one behaviour.
- **Cons:** 2–6 KB larger executables (9–19% of programs this small; the
  fixed part is the 1.5 KB printer, the rest is 16 bytes per call).

### Option B — `ail build --trace` turns it on
- **Pros:** smaller default builds.
- **Cons:** the failing binary is often the one already shipped to the agent;
  a rebuild costs the agent a round trip, against the project's tokens-first
  order.

## Decision

Option A. Run time is the criterion that matters here, and it doesn't change.
If code size becomes a goal, a flag can drop the table and print only the
trapping line.

## Consequences

- Revisit if a size budget appears (embedded targets).

## Rejected alternatives

Option B above; compressing the table (varint rows): saves under 1 KB, costs
a decoder in the runtime.
