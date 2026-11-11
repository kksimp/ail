# 14. FFI and C interoperability

Chapter key: `ffi`

> **Status:** Drafted (partial). Specified and implemented: `extern c fn`
> declarations, `use c "header.h"` (`ffi.decl.1`, a subset reader, ADR 0926; constants, enums and function-pointer types `ffi.decl.4-5`, ADR 1010), the
> type mapping, loading, and calls on Linux, macOS and Windows. Exports are specified (`ffi.export.1-3`); the library output is not built yet. Variadic `extern c fn` calls are specified (`ffi.var.1-4`). Not yet specified: callbacks,
> structs by value, and header search configuration.

## Declarations

**ffi.decl.1** — `use c "header.h" as name` makes the C declarations of
`header.h` available under `name`. The header is parsed by the
implementation's own C header parser. *(ADR 0103, ADR 0926)*

The header is found beside the file that imports it, else in the system's
include directories; a header that is not found is error `ffi.decl.1`. Only
the header's own text is read: `#include` is not followed and no `#if` or
macro is evaluated. Each function prototype whose types `ffi.decl.3` allows
becomes `extern c fn name(...)` under `name`, with `char` as `u8` behind a
pointer and `i8` otherwise, `int` as `i32`, `long` as `i64` (`i32` on
Windows), `double` as `float`, `float` as `f32`, `size_t` as `u64`, and any
pointer to a type that is not a scalar as `*u8`. A prototype the FFI cannot
carry (a variadic one with no named parameter, a struct by value, a function-pointer parameter, too many
register arguments) is left out, and using its name is the ordinary
`mod.use.2` error. Calling a declaration is unsafe (`ffi.call.1`).

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

## Exports

**ffi.export.1** — `pub export c fn name(params) Result` followed by a body
declares an AIL function that C can call. `export c "symbol" fn` names the C
symbol; without a string it is the AIL name. The declaration is at the top
level and starts with `pub`; `export` with anything but `c`, or without `pub`,
is error `ffi.export.1`. Inside AIL it is an ordinary function. *(ADR 0950)*

**ffi.export.2** — An exported function has the signature of an `extern c fn`
(`ffi.decl.3`): C-layout types only, no `mut` parameter, at most six
integer-like and eight float parameters, and no type, `self` or type
parameters. *(ADR 0950)*

```ail error=ffi.export.2
pub export c fn total(xs List[int]) int
    xs.len()

fn main()
    print(total([1, 2]))
```

```ail tool=run,FILE expect=7 rules=ffi.export.1
pub export c "ail_add" fn add(a i32, b i32) i32
    a + b

fn main()
    print(add(3, 4))
```

**ffi.export.3** — `ail header FILE [-o OUT.h]` prints a C header of the
exported functions in source order: `int` is `int64_t`, `float` is `double`,
`bool` is `bool`, sized numbers are the `<stdint.h>` types, a pointer to a
scalar is a pointer to its C type and any other pointer is `void *`. The
program is checked first. Building the library itself (`ail build --lib`)
is not yet specified. *(ADR 0950)*
**ffi.decl.4** — A header import also reads constants. An object-like
`#define NAME body` whose body is an integer expression, a float literal or a
string literal becomes `pub NAME := value`, and each enumerator of an `enum`
(also `typedef enum`) becomes an integer constant, numbered as C numbers it.
An integer expression may use integer and character literals, constants read
earlier in the same header, parentheses, unary `-`, `+` and `~`, and the
binary `*`, `/`, `%`, `+`, `-`, `<<`, `>>`, `&`, `^` and `|`, with C's
precedence. An integer constant is an `int` (use `.to_i32()` where a C
parameter is `i32`). A function-like macro, a body that needs a cast, `sizeof`
or a name the header does not define, a value beyond 2^61 and an enumerator
after one that could not be read are left out, like a prototype the FFI cannot
carry. *(ADR 1010)*
**ffi.decl.5** — A function-pointer parameter (`int (*cb)(int)`) and a
`typedef` of a function-pointer type are `*u8`. Calling a function that takes
one is as unsafe as any C call (`ffi.call.1`); AIL does not yet make a C
callback, so the value is a pointer the program got from C or `0`. A union
behind a pointer is `*u8` like a struct. A union or struct by value, a bit-field
struct's fields and a function that returns a function pointer are left out
until `@layout(c)` (ADR 0952) maps struct fields. *(ADR 1010)*

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

## Variadic functions

