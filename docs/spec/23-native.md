# 23. Native compilation

Chapter key: `native`

> **Status:** Drafted (partial). Specified: what the native subset is, how a
> program outside it is reported, what a native executable must match, and
> the `ail build` command. Implemented for Linux x86-64, Linux ARM64 and
> macOS ARM64.
> The backend's representations and the ABI are non-normative and live in
> [ir-architecture.md](../design/ir-architecture.md) and
> [native-abi.md](../design/native-abi.md).

Native is the only backend (ADR 0904): `ail run` builds the program natively
and runs it, so `ail run` and a built executable behave the same. A program
the checker accepts compiles; a rare construct the lowering cannot build yet
is rejected before any code is produced (native.subset.1). The backend is written in AIL under `compiler/` (ADR 0022)
with its own encoders and object writers (ADR 0010).

## The native subset

**native.subset.2** — `any Trait` is in the native subset for traits whose
implementations are structs: a struct converts to `any Trait` where one is
expected (trait.any.3), and a method call on an `any Trait` value runs the
method of the value's type (trait.any.1). Impls for other types, generic
impls in an `any` are reported under native.subset.1. A struct or enum
with no `to_str`, and an `any Trait` value, print as the interpreter prints
them (trait.auto.2) when their fields are `int`, `float`, `bool` or `str`,
or lists, maps, sets, options or other such values of them, including types
that hold themselves. Generic types have methods (`Stack[int].new()`,
`Pair[A, B].with_first[C]`, `Tree[T].Leaf`), `Self` names the implementing
type in a trait's signatures (trait.res.1), a trait's functions can be called
through the trait name (`Shape.area(x)`), and a user `impl Eq` or `impl Ord`
with its own `eq` or `cmp` is what `==` and `<` use (trait.derive.1, expr.cmp.1).
*(ADRs 0536, 0616)*

**native.subset.3** — A read-only argument or receiver of a struct, list or `any Trait` type that
may overlap a `mut` argument of the same call (the same variable, with fields
that agree and any index counting as equal) is copied before the call, as
own.excl.4 says, so the callee doesn't see its own changes through it.
*(ADR 0119)*

**native.subset.4** — `ref S` of a struct `S` is in the native subset (own.ref.1-6):
`ref e` makes a new object from a copy of `e`, a copy of the reference
shares the object, field access, assignment and method calls apply to it
(own.ref.3-4), `==` compares identity (own.ref.6), and a `ref` can be a
field, a list element or an optional. A `ref` of anything but a struct, the
text of a `ref`, and the trap of own.ref.7 are not in the subset yet.
*(ADRs 0067, 0192)*

**native.subset.1** — The native backend compiles every program `ail check`
accepts. A construct the lowering cannot yet build (a corner of the language,
named in the message) is reported at its position under this rule, all of
them at once with a `help:` line saying how to write it with what is built,
and nothing is built. The report is a gap in the compiler, not a language
rule: the other chapters say what each construct means. The `native_matrix`
gate suite builds every built-in method on every element type `ail check`
accepts, so such a gap is found before a release. *(ADRs 0384, 0904, 1109)*

```ail
fn steps(n int) int
    mut x := n
    mut k := 0
    while x != 1
        x = if x % 2 == 0 then x / 2 else 3 * x + 1
        k += 1
    k

fn main()
    print("27 takes {steps(27)} steps")
```

**native.str.1** — In a native program `str` and `char` behave as
type.str.3-14 and type.char.1-4 say, with the interpreter's results and
traps. The subset has: interpolation as a value, with `{x:.N}`, `{x:x}`,
width, fill and alignment (type.str.11), of a `List[int]` as `[1, 2]`; `==`, `!=`, `<`, `<=`, `>`, `>=` on
`str` and on `char`; on `str`, `len`, `byte_len`, `chars` (a `List[char]`),
`lines`, `trim`, `trim_start`, `trim_end`, `words`, `to_upper`, `to_lower`, `to_str`, `contains`,
`starts_with`, `ends_with` and `split` (each taking a `str` or a `char`),
`slice`, `replace`, `repeat`, `pad_left`, `pad_right`, `center`, `count`; on `char`, `code`,
`to_str`, `is_digit`, `is_alpha`, `is_alnum`, `is_space`, `is_upper`,
`is_lower`, `to_upper`, `to_lower`; `to_str` on `int`, `float` and `bool`;
`print` of a `char`, a `List[char]` and a `List[str]`. The methods with an
optional result are `find`, `rfind`, `char_at`, `strip_prefix`, `strip_suffix`,
`to_digit` and `char.from_code`, used with the optional forms of
native.subset.1.
The character predicates and case mappings, and `to_upper` and `to_lower`
on a `str`, are those of the interpreter on every scalar value, read from
tables generated from it (a capital sigma becomes the final form by the
Unicode rule, with its case-ignorable set approximated by the apostrophes,
dots, accents and combining marks). *(ADRs 0565, 0566, 0567)*

**native.chan.1** — `Channel[str](capacity)` is a bounded channel the runtime
builds (conc.share.3, conc.share.4): `send` waits while it is full, `recv`
returns `str?`, `close` ends it, `for v in ch` receives until it is closed
and empty (conc.share.7), and `task.select([a, b, ...])` returns the lowest
ready index or `none` (conc.select.1); a handle and its copies are the same
channel. A trap (`Channel(0)`, `send` on a closed channel, closing twice)
names the channel `Channel#N` in the order the program made them, as the
interpreter does. Other element types, a `select` over a list that isn't
written out, and cancellation (conc.cancel.2, conc.cancel.4) are not built.
A deadlock (conc.task.7) is found by the main task: when it waits in `send`,
`recv`, `for` or `select`, every unfinished task is waiting too, and nothing
has changed for 100 ms, it traps with the interpreter's message. A
deadlock the main task is not part of, or one reached through `t.wait()`,
hangs as before. *(ADR 0599, ADR 0186)*

**native.fnval.1** — A function value is a pointer to a heap object the
collector scans word by word: word 0 is the address of the code, and the
words after it are the literal's copied locals (type.fn.5), in the order the
compiler gives them. A call through the value loads word 0 and calls it with
the object as a hidden first argument, then the arguments. A declared
function named as a value is a one-word object whose code is a small thunk
that drops the hidden argument; a literal is a function of its own that reads
its copies from the object. A failing function value sets the error slot as a
failing call does. The runtime's own code keeps plain code addresses.
*(ADR 1067)*

**native.task.1** — Native code runs tasks and `par for` iterations in
parallel on a pool of threads, at most 64 and no more than the cores the
process may use, each region using only the workers its work repays
(`conc.par.17`), unless the environment variable `AIL_THREADS` (a whole
number, 1 to 64; anything else is ignored) sets the size (`conc.par.9`,
`conc.par.7`). `spawn f(args)` takes a free
function of the subset's types and copies a list argument (`conc.share.1`);
`t.wait()?` is the task's result, and if the task trapped it ends the
program as an uncaught `Trapped` does: `error: Trapped(rule: "R", message:
"L:C: M")` on standard error, exit status 1. A `par for` gives the
sequential result (`conc.par.1`): a chunk that traps, or a sum that would
leave `int` at any point, makes it run in order (`conc.par.4`,
`conc.par.11`), except in a loop that updates an atomic, which reports the
trap it met (`conc.atomic.6`). Atomic operations are sequentially consistent
whatever `order:` says (`conc.atomic.2`). The program's output is the
interpreter's wherever the interpreter defines it (`conc.task.9`); `print`
holds a lock for one statement, so lines from different tasks don't mix.
On macOS the threads are libSystem's pthreads, bound at load by dyld. *(ADRs
0599–0601)*

