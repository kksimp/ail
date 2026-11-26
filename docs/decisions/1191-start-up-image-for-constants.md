# 1191. Compute constants at build time and keep them in a start-up image

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the feature ("fast launch", turned into build-time start-up work)
- **Topic:** Native backend: start-up
- **Spec rules:** none (no language behaviour changes; [native-abi.md §12](../design/native-abi.md))
- **Amends:** [ADR 0604](0604-native-computed-constants.md) (where a computed constant's value comes from)

## Context

A module constant that isn't a constant expression (`TABLE := build(2000000)`, `WORDS := ["if", "else"]`,
`ORIGIN := Point(x: 0, y: 0)`) is filled by the generated `init.consts` before `main`, on every launch
(ADR 0604). Measured on the Mac on main `79e58f73`: hello launches in 2.0 ms, level with C; one constant of
2,000,000 `int`s adds 5 ms and 16 MB of private memory to every launch. A constant may only call functions with
no effects (ADR 0225), so its value is the same on every run. C programmers generate source files to get tables
into the binary; Go and Python run their initialisers at every start.

## Decision

`ail build` (and so `ail run` and `ail test`, which build through it) computes the constants once, at build
time, and puts the finished objects in the executable's data: the **start-up image**.

- **How it is made.** The program is lowered with one more call at the start of `main`, after `init.consts`:
  the call hands the constants to the runtime (`compiler/rt/native/image.ail`), which walks the objects they
  hold, writes them to a file as the image, and ends the process. The build runs that executable once, then
  lowers the program again with the image in place of those constants' code.
- **What an image holds.** Constants whose type is made of numbers, `bool`, `char`, text, `bytes`, lists,
  optionals, tuples, structs and enums. A constant of any other type (`Map`, `Set`, a `ref`, a function value,
  a tensor) is computed at start-up as before; the two kinds may read each other.
- **How pointers are told from numbers.** By type. The lowering gives the writer each constant's shape, the
  text a table already gets for a composite key (ADR 0866), extended to say which lists keep their structs in
  place (ADR 1149). The writer follows the shape, so a number is never taken for a pointer, and it checks what
  it finds against the shape (object kinds, element block kinds); a mismatch means no image.
- **When there is no image.** A constant that traps, an object that disagrees with its shape, a cross build, an
  instrumented or sanitized build, `AIL_OPT_OFF=constimage`, or a target other than macOS: the constants are
  computed at start-up, exactly as before. `AIL_IMAGE_WHY=1` makes the build say why. No program's behaviour
  depends on the image.
- **Where it lives.** In the part of `__DATA` the file backs, ending where the globals start, which is how the
  runtime finds it without a symbol or a fixup. At start-up `init.image.load` adds the load address to each
  pointer (one pass over a list of offsets) and copies each constant's word to its global slot. The collector
  already ignores addresses outside the heap, so it neither scans nor frees the image.
- **A program with no such constant** is lowered once and is the same bytes as before.

## Why tables stay out for now

A `Map` or `Set` remembers the last key looked up and the last key missed (ADR 0936), by the address of the key's
string object. In the heap the table keeps that string alive. A table in the image would not (the collector does
not scan the image), the string could be freed and its address reused, and a later lookup could hit the stale
entry. Tables need either a list of image tables for the collector to scan or caches keyed by content first.

## Audit: what can write to a constant's storage

- The language: a constant is never a place. Binding one to a `mut` name copies it (ADR 0060), a struct read
  from a place is copied (ADR 0392), and enum objects are never written. Nothing frees an object by hand.
- The runtime, on reads: lists, strings, structs, enum objects and boxes are not written. Tables are (above).
  A `ref` or cell could be assigned a heap pointer by the program, so those types are kept out by type.
- The collector: marks and sweeps only addresses inside the heap (`gc_find`).
- The image is writable memory, so the load-time pointer fix-up needs no change of protection, and a stray write
  would behave as it does to a heap constant today.

## Measured

Mac (Apple Silicon), median of 300 launches, the branch against the main it merged; job
`const-image/job.sh`.

MEASURED_TABLE

## Consequences

- Start-up no longer grows with the size of a program's constant tables; pages of a large table are read from
  the file when first touched and only the pages holding pointers become private memory.
- A build of a program with such constants lowers it twice and runs it once up to `main`. A constant that takes
  long to compute makes the build take that long, in exchange for no cost at launch.
- The two lowerings must lay values out alike. Both decide which lists keep structs in place from the program's
  own code, which is the same in both; the writer's kind checks catch a block that disagrees.
- The executable grows by the image; before, the same bytes were heap memory in every running process.
- Linux and Windows compute constants at start-up until their object writers get the same data section
  ([ADR 0008](0008-strict-lockstep.md): the image is an optimisation with a fallback, not a behaviour).

## Rejected alternatives

- **Compress the code and unpack it on demand** (the designer's first idea): the system already loads code page
  by page from the file cache; unpacked pages would be private memory, and signed code can't be produced at run
  time on Apple Silicon.
- **An evaluator inside the compiler.** The interpreter is deleted (ADR 0904), and a second implementation of the
  runtime's lists, text and formatting would have to agree with the native one in every case. Running the real
  code can't disagree with itself.
- **Telling pointers from numbers as the collector does** (a word is a pointer when it falls in a live object),
  checked by running the writer twice and comparing. The first version; measured on the Mac it never produced
  an image. The heap starts at the same address in every run, so two runs can't expose a number taken for a
  pointer, and a table of `int`s near the heap's addresses (squares around 4.8e11) is full of such numbers.
- **Serialisation code generated per type.** Exact, but a new generated function per type. The shape text
  already exists for table keys and one runtime walker reads it.
- **dyld rebase entries in place of the runtime's fix-up loop.** No runtime code, but macOS only; the loop is the
  same on every object format.
- **One lowering, with both paths in the executable.** Saves the second lowering but leaves the constants' code
  and the image writer in every shipped program.
