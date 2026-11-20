# Native std.regex test harness

Differential tests of `std.regex` ([ADR 0819](../../docs/decisions/0819-native-std-regex-in-ail.md)). They compare
an `ail build` executable with `ail run` on the same input: stdout and exit code. Each mode prints the build's own problems and exits 3 if the build fails.

| Mode | What it runs |
|---|---|
| `tools/native-regex/run.sh` | `vectors.ail` over `tests/regex/vectors.txt` (370 lines: searches with groups, `ALL`, `REPLACE`, `REPLACE_FIRST`, `SPLIT`, `ERR`) |
| `tools/native-regex/run.sh cases` | each `tests/cases/*regex*.ail`, built and run both ways |
| `tools/native-regex/run.sh fuzz [N [SEED]]` | `gen.py` writes N random vectors (RE2-subset patterns, flags, classes, malformed patterns); the same `vectors.ail` runs them |

`vectors.ail` prints one line per vector: the span and text of a
search (capture groups are compared through `REPLACE` templates until `regex.captures` builds natively), the spans of `find_all`, the replaced or split text, or the full
`RegexError` (`kind`, `at`, `rule`, `message`) of a pattern that does not
compile. It makes no claim about what is right: the `regex` cases check the engine
against the vector file's expectations, and this harness checks that the built
program and `ail run` agree.

A disagreement keeps both outputs in `$OUT` (a temporary directory, printed).
Measurement tooling only (ADR 0070); it is not part of `selftest`.

## Performance

`tools/perfbench/programs/regex_scan.{ail,py,c}`: six-token log lines from an
LCG, four patterns per line (digit runs, addresses, key=value pairs, a
replacement). The three programs print identical output (`regex_scan.out`, 100,000 lines). The C twin uses POSIX `regexec`, whose
leftmost-longest agrees with leftmost-first for these alternation-free
patterns. Run the three twins on your machine to compare; the C and Python twins
are single-threaded and the numbers depend on the host. Current race numbers are
in [benchmarks](../../docs/benchmarks.md).