**ffi.var.1** — `...` ends the parameter list of an `extern c fn` and follows
at least one named parameter: `extern c fn snprintf(buf *u8, size u64, fmt *u8, ...) i32`.
It is an error anywhere else. Only a call by the declared name takes the variadic
path for now; a header-imported variadic called as `lib.f(...)` is not yet
verified. *(ADR 1031)*

**ffi.var.2** — A call passes the named arguments as in any call and then any
number of further arguments, by value, with no label and no `mut`. The whole
call has at most six integer-like and eight float arguments (`ffi.decl.3`).
Calling is unsafe (`ffi.call.1`). *(ADR 1031)*

**ffi.var.3** — A further argument keeps its own type, which MUST have a C
layout (`ffi.decl.3`): `int` and the sized integers and `bool` are passed as C
`int` or `long long` would be (narrower than 64 bits is extended by its
signedness, `ffi.map.1`), `float` as `double`, a raw pointer as a pointer. An
`f32` is refused, since C passes it as a `double`: convert it first. A `str` is
refused; make a C string with `mem.cstr`. *(ADR 1031)*

**ffi.var.4** — The call follows the platform's variadic convention. System V
x86-64 sets `al` to an upper bound of the vector registers used. Win64 also
puts each of the first four float arguments in the integer register of its
position. Linux arm64 uses the plain AAPCS64 registers. Apple arm64 passes
every variadic argument on the stack, one 8-byte slot each. *(ADR 1031)*

```ail error=ffi.var.3
use std.mem

extern c fn snprintf(buf *u8, size u64, fmt *u8, ...) i32

fn main()
    unsafe
        buf := mem.alloc(8, 0.to_u8())
        snprintf(buf, 8, buf, "text")
```

```ail error=ffi.var.1
extern c fn bad(...) i32

fn main()
    print(1)
```

## Type mapping

**ffi.map.1** — `int` is C `int64_t`; `float` is `double`; `f32` is `float`;
`bool` is C `bool` (one byte); `i8`...`u64` are the fixed-width C integers of
the same name; `*T` is a pointer. A parameter narrower than 64 bits is passed
extended to 64 bits by its signedness. *(ADR 0432)*

```ail tool=run,FILE expect=5 rules=ffi.map.1
use std.mem

extern c fn strlen(s *u8) int

fn main()
    unsafe
        p := mem.cstr("hello")
        print(strlen(p))
        mem.free(p)
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
extern c fn toupper(c i32) i32

fn main()
    unsafe
        print(toupper(97) == 65)
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

**ffi.load.3** — On Windows the library is opened with `LoadLibraryA` and the
symbol found with `GetProcAddress`, and the call uses the Win64 convention.
`c`, `libc`, `m`, `libm` and `msvcrt` name the Universal CRT (`ucrtbase.dll`);
any other name is tried as `NAME.dll`, then `libNAME.dll`. A library or symbol
that can't be found traps with `ffi.load.1` or `ffi.load.2`. *(ADR 0431, ADR 0004, ADR 0980)*

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
# norun: on Windows the call runs through `LoadLibraryA`; on Linux and macOS the library
# "kernel32" can't be loaded, so the call traps.
extern c "kernel32" fn GetTickCount() i32

fn main()
    unsafe
        print(GetTickCount())
```

```c
/* mylib.h */
int abs(int x);
int toupper(int c);
double hypot(double a, double b);
int printf(const char *format, ...);   /* kept: variadic (ffi.var.1) */
```

```ail tool=run,DIR pkg=ffi_header expect=5.0 rules=ffi.decl.1
use c "mylib.h" as lib

fn main()
    unsafe
        print(lib.abs(-5))
        print(lib.hypot(3.0, 4.0))
```

```c
/* consts.h */
#define BUF 4096
#define FLAGS (1 << 3 | 0x10)
#define NEG -7
#define PI 3.5
#define GREETING "hi"
enum color { RED, GREEN = 5, BLUE };
typedef enum { SLOW = 1, FAST = 1 << 1 } speed_t2;
typedef int (*visit_fn)(int);
int abs(int x);
void walk(visit_fn f, void *user);   /* visit_fn is *u8 */
```

```ail tool=run,DIR pkg=ffi_header_consts expect=4096 rules=ffi.decl.4,ffi.decl.5
use c "consts.h" as k

fn main()
    unsafe
        print(k.BUF)
        print(k.FLAGS)
        print(k.abs(k.NEG.to_i32()))
        print(k.PI)
        print(k.GREETING)
        print(k.BLUE)
        print(k.FAST)
```

```ail tool=check,DIR pkg=ffi_header_missing status=2 expect=ffi.decl.1 rules=ffi.decl.1
use c "no_such_header.h" as lib

fn main()
    print("hi")
```
