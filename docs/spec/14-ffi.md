# 14. FFI and C interoperability

Chapter key: `ffi`

> **Status:** Drafted (partial). Specified and implemented in the bootstrap
> interpreter: `extern c fn` declarations, the type mapping, loading, and
> calls on Linux and macOS. Specified, not built: `use c "header.h"`
> (`ffi.decl.1`). Not yet specified: callbacks, exporting AIL functions to C,
> structs by value, variadic functions, and header search configuration.

## Declarations

**ffi.decl.1** — `use c "header.h" as name` makes the C declarations of
`header.h` available under `name`. The header is parsed by the
implementation's own C header parser. *(ADR 0103)*

**ffi.decl.2** — `extern c fn name(params) Result` declares one C function;
`extern c "lib" fn name(params) Result` names the library it lives in. The
declaration has no body, is at the top level, and has no receiver or type
parameters. The AIL name is the C symbol. *(ADR 0103, ADR 0431)*

**ffi.decl.3** — Types used in C declarations MUST have C-compatible fixed
layouts: `int`, `float`, `bool`, the sized numbers, and raw pointers `*T`
(`unsafe.ptr.1`); the result may also be omitted. A parameter is not `mut`.
At most six parameters are integer-like (everything but `float` and `f32`) and
at most eight are `float` or `f32`. *(ADR 0103, ADR 0013, ADR 0432)*

```ail fragment
extern c "libm" fn hypot(a float, b float) float
extern c fn abs(x i32) i32
extern c fn strlen(s *u8) int
```

## Calls

**ffi.call.1** — Calling a C function is an unsafe operation
(`unsafe.op.1`): outside an `unsafe` block it is a compile-time error.
*(ADR 0104)*

```ail error=ffi.call.1
extern c fn abs(x i32) i32

fn main()
    print(abs(-1))
```

```ail
extern c fn abs(x i32) i32

fn main()
    unsafe
        print(abs(-1))
```

**ffi.call.2** — A call runs the C function with the arguments passed by value
in the platform's C calling convention, and returns its result. The
implementation does not check that the declaration matches the C function; a
wrong declaration is undefined behaviour, inside the `unsafe` block that made
the call. Output printed by the program is flushed before the call. *(ADR 0432)*

```ail tool=run,FILE expect=7 rules=ffi.call.2
extern c fn abs(x i32) i32

fn main()
    unsafe
        print(abs(-7))
```

## Type mapping

**ffi.map.1** — `int` is C `int64_t`; `float` is `double`; `f32` is `float`;
`bool` is C `bool` (one byte); `i8`...`u64` are the fixed-width C integers of
the same name; `*T` is a pointer. A parameter narrower than 64 bits is passed
extended to 64 bits by its signedness. *(ADR 0432)*

```ail tool=run,FILE expect=5 rules=ffi.map.1
extern c fn labs(x int) int

fn main()
    unsafe
        print(labs(-5))
```

**ffi.map.2** — A result narrower than 64 bits is truncated to its declared
type and extended again: C leaves the upper bits unspecified, so `abs` is
declared `fn abs(x i32) i32`, not `int`. A `bool` result is true when its low
byte is not zero. *(ADR 0432)*

```ail tool=run,FILE expect=8 rules=ffi.map.2
extern c fn abs(x i32) i32

fn main()
    unsafe
        print(abs(-7) + 1)
```

## Native builds

**ffi.native.1** — `ail build` links no C library for the fixed libc subset the
runtime provides (`hypot`, `abs`, `toupper`, `strlen`, `atoi`, `memset`,
`memcpy`, `strtod`, declared with the C types of `examples/ffi_libc.ail` and
`examples/ffi_buffers.ail`), and compiles `std.mem` (`alloc`, `free`, `cstr`,
`from_cstr`, `cast`, `null`) over the runtime's own blocks. Any other
`extern c fn` is bound on its first call as `ffi.load.1-2` say, so the
executable then imports `dlopen` and `dlsym` from the system's C library; a
program that calls only the subset stays static. A native program does not
check `unsafe.ptr.5-6`: an access outside a block, or to a freed one, is
undefined as in C. *(ADR 0436, ADR 0844, ADR 0870)*

```ail tool=run,FILE expect=true rules=ffi.native.1
extern c fn isdigit(c i32) i32

fn main()
    unsafe
        print(isdigit(55) != 0)
```

## Loading

**ffi.load.1** — The library is loaded on the first call to one of its
functions, not when the program is checked or starts. A name with `/` or `.`
is the file as written. Any other name `n` is tried as the platform's shared
library, with and without a `lib` prefix: `n.so` and its versioned names (`.6`,
`.1`, `.0`, `.2`, `.3`) on Linux; `n.dylib`, `/usr/lib/n.dylib` and `/opt/homebrew/lib/n.dylib`
on macOS. A declaration without a library names a symbol of the running
program, which includes the C library. A library that cannot be loaded is a
run-time trap. *(ADR 0431)*

```ail tool=run,FILE status=1 expect=trap[ffi.load.1] rules=ffi.load.1
extern c "definitely_not_a_library" fn f(x int) int

fn main()
    unsafe
        print(f(1))
```

**ffi.load.2** — A symbol the loaded library does not have is a run-time trap
at the call. *(ADR 0431)*

```ail tool=run,FILE status=1 expect=trap[ffi.load.2] rules=ffi.load.2
extern c fn no_such_function_in_libc(x int) int

fn main()
    print("before")
    unsafe
        print(no_such_function_in_libc(1))
```

**ffi.load.3** — On Windows `extern c fn` is reported at run time as not
supported yet; the implementation needs `LoadLibraryA` and the Win64 calling
convention. *(ADR 0431, ADR 0004)*

## Linking (Linux)

**ffi.link.1** — On Linux, a program that declares or imports no C functions
MUST NOT depend on libc. A program that does is linked against the system
libc. *(ADR 0105, ADR 0020)*

## Runnable examples for the rules above

```ail error=ffi.decl.2
extern c fn abs(x i32) i32 = 3

fn main()
    print(1)
```

```ail error=ffi.decl.3
extern c fn strlen(s str) int

fn main()
    print(1)
```

```ail error=ffi.call.1
extern c fn abs(x i32) i32

fn main()
    print(abs(-5))
```

```ail rules=ffi.call.2,ffi.decl.2,ffi.link.1,ffi.map.1,ffi.map.2
extern c fn abs(x i32) i32

fn main()
    unsafe
        print(abs(-5))
```

```ail rules=ffi.map.1
extern c "libm" fn hypot(a float, b float) float

fn main()
    unsafe
        print(hypot(3.0, 4.0))
```

```ail norun rules=ffi.native.1
# norun: the process id differs on every run.
extern c fn getpid() int

fn main()
    unsafe
        print(getpid())
```

```ail norun rules=ffi.load.1
# norun: the library is loaded at the first call and the trap names every file tried.
extern c "nosuchlib_xyz" fn f() int

fn main()
    unsafe
        print(f())
```

```ail norun rules=ffi.load.2
# norun: the trap names the missing symbol at the call.
extern c fn no_such_symbol_xyz() int

fn main()
    unsafe
        print(no_such_symbol_xyz())
```

```ail norun rules=ffi.load.3
# norun: on Windows the call traps as "not supported yet"; on Linux and macOS the library
# "kernel32" can't be loaded, so the call traps too.
extern c "kernel32" fn GetTickCount() i32

fn main()
    unsafe
        print(GetTickCount())
```

```ail rules=ffi.decl.1
use c "stdlib.h" as libc

fn main()
    unsafe
        print(libc.abs(-5))
```
