# Systems ABI: exports, libraries, callbacks, layout (lane 6 of the replace-C plan)

Status: proposal, 2026-11-05 (rescoped by the designer: no no-GC, no 32-bit).
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

### S1. `pub export c fn` and `ail build --lib`

- `pub export c fn name(a i32) i32` (the designer-approved syntax, mirrors `extern c fn`; an optional `"c_name"` after `c` renames the symbol): a top-level fn whose params and result obey `ffi.decl.3`. It is emitted with the C ABI (System V, AAPCS64, Windows x64 later) under that exact symbol.
- `ail build FILE --lib static|shared [--header OUT.h]`: static = `.a` (ar of one object), shared = `.so` / `.dylib` / `.dll`. `--header` writes a C header from the exports, so the C side never hand-writes prototypes (AI-first: one source of truth).
- The ELF writer gains defined dynamic symbols (real DT_HASH or DT_GNU_HASH, `.dynsym` with STB_GLOBAL), Mach-O gets an export trie, PE an export directory. Today's "defines no symbols" hash goes away.
- Entry: an exported fn called from C runs `rt` init lazily on first call (heap, tasks). A trap inside an export aborts the process with the usual trap text; it never unwinds into C. Rule: `ffi.export.*`.
- Checker errors with `ail explain`: non-C-compatible type in an export, duplicate symbol, `mut` parameter, generic export.

### S2. Callbacks

- Pass an AIL function to C: type `extern c fn` pointer `cfn(i32) i32`, built from a named top-level `pub export c fn`-compatible fn, not a closure (no capture, no environment, no GC object behind a C pointer).
- Closures reach C only through the `void *user` idiom: `mem` block plus a plain fn. A helper `ffi.trampoline` is deferred until measured.
- Rules `ffi.cb.*`; calling a `cfn` is unsafe like any C call.

### S3. `@layout(c)` structs and by-value passing

- `@layout(c) struct P { x i32, y i32 }` has declaration-order fields with C padding; `@layout(c, packed)` removes padding and `@layout(c, align N)` raises alignment (the designer-approved scope: packed, align); `sizeof`, `alignof`, `offsetof` as compile-time consts. Allowed in `extern c fn` params and results by value (the ABI classification per platform: INTEGER/SSE regs on SysV, HFA on AAPCS64) and as `*P` everywhere.
- `use c "header.h"` then also imports structs, enums and `#define` integer consts (today they are left out).
- Variadics: `extern c fn printf(fmt *u8, ...)` with an explicit call-site type list, `printf(fmt, i32: 3, float: 1.5)`, so the checker still sees every argument's C type. Needed to replace C wholesale; lowest priority in this slice.

### Out of scope (Kaleb, 2026-11-05)

- No no-GC mode and no freestanding targets: AIL stays garbage-collected. Raw memory comes from `std.mem` and arenas under scoped unsafe (ADRs 0820, 0835).
- No 32-bit targets and no WASM (ADR 0007 stands).
- No inline assembly beyond the typed `hw.x64.*` / `hw.a64.*` intrinsics (OQ-137 option a); `asm fn` is not built.

## Finding: what a linkable library needs (2026-11-07)

Reading `compiler/codegen/image.ail`, `compiler/obj/link.ail` and `elf.ail`:

- Code addressing is already position-independent. Every relocation (`rel32`, `b26`, `adrp`, `add12`) is PC-relative to code, data or the globals block, and imports go through GLOB_DAT slots. So a shared object is mostly container work: `ET_DYN` at base 0, real `.dynsym` entries for the exports, a real `DT_HASH` chain (today an empty one), no `PT_INTERP`, no entry. Mach-O needs an export trie, and a static `.a` needs a relocatable `.o` that keeps these relocations as ELF/Mach-O relocation records (new writer code).
- The runtime is the hard part. It assumes it owns the process: `rt_start` initialises the globals block, `STACK_TOP` (the GC's stack scan root, `task.ail:789`) and the worker stacks region. A C caller enters on a foreign stack, maybe a foreign thread, with no initialised heap.
- Plan, single owner first: each export gets a generated entry stub that (1) takes one global library lock, (2) on first call runs runtime init exactly as `rt_start` does but on the caller's stack, (3) records the caller's `sp` as `STACK_TOP` for the outermost entry and restores it on return, (4) calls the AIL function, (5) releases the lock. Calls are serialised, so a collection never sees a foreign stack it does not know. Re-entry from a callback (S2) nests under the same lock and keeps the outer `STACK_TOP`.
- Needs a new runtime file (`rt/native/libentry.ail`, prefix `libentry_`) and a lowering file (`mir/lower_export.ail`); one dispatch line in `lower.ail`. Needs its own ADR for the lock and the stack-root rule (returned to Fable to number).

## Order and sizing

S1 (about 800 lines: parser, checker, symbol table, ELF/Mach-O writers, header emitter), then S3, then S2 (needs S1). S1 ships on Linux and macOS first; Windows export (PE export directory, Win64 ABI) follows once the Windows release gate is green. ADRs: S1 0950, S2 0951, S3 0952; 0953-0955 are returned to Fable.

Position-independent output is the hard part of S1: shared libraries need PIC code and relocations, while `elf.ail` links against fixed addresses today (`link(img, BASE+...)`). The static library (`.a`) comes first for that reason; `--lib shared` follows once the codegen can emit PIC.

## Rejected alternatives

- Generating C and calling a C compiler: breaks ADR 0010 (no external backend).
- Closures as C callbacks: hides a GC root behind a raw pointer.
- No-GC mode, freestanding targets and `asm fn`: declined by the designer (2026-11-05).
