# 0926. `use c "header.h"`: a subset C header reader that writes `pub extern c fn` declarations

- **Status:** Accepted (implements [ADR 0103](0103-c-header-import.md)'s `use c "header.h" as name`; builds on [ADR 0870](0870-native-extern-c-bound-lazily-with-dlopen.md))
- **Date:** 2026-10-28
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 14 — FFI
- **Spec rules:** [ffi.decl.1](../spec/14-ffi.md)

## Context

ADR 0103 chose header import with AIL's own parser (no clang), and the spec
has said since then that `use c "header.h" as name` makes a header's C
declarations available under `name`. Nothing built it. The native CLI binds
`extern c fn` outside the libc subset lazily (ADR 0870), so the missing piece
is only the source of the declarations.

## Options considered

### Option A — a small C header reader in AIL that writes an AIL module (chosen)
The reader tokenises the header (comments, preprocessor lines and literals
dropped), takes each top-level declaration, and maps every function prototype
whose types are in the FFI set (`ffi.decl.3`) to a `pub extern c fn` line. The
lines are the source of a module that `name` refers to, so the checker, the
formatter, `ail api` and the lowering see an ordinary module and need no new
construct.
- **Pros:** about 300 lines of AIL; no C preprocessor, no external tool (ADR 0009, ADR 0010); the module text can be shown by tooling, as ADR 0103 asks.
- **Cons:** not a full C parser; a prototype that needs a macro to read is skipped.

### Option B — a full C front end with a preprocessor
- **Pros:** reads any header.
- **Cons:** a project of its own (`#include` search, conditionals, macro expansion, GNU extensions) for a feature whose users call a few functions.

### Option C — libclang or clang's AST dump
- **Pros:** exact.
- **Cons:** an external tool and a dependency on its version, against ADR 0009 and ADR 0010; absent on a clean Windows machine.

### Option D — hand-written `extern c fn` only
- **Pros:** exists today (ADR 0103, item 2).
- **Cons:** signatures copied by hand drift and hold mistakes, the reason ADR 0103 chose header import.

## Decision

1. **Input.** The reader looks at the named header's own text only; it does not follow `#include` and evaluates no `#if`. The first of two declarations of a name wins, so the arms of an `#if` are not an error.
2. **Taken.** Function prototypes and function definitions' signatures, with `typedef` of scalar types read from the header itself. `extern "C" {` guards are looked through. `__attribute__`, `__asm__`, `__declspec` groups, qualifiers (`const`, `volatile`, `restrict`, `static`, `inline`, `extern`) and identifiers starting with `__` that are not types are ignored.
3. **Types** (ffi.map.1): `char` is `u8` behind a pointer and `i8` otherwise, then `short`, `int`, `long` and `long long` with `unsigned` as `i16`/`u16`, `i32`/`u32`, `i64`/`u64` (`long` is `i32`/`u32` on Windows, `i64`/`u64` elsewhere), `float` is `f32`, `double` is `float`, `_Bool` is `bool`, `size_t`/`int32_t` and the other standard typedefs by a fixed table. Each `*` is one `*`; `void *`, `struct X *` and a pointer to a typedef the header does not define are `*u8`; `T[]` in a parameter is `*T`; an `enum X` value is `i32`.
4. **Skipped, not an error:** variadic functions, struct or union by value, function-pointer parameters or results, `long double`, more than six integer-like or eight floating parameters (`ffi.decl.3`), names that are AIL keywords. A skipped name is simply absent, so using it is the ordinary `mod.use.2` "no item".
5. **Where the module comes from.** `use c "h.h" as libc` finds `h.h` next to the importing file, then in `/usr/include`, `/usr/local/include` and `/opt/homebrew/include`, then in the macOS SDK. A header that is not found is error `ffi.decl.1`. Parameters are named `a0`, `a1`, ... because the AIL call passes values by position.
6. **Hand-written declarations still work** (ADR 0103, item 2), and a header import adds nothing at run time: the first call of each function binds it by `dlsym` (ADR 0870).

## Rationale

The feature's value is not retyping signatures. A reader that is exact on the
declarations a C library actually exports (plain scalars and pointers) and
silent on the rest gives that value at a small cost, in AIL, with no tool the
designer would have to install. Skipping an unsupported prototype keeps the
rest of a large header usable.

## Consequences

- `ffi.decl.1` gets a runnable example (the block in `14-ffi.md` is no longer `skip=bootstrap`).
- Headers that declare their API through macros (for example `#define f(x) ...`) expose less than the C compiler sees.
- Callbacks, structs by value and variadic functions stay out until the FFI chapter specifies them.

## Rejected alternatives

- A C preprocessor and `#include` following (option B): the cost is out of proportion to the use.
- libclang (option C): external tool, against ADR 0009 and ADR 0010.
- Hand-written declarations only (option D): the drift ADR 0103 set out to remove.
