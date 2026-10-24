# Native test runner: tree counts

Measured with `ail test --native` (ADR 0679, native.test.1-5) on 2026-10-21 at the
batch head fbf9a99, Linux x86-64, four cores, about 15 s for the whole tree.
`tools/native-coverage` rewrites its own report, so these numbers live here.

| source | tests | match | DIFFER | not in the native subset yet | skipped (property tests, files the interpreter rejects) |
|---|---:|---:|---:|---:|---:|
| examples | 16 | 0 | 0 | 14 | 2 |
| tests/cases | 87 | 9 | 0 | 11 | 67 |
| tools | 152 | 3 | 0 | 124 | 25 |
| all | 255 | 12 | 0 | 149 | 94 |

No test differs: every native build that gets as far as running agrees with the
interpreter on pass or fail (one of the twelve matches is an expected failure).

## What blocks the 151

| blocker | tests |
|---|---:|
| `tools/garlic/garlic.ail` uses a dependency package whose own imports the backend can't resolve | 48 |
| `==` on `str` (`assert_eq` of strings; ADR 0565 / PR #184) | 23 |
| `==` on `List[str]` or `List[int]` | 17 |
| a helper outside the tests uses maps | 9 |
| a helper outside the tests uses `.split()` | 8 |
| `?` on an error value inside a test (the rewritten copy can't fail) | 5 |
| string interpolation outside `print` | 4 |
| a helper takes a `Json` value or an error enum | 7 |
| other (dates, hashes, nets, traits, sets, channels) | 30 |

The two largest groups clear as the queued string and list PRs land; rerun
`ail test --native` after each and add the new counts here. Module files of a package (tools/garlic/js/vm) are built in a staging directory
(native.test.5); the dependency group needs the backend to resolve a dependency's own imports.
