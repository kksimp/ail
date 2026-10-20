# 0599. Native threads: clone and futex on Linux, one pool, helpers wait

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer asked for multi-core by default on 2026-10-20)
- **Topic:** 11 — Concurrency; 22 — Runtime
- **Spec rules:** [conc.task.1–6](../spec/11-concurrency.md), conc.par.7, conc.par.9, [native.task.1](../spec/23-native.md)
- **Builds on:** [ADR 0095](0095-colorless-tasks.md), [ADR 0096](0096-scope-bound-tasks.md), [ADR 0387](0387-static-executables-and-ail-build.md)

## Context

Native executables link no libc (ADR 0387), so threads must come from the
kernel. The interpreter runs tasks one at a time; native code runs them in
parallel (conc.task.9). The designer wants at least four cores used by default.

## Decision

1. **Linux threads are raw `clone`** with `CLONE_VM|FS|FILES|SIGHAND|THREAD|SYSVSEM`
   on a stack from one `mmap` region (64 stacks of 8 MiB, lazily committed,
   a guard page at each low end). Two new primitives do the part that can't
   be written in AIL: `clone(top, arg)` returns in the creator the thread id
   and starts the child at `rt.worker_main(arg)` on the new stack; `restart(top, arg)`
   does the same in the running thread, abandoning its frames. Both are
   treated as calls by the register allocator. Sleeping is `futex` wait and
   wake on 8-byte words, and the locks are Drepper's three-state futex mutex,
   written from his description.
2. **One pool, sized to the cores.** `sched_getaffinity` gives the cores, at
   most 8 (conc.par.9); workers start lazily, one per queued task until the
   pool is full. Idle workers sleep on a sequence word; a push wakes one.
   Tasks wait in **one locked FIFO queue**, not per-worker deques. The queue
   lock is held for a pointer move, so coarse tasks and the loop chunks of
   ADR 0601 don't contend; per-worker deques with stealing replace the queue
   if a benchmark shows the lock (the interface, `task_start` and the worker
   loop, doesn't change).
3. **Waiting never runs tasks.** A waiting thread sleeps on the task's state
   word. A worker that waits stops counting as running, and a replacement
   worker starts when work is queued and nobody could take it (up to 64
   workers), so a task that waits on tasks can't starve the pool. The main
   thread isn't a worker: it sleeps in `wait`, so a pool of four keeps four
   cores busy. Running tasks inside `wait` would need a way to abandon a
   task's frames on a trap anyway, which `restart` gives only on a worker.
4. **A trap in a task ends the task** (conc.task.6). `trap` first calls
   `on_trap`, which on a worker saves the trap's operands in the task record,
   publishes state 3 and restarts the worker. `t.wait()?` in `main` then ends
   the program with the interpreter's line,
   `error: Trapped(rule: "R", message: "L:C: …")`, on standard error and
   exit 1; the interpreter's `at` lines after it may be absent (native.build.1).
5. **The runtime links only what a program uses.** `task.ail` is part of the
   runtime when the source mentions `spawn `, `par for` or `Atomic`;
   otherwise `notask.ail` supplies an empty `on_trap`. The shared runtime
   (`core.ail`) gains the locks, thread identity (a worker is found from its
   stack pointer), a reentrant output lock that `print` holds for one
   statement while tasks exist, and a lock round `alloc` once a second thread
   exists (`MULTI`). A program without tasks pays two loads and branches per
   allocation.
6. **Subset.** `t := spawn f(args)` for a free function over the subset's
   types (a list argument is copied, conc.share.1), `t.wait()?` as a
   statement or operand in `fn main() !`, whose `?` has no other source in
   the subset. Everything else about errors, `TaskGroup`, channels, mutexes
   and cancellation waits for the lanes that own optionals and errors.
7. **macOS.** Apple supports threads only through `libSystem`, so workers are
   `pthread_create` threads on the same stack region (`pthread_attr_setstack`),
   waits are `__ulock_wait`/`__ulock_wake`, and the core count is
   `sysconf(_SC_NPROCESSORS_ONLN)`. Raw `bsdthread_create` is private and
   unstable and is rejected. The runtime reaches libSystem through the general
   import mechanism of native-abi.md §11: `import_call("_name", a, b, c, d)` is
   a MIR call named `libsys._name`, and the Mach-O writer binds one pointer slot
   per distinct name from a bind table (`LC_DYLD_INFO_ONLY`). A trapped worker
   restarts at the stack pointer its `thread_entry` recorded, since the pthread
   keeps its own record at the top of the stack. Atomics need no thread.
8. **Windows** will use `CreateThread` and `WaitOnAddress` from kernel32 when
   lane 187's PE writer has imports; nothing here depends on it.

## Rejected alternatives

- **A thread per task:** simplest, but a loop of 10,000 `spawn`s would ask for
  10,000 stacks; the pool bounds threads and reuses stacks.
- **Per-worker deques with stealing now:** more code and more memory ordering
  to get right before anything shows the queue lock matters.
- **Helping (a waiting thread runs queued tasks):** keeps the pool smaller,
  but a task that traps inside a helper would need non-local exit on the main
  stack. Replacement workers cost a thread and no cleverness.
- **User-level stack switching (coroutines):** needs the compiler to know the
  frames; the kernel scheduler and futexes already do the job.
- **Conservative stop-the-world now:** the allocator never frees (ADR 0407);
  threads stop for collection when lane 171's collector exists (the plan is in
  native-abi.md §10).

## Consequences

- Four CPU-bound tasks run 3.2x faster than the same calls in a row on four
  cores (Linux x86-64, a shared cloud VM).
- The parse flag of `s.to_int()` is still one global (`PARSE_OK`): two tasks
  parsing at once can read each other's flag. It moves into the result when
  lane 175 gives the subset results.
- `AIL_THREADS` isn't read yet; the pool size follows the affinity mask.
