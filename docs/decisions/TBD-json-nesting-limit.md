# TBD. `parse_json` limits nesting to 512 levels; `parse_json_with` changes it

- **Status:** Accepted (number to be allocated by the integrator)
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), at the coordinator's request
- **Topic:** `std.json` robustness
- **Spec rules:** [json.parse.3](../spec/std-json.md)
- **Amends:** [ADR 0161](0161-std-json-in-the-prelude.md) (`parse_json` accepted any depth)

## Context

The parser is recursive. JSONTestSuite's `n_structure_100000_opening_arrays` and
`n_structure_open_array_object` are a few hundred KB of opening brackets; text from a
network or a file can make `parse_json` recurse until the stack is exhausted, which is a
crash and not an error a program can handle.

## Decision

`parse_json` fails with `JsonError` (`nesting deeper than 512`, `at` the opening bracket
of the first level over the limit) when arrays and objects nest more than 512 deep.
`parse_json_with(text, max_depth)` takes another limit. Scalars are at depth 0. Text that
nested 512 or fewer levels parses exactly as before. The limit is a parser field, so no
new syntax and no global setting.

512 is far above real data (the deepest JSON in common use is tens of levels) and about
1,000 parser frames, which the gate test `json_depth` runs (raised to 1,000 levels).

## Rejected alternatives

- **No limit, a bigger stack:** moves the crash, and the limit still depends on the host.
- **An explicit stack in the parser:** removes the crash but keeps a value too deep for the
  recursive `to_text`, `==` and drop that follow, so the failure moves.
- **A limit of 100 (Python's `json` has none, serde_json uses 128):** rejects the 500-deep
  document JSONTestSuite marks implementation-defined (`i_structure_500_nested_arrays`);
  512 accepts it.
- **A global setting:** hidden state; an argument is local to the call.
