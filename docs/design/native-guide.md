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
- Build time is about one to two seconds per program today, because the native
  compiler itself still runs on the interpreter. Lane 182 works on that.

## What builds natively today

A program builds when every function is a free function (no methods, generics,
contracts or traits) over `int`, `float`, `bool`, `str`, and lists of those and
of lists. The full list is native.subset.1 in [spec/23-native.md](../spec/23-native.md).
In short:

- locals, constants made of literals and arithmetic, `if`, `while`, `for` over
  a range or a list, `break`, `continue`, `return`, and calls between functions;
- `print`, and string interpolation inside `print` (`{x:.N}` for a float);
- `xs[i]`, `xs[i] = v`, `len`, `push`, `join`, `chars`, `env.args()`, and
  `s.to_int() else ...` as a binding's value;
- the methods `abs`, `min`, `max`, `to_float`, `to_int`, `sqrt`, `floor` and
  `ceil`, and the bit operations.

Examples that build and print what `ail run` prints (checked by
`bootstrap/tests/native_examples.rs`):

| File | Shows |
|---|---|
| [examples/native_collatz.ail](../../examples/native_collatz.ail) | `while`, a value `if`, a loop over a range |
| [examples/native_sieve.ail](../../examples/native_sieve.ail) | lists of bools built by `push`, indexing, assignment |
| [examples/native_fib.ail](../../examples/native_fib.ail) | recursion and a loop, interpolated `print` |
| [examples/native_words.ail](../../examples/native_words.ail) | `join`, `len`, `chars`, `to_int() else` |
| [examples/native_mandel.ail](../../examples/native_mandel.ail) | float arithmetic and `{x:.4}` |

The other programs in `examples/` do not build yet. Most use structs, enums,
methods, traits, maps, closures, or `main` returning a result. `examples/hello.ail`
is the only one that builds today. `ail build` reports the first construct of
each kind it meets, with its position:

```text
$ ./ail build examples/build.ail
examples/build.ail:3:8: error[native.subset.1]: struct `Finding` isn't in the native subset yet
```

When there is a problem, no executable is written. Lanes 171–177 widen the
subset (heap objects, strings, maps and sets, closures, optionals and errors, and
the OS layer). Lane 180's coverage tracker is the list of what is in and what is
not; update this section when it changes.

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

The interpreter adds a snapshot of the locals after that excerpt. The native
executable does not print the snapshot, so its standard error is shorter.

## Checking native output against the interpreter

The interpreter is the reference (ADR 0032). To check a program by hand:

```sh
./ail build FILE.ail -o /tmp/prog && /tmp/prog > native.txt
./ail run FILE.ail > interp.txt
cmp native.txt interp.txt
```

For the examples, the test suite does this for every `native_*` file.

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

- Only the subset above. Everything else is a diagnostic, not a slow fallback.
- Integer overflow, division by zero, bad indexes and bad shifts trap with the
  rule IDs of [spec/23-native.md](../spec/23-native.md); they do not wrap.
- Standard output and `env.args()` are the only I/O so far: no files, no network, no processes.
- Performance numbers change with each lane. See
  [native-performance.md](native-performance.md) for the latest race against C
  and Python.
