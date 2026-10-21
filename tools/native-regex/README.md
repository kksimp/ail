# Native std.regex test harness (lane 232)

Differential tests that hold whatever approach native `std.regex` takes
([note](/mnt/project-files/notes/232-native-regex.md)). They compare a native
build with `ail run` on the same input: stdout and exit code. Until native
`std.regex` builds, each mode prints the build's own problems and exits 3.

| Mode | What it runs |
|---|---|
| `tools/native-regex/run.sh` | `vectors.ail` over `bootstrap/tests/regex/vectors.txt` (370 lines: searches with groups, `ALL`, `REPLACE`, `REPLACE_FIRST`, `SPLIT`, `ERR`) |
| `tools/native-regex/run.sh cases` | each `bootstrap/tests/cases/*regex*.ail`, built and run both ways |
| `tools/native-regex/run.sh fuzz [N [SEED]]` | `gen.py` writes N random vectors (RE2-subset patterns, flags, classes, malformed patterns); the same `vectors.ail` runs them |

`vectors.ail` prints one line per vector: the span and the group texts of a
search, the spans of `find_all`, the replaced or split text, or the full
`RegexError` (`kind`, `at`, `rule`, `message`) of a pattern that does not
compile. It makes no claim about what is right: the Rust engine is checked
against the vector file's expectations by `cargo test` (`regex::vectors`), and
this harness checks that a second engine agrees with it.

A disagreement keeps both outputs in `$OUT` (a temporary directory, printed).
Measurement tooling only (ADR 0070); it is not part of `selftest`.

## Performance

`tools/perfbench/programs/regex_scan.{ail,py,c}`: six-token log lines from an
LCG, four patterns per line (digit runs, addresses, key=value pairs, a
replacement). The three programs print identical output (checked by hand at
100,000 lines; `regex_scan.out`). The C twin uses POSIX `regexec`, whose
leftmost-longest agrees with leftmost-first for these alternation-free
patterns. Before native `std.regex` (100,000 lines, Linux x86-64, wall time,
one run): C 0.26 s, Python `re` 0.82 s, `ail run` 2.79 s.
