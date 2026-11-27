# 1213. `proc` options: working directory, environment, input and time limit; `proc.pid`, `os.hostname`

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); lane 3 of the std gap plan, which the designer asked for on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.proc.11-15](../spec/17-std.md), [std.os.5](../spec/17-std.md)
- **Extends:** [ADR 0160](0160-process-runs.md), [ADR 0282](0282-std-proc-input-timeout-which.md), [ADR 0424](0424-streaming-child-processes.md)

## Context

`proc.run` and `proc.start` always inherit the working directory and the environment. Build scripts, test
runners and agent tools need "run `make` in `build/` with `CC=cc`" and fall back to `sh -c "cd build && CC=cc make"`,
which brings back the shell quoting `proc` exists to avoid. `run` already has two positional optionals after
`args` (`input`, `timeout_ms`); adding `cwd`, `env` and `clear_env` the same way would make five unlabelled
trailing arguments whose meaning depends on position.

## Decision

- `proc.options()` gives a `proc.RunOptions { cwd str?, env Map[str, str], clear_env bool, input str?, data bytes?,
  timeout_ms int? }`, the shape of `net.options()` (std.net.6): set fields by name, then pass the value.
- `proc.run_with(cmd, args, opts)`, `proc.run_bytes_with(cmd, args, opts)` and `proc.start_with(cmd, args, opts)`
  are `run`, `run_bytes` and `start` with the options. The plain forms are unchanged.
- `env` is added to the inherited set, replacing a variable of the same name (case-insensitive on Windows);
  `clear_env` starts from nothing. An empty name or one with `=` traps; a NUL is a `ProcError` of kind `other`,
  as a NUL in an argument already is.
- A relative program path (`./build.sh`) is found under `cwd`; a bare name is found on the parent's `PATH`.
- A `cwd` that can't be entered is a `ProcError` whose `cmd` is the directory, so the error names what was
  missing, not a program that exists. On Unix the child reports its `chdir` errno through the exec-status pipe
  with bit 32 set; Windows checks the directory before `CreateProcessW`.
- The input is `input` or `data`, whichever is set; both set traps. `start_with` traps on either, since a child's
  input goes through `c.stdin`.
- `proc.pid()` and `os.hostname()` (`gethostname`, `uname` on Linux, `GetComputerNameExW` on Windows) carry the
  `env` effect: they are facts of the run, not of the program.

## Rejected alternatives

- **More trailing optionals on `run`.** Five positional optionals is the opposite of local reasoning; a reader
  can't tell `run(c, a, none, none, "build")` from its neighbours without the signature.
- **Labelled trailing arguments (`run(c, a, cwd: "build")`).** Std calls take no labels today; adding them is a
  syntax-level change to call checking, out of scope for a std lane.
- **`run(cmd, args, opts)` overloaded on the third argument's type.** One name with two meanings for the same slot
  is worse for diagnostics (a `str` there could be an input or a mistake).
- **Naming the struct `proc.Options`.** Standard type names are one namespace (`load.ail` STD_TYPES) and
  `Options` is `net`'s; `RunOptions` says what it configures.
- **Looking the program up on the child's `PATH` when `env` sets one.** Platform-dependent elsewhere; the
  parent's `PATH` keeps `which` and `run_with` agreeing.
- **A builder (`proc.command("make").cwd("build").run()`).** A second API style for one module; the options struct
  already exists for `net`.

## Consequences

Native: `compiler/mir/lower_procopt.ail` maps the calls to `rt.proc_run_with`, `rt.proc_run_bytes_with`,
`rt.proc_start_with`, `rt.proc_options`, `rt.proc_pid` and `rt.os_hostname` (one dispatch line in `lower_os.ail`,
one struct line in `lower_struct.ail`). The internal `proc_run_with` of the runtime is now `proc_run_core`, and
`proc_launch`/`win_launch` take the options. Record and replay stops at the new calls (`rec_guard`) until they
get `rec_` wrappers. Windows builds the code but is untested until feature parity resumes.
