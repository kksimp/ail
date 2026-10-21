# 0563. The first native collector is conservative on the stack and registers

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by: AI (delegated, ADR 0076)**
- **Topic:** Native heap and collector (lane 171)
- **Spec rules:** [native.build.1](../spec/23-native.md)
- **Design note:** [native-abi.md](../design/native-abi.md)

## Context

The backend has no stack maps yet: it does not say which stack slots or registers
hold pointers. Lane 120 will emit them later.

## Options considered

A: precise collection now, requiring stack maps first. B: conservative scan of the
stack, registers and globals, with precision inside objects by kind (chosen).
C: reference counting.

## Decision

B. A collection stops the program, saves the callee-saved registers with the primitive `gc_regs(buf)` (which also
returns the stack pointer), and scans every word from the stack pointer to the saved
stack top (the `argv` address `rt.init` receives), those registers, and the global
slots. A word that falls inside an allocated block, at any offset, marks that
block. Marked blocks are scanned by kind: a list header's data pointer only; strings,
bytes and pointer-free blocks not at all; everything else (list data, structs, enums,
closures, maps, sets, boxed values) word by word, also conservatively. Interior
pointers are accepted because the optimiser may keep only a derived pointer in a loop.
A pointer one past an object's end is not accepted.

## Tasks (amended with lane 178, native threads)

With tasks on OS threads, a worker may run a loop that never allocates, so
"all tasks are at a call" does not hold. The protocol (native-abi.md §10):
`alloc` and the other runtime entry points that can collect hold `ALLOC_LOCK`;
the collector sets `GC_REQ`; every other thread parks at its next `alloc`,
lock wait or task wait, after `gc_regs(buf)` and publishing its stack pointer;
`rt.gc_stop_world()` returns when all are parked and `rt.gc_start_world()`
releases them. `gc_collect` scans, besides the main stack, each parked
thread's range from its published stack pointer to its stack top (plus its
saved registers) with `gc_scan_range`, then marks and sweeps as before.
Without tasks both calls are no-ops. A non-allocating worker delays a
collection until it next allocates or waits; that is accepted. Heap state
(free lists, bump pointers, page table) is only touched under `ALLOC_LOCK`.
Kinds 12 (task record), 13 (environment record) and 14 (atomic cell) are
scanned word by word, like structs.

## Later: precise stack maps

Lane 120 emits a safepoint table per call site (live pointer slots and registers).
The collector then scans only those, from the frame chain. The page table,
headers, mark stack and sweep stay as they are; only `gc_collect`'s root step changes.
Per-kind layouts (the element type of a list's data, a struct's pointer words) let
objects be scanned precisely and let kind 11 cover scalar lists too.

## Rejected alternatives and criterion

Criterion: how soon native programs stop leaking (the lane goal). A is blocked on stack
maps. C cannot free cycles and costs a count update on every store.

## Consequences

A scalar that looks like a heap address keeps a block alive. This costs memory,
never correctness. The collector is not moving, so addresses held in registers
and in `str_addr` integers stay valid. Programs where a pointer exists only as
an unaligned or tagged value would be missed: nothing in the ABI does that.
Tested with a collection every 4 KiB over every `run/` test and six perfbench programs.
