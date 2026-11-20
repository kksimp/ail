# 06. Memory model and allocation

Chapter key: `mem`

> **Status:** Drafted (partial). This chapter collects what a program may
> observe about memory, what it may not rely on, what `unsafe` changes, the
> limits and the variables the runtime reads. It decides nothing new: every rule
> transcribes an accepted ADR or points at the chapter that owns the rule
> (`own`, `conc`, `unsafe`, `run`). Where the ADRs describe the native
> runtime's heap, the rule is tagged `[definitional]`: it describes the
> implementation, and a program must not rely on it. Not yet specified: the
> formal weak-memory model (`conc.mm.1` waits for it), generational or precise
> collection (ADR 0563 names them as later work), and a block table for
> `std.mem` (ADR 0969).

AIL is garbage-collected and memory-safe, by decision. A program never frees
memory itself in safe code, and the collector never runs code of the
program's own (`mem.nospec.3`).

## What a program may rely on

**mem.safe.1** — Safe code has no dangling reference and no use after free. A
value, a `ref` cell or a list element that a program can still name stays valid
for as long as the program can name it. Storage is reclaimed only when no
binding, parameter, field, element, task or global can reach it. *(ADR 0062,
ADR 0063, ADR 0563)*

```ail rules=mem.safe.1,mem.ref.1
struct Counter
    n int

fn Counter.bump(mut self)
    n += 1

fn make() ref Counter
    ref Counter(n: 7)

fn main()
    c := make()
    c.bump()
    print(c.n)
```

**mem.safe.2** — Safe code cannot cause a data race (`conc.race.1`). The memory
model is defined on an abstract, weakly-ordered machine and not on any
processor's ordering (`conc.mm.1`); atomics are sequentially consistent unless
the program asks for less (`conc.atomic.1`, `conc.atomic.2`). *(ADR 0025)*

```ail rules=mem.safe.2
struct Profile
    id int

fn load_profile(id int) Profile
    Profile(id)

fn main() !
    a := spawn load_profile(1)
    b := spawn load_profile(2)
    print(a.wait()?)
    print(b.wait()?)
```

**mem.safe.3** — Safe code has no operation that frees memory, takes an
address, or reinterprets storage. Those are the unsafe operations
(`unsafe.op.1`), and `mem.unsafe.1` says what they change. *(ADR 0106)*

```ail error=unsafe.op.1 rules=mem.safe.3
use std.mem

fn main()
    xs := mem.alloc(4, 0)
    print("ok")
```

**mem.val.1** — Values are independent: binding, assignment, passing without
`mut` and returning each produce a value that no other name can change
(`own.val.1`). Sharing storage under the hood is allowed only when this holds
(`own.val.2`). Shared mutation exists only through `ref` (`own.ref.1`), and a
`ref` cell is confined to one task unless it is wrapped in a synchronised type
(`own.ref.2`). *(ADR 0060, ADR 0063, ADR 0025)*

```ail fragment rules=mem.val.1
struct Point
    x int
    y int

a := Point(x: 1, y: 2)
mut b := a
b.x = 6
assert(a.x == 1)
assert(b.x == 6)
```

**mem.ref.1** — A `ref T` cell is managed by the collector. It lives while any
copy of the `ref` value is reachable, even after the function that made it has
returned, and it is not copied when the `ref` value is (`own.ref.1`,
`own.ref.5`). Two `ref` values are equal exactly when they refer to the same
cell (`own.ref.6`); that identity is the only thing about a cell's place in
memory that a program can observe. *(ADR 0063, ADR 0192)*

```ail fragment rules=mem.ref.1,mem.nospec.2
struct Box
    v int

a := ref Box(v: 1)
b := ref Box(v: 1)
c := a
assert(a == c)
assert(a != b)
```

**mem.place.1** — [definitional] Where a value lives, in a register, on the
stack or on the heap, is not observable. An implementation MAY keep a value off
the heap when it can show that the value does not outlive its use, and MUST
keep every rule of this chapter when it does. `native.stack.2` is one such placement. *(ADR 0012, ADR 0923)*

## The collector

