# Example twins

Python twins of the examples, measured with tokbench (lane 247, night of
2026-10-21). A twin prints the same output as its example. The token
counts are cl100k and the mean of the tokenizers tokbench uses.

| Example | Twin | AIL cl100k | Python cl100k | AIL / Python |
|---|---|---|---|---|
| `examples/cookie_jar.ail` | `tools/tokbench/corpus/examples-twins/cookie_jar.py` | 277 | 746 | 0.37 |

## Notes

- The twin reimplements the cookie rules it needs (host-only and domain
  cookies, path defaults, `Secure`, `HttpOnly`, replacement, order). Python's
  standard library has no cookie jar that works offline with these rules, so
  the Python side is larger than a stdlib user would write. Treat the ratio as
  an upper bound for Python, not a fair comparison of a library call.
- AIL wins here, so no construct needs a loss note.

## Not yet covered

- `parse_json_lines`, `Json.text`/`Json.flag` and `Map.top` (lane 247's list)
  are not on the batch head yet, so no example uses them.
- `std.args`, `std.csv` and `std.log` already have examples (`std_args`,
  `std_csv`, `std_log`); their twins are not written here.
