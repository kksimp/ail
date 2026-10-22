# 0535. Native recursion traps when the stack runs out

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.1](../spec/23-native.md), [err.trap.1](../spec/08-errors.md#traps)
- **Builds on:** [ADR 0385](0385-x86-64-code-generation.md) (trap stubs), [ADR 0405](0405-native-traps.md)

## Context

Unbounded recursion in a native executable overran the stack and died with
SIGSEGV, with no message and no position. The interpreter traps under
err.trap.1 after 10,000 nested calls (`call depth exceeded 10000 in `f``).
That number is the interpreter's, not the language's: a native frame is a
few dozen bytes, and native programs, the compiler among them, may recurse
far deeper.

## Decision

1. **Limit.** At start-up `rt.init` stores a stack limit in a runtime global.
   It is the argument vector's address (the top of the main stack) less the
   stack size limit (`RLIMIT_STACK`, 8 MiB when unknown or unlimited, at
   most 1 GiB) less a 1 MiB reserve, which is left for the trap itself.
2. **Check.** A call to a program's function whose callee is in the same
   strongly connected component of the call graph as its caller, so a call
   that can recurse, first compares the stack pointer with the limit. Below
   it, the call traps under err.trap.1 at its position:
   `call depth exceeded the native stack in `f``. This takes two
   instructions on x86-64 and five on ARM64.
3. **No check** on other calls. A chain of non-recursive calls is at most
   as deep as the program has functions, and the reserve covers it. Runtime
   calls are never checked.
4. native.build.1 records the difference: a native executable traps when
   its stack is nearly exhausted, not after the interpreter's count.

## Consequences

- Deep recursion ends with the interpreter's rule ID, a position and the
  callee's name instead of a crash. A program may recurse deeper natively
  than under `ail run`.
- Cost, in instructions measured with callgrind: binarytrees +3.3%
  (recursive calls are its work), the other eight drag-race programs
  +0.00%. After #219 (the stack check on recursive calls, `recurse.ail`) it
  measured +5.0% on binarytrees (247.0M to 259.5M); that is the accepted
  cost of a guaranteed clean stack-overflow trap.
- Threads (lane 178) need their own limit; the global is the main thread's.
- Indirect calls (lane 174) can't be placed in the call graph, so they
  should be checked.

## Rejected alternatives

- **Count calls to the interpreter's 10,000.** Matches `ail run` exactly,
  but adds a counter update to every call and return, and caps native
  recursion at a limit chosen for the tree walker.
- **Check every call, or in every prologue.** Simpler, but costs
  non-recursive code too: 4.9% on binarytrees and some on nbody, against
  3.3% and nothing.
- **A guard page and a SIGSEGV handler on an alternate stack.** Free at run
  time, but it needs signal syscalls per platform and a table from return
  addresses to call sites to report the position. It stays possible later
  with no change to programs.
- **A fixed 7 MiB budget without reading the limit.** Segfaults again
  under a smaller `ulimit -s`.