**native.task.2** — A pool worker asleep in a blocking network wait
(`accept`, a read, a write or a connect) stops counting as running, as in
`wait` and channel waits, so a task queued meanwhile gets a thread even when
`AIL_THREADS` is 1, and the collector may scan the sleeper. *(ADR 0599)*

**native.build.1** — A native executable's standard output and exit status
equal the interpreter's on the same program and input, and a trap's first
line on standard error is the interpreter's (`PATH:LINE:COL:
trap[RULE]: MESSAGE`, err.trap.*); the interpreter's frame lines after it
may be absent. Recursion is the exception: the interpreter traps after a
fixed number of nested calls, a native executable when its stack is nearly
exhausted. It traps under err.trap.1 at the call, with `call depth exceeded
the native stack in `NAME``, so it may recurse deeper than the interpreter
allows. *(ADRs 0384, 0385, 0535)*
trap[RULE]: MESSAGE`, err.trap.*), then its source excerpt and the stack
without the frames' values (trace.frame.1). *(ADRs 0384, 0385, 0641)*

```ail rules=native.build.1
fn main()
    print(6 * 7)
```

**native.build.2** — An error inside the backend itself (MIR that fails the
verifier, ir-architecture.md §4) is reported under this rule as an internal
error, never turned into code. *(ADR 0384)*

