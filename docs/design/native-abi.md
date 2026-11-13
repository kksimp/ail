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
registers go on the stack, 8 bytes each, in order, the first lowest; the
caller reserves a 16-byte-aligned area below its frame for them and frees it
after the call, and the callee reads them above its frame record (ADR 0533).
Float arguments beyond the eighth aren't supported yet. Callee-saved registers are
the platform's, plus **xmm8–xmm13 on x86-64** between AIL functions (ADR 0385;
an `extern` call treats them as clobbered). No variadics. Structs are passed
by pointer.

On x86-64 the code generator keeps rax, r10, xmm14 and xmm15 as scratch; the
allocator (ADR 0386) uses rsi, rdi, r8, r9, rcx, rdx, r11, rbx, r12–r15 and
xmm0–xmm13, and keeps values live across a division out of rdx (ADR 1049). On ARM64 (ADR 0388) the scratch registers are x16, x17, x8, d30
and d31 (x0 between calls); x18 is never used; the allocator uses x1–x7,
x9–x15, x19–x28 and d0–d29. Mach-O executables are loaded by dyld, name
libSystem but import nothing, and are signed ad hoc by the backend
(ADR 0389).

## 2. Value layouts

All values are 64-bit and little-endian (ADR 0007). In registers and in list
elements each scalar takes 8 bytes, except that a `List[bool]` element takes
one byte, 0 or 1 (ADR 0391), a `List[u8]` element one byte, 0 to 255
(ADR 0907), a `List[u16]` element two bytes and a `List[u32]` element four
(ADR 0912). A list's length never decreases in the native
subset, and the optimiser relies on it (ADR 0578): an operation that
shortens a list must be added to the optimiser's decision first.

| Type | Layout |
|---|---|
| `int` | `i64` |
| `float` | `f64` (IEEE 754 binary64) |
| `bool` | `i64`, 0 or 1 |
| `char` | `i64`, a Unicode scalar value; `List[char]` has 8-byte slots (ADR 0566) |
| `str` | pointer to a **string object** |
| `List[int]`, `List[float]`, `List[bool]` | pointer to a **list object** |
| `Map[K, V]`, `Set[T]` | pointer to a **table object** (kind 7, 8) |
| `(A, B, ...)` | pointer to a struct object (kind 4), one 8-byte slot per element in order (ADR 0829) |

**String object** (immutable, 16-byte aligned):

```text
+0   u64  bytes    length in UTF-8 bytes
+8   u64  chars    length in characters  (str.len(), ADR 0208, O(1))
+16  u8[] data     UTF-8, `bytes` long, no terminator
```

A `bytes` object has the same layout with `chars` equal to `bytes`
(ADR 0702).

A literal is a string object in read-only data. `chars` is computed once when
the object is built, which makes `s.len()` free and byte indexing impossible.

**List object** (a value; copied on `mut` aliasing by the compiler, ADR 0060):

```text
+0   u64  len     elements in use
+8   u64  cap     elements allocated
+16  ptr  data    cap * 8 bytes (cap, 2 cap, 4 cap bytes for List[bool] and List[u8], List[u16], List[u32]), separate allocation
```

`data` is separate so `push` can grow without moving the header the caller
holds. A full list grows to 16 elements if it has fewer than 8, else doubles
(ADR 0562); blocks of 256 bytes or more are copied 64 bytes per round. Out-of-range reads and writes trap `type.coll.4` (the interpreter's rule
for list indexing).

**Optionals** (ADR 0574). `T?` is one register: 0 is `none`. A `str` or list
optional is the pointer itself. An `int`, `float` or `bool` optional points at
an 8-byte box (the value, or its bits for a `float`); the heap lane gives the
box kind 10. Built by `rt.box`, `rt.boxf`, `rt.box_bool`, read by
`rt.unbox`, `rt.unboxf`, `rt.unbox_bool`.

**Results** (ADRs 0575, 0576). A function that can fail returns its success
value (zero on failure); failure is the runtime's global slot 8 holding the
error's display text (a string object), 0 otherwise. Global 9 holds the return
trace, a list of string objects `  at PATH:LINE:COL`. `rt.failed`, `rt.err_set`,
`rt.err_trace`, `rt.err_take`, `rt.err_clear`, and for error structs and enums global 12 holds the error value (`rt.err_value`, `rt.err_take_value`, ADR 0804); `rt.exit` reports a failure
that reached `main` first (`error: TEXT`, the trace, status 1).