**mem.gc.1** — [definitional] The standard runtime reclaims heap storage by
tracing: storage that no root reaches is reclaimed, and nothing else is. The
roots are the running tasks' stacks and registers and the program's global
slots. There is no reference counting, so a cycle of values does not keep
itself alive. Programs keep memory alive by keeping it reachable, so a program
"leaks" only by holding what it no longer needs; `ail leak` (`san.leak.1`,
`san.leak.2`) reports that. *(ADR 0563, ADR 0969)*

```ail rules=mem.gc.1,mem.nospec.1
fn main()
    mut total := 0
    for i in 0..2000
        xs := [i, i + 1, i + 2]
        total += xs.len()
    print(total)
```

**mem.gc.2** — [definitional] The native collector stops the world, marks from
the roots and sweeps. It does not move objects. It reads the stack and registers
conservatively: a word that points anywhere inside an allocated block keeps that
block alive, interior pointers included, and a pointer one past the end of a
block does not. Objects are scanned precisely by kind (a string or a block of
scalars holds no pointers). With tasks on threads, every other thread parks at
its next allocation, lock wait or task wait before the collector scans its
stack, so a thread that never allocates delays a collection until it does.
*(ADR 0562, ADR 0563)*

**mem.gc.3** — [definitional] A scalar that happens to look like a heap address
can keep a block alive. The cost is memory, never correctness, and a program
MUST NOT rely on a particular object being reclaimed. *(ADR 0563)*

**mem.gc.4** — [definitional] The native heap is a reserved region of address
space (up to 16 GiB, halved until the system accepts it) cut into pages of
16,448 bytes, with a page table. Objects of up to 8,192 bytes, header word
included, come from 36 size classes; larger ones take whole pages. Every object
has a header word before its first byte: the kind in bits 0 to 7, the mark in
bit 8 and the payload size in bits 16 to 63. Memory returned by an allocation
is zeroed. *(ADR 0562)*

**mem.gc.5** — [definitional] A collection runs when the bytes allocated since
the last collection exceed the larger of 16 MiB and twice the live bytes after
it, so the heap peaks near three times the live data. When the process has
worker threads, each thread allocates from a cache of its own and takes the
shared lock only to refill it. When the heap spans 32 MB or more, the threads
parked for the collection help mark. Neither changes what a program observes.
*(ADR 0564, ADR 0943, ADR 0944)*

## What a program may not rely on

**mem.nospec.1** — The time of a collection is unspecified. A program MUST NOT
depend on whether or when memory is reclaimed. The only effects of collection a
program can see are its run time, its memory use (`mem.limit.1`) and what
`ail leak` and `AIL_GC_STATS` report (`mem.tune.1`). *(ADR 0564)*

**mem.nospec.2** — Addresses are unspecified. A program MUST NOT rely on the
address of a value, on the order of addresses, or on an address staying the
same, even though the native collector does not move objects at present
(`mem.gc.2`). Cell identity (`mem.ref.1`) is the only observation of a place
that the language defines. *(ADR 0563, ADR 0063)*

**mem.nospec.3** — The collector runs no code of the program's. There are no
finalisers. A unique value's `drop` runs when its scope releases it
(`own.uniq.3`, `own.uniq.7`), in the order `own.uniq.4` gives, and never at a
collection. Anything a program needs done at a known point, such as closing a
file or releasing a lock, it puts in a unique type or a `defer`
(`own.defer.1`). *(ADR 0066, ADR 0191)*

```ail rules=mem.nospec.3
unique struct Lock
    id int

impl Drop for Lock
    fn drop(mut self)
        print("unlock {id}")

fn main()
    a := Lock(id: 1)
    print("held {a.id}")
```

**mem.nospec.4** — [definitional] The size of the heap and of a process are unspecified. The
figures in the ADRs (`binarytrees 20` in 80 MiB, a 10 GB stream of short-lived
lists in 16 MiB) are measurements of one implementation, not guarantees.
*(ADR 0564)*

## What `unsafe` changes

