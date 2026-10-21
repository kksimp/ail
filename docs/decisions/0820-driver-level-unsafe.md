# 0820. A second manifest level, `unsafe driver`, unlocks hardware access through `std.hw`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** Designer (on the integrator's review of the proposal)
- **Topic:** 15 — Unsafe semantics; 14 — FFI; native targets
- **Spec rules:** none yet (`unsafe.*`, `mod.pkg.2` and `eff.infer.1` change with the first implementing PR)
- **Builds:** [ADR 0102](0102-unsafe-blocks.md), [ADR 0106](0106-unsafe-operations.md), [ADR 0430](0430-unsafe-blocks-and-functions-built.md), [ADR 0433](0433-raw-pointer-type.md)

## Context

The designer wants to write drivers and other low-level code in AIL, with
unsafe code allowed only where it is tagged ("driver"), and everything else
safe. AIL already confines unsafe: unsafe operations need an `unsafe` block
(ADR 0102), and a package may contain one only when `ail.pkg` says
`unsafe allow` (ADR 0430). What is missing is hardware access: there is no way
to get a pointer to a fixed address (ADR 0433 forbids int to pointer), no
volatile access, no barriers, no port I/O and no CPU instructions, which are
item 6 of `unsafe.op.1` without syntax.

The designer's "No" on a decision card on 2026-10-21 does not bear on this.
That card asked whether the bootstrap interpreter's Rust code may use `unsafe`
`get_unchecked` in the bytecode hot loop (ADRs 0619, 0621). It concerned the
Rust implementation, not AIL source code.

## Options considered

### Option A — a second manifest level, `unsafe driver`, with `std.hw` (chosen)
- **Pros:** reuses the opt-in that is already built and the place where
  dependencies are granted (`caps`, ADR 0232); a binding to a C library
  (`unsafe allow`) still can't touch physical addresses; one grep finds every
  package that can; zero cost to code that does not use it.
- **Cons:** two levels to learn instead of one.

### Option B — a `@driver` attribute on a module or function
- **Cons:** anyone editing the source can add it, and it is invisible where a
  dependency is accepted. Inside a package, `unsafe` blocks already mark the
  exact lines (ADR 0102), so the attribute adds a layer without removing risk.

### Option C — a capability `caps hw`
- **Cons:** effects propagate to callers, so every user of a driver's safe
  wrapper would need the grant. ADR 0430 rejected `caps unsafe` for the same
  reason.

### Option D — one level: `unsafe allow` also permits hardware access
- **Cons:** every C binding could then turn integers into pointers into
  device memory.

## Decision

1. **Levels.** `ail.pkg` takes `unsafe allow` (today's set) or `unsafe driver`
   (today's set plus `std.hw`), on the `package` line or a `dep` line. A
   package without `driver` that uses `std.hw` is an error at the use, naming
   the manifest line. A file with no `ail.pkg` needs
   `--unsafe driver` on `ail run`/`ail build` to use `std.hw`.
2. **`std.hw`** is its own module. Every operation in it is an unsafe operation
   (`unsafe.op.1` item 6), so it is only allowed inside an `unsafe` block:
   - `hw.at(addr, example)`, a `*T` at a fixed address, and the only route from
     an integer to a pointer;
   - `p.read_volatile()` and `p.write_volatile(v)`, never merged, removed or
     reordered against each other (spelling to be confirmed by tokbench);
   - `hw.fence()` and `hw.fence_io()`, in lockstep on x86-64 and ARM64
     (ADR 0008);
   - x86-64 port I/O `hw.in8/16/32` and `hw.out8/16/32`, a compile-time error
     on ARM64;
   - CPU intrinsics `hw.x64.*` and `hw.a64.*`, emitted by the in-house
     assemblers.
3. **Layout.** `@layout(c)` on a struct gives the fixed layout that ADR 0013
   promised. `*S` may point at such a struct, and the attribute is available to
   `unsafe allow` as well.
4. **Effect.** `std.hw` gives a new effect `hw` beside `unsafe`. Both propagate
   as before, and `@pure` rejects both.
5. **Reporting.** A tool reports, per package, its opt-in level and its counts
   of `unsafe` blocks, `unsafe fn`, `hw` sites, and public safe functions that
   contain them.
6. **Targets.** 64-bit only: the designer kept ADRs 0006 and 0007 (decision
   card, 2026-10-21). Drivers mean Linux user-space drivers and x86-64 or
   ARMv8.2-A machines. 32-bit microcontrollers, and Raspberry Pi 3 and 4 for
   bare-metal code, are out of scope.

## Rationale

The package is already the unit of trust and the unit of grants. A second
level keeps the boundary visible where a dependency is accepted, and splits C
bindings from hardware access, which carry very different risks. Free-form
inline assembly is left out ([OQ-137](../open-questions.md)): it needs a
register-constraint language and is opaque to agents. Intrinsics cover the
instructions that drivers need.

## Consequences

- Build order, each step its own PR: (1) `std.hw` and the `driver` level in
  the interpreter, with the checker ported to `compiler/check/` (ADR 0740);
  (2) native lowering of `*T`, `std.mem` and `std.hw`, with `ffi.native.1`
  narrowed to `extern c fn`, and volatile accesses as side-effecting MIR
  operations that bounds-check elimination, CSE, hoisting and auto-parallelism
  leave alone; (3) Linux user-space drivers through `mmap` of `/dev/mem`, UIO
  and VFIO; (4) freestanding targets `x64-none` and `a64-none`, with
  runtime hooks in place of the syscall shim (ADR 0406); (5) `@interrupt fn`,
  checked to allocate nothing.
- Language card: one line replaces the current `unsafe` line (estimated +20
  tokens). `std.hw` details go in a module card. The quick-reference row is
  paid for by a trim in the same PR.
- `gram.foreign.1` gains `volatile`, `asm`, and `&x`/`*p` as expressions,
  each pointing to the AIL spelling.
- macOS kernel or DriverKit drivers, Windows drivers and Linux kernel modules
  are out of scope.
- Proposal: `/mnt/project-files/notes/scoped-unsafe-design.md` (project files).

## Rejected alternatives

- A `@driver` attribute on a module or function (option B): rejected because
  it is invisible at the dependency grant, and anyone editing the source can add it.
- A capability `caps hw` (option C): rejected because it propagates to every
  caller of a safe wrapper.
- One level for C calls and hardware (option D): rejected because a C binding
  should not be able to reach device memory.
- Free-form `asm` now: deferred to OQ-137.
- 32-bit targets for microcontrollers: rejected by the designer; they would
  reopen ADRs 0006 and 0007.
