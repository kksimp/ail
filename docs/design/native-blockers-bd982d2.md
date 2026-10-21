# Native build blockers at bd982d2

Non-normative survey of why programs do not build with `ail build` (x86-64) on the batch head bd982d2, 2026-10-21. Of 101 programs (84 `examples/`, 17 `tools/perfbench/programs/`) 25 build and 76 do not (72 examples, 4 perfbench). A program can hit several features: `ail build` prints every problem, and this survey groups them. Owners are the PRs and lanes the integrator named for the area; "none known" means no PR in flight was named. Sources: the `error[native.*]` lines of each build.

## Features, most programs first

"Alone" counts programs this feature is the only blocker of, "first" counts programs whose first error it is.

| Feature | Programs | Alone | First | Owner |
|---|---|---|---|---|
| std.path module | 41 | 9 | 31 | none known |
| composite parameter and result types, generic functions | 18 | 3 | 12 | lane 179 |
| std.proc module | 17 | 1 | 2 | none known |
| list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum) | 16 | 3 | 6 | #197 |
| `@test` functions | 10 | 0 | 0 | none known |
| untyped empty `{}` or comprehension element | 10 | 0 | 0 | none known |
| std.args / csv / log / hash / date / compress / bytes modules | 9 | 6 | 7 | none known |
| std.json (`parse_json`, `Json`, derived `from_json`) | 7 | 0 | 0 | none known |
| tasks and channels (`spawn`, `Channel`, `TaskGroup`) | 6 | 0 | 0 | lane 178 |
| enum variants and error constructors (`Name(...)`, bare variants), error enums/structs | 6 | 0 | 2 | lane 179 |
| `str.to_int()` | 5 | 0 | 2 | #192 |
| optional receivers (`.f` / `.len()` / `.pop()` / indexing / `for` on a `T?`, `char?` compare) | 5 | 0 | 0 | none known |
| `else` must leave (`x else return`-style forms) | 4 | 0 | 0 | none known |
| `unsafe` blocks, contracts, `unreachable` | 4 | 0 | 0 | none known |
| std.net module | 4 | 0 | 2 | lane 194 |
| `match` on `char`, `char` maps, map and set comprehensions | 4 | 0 | 2 | #180 / #174 |
| traits and `impl` blocks, `to_str` methods, `any Trait` | 4 | 0 | 2 | lane 179 |
| std.tensor module | 4 | 4 | 4 | lane 193 |
| std.regex module | 3 | 1 | 1 | none known |
| `for mut` | 2 | 0 | 0 | none known |
| C library calls (`extern c`) | 2 | 0 | 2 | none known |
| `fs.glob`, `env.find`, `.pos` on `str` | 2 | 0 | 1 | none known |
| `int.checked_add/sub/mul/div` | 1 | 0 | 0 | none known |
| other (9 one-off messages) | 9 | 0 | 0 | none known |

## Which fix pays most

Greedy order: each step picks the feature that leaves the most programs with no blocker (ties by programs containing it), assuming nothing else hides behind it.

| Step | Feature | Programs with it | Newly buildable | Total buildable |
|---|---|---|---|---|
| 1 | std.path module | 41 | 9 | 34 of 101 |
| 2 | std.proc module | 17 | 13 | 47 of 101 |
| 3 | std.args / csv / log / hash / date / compress / bytes modules | 9 | 8 | 55 of 101 |
| 4 | list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum) | 16 | 4 | 59 of 101 |
| 5 | std.tensor module | 4 | 4 | 63 of 101 |
| 6 | composite parameter and result types, generic functions | 18 | 3 | 66 of 101 |
| 7 | tasks and channels (`spawn`, `Channel`, `TaskGroup`) | 6 | 3 | 69 of 101 |
| 8 | std.regex module | 3 | 3 | 72 of 101 |
| 9 | `@test` functions | 10 | 2 | 74 of 101 |
| 10 | std.json (`parse_json`, `Json`, derived `from_json`) | 7 | 5 | 79 of 101 |

Modules (`path`, `proc`, `args`, `csv`, `log`, `hash`, `date`, `compress`, `regex`, `net`, `tensor`, `json`) are named by their standard-library import: the native runtime has no implementation of them yet, so a program that touches one stops at its first use.

## Programs

