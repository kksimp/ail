# 0844. Native `std.mem` over runtime page blocks, and a fixed libc subset for `extern c fn`

- **Status:** Accepted
- **Date:** 2026-10-22
- **Decided by:** AI (delegated, ADR 0076), option (a) chosen by the integrator
- **Topic:** 23 — Compiler architecture
- **Builds on:** [ADR 0436](0436-native-c-calls-by-dynamic-imports.md), [ADR 0835](0835-native-raw-pointers-and-hw.md), [ADR 0823](0823-lower-ail-feature-files.md)

## Context

Native executables are static, with no libc and no dynamic linker (ADR 0387), so `ail build` rejected every `extern c fn` and all of `std.mem` (`ffi.native.1`). Of the examples, `ffi_libc` and `ffi_buffers` could not build. Real linking would be a project of its own.

## Decision

1. **`std.mem` natively.** `alloc`, `free`, `cstr`, `from_cstr`, `cast` and `null` (`unsafe.mem.1-3`), in `compiler/mir/lower_mem.ail` and `compiler/rt/native/ffi.ail`. A block is its own page mapping with a 16-byte header holding its length: `mem.free` returns it with `munmap` (Windows keeps it until exit). The collected heap never sees it. `alloc` takes the interpreter's traps for a negative or impossible count, `free` of null and `from_cstr` of null (same text); out-of-block and freed accesses are undefined, as in C (native-abi.md Memory).
2. **A fixed libc subset.** An `extern c fn` whose name and declared types equal one of `hypot(float, float) float`, `abs(i32) i32`, `toupper(i32) i32`, `strlen(*u8) int`, `atoi(*u8) i32`, `memset(*u8, i32, int) *u8`, `memcpy(*int, *int, int) *int`, `strtod(*u8, **u8) float` is a function whose body calls the runtime function `rt.ffi_NAME`. The library name is ignored. Anything else keeps `ffi.native.1` with a message that the build links only a fixed subset.
3. **Behaviour.** C locale: `toupper` changes `a-z` only; `atoi` and `strtod` read decimal text (`strtod` rounds correctly through the runtime's `to_float` digit code, no hex or `inf`/`nan` words); `hypot` scales large and tiny operands, so its last bit can differ from libm there.
4. **`lower.ail`** changes in three existing lines: `declare_fn` accepts a known `extern`, the missing body of a function dispatches to `lower_extern`, and the `mem.` problem becomes a call of `lower_mem_call` (net fewer lines).

## Rejected alternatives

- **Dynamic linking (option b).** A loader, GOT stubs and a Mach-O/PE equivalent; out of scope.
- **Allocating `mem` blocks on the collected heap.** The collector would reclaim a block C code still holds, and `free` could not be honoured.
- **A block table to keep `unsafe.ptr.5-6`.** It costs every access and the interpreter's table has no native counterpart; the differential harness runs those traps under the interpreter only.
- **Calling libm/libc by system call.** Neither exists as a syscall.

## Consequences

`examples/ffi_libc.ail` and `examples/ffi_buffers.ail` build natively and print what `ail run` prints. Tests `run/ffi_libc`, `ffi_free_null`, `ffi_alloc_negative` and `ffi_from_null` compare with `ail run`. Adding a libc function is one `FFI_NAMES` entry and one `ffi_NAME` runtime function.
