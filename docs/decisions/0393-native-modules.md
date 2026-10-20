# 0393. Modules in the native backend

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), native.build.1, mod.unit.1-5, mod.use.1-2
- **Builds on:** [ADR 0097](0097-directory-modules.md), [ADR 0099](0099-qualified-imports.md), [ADR 0153](0153-package-root-and-loading.md), [ADR 0392](0392-native-structs-and-methods.md)

## Context

The AIL compiler is 30 modules in directories under `compiler/`. For it
to build itself, `ail build` must load a package as `ail run` does and
keep names of different modules apart, and a trap in a module must name
that module's file and line.

## Decision

1. **Loading.** `ail build FILE` takes `FILE` as the root module and its
   directory as the package root; `ail build DIR` takes every `.ail` file
   in `DIR` (mod.unit.3, mod.unit.5). Each `use a.b` loads every `.ail`
   file of `ROOT/a/b`, once; standard modules are not loaded.
2. **Names.** Every item's MIR name starts with its module's import path
   and a dot (`syntax.lex`, `ast.Pos`, `mir.Lower.emit`), as the
   runtime's start with `rt.`; the root module's names have no prefix.
   Inside a module, bare names are its own; `m.x` goes through the file's
   imports (`use a.b` makes `b` mean `a.b`; `as` renames). Struct names
   in types, constructors (`geo.Point(...)`), functions of a type
   (`geo.Shape.unit()`) and constants (`geo.SCALE`) all resolve this way.
   A constant's value is lowered in its own module.
3. **Positions.** Each problem and each trap site records its file:
   problems print with it (`Problems.files`), and a trap's message and
   excerpt use it (`Site.file`, `Module.files`), as the interpreter does.
4. Struct names are declared for every file before any field, so a field
   may name a struct from any file of any module.
5. **The runtime is lowered on demand:** only the runtime functions the
   program reaches (from its calls and the few the code generators call
   by name) are lowered and linked. The runtime grows with every native
   lane, and most programs use little of it; compiling it once per target
   and caching the machine code was rejected for now (below).
6. The lowering no longer keeps or checks a function that used something
   already reported (a function or type outside the subset), so one
   problem doesn't cause an internal error later.

## Consequences

- `ail run compiler mir compiler` now lowers the whole compiler and lists
  its remaining problems in one run: about 1,070, nearly all enums,
  optionals, maps, `char` and string interpolation, which lanes 172–179
  own.
- The interpreter checks visibility (`pub`, mod.vis.1) and cycles
  (mod.use.5); `ail build` runs on programs `ail check` has accepted, so
  the backend doesn't check them again.

## Rejected alternatives

- **A cached, precompiled runtime per target** (raised when lane 177's OS
  layer took a cold `ail build` from about 1.1 s to 2.1 s). It would tie
  a cache format to code generators that change daily, and the
  self-compiled compiler will run lowering 20–100 times faster, removing
  most of the cost. Revisit if build time still matters after the
  self-compile.

- **Concatenating all files into one program.** Names of different
  modules would clash (`compiler/` has several `new`, `parse` and `Gen`),
  and traps would name the wrong file.
- **Separate compilation per module with a linker.** More work and slower
  for a whole-program backend that inlines across modules; one MIR module
  for the package keeps inlining and dead-code removal global.
