# 0614. Standard-library functions in the native runtime

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** std.path.1-4, std.hash.2-4, std.random.1-4, [native.subset.1](../spec/23-native.md)
- **Builds on:** ADR 0393 (native modules and the runtime), ADR 0580-0582 (the OS layer)

## Context

Lane 174 was asked to write standard modules in AIL and compile them like
user modules. The coverage tracker lists `std.path`, `std.hash` and
`std.random` in 14 programs. Their functions are pure computations on
numbers and `str`, so they need none of the types still missing (optionals,
results, maps, generics). Compiled as a user module they would pay for what
the checked subset costs them: a bounds check on every byte of a hash, a
trap on the wrapping multiplies that 64-bit hashes need, and every function
of the module in every program that names it.

## Decision

`path.NAME(...)`, `hash.NAME(...)` and `random.NAME(...)`, when the name is
not a local or a module the program imports, call the runtime function
`rt.MODULE_NAME`, written in AIL in `compiler/rt/native/path.ail`,
`hash.ail` and `random.ail`. Its parameter and result types are its
declaration's, so a call is type-checked like any other and a new function
is one declaration with no change to the lowering. The runtime is compiled
with wrapping arithmetic and unchecked indexing and only the functions a
program reaches are linked in (ADR 0393).

- **`std.path`:** `dir`, `base`, `ext` and `join` over `/` (std.path.2-4).
  The Windows separator and drive rules wait for the Windows target.
- **`std.hash`:** `crc32`, `hash64` (bit for bit the interpreter's, which
  stored values depend on), `sha256`, `sha384`, `sha512` and `hmac_sha256`
  over the string's UTF-8 bytes. The 64-bit product `hash64` folds is made
  from 32-bit halves.
- **`std.random`:** one splitmix64 generator in global slots 62 and 63,
  seeded by `random.seed` or lazily from the clock (std.random.3).
  `random.int` draws by rejection from the smallest covering mask, so every
  value is equally likely (std.random.2); the lowering checks `lo <= hi` and
  traps at the call with the interpreter's message. `random.shuffle(mut xs)`
  is a Fisher-Yates swap of the list's elements, whatever their type.
  The sequence for a seed differs from the interpreter's, which std.random.3
  allows. Tasks sharing the generator are not synchronised yet.

## Rejected alternatives

- **Modules in AIL compiled like user modules, loaded from a `std/`
  directory.** More machinery (a loader, import paths for `std`, implicit
  scope by text search) and slower code, for functions with no use for the
  checks. It stays the route for modules that need user-level types, such as
  `std.time.date`, once optionals and results land.
- **Dispatch by a table in the lowering.** Every function would be added in
  two places.

## Consequences

- `ail build` programs using these three modules build and match `ail run`
  on x86-64 and aarch64, but for random streams.
- The global-slot map in native-abi.md gains 62-63.
- Still outside: `std.regex`, `std.time.date`, `std.tz`, `std.compress`
  and `std.ui`, which need types and features other lanes are building.
