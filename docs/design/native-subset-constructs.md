# Constructs outside the native subset, and their owners (lane 242)

One list for two readers: `ail explain native.subset.1` and the `native_misc`
gate suite (`tools/gate/gate.ail`, which checks each row). Each row: a construct, the wording of its
`native.subset.1` message (plain text, backticks removed, as a substring of the
message), and its owner: a PR (`#184`), a lane (`lane 120`), several of those
separated by commas, or `unowned`. The first row whose wording matches a message
names the owner, so put the more specific rows first. A row leaves the table
when its construct becomes native and no message has its wording any more.

| Construct | Wording | Owner |
|---|---|---|
| Map[...] | Map[ | lane 173 |
| list contains and extend | .{name}() on | #197 |
| channels | Channel[ | #194 |
| TaskGroup, Mutex, spawn | calling TaskGroup | #194 |
| parse_json | calling parse_json | - |
| generic functions | generic functions | #219 |
| generic structs and enums | generic struct | #219 |
| trait methods and impl blocks | methods of traits | #222 |
| attributes such as @test | attribute @ | #207, #208 |
| unsafe blocks | unsafe blocks | #207, #208 |
| tensors | tensor | lane 193 |
| optional field and index access | ? on an optional | lane 175 |
| error structs and enums | error struct | lane 120 |
| comprehension of unknown type | comprehension whose | lane 179 |
| for mut | for mut | - |
| mut parameter of a number | mut parameter | lane 120, #205 |
| unsupported parameter, field or result type | its type isn't | lane 120 |
| module value that isn't a constant | only locals, parameters and constants | lane 179 |