**mem.unsafe.1** — Inside `unsafe` a program can leave the guarantees of
`mem.safe.1` to `mem.safe.3`. A raw pointer `*T` has no owner and is not
traced (`unsafe.ptr.1`), and the collector does not free what a raw pointer
points at. `std.mem` allocates from C's own allocator, outside the collected
heap, so a block that is not freed leaks, and one the program frees is its own
to free once (`unsafe.mem.1`, `unsafe.mem.2`, `std.mem.1`). A read or write
past the end of a `std.mem` block, or in one already freed, traps; the same
access to memory from anywhere else is not checked (`unsafe.ptr.5`,
`unsafe.ptr.6`). A program's `unsafe` blocks can be listed (`unsafe.block.4`).
*(ADR 0106, ADR 0434, ADR 0435)*

```ail rules=mem.unsafe.1
use std.mem

fn main()
    unsafe
        xs := mem.alloc(4, 0)
        for i in 0..4
            xs.offset(i).write(i * 10)
        print(xs.offset(2).read())
        mem.free(xs)
```

## Limits

**mem.limit.1** — `ail run --max-memory MB` and `ail test --max-memory MB`
stop a run that uses more than `MB` megabytes, with `trap[run.limit.2]`
(`run.limit.2`, `run.limit.4`). A native executable reads the same limit from
`AIL_MAX_MEMORY_MB`, or from the value `ail build --max-memory` baked in, and
prints its trap as `run.limit.5` and `run.limit.6` define (chapter
[21](21-trace.md)). The
native check counts heap bytes in use when the heap takes a new chunk, not
resident memory, so it fires a little later than a count of resident memory would. A program
the system cannot give more memory to also traps `run.limit.2`, so a run that
exhausts the heap ends with a trap and exit status 1, never silently.
*(ADR 0288, ADR 0562, ADR 0726)*

```ail tool=run,--max-memory,16,FILE status=1 expect=run.limit.2 rules=mem.limit.1
fn main()
    mut xs := [0]
    while true
        xs.push(1)
```

## Variables the runtime reads

**mem.tune.1** — [definitional] A native executable reads these environment
variables at start. Except for the two limits, none of them changes what the
program prints or returns.

| Variable | Effect | Rule or ADR |
|---|---|---|
| `AIL_MAX_MEMORY_MB` | Memory limit in megabytes; a positive whole number, else exit 2 | `run.limit.2`, `run.limit.4`, `run.limit.5`, ADR 0726 |
| `AIL_TIMEOUT_MS` | Wall-clock limit in milliseconds | `run.limit.1`, ADR 0726 |
| `AIL_THREADS` | Size of the task pool, 1 to 64; anything else is ignored | `native.task.1`, `conc.par.17`, ADR 0599 |
| `AIL_PARALLEL=0` | Runs `par for` and tensor regions in order | `conc.par.8` |
| `AIL_GC_STATS` | Before each sweep, live bytes and blocks per object kind on standard error | ADR 0564 |
| `AIL_GC_VERIFY` | `1` checks the heap at each collection; `LO,HI,STRIDE` also checks during allocation, and every allocation takes the counted, shared path | ADR 0943, ADR 0944 |
| `AIL_LEAK` | `1`, `json`, `all` or `all,json`: the leak report | `san.leak.3`, ADR 0969 |

The names are read by `compiler/rt/native/`; no other `AIL_` variable tunes
memory. The collector's constants (the 16 MiB minimum, the factor of two, the
32 MB parallel-marking threshold) are not settings. *(ADR 0564, ADR 0726,
ADR 0943, ADR 0944, ADR 0969)*

## ADRs used

0012 (memory tracks), 0025 (memory model and atomics), 0060 (value
semantics), 0062 (second-class borrows), 0063 (use-site `ref`), 0066 and 0191
(unique types and `Drop`), 0106 (unsafe operations), 0192 (`ref` equality),
0288 (run limits), 0434 and 0435 (`std.mem`), 0562 to 0564 (native heap,
collector, trigger), 0599 (native threads), 0726 (native memory cap), 0923
(escape placement), 0943 and 0944 (allocation caches, parallel marking), 0969
(leak report).
