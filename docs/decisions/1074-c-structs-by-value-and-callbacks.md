# 1074. C structs by value and C callbacks

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer put the language review's FFI item in 0.1.0
- **Topic:** 14 — FFI
- **Spec rules:** [ffi.struct.1-3, ffi.cb.1-4](../spec/14-ffi.md); amends ffi.decl.3 and ffi.decl.5
- **Builds on:** [ADR 0432](0432-c-calls-through-two-register-files.md), [ADR 0870](0870-native-extern-c-bound-lazily-with-dlopen.md), [ADR 1031](1031-variadic-c-calls.md), [ADR 0455](0455-function-types-and-values.md)
- **Amends:** [ADR 0952](0952-layout-c-structs.md) (structs by value no longer wait for `@layout(c)`)

## Context

Most real C libraries pass small structs by value (`div_t`, complex numbers,
vectors and colours in graphics libraries) and take callbacks (`qsort`, event
handlers, iteration). AIL could do neither: `ffi.decl.3` allowed only scalars
and pointers, and `ffi.decl.5` mapped a function pointer to an opaque `*u8`
that AIL could not produce. ADR 0952 planned by-value structs only for
`@layout(c)` structs, which live in raw memory and are deferred past 0.1.0.
The three platform conventions differ for aggregates: System V classifies
eightbytes and puts large structs on the stack, AAPCS64 has homogeneous float
aggregates and `x8` for large results, Win64 passes anything but 1, 2, 4 or 8
bytes by reference. An AIL struct is a GC object of 8-byte slots, and AIL
functions use the C register convention, except that Win64 makes the callee
keep rsi, rdi and xmm6-15, which AIL functions use freely there.

## Options considered

### Structs

#### Option A — any plain struct of C-typed fields, copied at the boundary (chosen)
- The struct stays an ordinary AIL value; the call lays it out as C does and
  the result is a new value. The checker names the offending field.
- **Pros:** no second struct representation, no annotation to learn; value
  semantics hold (C gets a copy); works today.
- **Cons:** a copy per call (a few dozen instructions in the runtime).

#### Option B — only `@layout(c)` structs (ADR 0952)
- **Cons:** needs the raw-memory struct feature first (deferred), and a
  by-value C struct would then be a type that is not a value.

#### Option C — an explicit marker on every C-shaped struct
- **Pros:** the intent shows at the declaration.
- **Cons:** more tokens for nothing the checker cannot already see; the error
  at the `extern` declaration names the field anyway.

### Where the classification runs

#### Option A — the runtime, from a descriptor string, through one call kernel per architecture (chosen)
- Lowering stays target-independent (MIR does not know the target); the
  runtime (`rt.ffi_cs`) reads `target()`, lays out, classifies and fills a
  frame; a 60-instruction kernel loads the registers, copies the stack words,
  calls and stores the result registers.
- **Pros:** one implementation of the three conventions in readable AIL; the
  code generators gain only a kernel each, nothing in the register allocator.
- **Cons:** a struct call costs a frame allocation and the classification at
  run time. Scalar-only calls keep the direct path (ADR 0870).

#### Option B — classify in the code generators
- **Pros:** no run-time cost.
- **Cons:** multi-register results, `x8`, stack aggregates and hidden pointers
  in two allocators and three conventions; much more code where bugs are
  hardest to see. A later speed slice can specialise hot shapes.

### Callbacks

#### Option A — a declared function's name, as a typed `fn(...)` parameter or `mem.c_fn(f)` (chosen)
- `extern c fn qsort(base *int, n u64, size u64, cmp fn(*int, *int) i32)`
  checks the callback's type against the function passed; `mem.c_fn(f)` gives
  the same entry as a `*u8` for header-imported function pointers, variadic
  arguments and pointers stored in C memory.
- **Pros:** the signature is checked where it can be; the compiler emits one C
  entry per function used, statically; no run-time code generation.
- **Cons:** no captured state: C keeps only a code address. State goes through
  the user-data pointer C libraries provide.

#### Option B — any `fn` value, including closures
- **Cons:** a closure (ADR 1067) is an object; C would need a trampoline made
  at run time per closure (executable memory, W^X on Apple). Can follow later.

#### Option C — only `mem.c_fn`
- **Cons:** the callback's signature would go unchecked in hand-written
  declarations.

## Decision

1. A struct whose fields are all C-layout types or such structs, with no type
   parameters, invariants or unique/error kind, can be a parameter or result of
   an `extern c fn` (`ffi.struct.1`). It crosses in C's natural layout and the
   platform's convention (`ffi.struct.2`); C gets a copy and the result is a
   new value (`ffi.struct.3`). Structs count toward neither register limit;
   whatever no longer fits goes on the stack as each convention says.
2. A `fn(...)` parameter of an `extern c fn` is a C function pointer of C types,
   at most six integer-like and eight float parameters (`ffi.cb.1`); its
   argument is the name of a declared function (`ffi.cb.2`). `mem.c_fn(f)` is
   that entry as a `*u8` (`ffi.cb.3`).
3. The entry `cb#f` checks that it runs on a thread AIL started (the main
   thread or a task worker; the collector scans only those stacks) and traps
   with `ffi.cb.4` otherwise, converts the arguments as `ffi.map.2` does for
   results, calls `f`, and converts the result for C. On Windows a generated
   adapter keeps rsi, rdi and xmm6-15 and moves the arguments from their Win64
   positions to the AIL order. A trap's stack walk stops at the C frames.
4. Not in this slice: structs by value in callbacks, exports and variadic
   calls; callbacks with captured state.

## Consequences

- `div`, `lldiv`, the C99 complex functions, and graphics and maths libraries
  that pass vectors and colours by value bind directly; `qsort`, `bsearch` and
  event-callback APIs work.
- `tools/ffi-abi/run.sh` (and `run.ps1`) builds a C library with every struct
  class and callback shape on the host and checks the AIL side against it.
- The header import (ADR 1010) still maps function pointers to `*u8` and still
  skips prototypes with structs by value; mapping C structs from headers is
  follow-up work that can now produce plain AIL structs.
- ADR 0952's `@layout(c)` remains the plan for structs in raw memory; when it
  lands, such a struct by value follows these same rules.

## Rejected alternatives

- `@layout(c)`-only by-value structs: blocks on a deferred feature and makes a
  value-like type that is not a value.
- A marker on C-shaped structs: tokens without new information.
- Classification in the code generators: the most bug-prone place for it, and
  not needed for 0.1.0 speed (struct calls are rare in hot loops; a later slice
  can specialise).
- Closures as C callbacks: needs run-time trampolines; user data covers state.
- `mem.c_fn` alone: loses the checked signature.
