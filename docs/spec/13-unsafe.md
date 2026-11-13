# 13. Unsafe

Chapter key: `unsafe`

> **Status:** Drafted (partial). Specified and implemented:
> unsafe blocks, unsafe functions, package opt-in, `unsafe driver` and
> `std.hw` (ADR 0820), also built natively (ADR 0835). Specified: the set of
> unsafe operations, C calls and raw pointers (ADRs 0431–0435).
> Not yet specified: the precise undefined-behaviour rules inside `unsafe`.

## Unsafe blocks and functions

**unsafe.block.1** — An unsafe operation (`unsafe.op.1`) is allowed only
inside an `unsafe` block. *(ADR 0102)*

**unsafe.block.2** — A function declared `unsafe fn` may be called only inside
an `unsafe` block. *(ADR 0102)*

**unsafe.block.3** — A package may contain `unsafe` blocks and `unsafe fn`
only if its manifest opts in: `unsafe allow` (or `unsafe driver`, `unsafe.hw.1`)
on its `package` line, or on the `dep` line that declares it (`mod.pkg.2`). A program with no `ail.pkg` may use
`unsafe`. The error is at the block or the `unsafe fn`. *(ADR 0102, ADR 0430)*

**unsafe.block.4** — An implementation MUST be able to list every `unsafe`
block and `unsafe fn` in a program (`ail api --ast`). *(ADR 0102, ADR 0430)*

**unsafe.block.5** — `unsafe` and an indented block is a statement. The block
is an ordinary block: its last expression is its value when it ends a body,
and `return` leaves the function. The body of an `unsafe fn` is not an
`unsafe` block. A function with an `unsafe` block has the effect `unsafe`
(`eff.infer.4`, `eff.pure.1`). *(ADR 0430)*

```ail rules=unsafe.block.5
unsafe fn twice(x int) int
    x * 2

fn safe_twice(x int) int
    unsafe
        twice(x)

fn main()
    print(safe_twice(21))
```

```ail error=unsafe.block.2
unsafe fn twice(x int) int
    x * 2

fn main()
    print(twice(3))
```

## Unsafe operations

**unsafe.op.1** — The unsafe operations are:
1. dereferencing a raw pointer `*T`, and pointer arithmetic;
2. converting a managed reference to a raw pointer, and pinning;
3. manual allocation and deallocation;
4. reinterpreting memory as another type;
5. calling an `unsafe fn` or a C function;
6. inline assembly and hardware intrinsics with preconditions.

*(ADR 0106)*

```ail error=unsafe.op.1
use std.mem

fn main()
    xs := mem.alloc(4, 0)
    print("ok")
```

```ail rules=unsafe.block.1,unsafe.op.2
use std.mem

fn main()
    unsafe
        xs := mem.alloc(4, 0)
        print("ok")
```

**unsafe.op.2** — Creating or copying a raw pointer value is not an unsafe
operation. *(ADR 0106)*

```ail rules=unsafe.op.2
use std.mem

fn null_ptr() *u8
    unsafe
        return mem.null()

fn main()
    q := null_ptr()
    r := q
    print(r.is_null())
```

## Raw pointers and memory

**unsafe.ptr.1** — `*T` is a raw pointer type: an address with no ownership
and no lifetime. `T` is `int`, `float`, `bool`, a sized number, or another
pointer. A pointer is passed, compared with `==` and stored freely
(`unsafe.op.2`); it is not a number and does not convert to one. *(ADR 0106,
ADR 0433)*

**unsafe.ptr.2** — A raw pointer `*T` has four methods. `p.read() T` loads a
`T` from the address and `p.write(v T)` stores one; `p.offset(n int) *T` is the
pointer `n * size_of(T)` bytes further (wrapping, and not checked until it is
used); `p.is_null() bool` is true for address 0. All but `is_null` are unsafe
operations (`unsafe.op.1`). Memory layout: `int`, `float` and pointers take 8
bytes, `bool` one byte holding 0 or 1, a sized number its width, little-endian
(ADR 0007); reads and writes need no alignment. *(ADR 0435)*

**unsafe.ptr.3** — A raw pointer has no other method; using one is a
compile-time error. *(ADR 0435)*

**unsafe.ptr.4** — Reading or writing through a null pointer (an address below
4096) is a run-time trap. *(ADR 0435)*

**unsafe.ptr.5** — Reading or writing past the end of a block that `std.mem`
allocated, including at its end exactly, is a run-time trap. Memory that did not
come from `std.mem` (a pointer from C, or an arbitrary address) is not checked:
that is what makes the access unsafe. *(ADR 0435)*

**unsafe.ptr.6** — Reading or writing a block that `mem.free` released is a
run-time trap, until `malloc` hands the same address out again. *(ADR 0435)*

**unsafe.mem.1** — `std.mem` (`std.mem.1`) allocates memory from C's own
allocator, so a block may be handed to C and freed there, or the reverse. Every
`std.mem` function is an unsafe operation. `from_cstr` of a null pointer, or of
a block of the program's own that has no NUL, is a run-time trap. *(ADR 0434)*

**unsafe.mem.2** — `mem.free` of a null pointer, or of a block of the program's
own that is already freed, is a run-time trap. Freeing a block that came from C
is the program's claim that C allocated it with `malloc`. *(ADR 0434)*

