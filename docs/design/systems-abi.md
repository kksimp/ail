# Systems ABI and no-GC mode (lane 6 of the replace-C plan)

Status: proposal, 2026-11-05. No ADR or OQ numbers yet (Fable allocates them).
Goal (Kaleb): AIL can replace C, C++ and Rust, so it must ship libraries C can
call, call C back, run without the collector, and drop to assembly. Everything
stays AI-first: one rule ID per behaviour, `ail explain` text for each error,
no new keywords where an attribute or a std call will do.

## What exists today

- `extern c fn`, `use c "header.h"`, dlopen binding (`ffi.decl.1-3`, `ffi.call.*`, `ffi.load.*`, `ffi.native.1`).
- Raw `*T`, `mem.alloc/free/cstr`, `std.hw` volatile and fences under `unsafe driver` (ADRs 0820-0822, 0835).
- ELF writer with a dynamic section that only imports (`compiler/obj/elf.ail`, `elf_dynamic`: empty DT_HASH, "the program defines no symbols"); Mach-O and PE writers.
- Native runtime always includes the heap and collector (`compiler/rt/native/heap.ail`, `gc.ail`, `gc_par.ail`).
- Not specified: exports, callbacks, structs by value, variadics, static or shared output, inline asm (OQ-137 recommends intrinsics only), freestanding targets.

## Slices, in landing order

Each slice is one PR, native-only, with a run program and `.out` golden.

### S1. `@export` and `ail build --lib`

- `@export fn name(a i32) i32` (also `@export("c_name")`): a top-level fn whose params and result obey `ffi.decl.3`. It is emitted with the C ABI (System V, AAPCS64, Windows x64 later) under that exact symbol.
- `ail build FILE --lib static|shared [--header OUT.h]`: static = `.a` (ar of one object), shared = `.so` / `.dylib` / `.dll`. `--header` writes a C header from the exports, so the C side never hand-writes prototypes (AI-first: one source of truth).
- The ELF writer gains defined dynamic symbols (real DT_HASH or DT_GNU_HASH, `.dynsym` with STB_GLOBAL), Mach-O gets an export trie, PE an export directory. Today's "defines no symbols" hash goes away.
- Entry: an exported fn called from C runs `rt` init lazily on first call (heap, tasks) unless the library is built `--no-gc` (S4). A trap inside an export aborts the process with the usual trap text; it never unwinds into C. Rule: `ffi.export.*`.
- Checker errors with `ail explain`: non-C-compatible type in an export, duplicate symbol, `mut` parameter, generic export.

### S2. Callbacks

- Pass an AIL function to C: type `extern c fn` pointer `cfn(i32) i32`, built from a named top-level `@export`-compatible fn, not a closure (no capture, no environment, no GC object behind a C pointer).
- Closures reach C only through the `void *user` idiom: `mem` block plus a plain fn. A helper `ffi.trampoline` is deferred until measured.
- Rules `ffi.cb.*`; calling a `cfn` is unsafe like any C call.

### S3. `@layout(c)` structs and by-value passing

- `@layout(c) struct P { x i32, y i32 }` has declaration-order fields with C padding; `sizeof`, `alignof`, `offsetof` as compile-time consts. Allowed in `extern c fn` params and results by value (the ABI classification per platform: INTEGER/SSE regs on SysV, HFA on AAPCS64) and as `*P` everywhere.
- `use c "header.h"` then also imports structs, enums and `#define` integer consts (today they are left out).
- Variadics: `extern c fn printf(fmt *u8, ...)` with an explicit call-site type list, `printf(fmt, i32: 3, float: 1.5)`, so the checker still sees every argument's C type. Needed to replace C wholesale; lowest priority in this slice.

### S4. `--no-gc` (freestanding-capable)

- `ail build FILE --no-gc` (or manifest `runtime none|gc`). The program is checked with `no_gc` as an effect-style capability: any construct that needs a collected object (`List`, `Map`, `str` growth, closures that capture, `ref T`, tasks, `defer` is fine) is a compile error naming the construct and the fix: use `std.mem` blocks, fixed arrays `[N]T`, slices over caller memory, or an `Arena`.
- Memory in this mode: `mem.alloc/free`, a stack-only `Buf[T, N]`, and `std.arena` (bump allocator over a caller or OS block, freed as a whole). All three are plain runtime code with no collector.
- Runtime layers split into `core` (traps, ints, floats, mem, hw), `heap` (GC, lists, maps, strings), `os` (files, net, tasks). `--no-gc` links only `core` plus what the program names, so a no-GC executable has no allocator beyond `mem`. Measured goal: hello-world in a few KiB.
- The checker enforces it, not the runtime: errors carry rule IDs `nogc.*` and `ail explain nogc.N` shows the replacement.
- Open choice (needs Kaleb): hard error versus warning-and-fallback for GC use. Recommendation: hard error; a library that silently links a collector defeats the point.

### S5. Freestanding targets

- Targets `x86_64-none` and `aarch64-none` (already on the ADR 0820 held list): no libc, no OS, entry symbol named by `@entry`, `@interrupt fn` for handlers, a linker script option, panic handler `@panic fn(msg str)` (no unwinding, no heap).
- Requires S4. Output: flat binary or ELF with chosen load address.

### S6. Inline assembly

- OQ-137 recommends intrinsics only. For the replace-C goal this is not enough for kernels and crypto. Proposal: land `hw.x64.*` / `hw.a64.*` intrinsics first, then `asm fn` (option c): whole functions in text assembly, fixed to the native ABI, assembled by the in-house assemblers, opaque to the optimiser, and one per target (`@asm("x86_64")`). An `asm { }` block with register constraints stays rejected: constraint languages are where agents fail.
- Needs Kaleb's confirmation because it changes OQ-137's recommendation.

## Suggested order and sizing

S1 (about 800 lines: checker attr, symbol table, ELF/Mach-O writers, header emitter), S4 (checker capability, runtime split, the largest), S3, S2, S5, S6. S1 and S4 are independent and can run as two PRs in parallel; S2 needs S1, S5 needs S4.

## Rejected alternatives

- Generating C and calling a C compiler: breaks ADR 0010 (no external backend).
- Closures as C callbacks: hides a GC root behind a raw pointer.
- A `nogc` runtime flag with silent fallback: a library would link the collector anyway.
- Copying Rust's `#[no_std]` crate split: AIL has no registry (Kaleb), so the split is of the runtime, not of std crates.

## Questions for the designer

1. S4: hard error on GC use in `--no-gc` (recommended) or warning?
2. S6: allow `asm fn` (changes OQ-137's recommendation)?
3. Windows x64 export/callback ABI in S1, or after the Windows release gate?