**Heap object header** (ADR 0562). Every heap object has an 8-byte word at
pointer minus 8: bits 0-7 kind, bit 8 mark (collector only), bits 9-15
reserved, bits 16-63 payload bytes. Kinds: 1 string, 2 list header, 3 list
data, 4 struct, 5 enum payload, 6 closure, 7 map, 8 set, 9 bytes, 10 boxed
optional/result, 11 pointer-free words (the data of `List[bool]`, `List[u8]`, `List[u16]` and `List[u32]`), 12-255 reserved.
The object pointer is 8-byte aligned (16-byte blocks start 8 bytes before it).
Literals in read-only data have no header and are never freed.

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
| 11 | pointer-free words |
| 12 | task record (scanned word by word) |
| 13 | environment record (scanned word by word) |
| 14 | atomic cell (scanned word by word) |

Strings, lists and structs all take their memory from `rt.alloc` (ADR 0562);
`rt.obj_new(size, kind)` is `alloc` for compiled code. Kind 11 (pointer-free
words) is also defined.

**Struct object** (ADR 0392): one 8-byte slot per field, in declaration
order (a `float` as its bits, a `bool` as 0 or 1, a string, list or struct
as a pointer). A struct is a value: reading one from a place to bind,
store, push or return it copies it, deeply through the lists and structs
it holds (a generated `copy.T` function); fields change in place.

**Table object** (`Map` and `Set`, ADR 0568; `compiler/rt/native/map.ail`):

```text
+0   u64  len     live entries
+8   u64  used    entry-array slots used, live or dead
+16  u64  ecap    entry-array capacity (a power of two)
+24  u64  mask    index size - 1
+32  u64  kind    key kind: 0 int, 1 str, 2 bool
+40  ptr  ents    ecap * 24 bytes: hash (-1 when dead), key, value
+48  ptr  index   (mask + 1) * 8 bytes: 0 empty, else entry number + 1
+56  u64  vmode   copy mode of values: 0 plain, 1 lists, 2 tables
+64  u64  vinfo   vmode 1: nesting depth * 256 + element bytes
+72  u64  lastkey the last key found or added
+80  ptr  laste   its entry, or 0 (cleared by a rebuild, removal or copy)
+88  u64  misskey the last key a probe missed (ADR 0936)
+96  u64  misshash its hash, reused by the insert that follows
+104 ptr  kcache  `str` keys only: 32 entry addresses, slot by key object address
```

A `str`-keyed table is 360 bytes, others 104. A key cache slot counts only
when its entry still holds that very string object (ADR 0936).

The entry array is in insertion order, which is iteration order
(type.map.1). A value is one word: a `float` as its bits, a `bool` as 0 or
A set uses the same layout and ignores the value word.

**Socket object** (`net.Conn`, `net.Listener`; ADR 0830; `compiler/rt/native/net.ail`).
A `Conn` or `Listener` value is a struct (kind 4) of one `int` field: an index
into a list of socket objects that global slot 40 keeps alive.

```text
+0   i64  fd       descriptor, -1 once closed
+8   u64  timeout  milliseconds a read, write or connect may wait
+16  ptr  buf      read buffer (pointer-free), or 0
+24  u64  pos      first unread byte in buf
+32  u64  len      bytes in buf
+40  u64  flags    1 end of stream seen
+48  ptr  peer     string: peer address of a Conn
+56  u64  kind     0 connection, 1 listener
+64  ptr  addr     string: bound address of a Listener
+72  u64  cap      size of buf
```

**Enum values** (ADR 0602, lane 179). An enum none of whose variants has
fields is an `i64`, the variant's index in declaration order. Any other enum
is a pointer to an **enum object** (heap kind 5, immutable, shared, never
copied):

```text
+0   i64  index   the variant, in declaration order
+8   ...  fields  one 8-byte slot per field of the variant, in order
```

A variant without fields of such an enum is one shared object per program,
built by the generated `init.enums` into a program global before `main`
runs; program globals (computed constants, ADR 0604) follow the runtime's
first `RT_GLOBALS` (16) slots, and the image's global area is sized from the
slots the MIR uses. `==` on an enum with fields compares the index when one
side is a variant without fields, the index and that variant's fields when
one side is built in place, else calls the generated `eq.E`.