**unsafe.mem.3** — `mem.alloc(n, example)` returns a pointer to `n` elements
of the example's type, each set to the example's value; a negative `n`, or a
size that overflows, is a run-time trap. `mem.cast(p, example)` is the same
address as a pointer to the example's type (reinterpretation, `unsafe.op.1`),
`mem.null()` is the null `*u8`, and `mem.free(p)` accepts any pointer.
`mem.c_fn(f)` is the C entry of the declared function `f` as a `*u8`
(`ffi.cb.3`). *(ADR 0435, ADR 1074)*

```ail rules=unsafe.mem.1,unsafe.mem.3,unsafe.ptr.1,unsafe.ptr.2
use std.mem

fn main()
    unsafe
        xs := mem.alloc(4, 0)
        for i in 0..4
            xs.offset(i).write(i * 10)
        print(xs.offset(2).read())
        mem.free(xs)
```

## Hardware access

**unsafe.hw.1** — `std.hw` and the volatile pointer methods (`unsafe.hw.3`)
reach hardware. They are unsafe operations (`unsafe.op.1`) and also need
`unsafe driver` where `unsafe allow` would do for other unsafe code: the word on
the `package` line of `ail.pkg` or on the `dep` line that grants the package.
`unsafe driver` includes `unsafe allow`. A program with no `ail.pkg` needs
`--unsafe driver` on `ail run`, `ail check` or `ail build`. Otherwise the error
is at the use and names the manifest line to change. *(ADR 0820)*

**unsafe.hw.2** — `hw.at(addr, example)` is a `*T` at the fixed address `addr`,
where `T` is the type of `example` (`unsafe.ptr.1`). It is the only way from an
`int` to a pointer. A negative address is a run-time trap. *(ADR 0820)*

**unsafe.hw.3** — `p.read_volatile()` and `p.write_volatile(v)` read and write
a `*T` like `read` and `write` (`unsafe.ptr.2`), in one access of the width of
`T`. They are never merged, removed or reordered against each other. The
address must be a multiple of the width, or it is a run-time trap. The checks
of `unsafe.ptr.4–6` apply. *(ADR 0820)*

**unsafe.hw.4** — `hw.fence()` orders all earlier memory accesses before all
later ones. `hw.fence_io()` does the same for device accesses. Both are unsafe
operations. *(ADR 0820)*

**unsafe.hw.5** — A call of `std.hw` or of a volatile method gives the effect
`hw`, which propagates like `unsafe` (`eff.infer.3`). `@pure` rejects it
(`eff.pure.1`), and it orders the iterations of a loop (`conc.par.2`).
*(ADR 0820)*

```ail tool=run,--unsafe,driver,FILE expect=7 rules=unsafe.hw.2,unsafe.hw.3,unsafe.hw.4,unsafe.hw.5
use std.mem

fn main()
    unsafe
        xs := mem.alloc(4, 0.to_u32())
        xs.offset(1).write_volatile(7.to_u32())
        hw.fence()
        print(xs.offset(1).read_volatile())
        mem.free(xs)
```

```ail error=unsafe.hw.1
fn main()
    unsafe
        p := hw.at(4096, 0)
        print(p.is_null())
```

```ail pkg=driver_denied tool=check,DIR status=2 expect=unsafe.hw.1 rules=unsafe.hw.1
```

```ail norun flags=--unsafe,driver rules=unsafe.hw.3
fn main()
    unsafe
        p := hw.at(4097, 0)
        print(p.read_volatile())
```

## Runnable examples for the rules above

```ail error=unsafe.ptr.3
fn f(p *int) int
    p.len()

fn main()
    print(1)
```

```ail tool=api,--ast,FILE expect="unsafe_fn":true rules=unsafe.block.4
unsafe fn twice(x int) int
    x * 2

fn main()
    unsafe
        print(twice(3))
```

```ail rules=unsafe.op.2
use std.mem
fn make() *int
    unsafe
        mem.alloc(1, 7)

fn pass(p *int) *int
    p

fn main()
    q := pass(make())
    unsafe
        print(q.read())
        mem.free(q)
```

```ail rules=unsafe.ptr.1,unsafe.ptr.2
use std.mem
fn same(a *int, b *int) bool
    a == b

fn main()
    unsafe
        xs := mem.alloc(2, 1)
        print(same(xs, xs.offset(1)))
        print(xs.offset(1).read())
        mem.free(xs)
```

```ail norun rules=unsafe.ptr.4
use std.mem
fn main()
    unsafe
        print(mem.null().read())
```

```ail norun rules=unsafe.ptr.5
use std.mem
fn main()
    unsafe
        xs := mem.alloc(4, 0)
        print(xs.offset(4).read())
```

```ail norun rules=unsafe.ptr.6
use std.mem
fn main()
    unsafe
        xs := mem.alloc(4, 0)
        mem.free(xs)
        print(xs.read())
```

```ail norun rules=unsafe.mem.2
use std.mem
fn main()
    unsafe
        xs := mem.alloc(4, 0)
        mem.free(xs)
        mem.free(xs)
```

```ail norun rules=unsafe.mem.3
use std.mem
fn main()
    unsafe
        xs := mem.alloc(-1, 0)
        print(1)
```

```ail skip=context rules=unsafe.block.3
# A package's manifest opts in: `package app unsafe allow` (or `dep NAME ... unsafe allow`).
# Without it, an `unsafe` block in the package is an error at the block. The package
# cases under tests/cases/packages/ cover the manifest rules (mod.pkg.2).
```
