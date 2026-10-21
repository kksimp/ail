# Example twins

Python twins of the examples, measured with tokbench (lane 247, night of
2026-10-21). A twin prints the same output as its example. Counts are cl100k.
The twins are in `tools/tokbench/corpus/examples-twins/`.

| Example | Twin | AIL | Python | AIL / Python |
|---|---|---|---|---|
| `examples/json_lines.ail` | `json_lines.py` | 214 | 190 | 1.13 |
| `examples/map_top.ail` | `map_top.py` | 146 | 153 | 0.95 |

## Notes

- `json_lines` is 13% over Python, under the 20% threshold. Most of the gap is
  the escaped JSON in the example's string literals: each brace inside a string
  needs `\{`, and each quote needs `\"`. Python's `json.loads` takes the same
  text with no escapes, so the cost is in the literal, not in `parse_json_lines`
  or `Json.text`/`Json.flag`.
- `map_top` is about even. `Map.top` does the sorting that the Python twin
  writes by hand.
- Both twins match their `.out` files byte for byte.
- This PR is stacked on `claude/trial-head`, which carries `parse_json_lines`,
  `Json.text`, `Json.flag` and `Map.top`. The `cookie_jar`, `std_args`,
  `std_csv` and `std_log` rows live on PR #311 (`lane/247-std-module-examples`).
  Both PRs add rows to this file; keep both sets of rows when merging.