**Generic instances** (ADR 0536). Each generic function, struct or enum is
compiled once per set of type arguments, as a plain function or type named
with them: `first[int]`, `Box[str]`, `Box[str].get`, `Pair[int, str]`.
An instance has the layout of the same struct or enum written out by hand;
there is no boxing and no type information at run time. The methods of an
`impl Trait for T` block are the symbols `T.m`, like any method of `T`.

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

**OS layer calls** (ADR 0580, `compiler/rt/native/os.ail`; a negative result is
`-errno` on every target):

| Operation | Linux x86-64 | Linux ARM64 | macOS ARM64 |
|---|---|---|---|
| `open` | `openat` 257, `AT_FDCWD` -100 | `openat` 56 | `open` 5 |
| `close` | 3 | 57 | 6 |
| `lseek` | 8 | 62 | 199 |
| `access` | `faccessat` 269 | `faccessat` 48 | `access` 33 |
| `mkdir` | `mkdirat` 258 | `mkdirat` 34 | `mkdir` 136 |
| `unlink` / `rmdir` | `unlinkat` 263, flag 0 / 0x200 | `unlinkat` 35 | `unlink` 10 / `rmdir` 137 |
| `rename` | `renameat` 264 | `renameat` 38 | `rename` 128 |
| directory entries | `getdents64` 217 (name at +19) | `getdents64` 61 | `getdirentries64` 344 (name at +21) |
| working directory | `getcwd` 79 | `getcwd` 17 | `fcntl` 92, `F_GETPATH` 50, on `.` |
| clock | `clock_gettime` 228, `CLOCK_REALTIME` | `clock_gettime` 113 | `gettimeofday` 116 |
| sleep | `nanosleep` 35 | `nanosleep` 101 | `select` 93 with a timeout |
| terminal test | `ioctl` 16, `TCGETS` 0x5401 | `ioctl` 29 | `ioctl` 54, `TIOCGETA` 0x40487413 |

`open` flags for writing: Linux `O_WRONLY|O_CREAT|O_TRUNC` = 577, append
1089; macOS 1537 and 521. Errors map to `fs.IoError.kind` (std.err.1) from
`errno` 2 not_found, 1 and 13 permission, 17 exists, 21 is_dir, 20 not_dir,
39 (Linux) or 66 (macOS) not_empty, else other. The environment follows
`argv`'s NULL (`argv + 8 * (argc + 1)`) on all three targets.

Linux clobbers `rcx` and `r11` on x86-64. The shim is four functions
(`sys_exit`, `sys_write`, `sys_mmap`, `sys_read`) so each target is one file
(`compiler/rt/sys_*.ail`).

**macOS risk.** Apple documents `libSystem` as the stable interface and
reserves the right to change raw syscall numbers. The raw route works on
current macOS ARM64 and is what the first slice uses. ADR 0534 moves the macOS
OS layer to libSystem calls (`_write`, `_exit`, `_mmap`, ...) through the dyld
imports lane 178 adds; Linux keeps raw syscalls.

## 5. Allocator and collector

**As built (lane 171, ADRs 0562-0564).** `rt.alloc(n, kind)` returns `n`
zeroed payload bytes with the header word before them; `rt.alloc_raw(n, kind)`
skips the zeroing for blocks the caller fills before reading. `rt.init` reserves up
to 16 GiB of address space (`mmap`, no-reserve; the size halves until it is
accepted), cut into 16,448-byte pages with a page table (free, small page of class
c, first or later page of a large span). Blocks up to 8192 bytes, header
included, come from 36 size classes (16 to 256 in steps of 16, then four per
doubling) with free lists; larger objects take whole pages. Failure to get
memory traps `run.limit.2`.

The collector (`compiler/rt/native/gc.ail`) is stop-the-world mark and sweep,
run from `alloc` when the bytes allocated since the last collection exceed
`max(16 MiB, 2 x live bytes)`. Roots: the stack from the stack pointer to the
`argv` address saved by `rt.init`, the callee-saved registers (primitive
`gc_regs`), and the global slots, all read as possible pointers, interior
pointers included. Objects are scanned by kind (list headers: the data
pointer; strings, bytes and kind 11: nothing; others: every word). The
collector does not move objects. Global slots 8 and 9 hold the heap state
block and the stack top; the runtime's slots are 0-63 and the program's constants start at 64; the collector scans them all (`globals_end`).

