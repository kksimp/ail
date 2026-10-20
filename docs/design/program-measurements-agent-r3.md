# Program measurements, agent round 3: ten more agent-tool programs

> **Non-normative design evidence,** 2026-10-20 (lane 144 of batch 2). Ten
> agent-tool programs that use what exists on this branch, each with a Python
> twin that prints the same bytes ([ADR 0477](../decisions/0477-agent-corpus-round-three.md)).
> The programs are in `examples/` with their `.out`; the twins are in
> `tools/tokbench/corpus/programs/NAME/`.

## Method

As in [round 8](program-measurements-r8.md): counts are the mean of the five
offline tokenizers (`tools/tokbench/run.sh`), the `.ail` files are the examples
without comment lines (`tools/tokbench/corpus_from_examples.py`), and both sides
keep their blank lines. Each twin uses the Python standard library the way an
agent would (`datetime`, `tarfile`, `subprocess`, `json`, `&` on bit flags).
Every pair was run: `python3 NAME.py` and `ail run examples/NAME.ail` print the
same bytes.

## Results

| Program | Features exercised | AIL | Python | AIL − Python |
|---|---|---|---|---|
| `logwatch` | `lines`, `strip_prefix`, `Map` counts, `sorted(by:)` | 220.6 | 235.4 | −14.8 (−6.3%) |
| `backup` | `proc.run` (system `tar`), `fs.remove`, `env.get` | 178.4 | 175.6 | +2.8 (+1.6%) |
| `poller` | `net.get` with fallback, `parse_json`, `doc.arr`, `doc.bool` | 235.2 | 267.8 | −32.6 (−12.2%) |
| `supervise` | `proc.run` time limit, `kind` on error, struct list | 266.8 | 267.8 | −1.0 (−0.4%) |
| `report_dates` | integer date arithmetic (no date module yet) | 410.4 | 181.6 | +228.8 (+126.0%) |
| `hexdump` | `proc.run` (system `od`), offset rows | 148.2 | 117.6 | +30.6 (+26.0%) |
| `route` | one GET route, `match`, request and response structs | 314.6 | 256.8 | +57.8 (+22.5%) |
| `perms` | permission bits by division (no bit operators yet) | 376.8 | 267.8 | +109.0 (+40.7%) |
| `csv2json` | `Json.Obj` / `JsonMember` built by hand | 189.0 | 164.8 | +24.2 (+14.7%) |
| `matrix` | `std.tensor` `matmul`, `transpose`, `sum(axis)` | 121.6 | 213.4 | −91.8 (−43.0%) |
| **Total** | | **2,461.6** | **2,148.6** | **+313.0 (+14.6%)** |

The ten programs cost more than Python in total: six cost more and four cost
less. `matrix` wins because `std.tensor` replaces a hand-written matrix product;
`poller` wins because `parse_json` and the typed accessors replace Python's
dictionary indexing and `float()` conversions. Four of the six that cost more
trace to a library or operator that does not exist yet:

- `report_dates` (+229 tokens) builds the civil date from a day number. The
  Python twin calls `datetime.fromtimestamp`. A date module (lane 129) removes
  almost all of the gap.
- `perms` (+109) reads each bit with `(mode / bit) % 2`. Python's `mode & bit`
  is shorter. Bit operators (lane 140) remove the gap.
- `route` (+58) declares two structs and a `match`. The Python twin uses plain
  values. Tuples or a literal map would close part of the gap.
- `hexdump` (+31) reads bytes through the system `od` tool because there is no
  bytes type yet (lane 123).

`csv2json` (+24) is the cost of building JSON values by variant; a conversion
from primitive values would remove most of it.

## Behaviour checked

- `poller` falls back to `examples/data/status.json` with `offline (refused)`
  when nothing listens on `127.0.0.1:1`, as `digest` does.
- `supervise` kills `sleep 5` at its 200 ms limit and reports `timed out`; the
  missing command reports `could not start`.
- `backup` writes its archive under `$TMPDIR` (or `/tmp`) and removes it.
- `route` runs the one route on sample request lines. The socket loop is not
  here: server sockets wait on networking (lane 124), so this measures the
  route, not a server.
- `csv2json` converts the `bad` units cell to `0.0` (`to_float() else 0.0`),
  and the twin does the same.

## Language gaps found

- `!x` and `a || b` are rejected with the words `not` and `or`
  (`expr.op.1`); both programs were fixed to use the words.
- Date formats need `0>4` (fill and align) because `04` is rejected
  (`lex.str.7`).
- `for row, i in xs` is rejected; the index form is `for i in 0..xs.len()`.
- A string cannot be indexed (`type.str.2`), and `+` cannot join strings
  (`trait.op.1`); build text with interpolation or `join`.
- A list has no `filter` method in this build (`expr.name.1`).
- Tuples are not values here (`gram.syntax.1`); a struct is required.
- `if` cannot sit inside a larger expression (`gram.syntax.1`); bind it first.

Proposals for the four programs that cost more are in
[ai-author-wishlist.md](ai-author-wishlist.md#7-agent-corpus-round-three-2026-10-20).
