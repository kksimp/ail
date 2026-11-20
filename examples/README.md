# Examples

Each example is a program (`NAME.ail`) with its expected output (`NAME.out`); the programs that need a window or a display have a `NAME.rules` file instead or as well.
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
| `bigint_sum.ail` | Totals that outgrow `int`: `+`, `*`, `/` and `<` on `std.bigint`'s `BigInt` (`trait.op.1`). |
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
| `chan_pipe.ail` | A producer task sends squares down a bounded channel and closes it; `main` receives until the channel is closed and empty, then sums them. |
| `task_sum.ail` | Sums 0 to 999 in four parts at once: a `par` comprehension runs one function per part on the thread pool and keeps the results in order. |
| `frames.ail` | Length-prefixed frames read from a byte buffer: one length byte, then that many payload bytes, sliced rather than copied by hand. |
| `hex_base64.ail` | Hex decoded into bytes, encoded as base64 and decoded back; `from_hex` and `from_base64` give optionals, so each is checked. |
| `digest_lines.ail` | The SHA-256 and CRC-32 of each record line, so a later run can tell which records changed (`std.hash`). |
| `gz_check.ail` | Gzips a file's text many times over, gunzips it and checks the round trip; prints only booleans (`std.compress`). |
| `redact_mail.ail` | Redacts e-mail addresses from a record file with one regex replace per line, and counts the lines that held one (`std.regex`). |
| `regex_codes.ail` | Agent-style log summary: finds every `code=NNN`, counts them and adds up the values with `group_all` (`std.regex`). |
| `days_left.ail` | Days left until each date in a file, counted from a fixed day; a date already past reports how many days late it is (`std.time.date`). |
| `shift_dates.ail` | Reads ISO dates from a file and prints each one's weekday and the date a week later (`std.time.date`). |
| `scan_usage.ail` | What takes the space: `fs.scan` lists a tree once with each entry's length, allocated size and inode, counting a hard-linked file once (`std.fs.23`). |
| `ffi_libc.ail` | Calls into the C library: `extern c` declares a function, a call is `unsafe`, and the safe wrappers are plain AIL (`ffi.call.1-2`). |
| `ffi_buffers.ail` | Raw pointers and buffers: C fills memory the program allocated and writes back a pointer into a string it parsed (`unsafe.ptr.1-3`, `unsafe.mem.3`). |
| `native_fib.ail` | Recursion and an iterative loop side by side; calls between free functions in the native subset. |
| `native_sieve.ail` | The sieve of Eratosthenes over a list of bools: a list built by `push`, indexed in place, and a `for` over a range. |
| `native_collatz.ail` | Collatz chain lengths: the longest chain below a limit, in a native loop. |
| `native_mandel.ail` | Floats in the native subset: the escape-time Mandelbrot iteration over a grid, printed to a fixed number of decimals. |
| `native_words.ail` | Strings in the native subset: `join`, `len`, `chars`, interpolation, and `to_int() else` as a binding's value. |
| `app_vectors.ail` | Vector shapes on a canvas (`std.app.32`): a disc, a ring, a rounded card, a triangle and a zig-zag line. Prints the display list; `--window` opens it in the app host. See `app_vectors.rules`. |
| `tool_api_outline.ail` | Agent task: the outline of a file from `ail api --text`, keeping only the function lines. |
| `tool_api_signatures.ail` | Agent task: each package module's functions from `ail api`, their signatures, and whether each has a doc comment. |
| `tool_card_budgets.ail` | Agent task: the size of a package's card at four budgets, so an agent picks the largest its context can spare. |
| `tool_check_summary.ail` | Agent task: `ail check --json` on every fixture, counting errors and warnings per file. |
| `tool_check_rules.ail` | Agent task: which spec rule IDs the fixtures' diagnostics cite, most frequent first. |
| `tool_fix_preview.ail` | Agent task: the fixes `ail fix` would apply to the broken fixtures, without writing anything, and what remains. |
| `tool_fmt_check.ail` | Agent task: which fixture files `ail fmt` would rewrite. |
| `tool_edit_preview.ail` | Agent task: a hunk-format edit applied with `ail edit --dry-run`, reporting the diagnostics the edited file would have. |
| `tool_diff_summary.ail` | Agent task: declarations added, removed and changed in each edited module, from `ail diff`. |
| `tool_diff_lines.ail` | Agent task: each changed declaration from `ail diff`, its kind of change, and how many source lines it touches. |
| `tool_effects_table.ail` | Agent task: each function's effects from `ail effects`, counted per effect. |
| `tool_explain_family.ail` | Agent task: a std family's rules found with `ail explain` without knowing their IDs. |
| `tool_explain_rules.ail` | Agent task: each rule ID in a list looked up with `ail explain`, with the first line of its explanation. |
| `tool_explain_chapters.ail` | Agent task: the spec rules per chapter from `ail explain --json`, and how many `ail fix` can repair. |
| `tool_test_summary.ail` | Agent task: a file's tests run with `ail test`, listing which passed and which failed, then the exit status. |
| `tool_test_coverage.ail` | Agent task: the coverage line from `ail test --coverage`. |
| `tool_test_update_preview.ail` | Agent task: the snapshot updates `ail test --update --dry-run` would make, and the changed-line count. |
| `tool_replay_check.ail` | Agent task: a run recorded with `ail run --record`, replayed with `--replay`, and the two outputs compared. |

Data files used by the examples are in `data/` and `configs/`.
