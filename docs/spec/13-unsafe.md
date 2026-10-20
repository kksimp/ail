# 13. Unsafe

Chapter key: `unsafe`

> **Status:** Drafted (partial). Specified and implemented in the bootstrap:
> unsafe blocks, unsafe functions, package opt-in. Specified: the set of
> unsafe operations (C calls and raw pointers arrive with ADRs 0431–0435).
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

**unsafe.op.2** — Creating or copying a raw pointer value is not an unsafe
operation. *(ADR 0106)*

## Raw pointers and memory

**unsafe.ptr.1** — `*T` is a raw pointer type: an address with no ownership
and no lifetime. `T` is `int`, `float`, `bool`, a sized number, or another
pointer. A pointer is passed, compared with `==` and stored freely
(`unsafe.op.2`); it is not a number and does not convert to one. *(ADR 0106,
ADR 0433)*

**unsafe.mem.1** — `std.mem` (`std.mem.1`) allocates memory from C's own
allocator, so a block may be handed to C and freed there, or the reverse. Every
`std.mem` function is an unsafe operation. `from_cstr` of a null pointer, or of
a block of the program's own that has no NUL, is a run-time trap. *(ADR 0434)*

**unsafe.mem.2** — `mem.free` of a null pointer, or of a block of the program's
own that is already freed, is a run-time trap. Freeing a block that came from C
is the program's claim that C allocated it with `malloc`. *(ADR 0434)*
