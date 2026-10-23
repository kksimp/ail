# Constructs outside the native subset, and their owners (lane 242)

One list for three readers: `ail explain native.subset.1` (bootstrap/src/explain.rs),
`tools/native-coverage/coverage.py` (its owner column), and the selftest in
`bootstrap/tests/native_subset_table.rs`. Each row: a construct, the wording of its
`native.subset.1` message (plain text, backticks removed, as a substring of the
message), and its owner: a PR (`#184`), a lane (`lane 120`), several of those
separated by commas, or `unowned`. The first row whose wording matches a message
names the owner, so put the more specific rows first.

| Construct | Wording | Owner |
|---|---|---|
| interpolation outside print | string interpolation outside | #184, #204 |
| to_int, to_float on str | to_int() on str | #184 |
| == and != on str | == on str | #184 |
| Map[...] | Map[ | lane 173 |
| list contains and extend | .{name}() on | #197 |
| channels | Channel[ | #194 |
| TaskGroup, Mutex, spawn | calling TaskGroup | #194 |
| parse_json | calling parse_json | - |
| generic functions | generic functions | #219 |
| generic structs and enums | generic struct | #219 |
| trait methods and impl blocks | methods of traits | #222 |
| contracts | contracts aren't | - |
| attributes such as @test | attribute @ | #207, #208 |
| unsafe blocks | unsafe blocks | #207, #208 |
| tensors | tensor | lane 193 |
| optional field and index access | ? on an optional | lane 175 |
| error structs and enums | error struct | lane 120 |
| comprehension of unknown type | comprehension whose | lane 179 |
| for mut | for mut | - |
| char patterns | char patterns | - |
| bytes literals | bytes literals | - |
| mut parameter of a number | mut parameter | lane 120, #205 |
| unsupported parameter, field or result type | its type isn't | lane 120 |
| module value that isn't a constant | only locals, parameters and constants | lane 179 |
