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
| `cookie_jar.ail` | An agent's session with one site: a cookie jar builds the `Cookie` header for each request (`std.net.jar`, offline). |
| `json_lines.ail` | An event log in JSON lines: `parse_json_lines` skips bad lines, `Json.text` and `Json.flag` read fields with defaults. |
| `map_top.ail` | Error codes counted in a `Map`, the most frequent with `Map.top`. |
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
| `checksums.ail` | Digests and checksums (`std.hash`): SHA-2, HMAC-SHA256, CRC-32, each against its standard test vector. |
| `compress.ail` | Compression (`std.compress`): gzip a log, check it shrinks, gunzip it back; bad input is refused. |
| `regex.ail` | Regular expressions (`std.regex`): find, capture groups, replacement with `${name}`, split. |
| `loopback.ail` | An HTTP round trip on 127.0.0.1 (`std.net` server and client). Runs offline. |
| `app_todo.ail` | A to-do list in a native window (`std.app`). Needs the app host; see `app_todo.rules`. |
| `ui_hello.ail` | A window that says hello (`std.ui`). Needs a display; see `ui_hello.rules`. |
| `bits.ail` | Bit operations on `int`: flags, a bitset, a hash mix; `shr` keeps the sign. |
| `keyword_names.ail` | Keywords as names: a field `error`, `o.match`, the label `in:`, a method `any`, and derived JSON (`lex.kw.3`). |
| `calendar.ail` | Dates, times and zones (`std.time.date`, `std.tz`): leap years, ISO 8601, DST, `in_zone`. Fixed values, same output every run. |
| `tensors.ail` | Float tensors: scalars, unary minus, division, and the activations `relu`, `sigmoid`, `tanh`. |
| `matrix.ail` | Agent-style matrix script: integer matrix product, transpose and row sums with `std.tensor`. |
| `backup.ail` | Agent-style backup check: archive a config directory with system `tar`, list it, remove it. |
| `csv2json.ail` | Agent-style converter: the sales CSV as a JSON array, numbers as JSON numbers. |
| `std_csv.ail` | `std.csv`: read a sales export with quoted fields, filter rows, write them back as CSV. |
| `std_args.ail` | `std.args`: a switch, an option, a positional and the rest of the words, with generated usage. |
| `std_log.ail` | `std.log`: leveled messages on standard error, filtered by `LOG_LEVEL`. |
| `std_math.ail` | `std.math` float methods: distances, a root-mean-square, decibels and an angle. |
| `std_regex_groups.ail` | `regex.group_all`: one group's text from every match. |
| `hexdump.ail` | Agent-style byte inspector: a file as offset-addressed hex rows of 16 bytes. |
| `logwatch.ail` | Agent-style log watcher: pull `code=` and `user=` out of each ERROR line. |
| `perms.ail` | Agent-style permission checker: a Unix mode as one number, each permission a bit (`bit_and`). |
| `poller.ail` | Agent-style poller: fetch a JSON status document over HTTP(S), fall back to the local copy. Takes a URL argument. |
| `report_dates.ail` | Agent-style daily report: Unix-second events grouped into UTC days. |
| `route.ail` | Agent-style HTTP route: one GET route, a method check and a 404, rendered as HTTP/1.1 text. |
| `supervise.ail` | Agent-style process supervisor: run each job with a time limit and report its exit status, timeout or start failure. |

Data files used by the examples are in `data/` and `configs/`.
