# 13. Unsafe

Chapter key: `unsafe`

> **Status:** Drafted (partial). Specified and implemented in the bootstrap:
> unsafe blocks, unsafe functions, package opt-in. Specified: the set of
> unsafe operations, C calls and raw pointers (ADRs 0431–0435).
> Not yet specified: the precise undefined-behaviour rules inside `unsafe`.

## Unsafe blocks and functions

**unsafe.block.1** — An unsafe operation (`unsafe.op.1`) is allowed only
inside an `unsafe` block. *(ADR 0102)*

**unsafe.block.2** — A function declared `unsafe fn` may be called only inside
an `unsafe` block. *(ADR 0102)*

**unsafe.block.3** — A package may contain `unsafe` blocks and `unsafe fn`
only if its manifest opts in: `unsafe allow` on its `package` line, or on the
`dep` line that declares it (`mod.pkg.2`). A program with no `ail.pkg` may use
`unsafe`. The error is at the block or the `unsafe fn`. *(ADR 0102, ADR 0430)*

**unsafe.block.4** — An implementation MUST be able to list every `unsafe`
block and `unsafe fn` in a program (`ail api --ast`). *(ADR 0102, ADR 0430)*

**unsafe.block.5** — `unsafe` and an indented block is a statement. The block
is an ordinary block: its last expression is its value when it ends a body,
and `return` leaves the function. The body of an `unsafe fn` is not an
`unsafe` block. A function with an `unsafe` block has the effect `unsafe`
(`eff.infer.4`, `eff.pure.1`). *(ADR 0430)*

```ail
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

```ail
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
`mem.null()` is the null `*u8`, and `mem.free(p)` accepts any pointer. *(ADR 0435)*

```ail
use std.mem

fn main()
    unsafe
        xs := mem.alloc(4, 0)
        for i in 0..4
            xs.offset(i).write(i * 10)
        print(xs.offset(2).read())
        mem.free(xs)
```
