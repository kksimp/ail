# 0580. The OS layer is direct system calls; failures use the error slot

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)**
- **Topic:** Native runtime and ABI (lane 177)
- **Spec rules:** [std.fs.1](../spec/17-std.md) to std.fs.7 and std.fs.11, [std.env.1](../spec/17-std.md) to std.env.4 and std.env.6, [std.io.1](../spec/17-std.md) to std.io.4, [std.time.1](../spec/17-std.md), std.time.2, [std.err.1](../spec/17-std.md), [native.subset.1](../spec/23-native.md)
- **Design note:** [native-abi.md](../design/native-abi.md) §4 and §9

## Context

The native compiler must read and write files, list directories, read the
environment and the clock, and read standard input, with no libc (ADR 0406),
on Linux x86-64, Linux ARM64 and macOS ARM64. The native subset has no
`Result`, optional or struct yet (lane 175 adds error values), but the
compiler driver's source is full of `fs.read_text(p) else ...`.

## Options considered

### Option A — system calls in AIL, a status beside the value (chosen)
- **Pros:** no new MIR or code generator work; one runtime function per
  operation; the `else` forms of err.else.1-2 lower to a call, a status call
  and a branch, the way `s.to_int() else ...` already does; error values can
  replace the status later without changing the runtime's callers.
- **Cons:** the wrapper reads a global status right after the operation;
  `err` is the error's text, so `err.kind` waits for lane 175's fielded errors.

### Option B — libSystem and libc through dyld and `extern`
- **Pros:** Apple's stable interface on macOS.
- **Cons:** needs the dynamic linking lane 126 hasn't landed; Linux has no
  libc to link; two code paths.

### Option C — wait for error values and write it all with `Result`
- **Pros:** the final shape directly.
- **Cons:** blocks the self-compile path on lane 175.

## Decision

A. `compiler/rt/native/os.ail` has one function per operation: `fs_exists`,
`fs_read_text`, `fs_write_text`, `fs_append_text`, `fs_list_dir`, `fs_remove`,
`fs_make_dir`, `fs_rename`, `env_get`, `env_home`, `env_cwd`, `env_program`,
`time_now_ms`, `time_sleep_ms`, `io_read_line`, `io_read_all`, `io_exit`.
Each `os_NAME` records its outcome in `os_ok()` and `os_kind()` (the
`fs.IoError.kind` of std.err.1, from one errno table that maps both Linux's
and macOS's numbers to the same name). The `fs_*` wrappers the lowering calls
follow lane 175's results (ADR 0575): on failure they set the error slot to
the display text of an `fs.IoError`, `fs.IoError(path: "P", kind: "K")`,
with the call's trace line, and return zero, so `?`, `else` and `else err`
work on them (`err` is that text as a `str`; its fields wait for lane 175's
fielded errors). `env_get`, `env_home`, `io_read_line` and `s.to_float()`
return optionals (a string pointer or 0; a boxed float). Paths go through two
4 KiB scratch buffers allocated once, so no call allocates for its path.
`lower_os.ail` lowers the calls and passes the call's trace line; the rules
for fallible calls (`?`, `else`, `else err`, a failing `main`) are lane 175's.

The system calls per target (native-abi.md §4): Linux uses the `*at` calls
with `AT_FDCWD` on both architectures (so the ARM64 numbers, which have no
`open`, `unlink`, `mkdir` or `rename`, differ only in the table); macOS uses
the classic numbers, `getdirentries64` for directories, and
`fcntl(F_GETPATH)` on `.` for the working directory because it has no
`getcwd` call. `read_text` sizes its buffer from `lseek` when the size is
plausible (a directory reports a huge one) and grows by doubling otherwise,
so `/proc` files work; it checks UTF-8 as std.fs.1 requires.

Gaps, each tracked: `io.exit` and `time.sleep_ms` report their out-of-range
argument (std.io.4, std.time.2) as `trap[RULE]: MESSAGE` without the source
excerpt, because a trap with an excerpt needs a site kind in the code
generators (lane 120). `fs.read_bytes`, `write_bytes` and `io.read_bytes`
wait for lane 123's `bytes` in the subset; `fs.metadata`, `copy`, `walk`,
`glob` and `env.vars` wait for structs, maps and the module system, and are
standard-library code over these calls. The macOS paths (`getdirentries64`,
`F_GETPATH`, `gettimeofday`, `select`) are checked only by the designer's
macOS gate. Windows (`plat.win.1`) has no native target.

## Rationale

Criterion: AI understanding (local, no ambiguity) and the critical path: the
driver needs files now. A keeps the runtime's source the same shape as the
spec and leaves the checker's error model to lane 175.

## Consequences

- `fs.*`, `env.*`, `io.*` and `time.*` calls compile natively, with `?`,
  `else` and `else err` on the failing ones (the compiler's own `fs.read_text(p)?`
  and `fs.list_dir(d)?`); the test programs `os_*.ail` match `ail run` on
  x86-64 and on ARM64 under qemu.
- Lane 175's fielded errors will let `os_error` build an `fs.IoError` value.

## Rejected alternatives

B: needs dynamic linking and doesn't exist on Linux. C: blocks the critical
path on another lane.
