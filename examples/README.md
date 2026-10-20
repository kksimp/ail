# Examples

Each example is a program (`NAME.ail`) with its expected output (`NAME.out`).
Run one with `./ail run examples/NAME.ail`. Every example is kept formatted
with `./ail fmt`.

| Example | Shows |
|---|---|
| `hello.ail` | The first program: a function and a print. |
| `gradebook.ail` | Structs, enums, methods, `match`, loops and interpolation. |
| `shapes.ail` | Traits, `any`, generics and the standard traits. |
| `config.ail` | Reading an agent's service config: maps, optionals and enums. |
| `json.ail` | A JSON parser, printer and query tool. |
| `interp.ail` | A small scripting language: tokenizer, parser and evaluator. |
| `sheet.ail` | A spreadsheet engine: formulas, dependencies, cycle detection. |
| `parallel.ail` | Loops the checker proves independent (`conc.par.2`–`4`). |
| `pipeline.ail` | A three-stage pipeline over bounded channels. |
| `workers.ail` | Fan-out and fan-in: four workers share one channel. |
| `tail.ail` | A log tailer: one task streams lines through a channel. |
| `build.ail` | Agent-style build script: check every example, summarise diagnostics. |
| `commands.ail` | A stock keeper: parses commands and reports bad input as errors. |
| `csvreport.ail` | Agent-style CSV report: revenue per region. |
| `digest.ail` | Agent-style fetch and summarise: CSV over HTTP, local fallback. Takes a URL argument. |
| `fetch.ail` | Fetch a URL and summarise the response (`std.net`). Needs network. |
| `logscan.ail` | Agent-style log scan: levels per hour, ranked error messages. |
| `merge.ail` | Agent-style config merge: a base, a local override, one typed result, unknown members reported. |
| `runner.ail` | Agent-style task runner: each command with its own time limit, keeps going after failures. |
| `scan.ail` | Agent-style repo scanner: glob a tree, count lines per extension. |
| `snapdiff.ail` | Compare two snapshots of a directory. |
| `tasks.ail` | Agent-style task runner over JSON tasks, written back to disk. |
| `tool.ail` | Agent-style tool: read a file, measure it, run a command, with each failure in its type. |
| `dashboard.ail` | A small dashboard on `std.ui.widgets`: directory sizes, rescanned by a task. |
| `ui_hello.ail` | A window that says hello (`std.ui`). Needs a display; see `ui_hello.rules`. |

Data files used by the examples are in `data/` and `configs/`.
