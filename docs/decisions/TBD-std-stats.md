# TBD. `std.stats`: descriptive statistics on a `List[float]`

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.stats.1 to std.stats.12](../spec/std-stats.md)

## Context

Agents asked for percentiles, a mean and a standard deviation (the corpus tasks `percentile_ranks`,
`u7_latency_percentiles`, `csv_stats`, `tsv_column_stats`) write them by hand each time, and the usual first draft gets
the empty list, the sample against the population divisor or the interpolation wrong. Python has `statistics`; AIL has
nothing.

## Decision

- A pure AIL module, `std/stats/stats.ail`, in scope without `use` as `stats` (std.use.4). No runtime or checker
  code: registration lines only.
- One float API. Every function takes a `List[float]`; `stats.floats(ints)` converts a `List[int]`. One name per
  statistic keeps the call predictable (AI understanding first) and AIL has no overloading or numeric trait to
  spell "int or float" with.
- Functions: `mean`, `median`, `mode` (smallest of the tied), `variance`/`stdev` (sample, n - 1),
  `pvariance`/`pstdev` (population), `percentile(xs, p)` (0 to 100, linear interpolation at rank `p/100*(n-1)`, the
  numpy default), `quantiles(xs, n)` (the "inclusive" method of Python's `statistics.quantiles`, so it agrees with
  `percentile`), `min_max`, `histogram(xs, buckets)`, `correlation`, `moving_average`.
- Input with no answer (empty list, `p` outside 0 to 100, lists of different lengths, a window longer than the list,
  no spread for `correlation`) gives `none`, never a trap (the failure is absence, not an enum error: there is one
  way to fail per call).
- Tests compare to Python `statistics` and numpy outputs for the same data, printed to six places.

## Rejected alternatives

- **A module called `std.stats` under another name** (`statistics`): the plan's measured local-name clash for `stats`
  is 8 uses, low enough that a local hiding the module is rare, and `stats` is shorter.
- **`List[int]` overloads or `_int` twins:** doubles the surface for no new statistic; `floats` is one call.
- **Python's default "exclusive" quantiles:** it disagrees with `percentile` on the same data; two methods in one
  module would be a trap for agents.
- **`T ! StatsError` for empty input:** an error type for one condition, and callers would write `else` anyway.
- **`mode` returning every tied value:** a list for a rare case; the smallest is deterministic.
- **Exact running sums or compensated summation:** costs speed for last-digit gains; std.stats.12 states the float
  caveat.
