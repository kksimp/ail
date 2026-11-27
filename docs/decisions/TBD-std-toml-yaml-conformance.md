# TBD. `std.toml` and `std.yaml` against the official test suites

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** std.toml.9-10, std.yaml.11 ([17-std](../spec/17-std.md))
- **Amends:** [0901](0901-std-toml.md), [1090](1090-std-yaml.md)

## Context

Kaleb asked that the std modules work, not only compile. `std.toml` and `std.yaml`
were tested with one hand-written case each. A Python port of the `std.toml` parser
run over the 717 files of toml-test (TOML 1.0.0) failed 185: 178 invalid documents
were accepted (leading zeros, `1__0`, `1.`, `2023-02-30`, control characters,
redefined tables, extended inline tables) and 7 valid ones were refused (a string
ending in quotes, `inf` and `nan`, a comment right after a bare value). The same port of `std.yaml` over
yaml-test-suite read 15 valid documents wrongly (block scalar chomping and blank
lines, a lone `...`, `-\t-1`) and accepted 17 invalid ones.

## Decision

Make both parsers strict where the format is unambiguous, so that text the format
forbids fails with the module's error and never reads as something else:

- `std.toml` validates numbers, dates and times and string contents itself, and
  records how each table path was defined (header, dotted key, inline table, fixed
  array, array of tables) to enforce the single-definition rules.
- `std.yaml` fixes block scalar blank lines and the final line break, flow
  collection entries and indentation, document markers, plain scalars that start
  with an indicator, and tab separators.

The suites' cases run as the gate cases `std_toml_suite` (708 documents) and
`std_yaml_suite` (371 documents), written verbatim by
`tools/conformance/gen_suite_tests.py`; the expected results are the suites', never
std's own output. `std.yaml` stays a subset (std.yaml.7): valid YAML outside it
must be refused or read exactly. Two invalid YAML documents are still read
(CML9, Y79Y/000) and are listed as known gaps.

## Consequences

- Documents that read before and the formats forbid now fail (`a = 01`,
  `[a]` twice). No document the formats allow changes, except the seven that were
  refused and now read.
- A TOML integer beyond 64 bits fails with a message that says so.
- The suite cases add about 190 KB of test data, MIT licensed (NOTICE).

## Rejected alternatives

- **Patch only the cases found.** The accepted classes (numbers, dates, table
  definitions) are open ended; validating by grammar closes them.
- **A full YAML 1.2 parser now.** Anchors, tags, multi-line scalars and several
  documents are the subset's documented limit (ADR 1090); a larger parser is a
  separate decision.
- **Run the suites from a checkout at test time.** The gate has no network and the
  suites change; the cases are pinned in the repository instead.
