# Standard library: `std.log` loggers

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR TBD). Rules
> std.log.1-2 (the plain functions and `LOG_LEVEL`) are in
> [17-std](17-std.md); this chapter adds `Logger`.

`std.log` has no global state. A `Logger` is a value; every setter returns a
changed copy. The module is written in AIL (`std/log/log.ail`) and is in scope
as `log` (`std.use.4`).

**std.log.3** — The plain functions `debug`, `info`, `warn` and `err` keep their
form (std.log.2): `LEVEL message`, no timestamp, filtered by `LOG_LEVEL`. They
ignore `LOG_FORMAT`.

**std.log.4** — `log.named(name) Logger` makes a logger: level from `LOG_LEVEL`
(std.log.1), format from `LOG_FORMAT` (std.log.13), a timestamp on each line, standard
error as the target. `l.child(n)` is a copy named `name.n` (just `n` when the
name is empty). An empty name is not written.

**std.log.5** — `l.with(key, value)` is a copy that adds the field to every
line. `l.info_kv(message, fields Map[str, str])` (also `debug_kv`, `warn_kv`,
`err_kv`) adds fields to one message, in key order after the logger's own. A
key set twice appears twice.

**std.log.6** — `l.debug(m)`, `l.info(m)`, `l.warn(m)` and `l.err(m)` write a
line when the level is at or above the logger's. `l.with_level(name)` fixes the
level (`debug`, `info`, `warn`, `error`, `off`, case-insensitive) instead of
`LOG_LEVEL`; an unknown name shows nothing (as std.log.1).

**std.log.7** — A text line is `[TIMESTAMP ]LEVEL [NAME: ]message key=value ...`.
A value that is empty or holds a space, `=`, `"` or a line break is written as a
JSON string. `l.text()` fixes the text format.

**std.log.8** — `l.json()` writes one JSON object per line with members `ts`
(unless std.log.9 turns it off), `level` (lower case), `logger` (unless the name
is empty), `msg`, then the fields, all values strings. A field whose key is `ts`,
`level`, `logger` or `msg` is written as `field.KEY`.

**std.log.9** — The timestamp is `date.now().iso()` (UTC, `std.date.9`).
`l.with_time(false)` leaves it out, which makes output reproducible.

**std.log.10** — `l.to_file(path, max_bytes, keep)` appends lines to `path`
(created when missing, std.fs) instead of standard error. `l.to_stderr()` goes
back. A line that cannot be written is dropped; logging never fails or traps.

**std.log.11** — Rotation: when `max_bytes > 0`, the file is not empty and a line
would take it past `max_bytes`, the file `path.keep` is removed, `path.i` is
renamed to `path.i+1` for `i` from `keep - 1` down to 1, and `path` is renamed to
`path.1`; the line then starts a new `path`. `keep` below 1 counts as 1. A single
line longer than `max_bytes` is written whole.

**std.log.12** — `log.debug_kv`, `info_kv`, `warn_kv` and `err_kv` (message and
`Map[str, str]`) are the plain functions with fields: `LEVEL message key=value`,
no timestamp, `LOG_FORMAT` not applied.

**std.log.13** — `LOG_FORMAT=json` (case-insensitive) makes a logger that has not
called `json()` or `text()` write JSON lines; any other value, or none, is text.

**std.log.14** — Level and format from the environment are read at each line, so
a logger made before the program changed them follows the change.

```ail rules=std.log.4,std.log.5,std.log.6,std.log.7
fn main()
    l := log.named("api").with_time(false).with("req", "42")
    l.info("served")                       # INFO api: served req=42 (standard error)
    l.child("db").warn_kv("slow", {"ms": "812", "q": "a b"})
    l.with_level("error").info("hidden")
    print("done")
```

```ail rules=std.log.8,std.log.10,std.log.11
fn main()
    j := log.named("job").json().with_time(false).to_file("job.log", 1000000, 3)
    j.info("start")
    print(fs.read_text("job.log") else "")     # {"level":"info","logger":"job","msg":"start"}
    fs.remove("job.log") else return
```

## Rejected alternatives

- **A global `log.set_level` or `log.set_output`.** AIL has no mutable globals; a
  `Logger` value and the environment cover both.
- **Fields as a `ToJson` value.** A string map is what call sites have; typed
  values can be formatted by the caller.
- **Rotation by age or compression.** Size covers the common case; compressing
  `path.N` can be done by the caller with `std.compress`.
- **Timestamps on the plain functions.** They would change ADR 0628 output
  that scripts already match.
