# 13. Unsafe

Chapter key: `unsafe`

> **Status:** Drafted (partial). Specified: unsafe blocks, unsafe functions,
> package opt-in, and the set of unsafe operations. Not yet specified: the
> precise undefined-behaviour rules inside `unsafe`.

## Unsafe blocks and functions

**unsafe.block.1** — An unsafe operation (`unsafe.op.1`) is allowed only
inside an `unsafe` block. *(ADR 0102)*

**unsafe.block.2** — A function declared `unsafe fn` may be called only inside
an `unsafe` block. *(ADR 0102)*

**unsafe.block.3** — A package may contain `unsafe` blocks only if its
manifest opts in to unsafe code. *(ADR 0102)*

**unsafe.block.4** — An implementation MUST be able to list every `unsafe`
block and `unsafe fn` in a program. *(ADR 0102)*

```ail
pub fn Buffer.read_u32(self, at int) u32
    if at + 4 > self.len()
        trap("out of range")
    unsafe
        load_u32(self.ptr + at)
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