| Program | Features |
|---|---|
| `examples/backup.ail` | std.path module; std.proc module |
| `examples/build.ail` | `@test` functions; composite parameter and result types, generic functions; std.path module |
| `examples/calendar.ail` | composite parameter and result types, generic functions |
| `examples/chan_pipe.ail` | composite parameter and result types, generic functions; tasks and channels (`spawn`, `Channel`, `TaskGroup`) |
| `examples/checksums.ail` | std.args / csv / log / hash / date / compress / bytes modules |
| `examples/commands.ail` | `else` must leave (`x else return`-style forms); `for mut`; `unsafe` blocks, contracts, `unreachable`; enum variants and error constructors (`Name(...)`, bare variants), error enums/structs |
| `examples/compress.ail` | composite parameter and result types, generic functions |
| `examples/config.ail` | `@test` functions; std.json (`parse_json`, `Json`, derived `from_json`); std.path module |
| `examples/csv2json.ail` | composite parameter and result types, generic functions; std.json (`parse_json`, `Json`, derived `from_json`); std.path module |
| `examples/csvreport.ail` | `@test` functions; `str.to_int()`; std.path module; untyped empty `{}` or comprehension element |
| `examples/dashboard.ail` | composite parameter and result types, generic functions; list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); optional receivers (`.f` / `.len()` / `.pop()` / indexing / `for` on a `T?`, `char?` compare); std.args / csv / log / hash / date / compress / bytes modules; std.path module; tasks and channels (`spawn`, `Channel`, `TaskGroup`) |
| `examples/days_left.ail` | std.args / csv / log / hash / date / compress / bytes modules; std.path module |
| `examples/digest.ail` | `@test` functions; list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); std.net module; untyped empty `{}` or comprehension element |
| `examples/digest_lines.ail` | std.path module |
| `examples/fetch.ail` | list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); std.net module; std.path module |
| `examples/ffi_buffers.ail` | C library calls (`extern c`); `unsafe` blocks, contracts, `unreachable`; composite parameter and result types, generic functions |
| `examples/ffi_libc.ail` | C library calls (`extern c`); `unsafe` blocks, contracts, `unreachable` |
| `examples/frames.ail` | std.args / csv / log / hash / date / compress / bytes modules |
| `examples/gz_check.ail` | std.args / csv / log / hash / date / compress / bytes modules; std.path module |
| `examples/hex_base64.ail` | std.args / csv / log / hash / date / compress / bytes modules |
| `examples/hexdump.ail` | std.path module; std.proc module |
| `examples/interp.ail` | `else` must leave (`x else return`-style forms); `fs.glob`, `env.find`, `.pos` on `str`; `int.checked_add/sub/mul/div`; `match` on `char`, `char` maps, map and set comprehensions; `str.to_int()`; `unsafe` blocks, contracts, `unreachable`; composite parameter and result types, generic functions; enum variants and error constructors (`Name(...)`, bare variants), error enums/structs; list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); optional receivers (`.f` / `.len()` / `.pop()` / indexing / `for` on a `T?`, `char?` compare); traits and `impl` blocks, `to_str` methods, `any Trait`; untyped empty `{}` or comprehension element |
| `examples/json.ail` | `else` must leave (`x else return`-style forms); `match` on `char`, `char` maps, map and set comprehensions; `str.to_int()`; composite parameter and result types, generic functions; enum variants and error constructors (`Name(...)`, bare variants), error enums/structs; list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); optional receivers (`.f` / `.len()` / `.pop()` / indexing / `for` on a `T?`, `char?` compare); std.json (`parse_json`, `Json`, derived `from_json`); traits and `impl` blocks, `to_str` methods, `any Trait` |
| `examples/logscan.ail` | `@test` functions; list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); std.path module; untyped empty `{}` or comprehension element |
| `examples/logwatch.ail` | std.path module; untyped empty `{}` or comprehension element |
| `examples/loopback.ail` | composite parameter and result types, generic functions; std.net module; tasks and channels (`spawn`, `Channel`, `TaskGroup`) |
| `examples/matrix.ail` | std.tensor module |
| `examples/merge.ail` | `@test` functions; composite parameter and result types, generic functions; std.json (`parse_json`, `Json`, derived `from_json`); std.path module |
| `examples/pipeline.ail` | composite parameter and result types, generic functions; tasks and channels (`spawn`, `Channel`, `TaskGroup`) |
| `examples/poller.ail` | list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); std.json (`parse_json`, `Json`, derived `from_json`); std.net module |
| `examples/redact_mail.ail` | std.path module; std.regex module |
| `examples/regex.ail` | composite parameter and result types, generic functions |
| `examples/regex_codes.ail` | std.path module; std.regex module |
| `examples/report_dates.ail` | `str.to_int()`; std.path module; untyped empty `{}` or comprehension element |
| `examples/runner.ail` | `@test` functions; list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); std.proc module |
| `examples/scan.ail` | `fs.glob`, `env.find`, `.pos` on `str`; std.path module; untyped empty `{}` or comprehension element |
| `examples/shapes.ail` | `@test` functions; composite parameter and result types, generic functions; enum variants and error constructors (`Name(...)`, bare variants), error enums/structs; optional receivers (`.f` / `.len()` / `.pop()` / indexing / `for` on a `T?`, `char?` compare); traits and `impl` blocks, `to_str` methods, `any Trait` |
| `examples/sheet.ail` | `else` must leave (`x else return`-style forms); `for mut`; `match` on `char`, `char` maps, map and set comprehensions; `str.to_int()`; composite parameter and result types, generic functions; enum variants and error constructors (`Name(...)`, bare variants), error enums/structs; list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); optional receivers (`.f` / `.len()` / `.pop()` / indexing / `for` on a `T?`, `char?` compare); other; other; other; other; other; traits and `impl` blocks, `to_str` methods, `any Trait`; untyped empty `{}` or comprehension element |
| `examples/shift_dates.ail` | std.path module |
| `examples/snapdiff.ail` | `@test` functions; `match` on `char`, `char` maps, map and set comprehensions; list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); std.path module; untyped empty `{}` or comprehension element |
| `examples/std_args.ail` | std.args / csv / log / hash / date / compress / bytes modules |
| `examples/std_csv.ail` | std.args / csv / log / hash / date / compress / bytes modules |
| `examples/std_log.ail` | std.args / csv / log / hash / date / compress / bytes modules |
| `examples/std_regex_groups.ail` | std.regex module |
| `examples/supervise.ail` | std.proc module |
| `examples/tail.ail` | composite parameter and result types, generic functions; list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); tasks and channels (`spawn`, `Channel`, `TaskGroup`) |
| `examples/task_sum.ail` | list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum) |
| `examples/tasks.ail` | `@test` functions; std.json (`parse_json`, `Json`, derived `from_json`); std.path module |
| `examples/tensors.ail` | std.tensor module |
| `examples/tool.ail` | other; other; std.proc module |
| `examples/tool_api_outline.ail` | std.path module; std.proc module |
| `examples/tool_api_signatures.ail` | std.path module |
| `examples/tool_card_budgets.ail` | std.path module; std.proc module |
| `examples/tool_check_rules.ail` | other; std.path module |
| `examples/tool_check_summary.ail` | std.path module |
| `examples/tool_diff_lines.ail` | std.path module; std.proc module |
| `examples/tool_diff_summary.ail` | std.path module; std.proc module |
| `examples/tool_edit_preview.ail` | std.path module; std.proc module |
| `examples/tool_effects_table.ail` | list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); std.path module; std.proc module |
| `examples/tool_explain_chapters.ail` | list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum); std.json (`parse_json`, `Json`, derived `from_json`); std.path module; std.proc module |
| `examples/tool_explain_rules.ail` | std.path module |
| `examples/tool_fix_preview.ail` | std.path module; std.proc module |
| `examples/tool_fmt_check.ail` | std.path module |
| `examples/tool_garlic_pages.ail` | std.path module |
| `examples/tool_pdf_grep.ail` | std.path module |
| `examples/tool_pdf_text.ail` | std.path module |
| `examples/tool_replay_check.ail` | std.path module; std.proc module |
| `examples/tool_test_coverage.ail` | std.path module; std.proc module |
| `examples/tool_test_summary.ail` | std.path module; std.proc module |
| `examples/tool_test_update_preview.ail` | std.path module; std.proc module |
| `examples/ui_hello.ail` | composite parameter and result types, generic functions; other |
| `examples/workers.ail` | composite parameter and result types, generic functions; enum variants and error constructors (`Name(...)`, bare variants), error enums/structs; tasks and channels (`spawn`, `Channel`, `TaskGroup`); untyped empty `{}` or comprehension element |
| `tools/perfbench/programs/fasta.ail` | list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum) |
| `tools/perfbench/programs/mandelbrot.ail` | list methods on `List[char]`, `List[str]` (join, sorted, first, last, contains, sum) |
| `tools/perfbench/programs/tensor.ail` | std.tensor module |
| `tools/perfbench/programs/tensor_ops.ail` | std.tensor module |

## Method

`ail build FILE -o OUT --target x86_64-linux` for every `examples/*.ail` and `tools/perfbench/programs/*.ail`, collecting every `error[native.*]` line, grouped by message pattern.
