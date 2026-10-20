# Native ABI: calling convention, value layout, runtime, traps

> **Non-normative design note,** 2026-10-20, lanes 120 (backend) and 122
> (runtime) of the [finish plan](finish-plan.md). The decided parts are
> ADRs 0384–0393 (backend) and 0404–0411 (runtime). Keep this note current
> when either side changes.

## 1. Scope

The native subset (ADR 0384): `int`, `float`, `bool`, string literals,
lists of numbers, locals, arithmetic, comparisons, `if`, `while`, `for`
over ranges and lists, calls, `return`, `print`. Everything here is for
code the AIL compiler under `compiler/` generates and for the runtime in
`compiler/rt/`, which is compiled by the same backend from AIL source.
There is no libc and no external linker or assembler (ADR 0010, ADR 0017).

## 2. Targets

| Target | Object | Entry | Syscalls |
|---|---|---|---|
| Linux x86-64 (x86-64-v3) | static ELF64, no interpreter | `_start` | `syscall`; number in `rax`; args `rdi rsi rdx r10 r8 r9` |
| Linux ARM64 (ARMv8.2-A) | static ELF64 | `_start` | `svc #0`; number in `x8`; args `x0`–`x5` |
| macOS ARM64 | Mach-O, `LC_MAIN`, dyld and libSystem loaded but unused, ad hoc signed | `_main` | `svc #0x80`; number in `x16`; args `x0`–`x5`; carry set on error |

Syscall numbers the runtime uses:

| Call | Linux x86-64 | Linux ARM64 | macOS ARM64 |
|---|---|---|---|
| `write` | 1 | 64 | 4 |
| `mmap` | 9 | 222 | 197 |
| `munmap` | 11 | 215 | 73 |
| `exit_group` / `exit` | 231 | 94 | 1 |

macOS raw syscalls are not a stable Apple interface, but `write`, `mmap`
and `exit` have not changed numbers since 10.0; the runtime confines them to
one function per call (`rt.sys_*`) so moving to libSystem imports later is
local. Windows: not targeted yet (ADR 0004 asks for awareness): the runtime
would call `kernel32` through an import table, so `rt.sys_*` is the only
layer that changes.

## 3. Calling convention (AIL to AIL)

The registers follow the platform C convention so FFI (lane 126) needs no
shuffling, with one difference on x86-64: **`xmm8`–`xmm13` are
callee-saved** in AIL code, mirroring AAPCS64's `d8`–`d15`, so float values
live across calls stay in registers on both targets. An `extern` call saves
them itself.

| | x86-64 | ARM64 |
|---|---|---|
| int, bool, pointer arguments | `rdi rsi rdx rcx r8 r9` | `x0`–`x7` |
| float arguments | `xmm0`–`xmm7` | `d0`–`d7` |
| result | `rax` or `xmm0` | `x0` or `d0` |
| callee-saved | `rbx rbp r12`–`r15`, `xmm8`–`xmm13` | `x19`–`x28`, `x29`, `x30`, `d8`–`d15` |
| backend scratch (never allocated) | `rax rdx r10 r11`, `xmm14 xmm15` | `x9 x10 x16 x17`, `d30 d31` |
| stack | 16-byte aligned at each call | 16-byte aligned always |

Arguments beyond the registers are not in the subset yet (a function with
more than 6 int or 8 float parameters is rejected, `native.subset.1`).

## 4. Value layout

| AIL type | Native form |
|---|---|
| `int` | 64-bit two's complement in a GPR |
| `float` | IEEE 754 binary64 in a vector register |
| `bool` | 0 or 1 in a GPR (full width) |
| `str` (literals) | pointer to a static record: `bytes i64`, `chars i64`, then the UTF-8 bytes |
| `List[int]`, `List[float]`, `List[bool]` | pointer to a heap header: `len i64`, `cap i64`, `data ptr`; elements are 8 bytes each |

Lists keep value semantics (own.val.*): binding or assigning a list from a
variable or element copies it (`rt.list_copy`); a fresh value (a call
result, a literal, `List[int]()`) is not copied; a read-only parameter is
not copied because the callee can't change it; a `mut` parameter passes the
header pointer, so the callee's changes are the caller's.

## 5. Memory

`rt.alloc(bytes)` bumps a pointer through 1 MiB-or-larger chunks from
`mmap`. Nothing is freed yet (no GC in the subset); a list's growth
doubles its capacity and copies.

The runtime keeps its state (output buffer pointer and length, heap
cursor and limit) in 8-byte global slots in a writable data segment,
reached with the `global(k)` primitive.

## 6. Runtime primitives

The runtime is ordinary AIL in `compiler/rt/` plus these primitives, which
the backend expands inline. They are declared in `compiler/rt/prim.ail`
(bodies `todo`) so the bootstrap's checker accepts the runtime; calling
them anywhere except `compiler/rt/` is an error (`native.prim.1`).

| Primitive | Meaning |
|---|---|
| `load64(a)`, `store64(a, v)` | 8-byte load and store |
| `load8(a)`, `store8(a, v)` | byte load (zero-extended) and store |
| `loadf(a)`, `storef(a, x)` | float load and store |
| `syscall(n, a, b, c, d, e, f)` | raw system call per §2; result in an `int` |
| `global(k)` | address of global slot `k` |
| `str_addr(s)` | address of a string literal's record |
| `float_bits(x)`, `bits_float(n)` | reinterpret between `float` and `int` |

## 7. Traps

A trap prints what the interpreter prints first, then exits with status 1
(ADR 0035):

```text
PATH:LINE:COL: trap[RULE]: MESSAGE
  LINE | source line
       |   ^
```

The message has the interpreter's text with the run-time values
(`` `9223372036854775807 + 1` overflows `int` ``, `index 5 is out of
bounds for a list of length 2`, ``division by zero in `5 / 0` ``). The
interpreter's frame lines (`at 3:13 in main(); x = …`) are not printed
yet: the native code keeps no frame map. Each trap site is a record in
read-only data (rule, message pieces, position, source line), and the
code jumps to a stub that passes the site and the operand values to
`rt.trap_*`, which flushes stdout first so the two streams interleave as
in the interpreter.

## 8. Output

`print` appends to a 64 KiB buffer that is flushed when full, before a
trap, and at exit. Integers print in decimal; floats as the interpreter
prints them (lane 122: shortest round-trip, Python's form); `bool` as
`true`/`false`; lists as `[1, 2, 3]`.
