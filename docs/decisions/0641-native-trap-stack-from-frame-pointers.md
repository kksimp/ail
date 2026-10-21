# 0641. A native trap prints its stack by walking frame pointers over a call table in the code

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Native runtime diagnostics
- **Spec rules:** `trace.frame.1`

## Context

A native program that traps already printed the interpreter's first line and
source excerpt (ADR 0405) but not the stack, so an agent that built with
`ail build` had to re-run under `ail run` to see who called the failing code.
The interpreter's report names each frame's function and the position it was
executing (trace.snap.2).

## Options considered

### Option A — walk the frame pointers; a table of call positions in the code (chosen)
Every native function already keeps `rbp` (x86-64) or `x29` (ARM64) as a frame
pointer, so the saved frame pointer and the return address of each frame are
at `[fp]` and `[fp + 8]`. The code generators record, for every user call,
the return address's offset in the code, the call's position (file, line,
column) and the function it is in. The object writers need nothing new: the
table is appended to the code (`trace.tab`, 8-aligned) as 8-byte words, so
its addresses are layout-independent. The runtime (`rt/native/trap.ail`)
finds the table with the `code_tab` primitive, takes the frame pointer with
`frame_ptr`, binary-searches the return addresses, and prints one line per
caller. Nothing runs on the non-trapping path.
- **Pros:** zero run-time cost (no instruction added to any call; the
  perfbench programs run at the same speed); the same on every target;
  independent of the object format; no unwinder.
- **Cons:** 16 bytes of table per user call, plus about 1.5 KB of code for
  the printer: examples grow 10–15% (2–6 KB).

### Option B — a shadow stack the program maintains
Push and pop a record at every call.
- **Pros:** can carry argument values.
- **Cons:** a run-time cost on every call, against the goal of beating C.

### Option C — DWARF unwind and line tables
- **Pros:** standard; debuggers read it.
- **Cons:** a large format and an unwinder for a result a 16-byte row gives;
  the brief says no DWARF.

### Option D — store the call position in each frame
- **Pros:** no lookup.
- **Cons:** a store per call; a bigger frame.

## Decision

Option A. Tokens first (ADR 0076): an agent reads the failing call chain from
the one run it already made. Speed second: nothing is added to the fast
path. The cost is code size, which is reported here and is not opt-in:
the table is part of every build (ADR 0644).

## Consequences

- `rt.trap` ends with `trace_frames(tail)`; a runtime path that raises its
  own trap (the stack check, contracts) calls `rt.trace_frames` after its
  text to get the same stack.
- A thread's entry must clear the frame pointer, as `_start` does, or the
  walk reads garbage.
- Calls from a constant's initialiser (`init.consts`, ADR 0604) have no row:
  the interpreter evaluates constants lazily and shows no frame for them.
- Windows (x86-64, ADR 0597) uses the same frames; checked under Wine by
  lane 202.

## Rejected alternatives

Options B, C and D above.