**native.build.3** — `ail build FILE [-o OUT] [--target T]` checks the
program as `ail check` does and stops on its errors, then reports
native.subset.1 problems, then writes an executable to OUT (default: FILE's
name without `.ail`, in the current directory; for a directory `DIR`, the file
`DIR/NAME` where NAME is the directory's name, `DIR/NAME.exe` on Windows). T is `x86_64-linux`,
`aarch64-linux` or `aarch64-macos` (a signed Mach-O); the default is the
host's. A host with no target of its own and no `--target`, or a failure to
write the file, is reported under this rule. A natively built compiler takes the
same command line (`NATIVE build FILE [-o OUT] [--target T] [--unsafe driver] [--timeout MS]
[--max-memory MB]`) and writes OUT itself; `AIL_NATIVE_COMPILER=PATH ail build ...`
checks the program, then runs PATH with the same arguments. In a checkout `./ail` builds the CLI it runs as stage 2 (the seed builds stage 1, stage 1 builds the tree); `AIL_USE_SEED=1` keeps stage 1. *(ADRs 0387, 0388, 0389, 1208)*

```ail rules=native.build.3
fn main()
    print("built")
```

**native.build.4** — A Windows x86-64 executable is a PE32+ image with a fixed base, four sections (`.text`, `.rdata`, `.idata`, `.bss`) and an import directory naming each imported function from its DLL. Calls to an import go through its address-table slot (ADR 0597, ADR 0598). A `libsys.NAME` call passes its arguments in Win64 positions and calls the import through its slot (ADR 0606). The writer is in `compiler/obj/pe.ail`; `ail build --target x86_64-windows` selects it, and the runtime reaches the OS through kernel32 imports (ADR 0608).

**native.build.5** — `ail build` keeps the executables it makes, under a
SHA-256 key of everything that decides their bytes: the path as given, the
target, the `.ail` sources of the backend and of the program's directory tree
(a tree over 4,000 files or 32 MB is not cached), and the build of `ail`
itself. Building again with the same key writes the stored bytes to OUT and
runs nothing else; the check of native.build.3 still runs first. A build
with problems is never stored. `--no-cache` or `AIL_NO_CACHE` reads and
writes nothing. The cache is in `$AIL_CACHE_DIR`, else
`$XDG_CACHE_HOME/ail/build`, else `%LOCALAPPDATA%\ail\build` or `~/.cache/ail/build`, and keeps the 64 most
recently used entries. A cached and an uncached build are the same bytes.
`ail` processes that build the same program at once each build into a file
of their own; the first to store the entry wins and the others use it.
*(ADR 0655)*

```text
$ ail build examples/hello.ail
$ ./hello
Hello, world!
```

## Optimisation

**native.bce.1** — An optimisation that removes a list bounds check
(type.coll.4) from a loop keeps native.build.1: when the lengths the check
would have tested are not known to be enough, the loop runs with the check,
and traps on the same iteration with the same message. A list that a call in
the loop receives, or a `push` in the loop grows, is never assumed to keep
its length. *(ADR 0577)*

```ail rules=native.bce.1
fn total(xs List[int]) int
    mut sum := 0
    for i in 0..xs.len()
        sum += xs[i]
    sum

fn main()
    print(total([1, 2, 3]))
```

**native.bce.2** — An access `xs[i]` has no bounds check when `i` was
already used to index `xs` on every path to it, or is `xs.len()` taken
before a `push` to `xs` that is on every path to it, or is a constant, or a
variable that only ever holds constants, below the number of `push` calls to
`xs` that are on every path to it. This holds while no operation in the
program shortens a list. *(ADR 0578, ADR 0816)*

```ail rules=native.bce.2
fn main()
    xs := [4, 5]
    a := xs[1]
    print(a + xs[1])
```

**native.bce.3** — A list's capacity is not observable: a native executable
may allocate room for a list ahead of the pushes that fill it, at most 256 MiB
of elements (2^25 elements of 8 bytes, 2^28 bytes in all for a narrower list) per list. A
list is allocated with room for exactly the elements asked for (at least 1);
one that fills its first block grows to 4 elements, then to 16, before it
doubles. *(ADR 0579, ADR 0816, ADR 0562, ADR 0907, ADR 0912, ADR 1064)*

```ail rules=native.bce.3
fn main()
    mut xs := [1]
    xs.push(2)
    print(xs.len())
```

**native.bce.4** — An overflow check (expr.arith.1) may be removed from a
loop only where the operands' values, tested before the loop, rule the
overflow out; otherwise the loop with the check runs and traps as the
interpreter does. A check on an operation that repeats an earlier identical
one on every path may be removed. *(ADR 0611)*

```ail rules=native.bce.4
fn main()
    mut sum := 0
    for i in 0..10
        sum += i * 2
    print(sum)
```

**native.bce.5** — The check on `r[j]`, with `r` a row of a list of lists
`b`, may be removed where `j` is at most the length of the shortest row of
`b`, taken before a loop in which the rows of `b` can't be replaced, and no
operation in the program shortens a list. *(ADR 0611)*

```ail rules=native.bce.5
fn main()
    b := [[1, 2], [3, 4]]
    mut total := 0
    for j in 0..b[0].len()
        total += b[0][j]
    print(total)
```

**native.bce.6** — A `push` in a counted loop may skip the test that the
list is full where, before the loop, the list's spare room is at least the
number of pushes the loop can make; otherwise the loop with the test runs.
*(ADR 0611)*

```ail rules=native.bce.6
fn main()
    mut xs := [0]
    for i in 0..3
        xs.push(i)
    print(xs.len())
```
**native.bce.7** — A counted loop whose start is computed before it is
treated as a counted loop where that start is tested, before the loop, to be
at least 0 and at most 2^30; otherwise the loop with its checks runs.
*(ADR 0622)*

```ail rules=native.bce.7
fn main()
    xs := [1, 2, 3, 4]
    mut total := 0
    mut i := 1
    while i < 3
        total += xs[i]
        i += 1
    print(total)
```

**native.bce.8** — The check on `xs[e]` may be removed where the arithmetic of
the loop bounds `e` between 0 and a constant `hi`, from a constant loop bound
and operands tested before the loop, and `xs` is longer than `hi`. *(ADR 0622)*

```ail rules=native.bce.8
fn main()
    xs := [1, 2, 3]
    mut total := 0
    for i in 0..xs.len()
        total += xs[i] + xs[i]
    print(total)
```

**native.bce.9** — A loop that adds an unchanging `s` to its induction value
is treated as a counted loop where `s` is tested, before the loop, to be
between 1 and 2^30. *(ADR 0622)*

```ail rules=native.bce.9
fn main()
    mut total := 0
    mut s := 2
    for i in 0..4
        total += i + s
    print(total)
```

**native.bce.10** — The check on `xs[y + k]` may be removed where `y` does not
change in the loop, `k` is a constant from 0 to 64, and `xs` is a list that
the loop does not resize or whose elements it does not replace through
another name: the loop is versioned on `0 <= y` and `y < len - k`, and the
original loop, which traps at the same access with the same message, runs
when they do not hold. When the counted loops around the loop keep `y` within
`lo..=hi` with `lo >= 0` (their counters run between constants, and a counter
is below its bound everywhere in its loop but the header), the test may
instead be `hi + k < len`, one test per list with the largest such index.
*(ADR 0816, ADR 1080)*

```ail rules=native.bce.10
fn main()
    xs := [1, 2, 3, 4, 5, 6]
    mut total := 0
    y := 1
    for j in 0..3
        total += xs[y + 2] * j + xs[y + 4]
    print(total)
```

**native.bce.11** — A loop that only reads and writes the elements of a
list it did not make (`xs[i]`, `xs[i] = v`, `xs.len()`) may read the list's
length and element pointer once, before the loop, and use them for every
access: the lists are a parameter of the function or the result of a call,
defined outside the loop, and the loop neither passes one to a call, pushes
to it, nor uses it in any other way. Each check keeps its place, its index,
its site and the length it reports, so a trap is the same. *(ADR 0890)*

```ail rules=native.bce.11
fn dot(a List[int], b List[int], n int) int
    mut s := 0
    for i in 0..n
        s += a[i] * b[i]
    s

fn main()
    print(dot([1, 2, 3], [4, 5, 6], 3))
```

**native.bce.12** — In a counted loop at least two loops deep, an `int` read
from a list, or from a row of a list of lists, that no call, `push` or
assignment can change in an enclosing loop may be taken to lie within
+-2^20 where a scan of the list before that enclosing loop says so; an
accumulator that every pass of the loop changes only by `+=` or `-=` of such
a value may be taken to lie within its start plus the loop's trip count times
that value's range, where the guard tests the trip count. The overflow checks
this proves can't fail may be removed (expr.arith.4); otherwise the loop with
its checks runs, and traps where it did. *(ADR 0941)*

```ail rules=native.bce.12
fn main()
    a := [[1, 2], [3, 4]]
    mut t := 0
    for i in 0..2
        for j in 0..2
            mut s := 0
            for k in 0..2
                s += a[i][k] * a[k][j]
            t += s
    print(t)
```

**native.bce.13** — A counted loop over `j` whose body sets an accumulator
`s` to a value from before the loop, runs a counted loop over `k` that only
adds a term to `s` (`s += t`), and then pushes `s` to a list made empty just
before the `j` loop and used nowhere else, may run with the `k` loop outside
and the `j` loop inside, adding each term to the list's element `j`, where a
guard before the loops proves that no index in the term is out of bounds and,
for `int`, that no operation overflows (native.bce.12). Each element adds the
same terms in the same order, so the list is the same. Otherwise the loops
run as written. *(ADR 0941)*

```ail rules=native.bce.13
fn main()
    a := [[1.0, 2.0], [3.0, 4.0]]
    mut c := List[List[float]]()
    for i in 0..2
        mut row := List[float]()
        for j in 0..2
            mut s := 0.0
            for k in 0..2
                s += a[i][k] * a[k][j]
            row.push(s)
        c.push(row)
    print(c[1][1])
```

**native.bce.14** — The overflow check of an `int` `+` or `-` may be removed
where a fact on every path to it bounds an operand: on the edge where `x < y`
holds, `x + 1` can't overflow; after `x` (or `x * c` for a constant `c >= 1`) is a
checked list index, `0 <= x` and
`x` is below any list's length (under 2^62), so `x + c` and `x - c` for a
constant `c` from 0 to 2^62, and the sum of two such indexes, can't overflow.
In a loop where `l` starts at 0 and every path back adds 1 to it without
wrapping, the check of `a + l`, for an `a` defined before the loop, may be
removed where `a + l < y` or an index use of `a + l` holds at the end of every
path back: each iteration's sum is the last one's plus 1. This holds for
loops that leave in more than one place, which native.bce.7-9 do not cover.
Checks with no such fact stay and trap where they did (expr.arith.4).
*(ADR 0994)*

```ail rules=native.bce.14
fn match_len(data List[int], a int, b int, n int) int
    mut l := 0
    while l < 66 and a + l < n and data[b + l] == data[a + l]
        l += 1
    l

fn main()
    print(match_len([1, 2, 3, 1, 2, 3, 9], 3, 0, 7))
```

**native.bce.15** — A list made empty and pushed to a known number of times,
either all in the block that makes it or all in one counted loop with constant
bounds that leaves only from its header (each push in a block every pass goes
through), has that length once the pushes are done, if every other use of it
is `xs[i]`, `xs[i] = v` or `xs.len()` after them. `xs.len()` is then that
constant, and an access outside a counted loop may compare its index to the
constant instead of loading the length. Any other use (a call, a store, a
return, a later push) rules the list out. A trap names the same site, index
and length. *(ADR 1061)*

```ail rules=native.bce.15
fn main()
    mut st := [0 for _ in 0..8]
    mut sp := 0
    while sp < 5
        st[sp] = sp * 2
        sp += 1
    print(st[4] + st.len())
```

**native.bce.16** — A counted loop (native.bce.1) that also leaves from blocks
other than its header (a failing `and` term, `break`, `return`) may be
versioned as one that leaves only from its header, where every block it leaves
to is entered only from the loop. The check on `xs[a + i]`, with `a` fixed in
the loop and `i` its counter, and the overflow check of `a + i`, may be removed
where the guard tests `0 <= a` and that the loop's bound is at most
`xs.len() - a` (one less for the falling `j` of a two-pointer loop). The guard
covers every pass the bound allows, so a loop that would leave early runs with
its checks when the list is shorter than that, and traps where it did.
*(ADR 1062)*

```ail rules=native.bce.16
fn count_after(xs List[int], k int) int
    mut total := 0
    mut l := 0
    while l < 4
        if k + l >= xs.len()
            break
        total += xs[k + l]
        l += 1
    total

fn main()
    print(count_after([1, 2, 3, 4, 5], 1))
```

**native.bce.18** — The check of an element kept in place (native.flat.1),
as in `ps[i].x`, is removed from a counted loop under the conditions of
native.bce.1 and with the same guard as the check of `xs[i]`: the guard
compares the bound with `ps.len()` once, the loop that runs under it has no
check, and the loop with its checks runs otherwise, so a trap is the one the
checked loop gives. `AIL_OPT_OFF=bceflat` keeps the checks. *(ADR 1160)*

```ail rules=native.bce.18
struct P
    a int
    b int

fn bump(mut ps List[P], n int)
    for i in 0..n
        ps[i].a = ps[i].a + ps[i].b

fn main()
    mut ps := List[P]()
    for i in 0..4
        ps.push(P(a: i, b: 10))
    bump(mut ps, 4)
    print(ps[3].a)
```

**native.bce.19** — In a counted loop (native.bce.1) that steps its counter by
1, a second loop-carried `int` `m` that starts at a value `m0` fixed before the
loop and grows by the same constant `S` (1 to 64) on every pass is `m0 + S * j`
on pass `j`. The check on `xs[m + k]` (`k` a constant from 0 to 64, or `xs[m]`),
and the overflow check of `m + k`, may be removed where the guard tests `0 <=
m0`, that the loop's bound and start are within 2^30 of 0, and that
`S * (hi - lo - 1) + m0 + k < xs.len()`. The guard covers every pass the bound
allows, so a loop that would run past the list's end, or start below 0, runs
with its checks and traps where it did, after the same effects.
`AIL_OPT_OFF=ivstep` keeps the checks. *(ADR pending: integrator allocates)*

```ail rules=native.bce.19
fn row(pk List[int], words int) int
    mut c := 0
    mut s := 0
    for _w in 0..words
        s += pk[c] + pk[c + 1] * 2 + pk[c + 2] * 3 + pk[c + 3] * 4
        c += 4
    s

fn main()
    print(row([i for i in 0..12], 3))
```

**native.promote.1** — In a counted loop (native.bce.1) that has no loop inside
it, whose every operation is unchecked (no trap can happen in it) and calls
nothing, the elements `xs[x + k]` it assigns, with `xs` and `x` unchanged by
the loop, may be held in registers: read once before the loop and written
once when it ends, with each element's last assigned value. Every other
element the loop reads or assigns must be `xs[j * c + k']` for the loop's
counter `j`, which starts at `x0 + d` and steps by 1, where `x = x0 * c` and
`c * d > k - k'`, so it is never one of them. A loop that runs no iteration
reads and writes none of them. `AIL_OPT_OFF=promote` turns it off.
*(ADR 1079)*

```ail rules=native.promote.1
fn kick(mut s List[float], n int)
    for i in 0..n
        bi := i * 3
        for j in i + 1..n
            bj := j * 3
            d := s[bi] - s[bj]
            s[bi + 1] -= d * 0.5
            s[bj + 1] += d * 0.5

fn main()
    mut s := [1.0, 0.0, 0.0, 4.0, 0.0, 0.0, 9.0, 0.0, 0.0]
    kick(mut s, 3)
    print(s)
```

**native.fwd.1** — An access `xs[i]` or `xs[i] = v` may run without its bounds
check when an access to the same list with the same index value is on every
path to it, and nothing on those paths can shorten a list: only element
writes, field writes, arithmetic, reads, `push` and allocation come between.
A call, a write to the list itself or a raw store ends this. Where an access
is not covered so, its check stays, and traps with the same message
(native.build.1). Unlike native.bce.2, this holds in a program that shortens
lists. *(ADR 1050)*

```ail rules=native.fwd.1
struct Coder
    rng int
    probs List[int]

fn Coder.bit(mut self, idx int) int
    p := self.probs[idx]
    self.rng = self.rng / 2 + p
    self.probs[idx] = p + 1
    self.rng % 2

fn main()
    mut c := Coder(rng: 100, probs: [5, 6, 7])
    print(c.bit(1) + c.bit(1))
    print(c.probs)
```

**native.fwd.2** — A write to a field of a struct, tuple or enum value
changes only that field's word, in whichever value it reaches: another field,
a list's length or elements, read before it, may be reused after it. A write
through anything else, and any call, may change any memory. *(ADR 1050)*

```ail rules=native.fwd.2
struct Bag
    items List[int]
    n int

fn fill(mut b Bag) int
    a := b.items.len()
    b.n = a
    a + b.items.len()

fn main()
    mut b := Bag(items: [1, 2], n: 0)
    print(fill(mut b))
    print(b.n)
```

**native.fwd.3** — A value read from memory, or a check passed, on every
path into a block whose paths join (after an `if`) counts as read or passed
in the block when all paths agree on it (native.fwd.1-2). Nothing read inside
a loop counts on its next iteration. *(ADR 1050)*

```ail rules=native.fwd.3
fn pick(xs List[int], i int, up bool) int
    mut t := 0
    if up
        t = xs[i] + 1
    else
        t = xs[i] - 1
    t + xs[i]

fn main()
    print(pick([4, 5, 6], 2, true))
    print(pick([4, 5, 6], 0, false))
```

**native.fwd.4** — Two lists never share part of their elements. Two list
values are known to be different lists when each is a parameter or a list the
function made, and they are two different values of which at least one was
made in the function, or both are parameters (own.excl.1, own.excl.4). A
write to an element of one list then leaves what is known about the other
list's elements (native.fwd.3). A loop's first block starts with what was
known before the loop, less what any part of the loop may write, so a value
read before a loop serves the loop's reads of the same memory. A read of a
struct or tuple field, or of a list's length or elements address, whose value
is an object on every path (a parameter, a call's result, a field or element)
and whose memory nothing in the loop may write (no call, no raw store, no
write to that field) may run once before the loop. Such a read can't trap.
`AIL_OPT_OFF=fwdalias`, `fwdloop` and `fwdhoist` turn the three parts off.
*(ADR 1142)*

```ail rules=native.fwd.4
struct Grid
    w int
    cells List[float]

fn bump(mut out List[float], src List[float], k int) float
    x := src[k]
    out[k] = x + 1.0
    src[k] + out[k]

fn walk(g Grid) float
    mut s := 0.0
    mut i := 0
    while i < g.cells.len()
        s += g.cells[i] * g.w.to_float()
        i += 1
    s

fn main()
    mut a := [1.0, 2.0, 3.0]
    print(bump(mut a, [4.0, 5.0, 6.0], 1))
    print(bump(mut a, a, 2))
    print(walk(Grid(w: 2, cells: a)))
```

**native.range.1** — An overflow, division or shift check may be omitted
where intervals of its operands, computed over the whole function, rule the
trap out. An interval comes from constants, `len()` (below 2^48), masks
(`bit_and` with a value never negative), shifts, `%`, bytes, and the
comparisons of the branches that lead to the operation; a loop's value joins
what every path brings into the loop. A checked operation that can overflow
keeps its check, and its result counts as within `int` after it. Where the
intervals don't rule the trap out, the check stays and traps with the same
message (native.build.1). *(ADR 1063)*

```ail rules=native.range.1
fn hash3(a int, b int, c int) int
    x := a.bit_and(255)
    y := b.bit_and(255)
    z := c.bit_and(255)
    (x * 65536 + y * 256 + z) * 2654435761

fn main()
    print(hash3(300, 255, 511))
```

**native.range.2** — A comparison of two `int`s whose intervals
(native.range.1) give one answer may be replaced by that answer, and the
branch it decides by a jump. *(ADR 1063)*

```ail rules=native.range.2
fn low(n int) int
    m := n.bit_and(15)
    if m < 16
        return m
    -1

fn main()
    print(low(47))
```

**native.range.3** — `x / 2^k` and `x % 2^k` (k >= 1) of an `x` whose
interval (native.range.1) has no negative value may compile as a shift and a
mask, as for a loop counter. `x / c` and `x % c` of such an `x` by any other
constant `c >= 3` may compile as one unsigned high multiply and a shift,
with no correction for a negative dividend. *(ADR 1063, ADR 1150)*

```ail rules=native.range.3
fn split(n int) int
    m := n.bit_and(1023)
    m / 4 + m % 8

fn main()
    print(split(1037))
```

**native.select.1** — A two-way branch whose arms are at most three
integer instructions each (five in all) with no call, load, store or trap
other than an overflow check, and that rejoin one block with `int`, `bool`
or `char` values, may compile as both arms and one branch-free select per
value (`csel`/`csinc` on arm64, `cmov` on x86-64). Each checked `+`, `-`,
`*` or negation in an arm that is an affine form `c * x + d` of one input
gives a range of `x` in which it can't overflow; the arms run unchecked only
when a guard finds every such input inside its range, and otherwise the
original branch and its checked arms run, so an overflow traps with the same
message (expr.arith.1, native.build.1). An arm with another checked
operation, or more than two guarded inputs, keeps its branch. So does a
branch that picks a value a list index is computed from when its condition
or one of its values is computed from that pick in turn, round a loop; an
index that only reaches its element is picked freely. An `if`/`elif` chain
converts from the innermost level out, up to four levels, an arm of selects
counting by its comparisons; only an innermost level may need a guard.
`AIL_OPT_OFF=select` turns it off. *(ADR 1141, ADR 1150)*

```ail rules=native.select.1
fn next(n int) int
    mut x := n
    if x % 2 == 0
        x = x / 2
    else
        x = 3 * x + 1
    x

fn main()
    print(next(7))
    print(next(3074457345618258601))
```

**native.select.2** — On the arm of a branch on `x % 2^j == 0` (or `!= 0`)
where the low `j` bits of `x` are zero, `x / 2^k` with `k <= j` is exact and
may compile as one arithmetic shift, for any sign of `x`. *(ADR 1141)*

```ail rules=native.select.2
fn half_if_even(n int) int
    mut x := n
    if n % 2 == 0
        x = n / 2
    x

fn main()
    print(half_if_even(-6))
    print(half_if_even(7))
```

**native.select.3** — A two-way branch whose arms (four instructions each
at most, six in all) only compute integers, read slots of objects the
branch's block itself reads or writes, and store integers to at most two
slots of objects that block itself stores to, may compile as both arms, a
select per stored slot between an arm's value and the slot's old one, and
stores that always run. A checked `+` or `-` in an arm then runs unchecked,
and when the sign bits show that one overflowed, the original branch and its
checked arms run instead, so an overflow traps with the same message
(expr.arith.1); an arm with any other checked operation keeps its branch.
`AIL_OPT_OFF=selstore` turns it off. *(ADR 1151)*

```ail rules=native.select.3
struct Coder
    code int
    rng int

fn Coder.bit(mut self) int
    self.rng = self.rng.shr(1)
    mut b := 0
    if self.code >= self.rng
        self.code -= self.rng
        b = 1
    b

fn main()
    mut c := Coder(code: 5, rng: 12)
    print(c.bit())
    print(c.bit())
    print(c.code)
```

**native.select.4** — A two-way branch with one arm of up to three checked
`+` or `-` on `int`s the other arm passes on unchanged (`if c: n += w`),
`w` computed before the branch, and plain assignments of values computed
before the branch (`last = k`), may compile as `n + select(c, w, 0)`
(`n - select(c, w, 0)`) for each sum, `select(c, k, last)` for each
assignment, and for `w` of 1 as `n` plus the condition as 0 or 1. Each sum
is still the checked operation: it traps when the condition holds and the
sum overflows, with the same message, and never otherwise (expr.arith.1),
so it needs no guard. A sum whose result a
list index is computed from keeps the rule of native.select.1.
`AIL_OPT_OFF=selsum` turns it off. *(ADR 1162)*

```ail rules=native.select.4
fn count(xs List[int], k int) int
    mut n := 0
    for x in xs
        if x == k
            n += 1
    n

fn main()
    print(count([3, 1, 3, 2], 3))
    print(count([3, 1, 3, 2], 5) + 9223372036854775807)
```

**native.vec.1** — In a nest swapped under native.bce.13 whose term is
`x * y`, with `y` the element `j` of a list and nothing else depending on
`j`, the inner loop may be one call of a vector kernel per `k` that computes
`row[base + j] + x * y` for every `j` with the same rounding, operand order
and result as the loop (for `int`, where the guard keeps `x` within +-2^31
and `y` within +-2^20). *(ADR 0942)*

```ail rules=native.vec.1
fn main()
    a := [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
    mut c := List[List[int]]()
    for i in 0..3
        mut row := []
        for j in 0..3
            mut s := 0
            for k in 0..3
                s += a[i][k] * a[k][j]
            row.push(s)
        c.push(row)
    print(c[2][2])
```

**native.vec.2** — A counted `for` over `x` whose body makes `cr` by float
arithmetic of `x` and values from before the loop, then runs the escape loop
`while k < MAX` (`MAX` a constant from 1 to 2^20) with the exit test
`zr * zr + zi * zi > T`, the step `t = zr * zr - zi * zi + cr`,
`zi = 2.0 * zr * zi + ci` (`ci` from before the `x` loop), `zr = t`, `k += 1`
from `zr = zi = 0.0`, may take each `k` from one vector kernel call per
execution of the `x` loop that counts every point of the row at once, when
`n <= 16777216`. Each lane performs the scalar loop's operations in the scalar
order (no fused multiply-add, no reassociation) and a lane that has left keeps
its state, so every `k` is the scalar `k`; a NaN never leaves. Otherwise the
loop runs as written. *(ADR 0945)*

```ail rules=native.vec.2
fn escape(cr float, ci float) int
    mut zr := 0.0
    mut zi := 0.0
    mut k := 0
    while k < 50
        if zr * zr + zi * zi > 4.0
            break
        t := zr * zr - zi * zi + cr
        zi = 2.0 * zr * zi + ci
        zr = t
        k += 1
    k

fn main()
    mut line := []
    for x in 0..7
        line.push(escape(-2.0 + 3.0 * x.to_float() / 7.0, 0.3))
    print(line)
```

**native.vec.3** — Inside one basic block, the stores of two `float`s to
elements `k` and `k + 1` of the same list at the same index value may become
one two-lane vector store, and the values stored one two-lane computation,
when both are the same operation (`+`, `-`, `*`, `/`) on operands that pair
the same way, down to loads of two adjacent elements; a value both lanes use
is copied into both, and `x * x + y * y` over the two lanes of a pair may be
one vector multiply and one addition of the lanes. Each lane performs its
scalar operation with the operands in the written order (no fused
multiply-add, no reassociation), so every result has the bits of the scalar
code. A paired load is done at the earlier of its two loads and a paired
store at the later of its two stores, only when every two accesses that may
touch the same element, one of them a store, keep their order, and no access
moves across an operation that can trap or call. Checked accesses (`get`,
`set` with a bounds check) are never paired. `AIL_OPT_OFF=slp` turns the
pairing off. *(ADR 1066)*

```ail rules=native.vec.3
fn main()
    mut s := [0.0 for _ in 0..12]
    for k in 0..12
        s[k] = k.to_float() * 0.5
    for i in 0..3
        b := i * 4
        for j in i + 1..3
            c := j * 4
            dx := s[b] - s[c]
            dy := s[b + 1] - s[c + 1]
            w := 1.0 / (dx * dx + dy * dy)
            s[b + 2] -= dx * w
            s[b + 3] -= dy * w
            s[c + 2] += dx * w
            s[c + 3] += dy * w
    print(s)
```

**native.vec.4** — A counted loop over `i` whose body only loads `float`
elements, computes with `+`, `-`, `*` and `/` on `float`s, and stores `float`
elements, every access at a counter that steps by 1 (the loop's `i`, or the
length a comprehension's `push` writes at) with no offset, and with no bounds
check left (native.bce.1), may run two iterations at a time with two-lane
vector operations (native.vec.3) while two remain, the rest one at a time. No
run-time test for overlapping lists is made: two lists never share part of
their elements, so the two lanes never touch the same element unless the
scalar loop's one iteration did. Accesses at two different counters need the
two lists known to be different lists (native.fwd.4). Each lane performs its
scalar operation with the operands in the written order (no fused
multiply-add), and a loop that carries a `float` from one iteration to the
next (a sum) stays scalar, so every result has the bits of the scalar loop.
`AIL_OPT_OFF=vmap` turns the pass off. *(ADR 1142)*

```ail rules=native.vec.4
fn axpy(mut c List[float], a List[float], b List[float], k float)
    for i in 0..a.len()
        c[i] = a[i] * k + b[i]

fn main()
    a := [0.5, 1.5, 2.5, 3.5, 4.5]
    b := [1.0, 2.0, 3.0, 4.0, 5.0]
    mut c := [0.0 for _ in 0..5]
    axpy(mut c, a, b, 2.0)
    print(c)
    print([a[i] * 0.5 - b[i] for i in 0..a.len()])
```

**native.vec.5** — A loop that native.vec.4 takes may run four iterations at
a time while four remain, then two at a time, then one: on x86-64 with one
256-bit register a value (AVX, in the baseline, plat.isa.1), on ARM64 with two
128-bit registers. Before the first vector pass, single iterations run until
the first store's element is on a 32-byte boundary (at most three). Each lane
still performs its scalar operation with the operands in the written order,
so every result has the bits of the scalar loop, and the conditions of
native.vec.4 are unchanged. *(ADR 1144)*

```ail rules=native.vec.5
fn blend(mut c List[float], a List[float], b List[float], k float)
    for i in 0..a.len()
        c[i] = (a[i] - b[i]) * k + b[i]

fn main()
    a := [x.to_float() * 0.5 for x in 0..11]
    b := [10.0 - x.to_float() for x in 0..11]
    mut c := [0.0 for _ in 0..11]
    blend(mut c, a, b, 0.25)
    print(c)
```

**native.vec.6** — The loops of native.vec.4 and native.vec.5 may also be
loops over `int` elements whose body computes with `+`, `-`, `&`, `|` and `^`
on `int`s; the other operand of an operation may be a constant or a value from
before the loop. A body has one kind of element. An overflow still traps as
expr.arith.1 says, at the same iteration and with the same message: each vector
pass tests whether any of its lanes overflowed before it stores anything, and
if one did the pass writes nothing and the remaining iterations run one at a
time. A checked `+` or `-` that follows a store in the body keeps the loop
scalar. Values that do not change in the loop (constants, a value from before
the loop in every lane) are made once before the vector passes. *(ADR 1145)*

```ail rules=native.vec.6
fn total(mut c List[int], a List[int], b List[int], k int)
    for i in 0..a.len()
        c[i] = a[i] + b[i] - k

fn main()
    a := [x * 3 for x in 0..11]
    b := [100 - x for x in 0..11]
    mut c := [0 for _ in 0..11]
    total(mut c, a, b, 7)
    print(c)
```

**native.match.1** — A `match` over an `int` or an enum whose arms each test
int literals (alone or with `|`) or variants, with no guard and no range, and
at least five distinct values, may pick its arm by comparing against the
sorted values instead of testing the arms in order. The arm that runs, its
bindings and the order of effects are those of expr.match.1-12: the first arm
that tests a value takes it. *(ADR 0890)*

```ail rules=native.match.1
fn name(x int) str
    match x
        0 => "zero"
        1 | 2 => "small"
        3 => "three"
        7 => "seven"
        _ => "other"

fn main()
    print(name(2))
```

**native.match.2** — A `match` that native.match.1 takes, whose distinct values
span at most 1024 and fill at least a third of their span, may jump through
a table indexed by the value minus the smallest: a value outside the span or
in a gap goes where the in-order test sends it (the `_` arm, or nowhere for an
exhaustive `match`). Code that jumps back to such a `match` (the next
iteration of a loop around it) may repeat the selection there. The arm that
runs, its bindings and the order of effects are those of native.match.1.
`AIL_OPT_OFF=switch` turns the table off and `AIL_OPT_OFF=taildup` the
repeat. *(ADR 0939)*

```ail rules=native.match.2
fn step(op int, acc int) int
    match op
        0 => acc + 1
        1 => acc * 2
        2 => acc - 3
        3 => 0
        4 => acc
        _ => -1

fn main()
    mut acc := 0
    for op in [0, 0, 1, 2, 9]
        acc = step(op, acc)
    print(acc)
```

**native.inline.1** — A call may be replaced by a copy of the callee's body
when the callee is not on a cycle of the call graph, is not `main` or part of
the runtime, has no loop, and makes no indirect call, and the call is inside
a loop of the caller: the one call of a function called from one place, up
to 200 instructions, and any other call up to 48, with each caller grown by
at most 400 instructions this way;
besides the leaf functions of ADR 0391. A function that nothing calls or
names after this is left out of the executable. A trap in, or below, a copied
body reports the callee's frame and every frame it was copied into
(trace.frame.2). `AIL_OPT_OFF=wide` turns it off. *(ADR 0940)*

```ail rules=native.inline.1
fn area(w int, h int) int
    print("area of {w} by {h}")
    w * h

fn main()
    for k in 1..3
        print(area(k, 4))
```

**native.map.1** — A lookup in a `Map` or `Set` (`m.get(k)`, `m[k]`,
`m.get_or(k, d)`, `m.contains(k)`, `m[k] op= v`, `m.push(k, x)`) makes no
call when it hits at once. For a key stored as a word (`int`, `bool`, `char`,
a sized integer) the hash and the first two index slots are compiled into the
caller; a longer run of slots, or a missing key, goes to the runtime. For a
`str` or `bytes` key the table's key cache is read in the caller (the entry
this key object was last found at), and the runtime hashes the bytes
otherwise. `m[k] op= v` stores through the entry its lookup found. `m[k] = v`
stores through the entry `m.get(k) else d` found when the table and the key
are variables, neither is assigned in between (`d` is a literal or leaves),
and no call lies on any way from the lookup to the store: only a call can
add to, remove from or rebuild a table, so the entry is still the key's. A missing key is added by the runtime in every case, last in
iteration order (type.map.1). Results, order and traps are those of the
runtime calls. `AIL_OPT_OFF=mapfast` keeps the calls. A `str` key that
`xs.join("")` of a `List[char]` makes, in the key or bound by the statement
before, is looked up from the characters, and the string is made only when
the map has no equal key or a character is past ASCII
(`AIL_OPT_OFF=joinkey` joins first). *(ADR 1148, ADR 1197)*

```ail rules=native.map.1
fn main()
    mut counts := Map[int, int]()
    mut words := Map[str, int]()
    names := ["ash", "oak", "ash", "elm", "ash"]
    for i in 0..20
        k := i * i % 7
        c := counts.get(k) else 0
        counts[k] = c + 1
        words[names[i % 5]] += 1
    cur := ['e', 'l', 'm']
    words[cur.join("")] += 1
    print(counts)
    print(words)
    print(counts.contains(3))
```

**native.flat.1** — A `List[S]` keeps its elements in place, not as one
object each, when `S` is a struct of one to 16 fields that are each an
`int`, `float`, `bool` or `char`, and every use of a `List[S]` in the program
is one the in-place layout serves: `xs[i]`, `xs[i] = s`, `xs.push(s)`,
`xs.len()`, `for` and `for mut`, a copy of the list, and passing, returning
or storing the list itself. Element `i` is then the `8 * fields` bytes at
`data + i * 8 * fields` of the list's block, and `xs[i]` is that address
after the bounds check (type.coll.4); a field of it is read and written as a
field of any struct. Any other use of a `List[S]` anywhere in the program (a
list method not named here, a list literal, a comparison, text) keeps every
`List[S]` as a list of objects. Values, copies (own.val.1), order and traps
are the same either way. `AIL_OPT_OFF=flat` keeps every list a list of
objects. *(ADR 1149; 16 fields by ADR 1161)*

```ail rules=native.flat.1
struct Body
    x float
    v float
    hits int

fn main()
    mut bodies := List[Body]()
    for i in 0..5
        bodies.push(Body(x: i.to_float(), v: 0.5, hits: 0))
    for _ in 0..3
        for i in 0..bodies.len()
            bodies[i].x += bodies[i].v
            bodies[i].hits += 1
    kept := bodies[4]
    bodies[4].x = 0.0
    mut far := 0.0
    for b in bodies
        far += b.x
    print(far)
    print(kept.x)
    print(bodies[0].hits)
```

**native.flat.2** — The fields of a struct kept in place (native.flat.1) may
also be a `str`, `bytes`, a list, a map, a set, an optional, a tuple, an enum
or another struct, when none is a unique type: such a field is the 8-byte
pointer a struct object would hold in that slot. The list's block then has a
kind that tells the collector the size of an element, and the collector reads
the first `len` elements of it for pointers. A copy of an element or of the
list copies what a copy of the struct copies: a field that is a list, a map,
a set, a tuple or a struct is copied, and a text is shared (own.val.1), so
`xs.push(s)` and `xs[i] = s` copy such fields when `s` is read from a place.
`AIL_OPT_OFF=flatref` keeps in place only the structs of native.flat.1.
*(ADR 1156)*

```ail rules=native.flat.2
struct Entry
    name str
    marks List[int]
    total int

fn main()
    mut entries := List[Entry]()
    for i in 0..4
        entries.push(Entry(name: "entry {i}", marks: [i, i + 1], total: 0))
    for i in 0..entries.len()
        entries[i].marks.push(i * 10)
        for m in entries[i].marks
            entries[i].total += m
    mut kept := entries[3]
    kept.marks.push(100)
    kept.name = "kept"
    print(entries[3].name)
    print(entries[3].marks.len())
    print(kept.marks.len())
    print(entries[2].total)
```

**native.jump.1** — In x86-64 code no jump, call or return crosses a 32-byte
line of the code or ends on one, nor does a comparison together with the
conditional jump right after it: such an instruction starts on the next
line, after instructions that do nothing. Lines are counted from the first
byte of the code, which is loaded at a multiple of 64, and each function
starts on a line. The stubs that report a trap are left as they fall. No
result changes: processors that decode a jump on a line's edge slowly (Intel,
2015 to 2019) no longer run a loop slower or faster by where an edit
elsewhere happened to put it. ARM64 code is not changed.
`AIL_OPT_OFF=jumpline` turns it off. *(ADR 1096)*

```ail rules=native.jump.1
fn count(xs List[int], want int) int
    mut n := 0
    for x in xs
        if x == want
            n += 1
    n

fn main()
    xs := [k % 7 for k in 0..1000]
    print(count(xs, 3))
```

**native.reuse.1** — A list made empty inside a loop may be one object that
each iteration empties (length 0, capacity kept) instead of a new object per
iteration, where no reference to the list can outlive the iteration: it is
not stored, returned, passed to a function value, or passed to a function
whose parameter can keep it, and it isn't carried to the next iteration. A
list bound, assigned or stored from a variable is copied (own.val.1), so the
copy, not the reused list, is what's kept. *(ADR 0824)*

```ail rules=native.reuse.1
fn main()
    mut rows := List[List[int]]()
    for i in 0..3
        mut row := []
        row.push(i)
        rows.push(row)
    print(rows)
```
**native.sroa.1** — A struct built in a function and only read back there
(its fields read by `.f`, never returned, stored, passed to a call, joined at a
block or changed after it is built) may have no object: the field reads use
the values it was built from. Constructors that only build their result
(`Vec.add`) are inlined into their callers first, so that chains such as
`a.sub(b).unit()` qualify. Nothing a program can observe changes, only the
allocation. `AIL_OPT_OFF=sroa` turns it off. *(ADR 0923)*

```ail rules=native.sroa.1
struct P
    x float
    y float

fn P.add(self, o P) P
    P(x: x + o.x, y: y + o.y)

fn main()
    a := P(x: 1.0, y: 2.0)
    print(a.add(a).add(a).y)
```

**native.sroa.2** — Struct objects that meet at a block parameter (a `mut`
variable assigned in a branch or carried round a loop) may also have no
object, when each of them qualifies under native.sroa.1 and the parameter is
only read by field access: the parameter becomes one parameter per field. *(ADR 0923)*

```ail rules=native.sroa.2
struct P
    x float
    y float

fn main()
    mut p := P(x: 1.0, y: 2.0)
    for i in 0..3
        p = P(x: p.x + 1.0, y: p.y * 2.0)
    print(p.y)
```

**native.sroa.3** — After native.sroa.1 and 2, a call that is given a struct
the caller has just built may be inlined when the callee calls nothing and
only reads it, and a self-recursive function that hands such a struct to
itself may have one copy of its own body inlined (the code that makes a
trap's stack list every source frame is unchanged), so that the struct needs
no object in the first level of the recursion. *(ADR 0923)*

```ail rules=native.sroa.3
struct P
    x float
    y float

fn len2(p P) float
    p.x * p.x + p.y * p.y

fn go(p P, d int) float
    if d == 0
        return len2(p)
    go(P(x: p.x + 1.0, y: p.y), d - 1)

fn main()
    print(go(P(x: 0.0, y: 3.0), 4))
```

**native.stack.1** — A struct built in a function that nothing but calls and
field accesses ever sees may live in the function's frame, not the heap: when
every use is a field read or write, or an argument for a parameter that the
callee only reads (or passes on to such a parameter), the object is not
carried past the call, so each activation and each iteration has its own copy
and the callee cannot keep it. *(ADR 0923)*

```ail rules=native.stack.1
struct P
    x float
    y float

fn depth(p P, d int) float
    if d == 0
        return p.x + p.y
    depth(P(x: p.x + 1.0, y: p.y * 2.0), d - 1)

fn main()
    print(depth(P(x: 0.0, y: 1.0), 3))
```

**native.stack.2** — A function whose every result is an object it built
itself, and that is not used as a function value, may build the result in an
object its caller provides; each direct caller provides one in its frame when
it only reads the result or lends it to a call, else on the heap. *(ADR 0923)*

```ail rules=native.stack.2
struct P
    x float
    y float

fn shift(p P) P
    P(x: p.x + 1.0, y: p.y)

fn main()
    print(shift(shift(P(x: 0.0, y: 2.0))).x)
```

**native.stack.3** — A present optional of an `int`, `float`, `bool`, `char`,
sized integer or fieldless enum that a function makes and returns at once may
be stored in a cell of the caller's frame instead of a heap box, when the
caller only tests the result against `none` and unwraps it (`else`, `?`,
`if x :=`, `==`, printing). A result that is stored, passed on, returned or
carried round a loop keeps its box, and so does a call through a function
value. The value, the traps and their order are those of the boxed result.
`AIL_OPT_OFF=optret` turns it off. *(ADR 1146)*

```ail rules=native.stack.3
fn pick(i int) int?
    if i % 10 == 9
        return none
    i * 3 + 5000

fn main()
    mut total := 0
    for i in 0..20
        k := pick(i) else -1
        total += k
    print(total)
```

**native.stack.4** — An optional of an `int`, `float`, `bool`, `char` or sized
integer that is carried from block to block (a `mut` variable assigned in a
loop or a branch) may be held as two values, whether it is present and the
number, with no box, when it is only tested against `none`, unwrapped and
assigned, and each value given to it is `none`, a number, another such
variable, or the result of a function whose every result is `none` or a
number it has just made. A variable that is stored, passed or returned
keeps its box. The value, the traps and their order are those of the boxed
variable. `AIL_OPT_OFF=optphi` turns it off. *(ADR 1152)*

```ail rules=native.stack.4
fn pick(i int) int?
    if i % 10 == 9
        return none
    i * 3 + 5000

fn main()
    mut best := pick(9)
    mut total := 0
    for i in 0..20
        c := pick(i)
        if i % 3 == 0
            best = c
        total += best else 1
    print(total)
```

**native.reuse.2** — Where a list is replaced by a new empty one (`buf = []`)
and no reference to the old list survives, the old list may be emptied
instead, under the conditions of native.reuse.1 and when the old list is
the compiler's own: not a parameter of the function. *(ADR 0826)*

```ail rules=native.reuse.2
fn main()
    mut line := []
    for i in 0..7
        line.push(i)
        if line.len() == 3
            print(line)
            line = []
    print(line)
```
**native.test.1** — *Retired (ADR 0900).* `ail test --native` compared each native
verdict with the interpreter's. With the interpreter gone, `ail test` runs every
`@test` function natively (native.test.2-5), and `ail test --native` is refused
with exit 2. *(ADR 0679, ADR 0900)*
**native.test.2** — The native build is of a rewritten copy of the file: `@test`
marks go and the copy's `main` is a dispatcher. `assert` and `assert_eq`
lower natively and trap as test.assert.1 says. The runner maps lines back, so
a construct the backend refuses is reported at its place in the original file.
*(ADR 0679, ADR 0858)*
**native.test.3** — A test the backend refuses (native.subset.1) is reported
as not in the native subset yet, with the first blocking construct and its
line. The same holds for a test whose file has a refused construct outside any
test; the report says so. A property test, which takes arguments, is skipped.
*(ADR 0679)*
**native.test.4** — *Retired with native.test.1 (ADR 0900).* The per-test
verdicts `match`, `DIFF`, `not-native` and `skipped`, the counts line and the
`--json` summary belonged to the comparison with the interpreter. `ail test`
reports each test as `ok`, `FAIL` or `skip` (test.run.1). *(ADR 0679, ADR 0900)*

```ail rules=native.bce.9
fn main()
    mut total := 0
    mut s := 2
    for i in 0..4
        total += i + s
    print(total)
```
**native.test.5** — A file inside a package (a directory under an `ail.pkg`) is
built in a staging directory that holds the package's other module directories
and the rewritten copy as the root module's only file, so its `use` lines
resolve. A package that has a dependency (`dep NAME path DIR`) isn't staged
yet: the dependency's own imports resolve against the wrong root, and its tests
are reported as not in the native subset yet. *(ADR 0679)*

## Checked examples for the native rules

Each block runs `ail test` or `ail build` with the options on its fence. The rules below have no observable output the harness can check (a Windows image, the cache, a backend fault, a package staging run), so their blocks are `norun`: checked, not run (native.build.2, native.build.4, native.build.5, native.test.5).

```ail tool=test,FILE expect=ok rules=native.str.1,native.test.1
@test fn strings()
    s := "hello"
    assert(s.len() == 5)
    assert("{3:>4}" == "   3")
    assert("ab" < "b")

fn main()
    print("ok")
```

```ail tool=test,FILE expect=ok rules=native.fnval.1
fn adder(n int) fn(int) int
    |x| x + n

fn double(n int) int
    n * 2

@test fn values()
    fs := [adder(3), double, |x int| x - 1]
    assert(fs.map(|f| f(10)) == [13, 20, 9])

fn main()
    print("ok")
```

```ail tool=test,FILE status=1 expect=FAIL rules=native.test.2
@test fn wrong()
    assert(1 == 2)

fn main()
    print("ok")
```

```ail norun rules=native.test.3
struct Tag
    n int

fn Tag.pick[T](self, xs List[T]) T
    xs[n]

@test fn picks()
    assert(Tag(n: 0).pick([1, 2]) == 1)

fn main()
    print("ok")
```

```ail norun rules=native.test.4
@test fn strings()
    s := "hello"
    assert(s.len() == 5)
    assert("{3:>4}" == "   3")
    assert("ab" < "b")

fn main()
    print("ok")
```

```ail tool=build,FILE status=1 expect=subset rules=native.subset.1
fn nine(a float, b float, c float, d float, e float, f float, g float, h float, i float) float
    a + i

fn main()
    print(nine(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0))
```

```ail norun rules=native.build.2
fn main()
    print("ok")
```

```ail norun rules=native.build.4
fn main()
    print("ok")
```

```ail norun rules=native.build.5
fn main()
    print("ok")
```

```ail norun rules=native.test.5
fn main()
    print("ok")
```

## Instrumentation

**native.instr.1** — A native build is instrumented only when the environment of
`ail build` has `AIL_INSTRUMENT` (a comma list of `lines`, `calls`, `tests`). Without it
the program has no instrumentation call and no table. *(ADR 0871)*

```ail rules=native.instr.1
fn main()
    print("no instrumentation without AIL_INSTRUMENT")
```

**native.instr.2** — `lines`: each statement of the program's own files (not the
prelude, `std`, generated code or the runtime) starts with a hit of its site. The build
writes the sites to `$AIL_INSTRUMENT_SITES`, one `ID<TAB>PATH<TAB>LINE<TAB>FN<TAB>FIRST`
line each. When the program exits or traps it appends the ids hit, one per line, to
`$AIL_INSTRUMENT_OUT`, else to `EXE.hits`. *(ADR 0871)*

```ail rules=native.instr.2
fn square(n int) int
    n * n

fn main()
    print(square(3))
```

**native.instr.4** — `counts`: the sites of `lines`, and each statement adds one to its site's counter.
At exit the program writes `ID<TAB>COUNT` for each site that ran, in place of the ids. Tasks count
without a lock, so a count may fall short. *(ADR 1186)*

```ail rules=native.instr.4
fn main()
    mut n := 0
    for i in 0..3
        n += i
    print(n)
```

**native.instr.3** — `calls`: each function of the program's own files starts a
`call<TAB>FN<TAB>LINE<TAB>COL` event with `<TAB>NAME=VALUE` for each argument that is a
number, a `bool` or a `str` (cut at 60 bytes). At most `$AIL_TRACE_LIMIT` (default 1000)
events are kept; a last line gives the count dropped. The events go to `$AIL_TRACE_OUT`,
else `EXE.trace`. `tests` is `calls` without the test runner's `main` and `ail_*` helpers.
*(ADR 0871)*

```ail rules=native.instr.3
fn greet(name str) str
    "hi {name}"

fn main()
    print(greet("bo"))
```
