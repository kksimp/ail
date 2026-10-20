# Native ABI and runtime

> **Non-normative design note,** 2026-10-20 (lane 122 of the
> [finish plan](finish-plan.md); shared with lane 120, the native backend, and
> lane 126, FFI). Decided parts: ADRs
> [0404](../decisions/0404-native-calling-convention-and-value-layouts.md) to
> [0409](../decisions/0409-float-text-by-exact-digit-generation.md). Lane 120
> may amend this note; keep it current.

The native executables of `ail build` link no libc. The **runtime** is AIL code
under `compiler/rt/` that the backend compiles into every executable. Runtime
files carry the Runtime Exception header (see [Licensing](#licensing)).

## 1. Targets and calling convention

| Target | Object | Int args | Float args | Return | Stack |
|---|---|---|---|---|---|
| Linux x86-64 | ELF | rdi rsi rdx rcx r8 r9 | xmm0-7 | rax, xmm0 | 16-byte aligned at call |
| Linux ARM64 | ELF | x0-x7 | d0-d7 | x0, d0 | 16-byte aligned |
| macOS ARM64 | Mach-O | x0-x7 | d0-d7 | x0, d0 | 16-byte aligned |
| Windows x86-64 | PE | Win64: rcx rdx r8 r9, 32-byte shadow space (ADR 0598) | xmm0-3 | rax, xmm0 | 16-byte aligned at call |

AIL functions use the platform's C convention (System V AMD64, AAPCS64), so an
`extern` call (lane 126, `ffi.call.1`) needs no thunk. Arguments beyond the
registers go on the stack, 8 bytes each, in order. Callee-saved registers are
the platform's, plus **xmm8–xmm13 on x86-64** between AIL functions (ADR 0385;
an `extern` call treats them as clobbered). No variadics. Structs are passed
by pointer.

On x86-64 the code generator keeps rax, rdx, r10, r11, xmm14 and xmm15 as
scratch; the allocator (ADR 0386) uses rsi, rdi, r8, r9, rcx, rbx, r12–r15,
xmm0–xmm13. On ARM64 (ADR 0388) the scratch registers are x16, x17, x8, d30
and d31 (x0 between calls); x18 is never used; the allocator uses x1–x7,
x9–x15, x19–x28 and d0–d29. Mach-O executables are loaded by dyld, name
libSystem but import nothing, and are signed ad hoc by the backend
(ADR 0389).

## 2. Value layouts

All values are 64-bit and little-endian (ADR 0007). In registers and in list
elements each scalar takes 8 bytes, except that a `List[bool]` element takes
one byte, 0 or 1 (ADR 0391).

| Type | Layout |
|---|---|
| `int` | `i64` |
| `float` | `f64` (IEEE 754 binary64) |
| `bool` | `i64`, 0 or 1 |
| `char` | `i64`, a Unicode scalar value |
| `str` | pointer to a **string object** |
| `List[int]`, `List[float]`, `List[bool]` | pointer to a **list object** |

**String object** (immutable, 16-byte aligned):

```text
+0   u64  bytes    length in UTF-8 bytes
+8   u64  chars    length in characters  (str.len(), ADR 0208, O(1))
+16  u8[] data     UTF-8, `bytes` long, no terminator
```

A literal is a string object in read-only data. `chars` is computed once when
the object is built, which makes `s.len()` free and byte indexing impossible.

**List object** (a value; copied on `mut` aliasing by the compiler, ADR 0060):

```text
+0   u64  len     elements in use
+8   u64  cap     elements allocated
+16  ptr  data    cap * 8 bytes (cap bytes for List[bool]), separate allocation
```

`data` is separate so `push` can grow without moving the header the caller
holds. Out-of-range reads and writes trap `type.coll.4` (the interpreter's rule
for list indexing).

**Heap object header** (ADR 0392). Objects allocated with `rt.obj_new(size,
kind)` have an 8-byte word just before the address the program holds, so a
pointer is to the first field:

```text
-8   u64  header  bits 0-7 kind, bit 8 mark, bits 16-63 size in bytes
+0        fields  `size` bytes
```

| kind | object |
|---:|---|
| 1 | string |
| 2 | list header |
| 3 | list data |
| 4 | struct |
| 5 | enum payload |
| 6 | closure |
| 7 | map |
| 8 | set |
| 9 | bytes |
| 10 | boxed optional or result |

Strings and lists don't carry the header yet; lane 171 moves every
allocation to `rt.obj_new` with the collector.

**Struct object** (ADR 0392): one 8-byte slot per field, in declaration
order (a `float` as its bits, a `bool` as 0 or 1, a string, list or struct
as a pointer). A struct is a value: reading one from a place to bind,
store, push or return it copies it, deeply through the lists and structs
it holds (a generated `copy.T` function); fields change in place.

## 3. Entry and exit

The object writer points the entry at `_start` (ELF) or `_main` (Mach-O,
`LC_MAIN`). On Linux x86-64 `_start` is emitted by the code generator: clear
`rbp`, align the stack, `call rt.init`, `call main`, `rt.exit(0)` (ADR 0387).
The steps:

1. take the stack pointer, set up the heap (§5), save `argc`/`argv` for
   `env.args()` (later);
2. call the program's `main`;
3. `exit(0)`.

`exit(code)` flushes the stdout buffer (§6) first. A trap exits with **1**
(`err.trap.3`).

## 4. Syscalls (the only OS interface)

| Operation | Linux x86-64 | Linux ARM64 | macOS ARM64 |
|---|---|---|---|
| instruction | `syscall` | `svc #0` | `svc #0x80` |
| number register | rax | x8 | x16 |
| args | rdi rsi rdx r10 r8 r9 | x0-x5 | x0-x5 |
| result | rax (`-errno` on error) | x0 (`-errno`) | x0, carry set = error (x0 is errno) |
| `exit` | 60 (`exit_group` 231) | 93 (`exit_group` 94) | 1 |
| `write` | 1 | 64 | 4 |
| `mmap` | 9 | 222 | 197 |
| `munmap` | 11 | 215 | 73 |
| `read` | 0 | 63 | 3 |

`mmap` flags: `PROT_READ|PROT_WRITE` = 3; `MAP_PRIVATE|MAP_ANONYMOUS` = `0x22`
on Linux and `0x1002` on macOS (`MAP_ANON` is `0x1000`). fd `-1`, offset 0.

Linux clobbers `rcx` and `r11` on x86-64. The shim is four functions
(`sys_exit`, `sys_write`, `sys_mmap`, `sys_read`) so each target is one file
(`compiler/rt/sys_*.ail`).

**macOS risk.** Apple documents `libSystem` as the stable interface and
reserves the right to change raw syscall numbers. The raw route works on
current macOS ARM64 and is what the first slice uses; the fallback is to import
`_write`, `_exit` and `_mmap` from `libSystem` through the dyld stubs lane 126
needs for `extern` anyway. Only the four shim functions change (ADR 0406).

## 5. Allocator

A bump allocator over `mmap`ed chunks: first chunk 1 MiB, each later chunk
`max(request, 2 * previous)`, 16-byte alignment, zeroed by the kernel, never
freed. `alloc(n)` aborts with trap `run.limit.2` if `mmap` fails. This is
the interim for the GC (ADR 0407).

## 6. Output

`print` appends to a 64 KiB stdout buffer and flushes when full, at exit, at a
trap, and before reading stdin. Standard error is unbuffered. The interpreter's
text is the oracle: integers in decimal with a leading `-`; floats per
`type.num.7` (ADR 0409); strings as is.

## 7. Traps

**As built (lane 120, ADR 0385).** A failed check branches to a stub at the
end of its function, which calls
`rt.trap(kind, a, b, head, op, tail)`: `kind` picks the message (0 overflow,
1 division by zero, 2 negation, 3 index, 4 float to `int`, 5 `abs`, 6 shift
count), `a` and `b` are the operands (for 4, the float's bits), `head` is the
constant `PATH:LINE:COL: trap[RULE]: `, `op` the operator, and `tail` the
constant source excerpt (`  L | line` and the caret line). The output is the
interpreter's first line and excerpt, byte for byte; the snapshot lines
after them are not printed. Runtime code has no checks: its arithmetic wraps.

The trap-record design below was the first draft; a record can't hold the
operands the interpreter's messages contain.

`rt_trap(rec)` takes a pointer to a **trap record** the compiler emits per
trap site, then writes to standard error and exits 1:

```text
+0   ptr  file      string object, the source path
+8   i64  line
+16  i64  col
+24  ptr  rule      string object, e.g. "expr.arith.3"
+32  ptr  message   string object, e.g. "division by zero in `1 / 0`"
```

Output, byte for byte the first line the interpreter prints:

```text
FILE:LINE:COL: trap[RULE]: MESSAGE
```

The interpreter then adds a source excerpt and a stack trace. The native
version doesn't yet (frame metadata is a later ABI addition, ADR 0405). The
differential harness compares exit status, the rule ID and the first line;
`.trap` files hold the rule ID.

Runtime-raised traps use records the runtime owns: `type.coll.4` (index out of
range), `run.limit.2`. Compiler-emitted checks: `expr.arith.3` (division
by zero), `expr.arith.1` (overflow), per `expr.arith` in spec chapter 5.

## 8. What the runtime provides to compiled code

**As built (lane 120, ADR 0387).** `compiler/rt/native/core.ail`, compiled
with every program, provides `rt.init`, `rt.exit`, `rt.alloc`, `rt.trap`,
`rt.out_str`, `rt.out_int`, `rt.out_float`, `rt.out_bool`, `rt.out_nl`,
`rt.out_list_int`/`_float`/`_bool`, `rt.list_new`, `rt.list_push`,
`rt.list_pushf` and `rt.list_copy`; `print` writes its pieces straight into
the buffer instead of building a string. List indexing and `len` are inline.
The runtime is written over the primitives of `compiler/rt/native/prim.ail`
(`load64`, `store64`, `load8`, `store8`, `loadf`, `storef`, `syscall`,
`global`, `str_addr`, `list_addr`, `list_at`, `float_bits`, `bits_float`,
`target`), which the code generator expands inline and only runtime code may
call. Its float text is lane 122's algorithm (ADR 0409), on lists of limbs.

The table below is the first draft:

| Symbol | Meaning |
|---|---|
| `rt_start` | entry, §3 |
| `rt_exit(code)` | flush, then exit |
| `rt_alloc(n)` | §5 |
| `rt_trap(rec)` | §7 |
| `rt_str_new(ptr, bytes)` | build a string object, computing `chars` |
| `rt_str_cat(a, b)` | concatenate |
| `rt_str_int(n)`, `rt_str_float(x)` | number to string object |
| `rt_print_str(s)` | append `s` and a newline to the stdout buffer |
| `rt_list_new(cap)`, `rt_list_push(l, v)`, `rt_list_get(l, i)`, `rt_list_set(l, i, v)`, `rt_list_len(l)`, `rt_list_copy(l)` | §2 |

The AIL source of record (`compiler/rt/*.ail`) is today written against a
**memory model** (a `List[int]` of bytes and word-addressed helpers) so it runs
and is tested under the interpreter. When lane 126 lands raw pointers the same
modules switch their load/store helpers to `unsafe` pointer operations; the
algorithms don't change (ADR 0404).

## 9. FFI hook (lane 126, ADRs 0431–0436)

The interpreter already calls C (`extern c fn`, `docs/spec/14-ffi.md`). This
section is the plan for compiled programs. Nothing here is built yet; until it
is, `ail build` rejects a program that declares an `extern c fn` or uses
`std.mem` allocation with `ffi.native.1` (spec `ffi.native.1`).

**Source of truth for the call shape.** The checker already limits a C call to
six integer-like and eight float arguments (`ffi.decl.3`) and types it with
`int`/`float`/`bool`/sized numbers/`*T` (`ffi.map.1`). A native call is
therefore the platform's register convention of §1 with no thunk and no stack
arguments:

| Step | Rule |
|---|---|
| Arguments | integer-like in order into the integer registers, `float` and `f32` in order into the float registers (`f32` in the low half); narrow integers are extended to 64 bits by signedness first (`movsx`/`movzx`, `sxtb`/`uxtb`...), because Apple's ARM64 convention makes the caller extend |
| Result | integer register or float register; a narrow result is truncated to its declared type and extended again (`ffi.map.2`); `bool` is the low byte `!= 0` |
| Registers | caller-saved registers are assumed clobbered across the call |
| Stack | 16-byte aligned at the call (§1) |
| Not supported | variadic functions, structs by value, stack arguments, callbacks (`ffi.decl.3`, ADR 0432) |

**Resolution: static or dynamic.** A symbol the runtime provides (`rt_*`) is
a direct call. Any other symbol is **dynamic**, from the library written in the
declaration (`extern c "libm" fn`) or from libc when none is.

| Target | What the writer emits |
|---|---|
| Linux x86-64, ARM64 (ELF) | `PT_INTERP` (the system loader), `DT_NEEDED` for each library, a `.got` slot per symbol bound at load time (`R_X86_64_GLOB_DAT`, `R_AARCH64_GLOB_DAT`; `DT_FLAGS` `BIND_NOW`, so no lazy PLT), calls as `call [rip+slot]` / `ldr x16, slot; blr x16` |
| macOS ARM64 (Mach-O) | `LC_LOAD_DYLIB` per library (`/usr/lib/libSystem.B.dylib` always), chained fixups for the `__got` slots, same `ldr`/`blr` call; library names resolve as in `ffi.load.1` |
| Windows x86-64 | stub, `plat.win.1` and `ffi.load.3` |

A program with **no** `extern` links no libc and has no `PT_INTERP`
(`ffi.link.1`, ADR 0105): the static executable of §3 is unchanged. A program
with one becomes a dynamic executable; its entry is still `rt_start`, which
does not go through `__libc_start_main`. That is safe for the pure functions
the language targets (`strlen`, `sqrt`, `atoi`), because the loader has already
initialised libc and its thread-local storage. It is not enough for libc's
buffered stdio, so `rt_exit` calls `fflush(0)` through the dynamic symbol when
libc is linked, before the `exit` syscall. This is an open risk until a
differential test with `printf` passes on both Linux ARM64 and macOS.

**Memory.** `mem.alloc`, `mem.cstr` and `mem.free` call libc `malloc` and
`free` when libc is linked, so a block can cross the boundary in either
direction (`unsafe.mem.1`); without libc they use `rt_alloc` and `free` is a
no-op until the collector exists (§5). `read`, `write` and `offset` compile to
a load, a store and an `add` of `n * size_of(T)`. The interpreter's block table
(`unsafe.ptr.4–6`) does not exist natively: only a null access is trapped, by
a guard page and a signal handler the runtime installs, reporting
`unsafe.ptr.4`; out-of-block and freed accesses are undefined behaviour, as in
C, and the differential harness runs them under the interpreter only.

**Runtime Code.** The call stubs, the GOT slots and `rt_exit`'s flush belong
to the executable and fall under the Runtime Exception (ADR 0510); the
interpreter's `dlopen` glue (`bootstrap/src/interp/ffi.rs`) is toolchain code
and does not.

**Order of work (lane 120 and lane 122).** (1) `compiler/rt/` switches its
word-addressed helpers to `unsafe` pointer methods once lane 122 has the
interpreter-side tests (ADR 0435 is merged); (2) the ELF writer gains
`PT_INTERP`/`DT_NEEDED`/GOT and the first `extern` call; (3) the Mach-O writer
follows in lockstep (ADR 0008); (4) `ail build` drops `ffi.native.1` and the
selftest diffs `examples/ffi_libc.ail` and `examples/ffi_buffers.ail` between
the interpreter and the native executable on both platforms.

## 10. Threads, tasks and atomics

**As built (lane 178, ADRs 0599–0601).** `compiler/rt/native/task.ail` is part
of the runtime of a program whose source mentions `spawn `, `par for` or
`Atomic`; `notask.ail` otherwise. Linux x86-64 and ARM64, and macOS ARM64 through §11.

- **Threads.** `clone` with `CLONE_VM|FS|FILES|SIGHAND|THREAD|SYSVSEM` (flags
  `0x50F00`; syscall 56 on x86-64, 220 on ARM64), no TLS. Worker `k` has the
  stack `STACKS + k * 8 MiB` (a lazily committed region of 64, `PROT_NONE`
  guard page at the low end); `STACKS` is global slot 46, and a thread is a
  worker exactly when its stack pointer lies in that region. The child starts
  in `rt.worker_main(k)` with the stack top (16-byte aligned) in `rsp`/`sp`;
  the argument rides in `r8` (x86-64) or `x4` (ARM64), which `clone` ignores
  without `CLONE_SETTLS`. The `restart` primitive re-enters `rt.worker_main`
  on a fresh stack after a trap.
- **Sleeping.** `futex` wait and wake with `FUTEX_PRIVATE` (`128`, `129`) on
  the low 32 bits of an 8-byte word (syscall 202 on x86-64, 98 on ARM64).
  Locks are three-state futex mutexes (0 free, 1 held, 2 held and waited
  for) over `atomic_cas`, `atomic_add` and `atomic_store`.
- **Atomics.** x86-64: `mov`, `xchg`, `lock xadd`, `lock cmpxchg`. ARM64:
  `ldar`, `stlr`, `swpal`, `ldaddal`, `casal` (LSE). Always sequentially
  consistent.
- **Globals** 40–46: `TASKS` (the pool block), `MULTI` (1 once the pool
  exists: `alloc` then takes `ALLOC_LOCK`), `ALLOC_LOCK`, `OUT_LOCK`,
  `OUT_OWNER` and `OUT_DEPTH` (the reentrant output lock), `STACKS`. The image
  has 64 slots.
- **Task record** (8-byte words): 0 state (0 queued or running, 2 done,
  3 trapped), 1 function id, 2 result, 3 next in the queue, 4–6 the trap's
  kind and operands, 7 its head string, 8 its operator, 9 its source
  excerpt, 10–23 arguments (a chunk function: environment, `lo`, `hi`, then
  sum, lowest and highest running sum per accumulator). The compiler writes
  `task.K(rec)` for each `spawn` and `par for` site, and `rt.dispatch(id,
  rec)` to call it.
- **Pool block** (one `mmap` page): lock, queue head and tail, the sequence
  word idle workers sleep on, live, blocked and idle counts, the target, and
  the task each worker runs.
- **Atomic cell:** 8 bytes, no header yet.

**With the collector (lane 171, PR #174).** Its collector is conservative and
assumes every task is at a call; a worker in a loop that never allocates is
not. The protocol for threaded programs:

1. The collector runs inside `alloc` holding `ALLOC_LOCK` and sets `GC_REQ`.
2. Every other thread parks at its next `alloc`, lock wait or task wait: it
   calls `gc_regs(buf)` on its own stack, publishes the returned stack pointer
   in the pool block, and sleeps on a futex until the collection ends. A thread
   already blocked in a futex wait publishes its stack pointer before sleeping.
3. The collector waits until every live worker has parked, then scans the main
   stack (to `STACK_TOP`), each worker stack (published pointer up to
   `STACKS + k * STACK_SZ + STACK_SZ`), the registers, the globals and the pool
   block. A loop that never allocates delays a collection but nothing else.
4. `rt.gc_stop_world()` and `rt.gc_start_world()` do steps 1-3 and the wake;
   they are no-ops in `notask.ail`. Lane 171 calls them around `gc_collect`.

Header kinds (11 is lane 171's pointer-free words): 12 task record and 13
environment or group record (every word may be a pointer), 14 atomic cell (no
pointers). Until the collector merges, nothing is freed and `alloc` only
serialises on `ALLOC_LOCK`.

## 11. Imports from libSystem (macOS)

A runtime file calls a libSystem function with the primitive
`import_call("_name", a, b, c, d)` (the name is a string literal with its
leading underscore, the four arguments integers; unused ones are 0). It lowers
to an ordinary MIR `call` named `libsys._name`, so it clobbers the caller-saved
registers like any call. The ARM64 generator passes the arguments as for any
call, then `adrp x16, slot; add; ldr x16, [x16]; blr x16`. Slot `k`, for the
`k`th distinct name in `Image.imports`, is the 8 bytes after the runtime's
globals (`globals + k`), so the images grow by one word per import and
nothing else moves.

The Mach-O writer adds `LC_DYLD_INFO_ONLY` with a bind table only when the
image imports something: ordinal 1 (libSystem), one `SET_SYMBOL`,
`SET_TYPE_IMM pointer`, `SET_SEGMENT_AND_OFFSET_ULEB` (segment 2, `__DATA`) and
`DO_BIND` per name. dyld fills the slots before `main` runs. On Linux a
`libsys.` call is `ud2`/`brk`: the runtime guards each with `target() == 2`.
`fn_addr("rt.name")` gives a function's address (for `pthread_create`).

Used by `task.ail` (`pthread_attr_init`, `pthread_attr_setstack`,
`pthread_create`, `sysconf`) and `core.ail` (`__ulock_wait`, `__ulock_wake`).

## Licensing

[ADR 0510](../decisions/0510-licence-apache-2-with-runtime-exception.md): the
repository is Apache 2.0 with a Runtime Library Exception, and `compiler/rt/`
is Runtime Code. Every file there starts with the header

```text
## Runtime: part of the AIL native runtime; linked into user executables
## under the Runtime Exception (see LICENSE).
```

Nothing in `compiler/rt/` is ported from another project; the float text
algorithm is written from the textbook description (ADR 0409), so there is no
third-party notice to carry.