## 6. Output

**Global slots.** The runtime's globals are 64 eight-byte slots (`global(k)`,
`Image.globals`): 0-15 `core.ail`, 16-31 the heap and collector, 32-39
`os.ail` (status, scratch area, output descriptor, input buffer, terminal
flag), 40 `net.ail` (the table of sockets, ADR 0830), 41 `net_http.ail` (the pool of idle HTTP connections, std.net.19), 42 `tls_client.ail` (the cache of trusted roots, ADR 0852), 44 `memtab.ail` (the table of `std.mem` blocks, unsafe.ptr.5-6), 48 `unwind.ail` (cleanups on a trap, own.uniq.3), 45-47, 49-51 and 55 `limits.ail` (run limits, ADR 0726), 52-54 `fmt.ail` (the byte builder), 56 `instr.ail` (the hit table of an instrumented build, native.instr.2), the rest free (the concurrency slots of #194 take 16-24, channels 25-30). Each slot is a
constant `NAME := N` read as `global(NAME)`; two names may share a slot only
if the doc comment says it `shares` it (`TRAP_LEN` and `PARSE_OK`).
`77592931:bootstrap/tests/native_globals.rs` fails on two names in one slot, a slot
past `Image.globals`, an undeclared name, or a code generator constant of
the same name with another value.

**Flush points** (ADR 0581): the buffer fills; `rt.exit` (called by the entry
after `main` and by `io.exit`); every trap; before reading standard input;
before `io.eprint`; and each newline when standard output is a terminal.
`io.eprint` writes into the same buffer with `OUT_FD` set to 2, so every
flush it causes goes to standard error, then restores 1.

`print` appends to a 64 KiB stdout buffer and flushes when full, at exit, at a
trap, and before reading stdin. Standard error is unbuffered. The interpreter's
text is the oracle: integers in decimal with a leading `-`; floats per
`type.num.7` (ADR 0409); strings as is.

## 7. Traps

**As built (lane 120, ADR 0385).** A failed check branches to a stub at the
end of its function, which calls
`rt.trap(kind, a, b, head, op, tail)`: `kind` picks the message (0 overflow,
1 division by zero, 2 negation, 3 index, 4 float to `int`, 5 `abs`, 6 shift
count, 7 slice, 11 stack exhausted, 14 `unreachable`; 15 prints a runtime-built message string, `a`), `a` and `b` are the operands (for 4, the float's bits), `head` is the
constant `PATH:LINE:COL: trap[RULE]: `, `op` the operator, and `tail` the
constant source excerpt (`  L | line` and the caret line). The output is the
interpreter's first line and excerpt, byte for byte; the snapshot lines
after them are not printed. Runtime code has no checks: its arithmetic wraps.

**Stack check (ADR 0535).** `rt.init` stores a limit in global slot 15: the
argument vector's address (the top of the main stack) less the stack's size
limit (`RLIMIT_STACK`; 8 MiB when unknown or unlimited, at most 1 GiB) less a
1 MiB reserve for the trap itself. A call that can recurse, one whose callee
is in its caller's strongly connected component of the call graph, first
compares the stack pointer with it (`cmp rsp, [globals+96]; jb stub` on
x86-64; `adrp`, `add`, `ldr`, `cmp sp, x16`, `b.lo` on ARM64) and traps with
kind 11 at the call: `call depth exceeded the native stack in `NAME``. Other
calls are unchecked: their chains are bounded by the program's functions.
Threads other than the main one need their own limit (lane 178).
**Stack (lane 195, ADR 0641).** The stub's `tail` text also holds the
trapping function's frame line (and, for a check inlined from a callee, the
frames it was inlined through, ADR 0643). `rt.trap` then calls
`trace_frames(tail)` (`rt/native/trap.ail`), which walks the frame pointers
(`[fp]` is the caller's frame, `[fp + 8]` the return address; `_start` and
thread entries clear the frame pointer) and prints a line for each return
address found in the **call table**. The table is the symbol `trace.tab`, in
the code (8-aligned), in 8-byte words: `count`, the table's own offset in
the code, the number of functions; then `count` rows of two words, in
address order, `return offset | line << 32` and `col | function << 32`;
then two words per function, the offsets from `trace.tab` of its
`NAME(..)` string object and of its `FILE:` string object (0: the first
file); then those string objects. Primitives `code_tab()` and `frame_ptr()`
(runtime only) give the table's address and the frame pointer. A user
`call` instruction carries its position in `imm` (`mir.call_pos`); runtime
calls carry 0 and get no row. A runtime path with its own trap text calls
`trace_frames` after printing it. Code-size cost: 16 bytes per user call and
about 1.5 KB of runtime (ADR 0644).

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

The interpreter then adds a source excerpt and a stack trace. Native code
prints the excerpt and the stack too (trace.frame.1, ADR 0641; see below). The
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
`rt.list_pushf`, `rt.list_copy` and `rt.list_reserve(xs, lo, hi, per, size)`
(room for `(hi - lo) * per` more elements, inserted by the optimiser before
loops that fill a list, ADR 0579); `print` writes its pieces straight into
the buffer instead of building a string. List indexing and `len` are inline.
The runtime is written over the primitives of `compiler/rt/native/prim.ail`
(`load64`, `store64`, `load8`, `store8`, `loadf`, `storef`, `loadf32`, `storef32` (one packed single, widened and narrowed exactly; ADR 0875), `syscall`,
`global`, `str_addr`, `list_addr`, `list_at`, `float_bits`, `bits_float`,
`f32_round`, `target`), which the code generator expands inline and only runtime code may
call. Its float text is lane 122's algorithm (ADR 0409), on lists of limbs.

**OS layer and number text (lane 177, ADRs 0580-0582).** `os.ail` provides
`fs_*`, `env_*`, `time_*`, `io_*`, `err_begin`/`err_end`, `os_ok`, `os_kind`
and `kind_name`; `fmt.ail` provides the byte builder (`sb_*`), `put_*`,
`fmt_int`, `fmt_hex`, `fmt_bool`, `fmt_float`, `fmt_float_fixed`, `fmt_pad`,
`out_float`, `out_float_fixed` and `str_to_float`. A fallible function returns
its value and, on failure, sets the error slot (global 8) to the display text
of an `fs.IoError` and returns zero (ADR 0580); `env_get`, `env_home` and
`io_read_line` return a string or 0. Runtime code that needs a deadline calls
`time_mono_ns()` or `time_mono_ms()` (`CLOCK_MONOTONIC` on Linux; on macOS it is NOT
monotonic yet, it reads the wall clock, until the libSystem switch moves it to
`clock_gettime_nsec_np` or `mach_continuous_time`: a follow-up for the Mac gate); `time_now_ms()` is the wall clock of `time.now_ms`.

**Strings and characters (lane 172, ADRs 0565-0567).** `compiler/rt/native/str.ail`
adds: `str_cat`, `str_eq`, `str_cmp`, `str_int`, `str_hex`, `str_bool`,
`str_float`, `str_fixed`, `str_char`, `str_pad`, `str_pad_checked`,
`str_slice`, `str_contains`, `str_starts_with`, `str_ends_with`, `str_find`
(character index or -1), `str_find_opt`, `str_char_at`, `str_strip_prefix`, `str_strip_suffix`,
`char_to_digit`, `char_from_code` (optional results), `str_replace`, `str_split`,
`str_lines`, `str_chars` (scalar values), `str_repeat`, `str_trim`, `str_trim_start`,
`str_trim_end`, `str_words`, `str_count`, `str_rfind_opt`,
`str_to_upper`, `str_to_lower`, the `char_*` predicates and mappings,
`out_char`, `out_list_char`, and `str_trap`. A function that
can trap takes the constant head and excerpt as its last arguments (`str`
values the lowering builds from the call's position). A character crosses the
runtime as an `int`.

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
section is the plan for compiled programs. Dynamic linking is not built.
Instead (ADR 0844) `ail build` provides `std.mem` over the runtime's own page
blocks and a fixed libc subset (`hypot abs toupper strlen atoi memset memcpy
strtod`) as runtime functions; any other `extern c fn` is `ffi.native.1`.

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
| Windows x86-64 | no import table entry per symbol: `LoadLibraryA`/`GetProcAddress` bind it on the first call (`ffi.load.3`, ADR 0980), called with the Win64 convention; ARM64 binds the same way with AAPCS64 (experimental, ADR 0968) |

A program with **no** `extern` links no libc and has no `PT_INTERP`
(`ffi.link.1`, ADR 0105): the static executable of §3 is unchanged. A program
with one becomes a dynamic executable; its entry is still `rt_start`, which
does not go through `__libc_start_main`. That is safe for the pure functions
the language targets (`strlen`, `sqrt`, `atoi`), because the loader has already
initialised libc and its thread-local storage. It is not enough for libc's
buffered stdio, so `rt_exit` calls `fflush(0)` through the dynamic symbol when
libc is linked, before the `exit` syscall. This is an open risk until a
differential test with `printf` passes on both Linux ARM64 and macOS.

**As built (ADR 0870).** An `extern c fn` outside the runtime's libc subset is
bound on its first call by `rt.ffi_addr` (`compiler/rt/native/ffi_load.ail`):
`dlopen` of the library, `dlsym` of the symbol, the address kept in a cell of
the table in global slot 57. The body then makes an `icall` named `c`, which
the x86-64 generator brackets with a save and restore of `xmm8`-`xmm13`. The
executable imports only `dlopen` and `dlsym`. On Linux `obj/elf.ail` writes a
dynamic ELF in that case (`PT_INTERP`, `DT_NEEDED` libc, `DT_BIND_NOW`, two weak
`GLOB_DAT` relocations into the import slots after the globals); the import
call is `call [slot]` on x86-64 and `adrp`/`ldr`/`blr` on ARM64. On macOS the
imports are `_dlopen` and `_dlsym` in the bind table of §11. Programs that
call only the libc subset still have no `PT_INTERP`.

**Memory.** `mem.alloc`, `mem.cstr` and `mem.free` call libc `malloc` and
`free` when libc is linked, so a block can cross the boundary in either
direction (`unsafe.mem.1`); without libc they use `rt_alloc` and `free` is a
no-op until the collector exists (§5). `read`, `write` and `offset` compile to
a load, a store and an `add` of `n * size_of(T)`. The interpreter's block table
(`unsafe.ptr.4–6`) does not exist natively: only a null access is trapped, by
a guard page and a signal handler the runtime installs, reporting
`unsafe.ptr.4`; out-of-block and freed accesses are undefined behaviour, as in
C, and the differential harness runs them under the interpreter only.

**Hardware access (ADR 0835).** `*T` is an address in an integer register. `read`, `write`, `read_volatile` and `write_volatile` are the prims `pload8/16/32/64` and `pstore8/16/32/64`, which the optimiser treats as barriers; `hw.fence` and `hw.fence_io` are `pfence` and `pfence_io` (`mfence`; `dmb ish` and `dsb sy`). The null check is a compare against 4096, not a signal.

**Runtime Code.** The call stubs, the GOT slots and `rt_exit`'s flush belong
to the executable and fall under the Runtime Exception (ADR 0510); the
interpreter's `dlopen` glue (`77592931:bootstrap/src/interp/ffi.rs`) is toolchain code
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
- **Globals** 16–24: `TASKS` (the pool block), `MULTI` (1 once the pool
  exists: `alloc` then uses the thread's cache and takes `ALLOC_LOCK` only to refill it), `ALLOC_LOCK`, `OUT_LOCK`,
  `OUT_OWNER` and `OUT_DEPTH` (the reentrant output lock), `STACKS`. The image
  has 72 slots (64–71 are unused scratch slots, which `tests/native/driver/mem_write_read` writes).
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
- **Atomic cell:** 8 bytes, heap kind 11.
- **Allocation under threads ([ADR 0943](../decisions/0943-per-thread-allocation-caches.md)):** once the pool exists each thread slot has an allocation cache (heap state word 280 points at them, 1536 bytes each, laid out like the state block's per-class words: free-list heads, bump page, end, dirty flag, plus a three-word mark stack at words 8-10). `H_FASTLIM` is 0 then, so the fast path's limit test fails into a second branch that serves the running thread's cache (its slot from the stack pointer: workers' stacks lie in `STACKS`) with no lock and no counting; one thread never reaches that branch. `alloc_slow` refills a class under `ALLOC_LOCK`: about 4 KiB of the shared free list, else the shared page being filled, else a new page, and counts it toward the next collection then. A collection empties every cache first (`tb_flush`, clearing the headers of blocks not yet handed out). With `AIL_GC_VERIFY` every allocation stays on the shared, locked path.
- **Channel record** (`chan.ail`, heap kind 13; a `Channel[str]` is its
  address): 0 lock word, 1 capacity, 2 count, 3 head, 4 closed, 5 ring
  pointer (string pointers), 6 source (nonzero for a channel fed from outside:
  `chan_pump`, `chan_source_send` and `chan_source_close` then take over),
  7 wake word (futex), 8 ring capacity, 9 id (`Channel#N`). Globals 25–27:
  the change counter `select` sleeps on (task ends bump it too), how many sleep, channels made. 28–30: the deadlock message the main task found, the unfinished tasks, and the threads sleeping on a program-made change; `prog_wait` counts a sleeper, and the main task in 25 ms slices reports a deadlock after 4 idle slices when the sleepers number the unfinished tasks plus main.

**With the collector (lane 171).** The collector is conservative and assumes
every other thread is at a call; a worker in a loop that never allocates is
not. As built, a threaded program works like this:

1. Once the pool exists (`MULTI`), each thread allocates from its own cache
   and `alloc_slow` refills it under `ALLOC_LOCK`. A collection starts inside
   it, so the collector holds the lock and no other thread can refill; the
   others are parked, never inside the fast path, since it has no safe point.
2. `gc_stop_world` sets `GC_REQ` and waits until every other live thread has
   published a stack pointer (`P_PARK`). A thread publishes at each safe point:
   before any futex sleep (`futex_wait`, which saves the callee-saved registers
   with `gc_regs` first), when it finds `GC_REQ` set in `lock`, and between
   tasks. After waking it stays published while `GC_REQ` is set, and clears its
   mark first and re-checks, so the collector never scans a thread that runs.
3. The collector scans its own stack, then each other thread's registers and
   stack (published pointer up to the main stack's top or the worker's slice of
   `STACKS`), the pool block (queued and running task records), each thread's
   text builder, and the globals. A thread that computes without reaching the
   runtime delays a collection, nothing else.
4. On a heap of 32 MB or more the parked threads help mark
   ([ADR 0944](../decisions/0944-parallel-gc-marking.md), `gc_par.ail`): the
   collector sets `GC_REQ` to 2 and wakes them (idle workers through the
   pool's `seq` word). Each marker moves a share of the shared mark stack to
   its own, scans from it, and gives half back while the shared stack is
   empty. Marking ends when the shared stack is empty and no marker holds work
   (heap state words 281-284: lock, busy, done, joined). The collector sets
   `GC_REQ` back to 1 and waits until every helper has left before it sweeps.
5. `gc_start_world` clears `GC_REQ` and wakes the parked threads.
   `notask.ail` stubs all of it.

Heap kinds: 11 (`KIND_ATOMIC`, no pointers) is an atomic cell; 12 is a task
record and 13 an environment or group record (every word may be a pointer).
`str` formatting keeps one text builder per thread in the pool block
(`fmt.ail`, `sb_base`). A trap inside the allocator (a run limit) releases its
lock before the task ends. What is still shared and not thread-safe: the error
slot (`rt.err_set`), `PARSE_OK` and the OS layer's result slots (`os.ail`), so
tasks must not use fallible calls or the file and environment functions at
the same time.

## 11. Imports from libSystem (macOS)

A runtime file calls a libSystem function with the primitive
`import_call("_name", a, b, c, d)` (the name is a string literal with its
leading underscore; the arguments are integers, any number the target's
calls support, so the declaration in `prim.ail` doesn't limit a call site). It lowers
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

## Tensors (ADR 0815)

`tensor.Tensor[T]` is a pointer to an object of heap kind 4: `+0` element kind
(0 `int`, 1 `float`), `+8` rank, `+16` element count, `+24` pointer to the
elements (8 bytes each, heap kind 11), `+32` the axis sizes, one word each. A
tensor is never changed in place, so a copy shares the object. Calls: `rt.tensor_from`,
`rt.tensor_shape_fault` and `rt.tensor_from_shape` (the fault helper returns a
message string object or 0, and a message traps with runtime trap kind 15),
`rt.tensor_to_list`, `rt.tensor_shape`, `rt.tensor_eq`, `rt.tensor_text` and
`rt.out_tensor`.

Step 2 (elementwise operators, reductions, transpose): `rt.tensor_zip_fault`
and `rt.tensor_zip` (`op` 0 `+`, 1 `-`, 2 `*`, 3 `/`; broadcasting as
std.tensor.4), `rt.tensor_scalar_f` and `rt.tensor_scalar_i`, `rt.tensor_neg`,
`rt.tensor_sum_f`, `rt.tensor_sum_i`, `rt.tensor_dot_f`, `rt.tensor_dot_i`,
`rt.tensor_reduce_fault` and `rt.tensor_reduce` (`kind` 0 sum, 1 max, 2 mean),
`rt.tensor_transpose_fault` and `rt.tensor_transpose`. A kernel that can
overflow an `int` (std.tensor.6) returns 0, and `sum`/`dot` of `int` return a
one-element tensor read with `rt.tensor_first_i`; the lowering traps with the
constant message when it sees 0. A float `sum`, `dot` and axis sum follow
std.tensor.5's 16 partial sums.

Step 3 (matmul, activations): `rt.tensor_matmul_fault` and `rt.tensor_matmul`
(the plain triple loop of std.tensor.7, each product and sum rounded, no fused
multiply-add; 0 on `int` overflow, as above), and `rt.tensor_activate` (`kind`
0 `relu`, 1 `sigmoid` through `rt.float_exp`, 2 `tanh` through `rt.float_tanh`,
fdlibm's, which uses `rt.float_expm1`).

Step 4 (`Tensor[f32]`): element kind 2. An element is still a `float` word, but
it always holds a value already rounded to `f32`; `f32_round` (a primitive:
round to single precision and widen back, `vcvtsd2ss` and `vcvtss2sd` on
x86-64, `fcvt s,d` and `fcvt d,s` on ARM64) is applied after every operation.
One rounding of the double-precision result is the correctly rounded
single-precision result for `+ - * /` (a double has more than twice an `f32`'s
precision), so the values are the interpreter's `f32` kernels'.
`rt.tensor_to_f32` and `rt.tensor_to_float` convert. A `Tensor[f32]` has type
`Ty.Tensor(Ty.F32)`. A scalar `f32` (ADR 0834) is a `Ty.F32` value in a float
register, rounded after every operation, but `sum`, `dot`, `max`, `mean`,
`item`, printing and the text form of a `Tensor[f32]` are still
`native.subset.1` problems, and `to_list` gives a `List[float]`.

Step 5 (vector kernels, x86-64): `rt.vec_axpy` (matmul's inner loop), `rt.vec_zip`
and `rt.vec_scal` (elementwise operators) are declared in `tensor.ail` with
scalar bodies, and `compiler/x64/kernels.ail` assembles AVX loops in their
place (four doubles at a time, then element by element; a multiply and an add,
never a fused one, so every result is the scalar one bit for bit; a non-zero
`r` rounds each result to `f32` through `vcvtpd2ps`/`vcvtps2pd`). They are
leaf code using only rax, rcx-r11 and xmm0-xmm4, and `inline.ail` never inlines
an `rt.vec_*` function. On ARM64 `compiler/a64/kernels.ail` assembles NEON
loops in the same place (two doubles at a time, `fcvtn`/`fcvtl` for the `f32`
rounding, x0-x9 and v0-v5 only), with the same results bit for bit.

Step 5b (ARM64 `matmul`): `rt.vec_mm(c, a, b, n, k, r)` computes one row of
the product, `compiler/a64/kernels.ail` assembles it with eight columns' sums
held in registers across the whole `p` loop (`f32` as four singles per register,
which rounds like the `float` operation rounded to `f32`; doubles otherwise),
and the remaining columns one at a time. The order of the sums is the same as
`k` calls of `rt.vec_axpy` on a zeroed row, which is the body on other targets.

Step 6 (`softmax`, std.tensor.24): `rt.tensor_softmax_fault(a)` returns the
fault message (no axis, empty last axis) or 0, and the lowering traps with
imm 15 like the other tensor faults. `rt.tensor_softmax(a)` writes `exp(x - top)`
per row of the last axis (top skips NaN), sums them in element order and
divides, in `float` and rounded once for `f32`. `exp` is the runtime's own
(ADR 0592), so a result may differ from `ail run` in the last bit.
