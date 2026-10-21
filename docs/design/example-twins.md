# Example twins

Python twins of the examples, measured with tokbench (lane 247, night of
2026-10-21). A twin prints the same output as its example. The token
counts are cl100k and the mean of the tokenizers tokbench uses.

| Example | Twin | AIL cl100k | Python cl100k | AIL / Python |
|---|---|---|---|---|
| `examples/cookie_jar.ail` | `tools/tokbench/corpus/examples-twins/cookie_jar.py` | 277 | 746 | 0.37 |
| `examples/std_args.ail` | `tools/tokbench/corpus/examples-twins/std_args.py` | 219 | 290 | 0.76 |
| `examples/std_csv.ail` | `tools/tokbench/corpus/examples-twins/std_csv.py` | 176 | 182 | 0.97 |
| `examples/std_log.ail` | `tools/tokbench/corpus/examples-twins/std_log.py` | 108 | 87 | 1.24 |

## Notes

- The twin reimplements the cookie rules it needs (host-only and domain
  cookies, path defaults, `Secure`, `HttpOnly`, replacement, order). Python's
  standard library has no cookie jar that works offline with these rules, so
  the Python side is larger than a stdlib user would write. Treat the ratio as
  an upper bound for Python, not a fair comparison of a library call.
- `std_log` is the one loss above 20%. Its header comment is most of the
  difference: with the comment lines removed, AIL is 51 and the Python twin
  70. The four `log.*` calls cost the same in both languages, so there is no
  construct to change. The comment is part of the example for agents, so it
  stays.
- `std_args`: the Python twin hand-codes the parser and the usage text, which
  `std.args` generates from the spec. The usage text counts for Python, so the
  ratio is a fair reading of what an agent writes, not of a library call.
- `std_csv`: Python's `csv` module does the quoting in both directions; the
  two are close, and the AIL side includes the `csv.write_records` call.

## Not yet covered

- `parse_json_lines`, `Json.text`/`Json.flag` and `Map.top` (lane 247's list)
  are not on the batch head yet, so no example uses them.
