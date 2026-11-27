# 1290. `std.wasm`: WebAssembly binary modules, read, checked and written

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.wasm.1 to std.wasm.10 in [std-wasm.md](../spec/std-wasm.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)

## Context

Programs that build, inspect or rewrite WebAssembly (a linker step, an import or export lister, a size report, a
patcher for a module's exports) have no std support: the binary format is LEB128 numbers, nested section
vectors and about 200 operations with six immediate shapes, and each program that needs it writes the reader again,
usually without the validation rules. The std plan (`std-gap-plan.md`, binary formats) lists it as a pure AIL module
on top of the `bytes` reads and pushes of type.bytes.15-18.

## Decision

A new module written in AIL, `std/wasm/wasm.ail`, in scope without `use` (std.use.4). No syntax change, no runtime,
checker or lowering work beyond the registration lines.

- `parse(bytes) Module ! WasmError` reads the header and every standard section. A `Module` is a struct of public
  lists (types, imports, function type indexes, tables, memories, globals, exports, start, element and data
  segments, bodies, custom sections) so a program edits it with ordinary list code.
- Function bodies and constant expressions are flat `List[Instr]` in file order, `end` included. An `Instr` is the
  opcode, a list of integer immediates and, for the two 16-byte SIMD forms, `data`. Flat lists keep a round trip
  exact and let a program insert an instruction with one `insert`; `block`, `loop`, `if`, `else` and `end` are the
  nesting.
- `write(Module) bytes ! WasmError` writes the sections in the format's order, each custom section after the one it
  followed. `parse(write(m)) == m` for every module `parse` makes. Numbers are written shortest, so a file keeps its
  content, not always its bytes (padded LEB128 and empty sections are not kept).
- `validate(Module) ! WasmError` runs the specification's validation algorithm: index ranges, limits, unique export
  names, the start function, the data count, typed constant expressions, balanced blocks, branch labels, local indexes,
  memory operations, and the operand stack and the types of every operation.
- Failures are `WasmError` with an enum `kind` (`BadHeader`, `Truncated`, `BadSection`, `BadValue`, `BadOpcode`,
  `Unsupported`, `Invalid`) and a text that names the byte offset (type.enum.7).
- Supported: the 1.0 core plus sign extension, saturating conversion, bulk memory, reference types, multi-value,
  tail calls and the fixed-width SIMD operations. Refused as `Unsupported`: the tag section (exception handling),
  threads and atomics, 64-bit and multiple memories, shared limits.

## Rejected alternatives

- **Built-in module in the runtime.** It would add checker, effects and lowering work and collide in
  `check/stdlib.ail`; reading a file once is not hot enough to need it.
- **A nested instruction tree** (`Block(kids)`). It makes a flat edit (insert before the final `end`) a tree walk
  and a round trip harder to keep exact; the nesting is recoverable from the flat list with a depth counter.
- **One enum variant per operation.** About 250 variants with their immediates would take the whole card budget of
  the module and make every walker match them all; `op int` plus `name()` is what the tools in this space need.
- **A structural-only `validate`.** The first draft did not follow the operand stack. Checking it against the
  WebAssembly specification test suite (2,200 `assert_invalid` modules) showed that a `validate` that passes a module no
  engine accepts is not one a program can trust, so `validate` now runs the specification's validation algorithm;
  an editing program gets the same answer as an engine would.
- **Keeping the original bytes of each section for exact round trips.** It would make an edited module disagree with
  its stale bytes; the shortest form is the canonical one.
- **A module named `module` or `binary`.** Common local names that would hide the module (std.use.4); `wasm` clashed
  with none.
