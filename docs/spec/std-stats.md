# Standard library: `std.stats`

Chapter key: `std`

> **Status:** Drafted (ADR 1224). Written in AIL (`std/stats/stats.ail`).

Descriptive statistics over a `List[float]`. Every function is pure
(`eff.infer.1`), needs no capability, and never traps: input it has no answer
for gives `none`.

## The module

**std.stats.1** — `stats` is in scope without `use` (`std.use.4`); `use std.stats`
stays valid. Every function takes a `List[float]`. `floats(xs List[int])
List[float]` turns a list of `int` into one, so `stats.mean(stats.floats(ns))`
works on integers. There is one name per statistic, not one per element type.
*(ADR 1224)*

**std.stats.2** — `mean(xs) float?` is the sum divided by the length; `none` for an
empty list. *(ADR 1224)*

**std.stats.3** — `median(xs) float?` is the middle value of the sorted list, or
the mean of the two middle values when the length is even; `none` for an empty
list. *(ADR 1224)*

**std.stats.4** — `mode(xs) float?` is the value that occurs most often; when
several tie, the smallest. `none` for an empty list. *(ADR 1224)*

**std.stats.5** — `variance(xs)` and `stdev(xs)` are the sample statistics (the
squared distances from the mean divided by n - 1); `pvariance(xs)` and
`pstdev(xs)` are the population statistics (divided by n). Each gives
`float?`. The sample forms are `none` below two values, the population forms
for an empty list. `stdev` is the square root of `variance`, `pstdev` of
`pvariance`. *(ADR 1224)*

**std.stats.6** — `percentile(xs, p float) float?` is the `p`th percentile, `p`
from 0 to 100. The sorted list is indexed by rank `p / 100 * (n - 1)`, counted
from 0, and a rank between two values is interpolated linearly (the
"inclusive" method, numpy's default). `none` for an empty list or a `p` outside
0 to 100. *(ADR 1224)*

**std.stats.7** — `quantiles(xs, n int) List[float]?` gives the `n - 1` cut points
that split the data into `n` groups of equal share: element `i - 1` is
`percentile(xs, 100 * i / n)`. It matches Python's
`statistics.quantiles(..., method="inclusive")`. `none` for an empty list or an
`n` below 2. *(ADR 1224)*

**std.stats.8** — `min_max(xs) (float, float)?` is the smallest and the largest
value; `none` for an empty list. *(ADR 1224)*

**std.stats.9** — `histogram(xs, buckets int) List[int]?` counts the values in
`buckets` equal-width ranges from the smallest value to the largest. A value on
a boundary goes in the upper bucket, the largest value in the last one; when
all values are equal they all go in the first. `none` for an empty list or
`buckets` below 1. *(ADR 1224)*

**std.stats.10** — `correlation(xs, ys) float?` is Pearson's coefficient, from
-1 to 1. `none` when the lists differ in length, hold fewer than two values, or
either has no spread. *(ADR 1224)*

**std.stats.11** — `moving_average(xs, n int) List[float]?` is the mean of each run
of `n` neighbouring values, `xs.len() - n + 1` of them, in order. `none` for an
empty list, an `n` below 1 or an `n` above the length. *(ADR 1224)*

**std.stats.12** — A list holding `nan` gives unspecified results; no function
checks for it. Results are computed in `float` arithmetic, so they can differ
from exact decimal answers in the last digits. *(ADR 1224)*

```ail rules=std.stats.1,std.stats.2,std.stats.3,std.stats.4,std.stats.5
fn main()
    xs := [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]
    print(stats.mean(xs) else 0.0)                          # 5.0
    print(stats.median(xs) else 0.0)                        # 4.5
    print(stats.mode(xs) else 0.0)                          # 4.0
    print(stats.pstdev(xs) else 0.0)                        # 2.0
    print(stats.mean(stats.floats([1, 2, 3, 4])) else 0.0)  # 2.5
    print(stats.variance([1.5]) == none)                    # true: one value has no sample variance
```

```ail rules=std.stats.6,std.stats.7,std.stats.12
fn main()
    xs := [15.0, 20.0, 35.0, 40.0, 50.0]
    print(stats.percentile(xs, 40.0) else 0.0)              # 29.0
    print(stats.percentile(xs, 120.0) == none)              # true
    print(stats.quantiles(xs, 3) else [])                   # [25.0, 38.333333333333336]
```

```ail rules=std.stats.8,std.stats.9,std.stats.10,std.stats.11
fn main()
    print(stats.min_max([3.0, -1.5, 9.0]) else (0.0, 0.0))  # (-1.5, 9.0)
    print(stats.histogram([0.0, 1.0, 2.0, 3.0, 4.0, 10.0], 2) else [])   # [5, 1]
    print(stats.correlation([1.0, 2.0, 3.0], [6.0, 4.0, 2.0]) else 0.0)  # -1.0
    print(stats.moving_average([1.0, 2.0, 3.0, 4.0], 2) else [])         # [1.5, 2.5, 3.5]
```
