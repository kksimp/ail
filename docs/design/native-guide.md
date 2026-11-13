# Native builds: a guide

> **Non-normative guide,** 2026-10-20 (lane 186 of the
> [finish plan](finish-plan.md)). The rules are in
> [spec/23-native.md](../spec/23-native.md) (native.subset.1, native.build.1–3);
> the backend's layouts are in [native-abi.md](native-abi.md). Keep this page
> in step with the subset and with [quick-reference.md](../quick-reference.md).

## Build and run

```sh
./ail build examples/native_fib.ail              # writes ./native_fib
./native_fib                                     # runs it; same output as ./ail run
./ail build FILE.ail -o OUT --target aarch64-linux
```

- `ail build FILE [-o OUT] [--target T]` checks the program the way
  `ail check` does, stops on its errors, then reports anything outside the
  native subset. It writes an executable to `OUT`, or to `FILE`'s name without
  `.ail` in the current directory.
- Targets: `x86_64-linux` (the default on Linux x86-64), `aarch64-linux`, and
  `aarch64-macos` (the default on Apple Silicon; the Mach-O file is signed by
  the backend). Any other name is an error (native.build.3).
- The executables link no libc and import nothing. Each one is a single static
  file.
- `ail run FILE` builds the program the same way and runs it; an unchanged
  program starts from the build cache (native.build.5).

## What builds natively

Everything `ail check` accepts: structs, enums, methods, traits, generics,
maps and sets, optionals, errors, tasks and channels, contracts and the
standard library (native.subset.1 in [spec/23-native.md](../spec/23-native.md)).
Native is the only backend (ADR 0904). A rare construct the lowering cannot
build yet is reported with its position and a `help:` line, and no executable
is written:

```text
error[native.subset.1]: not in the native subset yet: a `Map[str, int]` in a tuple
help: maps are native with `int`, `bool`, `char` or `str` keys (ADR 0568); ...
```

## When a program traps

A trap has the same first line on standard error in the native executable as
in `ail run` (native.build.1). The line gives the position and the rule that
failed. The native executable then prints a source excerpt, and exits with
status 1. Standard output before the trap is kept:

```text
$ ./trap_index
1
2
3
PATH:5:17: trap[type.coll.4]: index 3 is out of bounds for a list of length 3
  5 |         print(xs[i])
```

`ail run --trace` adds a snapshot of the locals.

## Looking at the code the backend produces

`ail run compiler mir FILE.ail` prints the MIR (unstable, ir-architecture.md §4)
of a program, before register allocation. Use it to see what a loop lowers to,
and whether a helper was inlined:

```sh
./ail run compiler mir examples/native_fib.ail
```

Native executables keep their function symbols in `.symtab` (`STT_FUNC`), so a
Linux profiler names them. With `perf` installed:

```sh
perf record ./native_collatz && perf report
```

On macOS, use Instruments' Time Profiler on the executable. Neither tool was
run as part of writing this guide, so report any mismatch.

## Limits

- Integer overflow, division by zero, bad indexes and bad shifts trap with the
  rule IDs of [spec/23-native.md](../spec/23-native.md); they do not wrap.
- Performance numbers change with each lane. See
  [native-performance.md](native-performance.md) and [../benchmarks.md](../benchmarks.md).
