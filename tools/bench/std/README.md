# Std-module benchmarks

AIL's standard library against what each other language ships: Python (stdlib), Go (stdlib), C (libc, and
zlib for gzip) and Rust (std only, so only sort and maps have a Rust twin: JSON, hashing, regex and the rest
are crates there, which this suite does not pull in). Method follows `tools/bench` and `tools/perfbench/race.py`:
same algorithm and the same printed bytes in every twin (a differing column is `DIFF` and dropped), whole-process
wall time, median of the runs, ratios on the best run.

| program | AIL std | twins |
|---|---|---|
| `json_parse` | `parse_json`, `Json.to_text` | Python `json`, Go `encoding/json` |
| `sha_hmac` | `hash.sha256`, `hash.hmac_sha256` | Python `hashlib`/`hmac`, Go `crypto/*` |
| `gzip_roundtrip` | `compress.gzip`, `compress.gunzip` | Python `gzip`, Go `compress/gzip`, C zlib |
| `regex_scan` | `std.regex` | Python `re`, Go `regexp` |
| `csv_parse` | `std.csv` | Python `csv`, Go `encoding/csv` |
| `url_parse` | `std.url` | Python `urllib.parse`, Go `net/url` |
| `xml_parse` | `std.xml` | Python `xml.etree`, Go `encoding/xml` |
| `sort_ints` | `List.sorted` (ints, strings) | Python, Go, C `qsort`, Rust |
| `map_counts` | `Map[int, int]`, `Map[str, int]` | Python, Go, Rust |
| `db_rows` | `std.db` (in memory) | Python `sqlite3` |

Run (designer's machines only, quiet host, load below 3; the Mac needs Homebrew python for `>= 3.10`):

    tools/bench/std/run.sh                    # writes tools/bench/std/out/std-results.{md,json}
    tools/bench/std/run.sh --runs 3 regex_scan
    tools/bench/std/run.sh --scale 0.1        # smoke run

Sizes are in `programs/NAME.args` and were chosen without a measurement (the first run tunes them: aim for
the slowest twin above 0.3 s, `--scale` multiplies them). Encoder levels differ between libraries, so the
gzip program prints the input size, the CRC-32 and the round-trip result, not the compressed size.

## Not covered yet (add as each lands)

- HTTP server throughput (router, keep-alive) and client: needs a load generator process; a `wrk`-style
  driver in Python against AIL, Go `net/http` and Python `http.server` is the plan.
- brotli, zstd, text diff (Python `difflib` and Go have no Myers minimal diff to compare byte for byte),
  CBOR/msgpack, markdown, decimal: modules not on main at 94c2607b or no twin with equal output.

The AIL programs were written without a compiler in reach: the first run may need a fix before a
column appears (the `ail` cell reads `build`). The Python, Go, C and Rust twins were run and agree.
