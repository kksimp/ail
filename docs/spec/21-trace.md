# 21. Runtime traces and trap snapshots

Chapter keys: `trace`, `probe`

> **Status:** Drafted (partial). Specified: `ail run --trace`, its event
> budget and summary, `--watch NAME`, and the snapshot a trap carries.
> The exact line and JSON formats are non-normative and live in
> [semantic-api.md](../design/semantic-api.md#runtime-traces-ail-run---trace-adr-0265).
> Tests are traced by `ail test --trace` (trace.test.*). Not yet designed:
> breakpoints, and a trace of one function only.

A trace lets the person or agent who ran a program see what it did without
adding prints: which functions ran with which values, which effects it
reached, and, when it traps, the stack and its values at that moment.
Tracing is a tool option; it changes nothing the program can observe.

## Trace mode

**trace.run.1** — `ail run --trace[=calls|effects|all] FILE` writes one
event per line to standard error. `calls` traces calls, returns and contract
checks; `effects` traces effects reached and tasks spawned and waited; `all`
(the default for a bare `--trace`) traces both. With `--json`, each event is
one JSON object. The tracing options (`--trace`, `--trace-limit`, `--watch`,
`--verbose`) belong to `ail run` and `ail test` (trace.test.1); with another
command, or malformed, they are an error before anything runs (exit 2).
*(ADRs 0265, 0326)*

**trace.run.2** — A `call` event names the function, its receiver and
arguments with their values, its call depth (`main` is 1), and the position
of the call. A `return` event names the function and the value it returned.
Calls of the prelude's functions are not traced. A position in the program's
first file is `LINE:COL`; elsewhere it is `FILE:LINE:COL`. *(ADR 0265)*

**trace.run.3** — An `effect` event is each call of a standard module
function (`fs.read_text`, `proc.run`, `time.now`, ...) with its argument
values, before it runs. A `spawn` event is each `spawn` with the function and
arguments; a `wait` event is each `Task.wait()`. A `contract` event is each
`requires` or `ensures` check (contract.run.1) with its condition and whether
it held. *(ADR 0265)*

**trace.run.4** — At most `--trace-limit N` events are shown (default
1,000); later events are counted, not shown. When the program ends, by
finishing, failing or trapping, one summary line gives the events shown, the
events dropped, and the count of each kind. *(ADR 0266)*

**trace.run.5** — Tracing MUST NOT change the program's behaviour: standard
output, exit status, and the output of a `--deterministic` or `--seed` run
are the same with and without it. With no tracing option, the interpreter's
speed is within measurement noise of a build without tracing. *(ADR 0265)*

```ail
fn fact(n int) int
    requires n >= 0
    if n <= 1
        return 1
    n * fact(n - 1)

fn main()
    print(fact(3))
```

**Note:** `ail run --trace=calls` on this program shows `call fact(n = 3)`,
the nested calls, a `contract` line for each check, `fact returned 6`, and
the summary.

## Values

**trace.val.1** — Traces, watches and snapshots show values in their
structural text form (trait.auto.2, strings and characters quoted), without
calling any `to_str`: showing a value runs no user code, so it has no effects
and can't trap. *(ADR 0266)*

```ail tool=run,--watch,s,FILE expect="hi" rules=trace.val.1
fn main()
    mut s := "hi"
    s = "there"
    print(s)
```

**trace.val.2** — A value is cut to a character budget and ends in `…` when
cut; a list, map or set that is cut says how many elements are left. The
budget is 60 characters in trace lines and in the default trap report, and
1,000 in a watch, a JSON snapshot and `--verbose`. *(ADR 0266)*

## Watch

**trace.run.6** — Natively, `ail run --trace` builds the program with the events asked for
(only then does it cost anything) and writes them to standard error as the program runs, then
the summary (trace.run.4), in the text form or, with `--json`, as the JSON objects of
trace.run.1. `calls` gives the `call` and `return` events (trace.run.2) and a `contract` event
for each condition checked; `effects` gives the `effect` events of the OS and network modules,
and `spawn` and `wait` (`Task#N`, numbered from 1 in spawn order). A call's position is the
call's, `main`'s its own. A value is cut to 60 bytes and ends in `…` (trace.val.2); a returned
list or map is `…`, and a cut list or map does not say how many elements are left. *(ADR 0868)*

**trace.watch.1** — `ail run --watch NAME FILE` shows, after each
assignment whose target is rooted in a variable called `NAME` (`NAME = e`,
`NAME op= e`, `NAME.f = e`, `NAME[i] = e`) and after each `mut NAME := e`,
the variable's whole new value and the position of the assignment. It counts
against the event budget (trace.run.4). *(ADR 0268)*

```ail tool=run,--watch,s,FILE expect=watch: rules=trace.watch.1
fn main()
    mut s := "hi"
    mut xs := [1]
    s = "there"
    xs.push(2)
    xs[0] = 5
    print(s)
```

**trace.watch.2** — A builtin mutating method (`NAME.push(x)`) is shown as
a write to the element (trace.watch.4). A change
made by a user function through a `mut` argument or `mut self` is shown
(trace.watch.3). *(ADRs 0268, 0327)*

```ail tool=run,--watch,xs,FILE expect=push rules=trace.watch.2
fn main()
    mut s := "hi"
    mut xs := [1]
    s = "there"
    xs.push(2)
    xs[0] = 5
    print(s)
```

**trace.watch.4** — A write to one element of the watched collection is a
watch event with the whole new value and what was written: the list methods
`push`, `pop`, `insert`, `remove_at`, `swap`, `extend`, `clear`, `reverse`,
`sort`, the map method `remove`, the set methods `add` and `remove`, and
`NAME[i] = e` and `NAME[k] = e` (also `op=`). In text the event ends with
`[push 3 at index 2]`, `[pop 3 at index 3]`, `[insert 9 at index 0]`,
`[remove_at 9 at index 0]`, `[set 5 at index 1]`, `[set 2 at key "b"]`,
`[remove 1 at key "a"]` or `[add 3]`; in JSON it adds `"op"`, `"element"` and
`"index"` or `"key"`. A `pop`, `remove_at`, `remove` or `add` that changes nothing
is not shown. *(ADR 0347)*

```ail tool=run,--watch,xs,FILE expect=set rules=trace.watch.4
fn main()
    mut xs := [1]
    xs[0] = 9
    print(xs.len())
```

**trace.watch.3** — A user function or method called with the watched variable
(or a name that stands for it) as a `mut` argument or `mut` receiver follows it
under the callee's own name for it: an assignment rooted in that name shows the
watched variable's whole new value in that frame, until the call returns. The
event names the frame, the function that wrote, and, when the callee's name
differs, that name (`via`). In JSON: `"frame":"Counter.bump","via":"self"`;
in text, ` in Counter.bump (as \`self\`)` after the position, left out for the
first frame so a plain run reads as before. A call that passes the value by
copy (not `mut`) does not follow it. *(ADR 0327)*

```ail tool=run,--watch,x,FILE expect=bump rules=trace.watch.3
fn bump(mut n int)
    n += 1

fn main()
    mut x := 1
    bump(mut x)
    print(x)
```

**trace.watch.5** — Natively, `ail run --watch NAME FILE` builds the program with
the watch (only then does it cost anything) and shows trace.watch.1's events on
standard error, `watch: NAME = VALUE at LINE:COL`, then the summary line
(trace.run.4); over `--replay R` (replay.run.1) the same run is repeated exactly,
so it answers "when did `NAME` become 0?" on a recorded run. The events are the
assignments and `mut NAME := e` of trace.watch.1; the element writes of
trace.watch.4 `push`, `pop`, `insert`, `remove_at`, `swap`, `extend`, `clear`, `reverse`,
`sort`, `NAME[i] = e`, `NAME[k] = e`, and a map's `remove` and a set's `add` and `remove`
(only when they change it) end in their note (`[push 3 at index 2]`, `[swap at index 0, 2]`,
`[add 3]`); and a `mut` argument or `mut self`
that holds the watched variable is followed into the callee (trace.watch.3), the event
ending in ` in FN` and ` (as \`p\`)` when the callee's name differs. With `--json` each
event is the one-line object of trace.watch.4's JSON form (`{"api":1,"trace":"watch",...}`).
*(ADRs 0868, 0886)*

```ail tool=run,--watch,n,FILE expect=watch: rules=trace.watch.5
fn main()
    mut n := 3
    n -= 3
    print(n)
```

## Tracing tests

**trace.test.1** — `ail test --trace[=calls|effects|all]` (and `--trace-limit`,
`--watch`) trace each test as `ail run` traces a program, but keep each
test's events instead of writing them: only a failing test reports them. With
`--json` the failing test's entry gets `"trace"`, a list of the event objects
`ail run --trace --json` writes (without the summary line); a passing test,
a property test, and a run without `--trace` have no `trace` key. In the human
form the events are printed under `FAIL`, indented, before the test's output.
*(ADR 0326)*

```ail tool=test,--trace=calls,FILE status=1 expect=adds() rules=trace.test.1
@test
fn adds()
    assert_eq(1 + 1, 3)
```

**trace.test.2** — The budget (trace.run.4) applies per test. When it drops
events, the list ends with `{"api":1,"trace":"dropped","count":N}` (text:
`trace: N more events dropped`). Tracing never changes a test's result.
*(ADR 0326)*

```ail tool=test,--trace=calls,--trace-limit,2,FILE status=1 expect=dropped rules=trace.test.2
fn step(n int) int
    n + 1

@test
fn counts()
    mut n := 0
    for i in 0..5
        n = step(n)
    assert_eq(n, 99)
```

## Trap snapshots

**trace.snap.1** — A trap that ends `ail run` (err.trap.3) is reported with
a snapshot: the source line of the trap's position, and the call stack at
the trap. *(ADR 0267)*

```ail tool=run,FILE status=1 expect=trap[type.coll.4] rules=trace.snap.1
fn inner(xs List[int]) int
    xs[5]

fn main()
    print(inner([1]))
```

**trace.snap.2** — The stack lists frames innermost first. Each frame names
its function, the position it was executing (the trap for the innermost
frame, the call of the next frame for the others), its receiver and
parameters with their values, and its other locals in scope with their
values. At most 64 frames are kept; the rest are counted. A failed
`requires` or `ensures` also gives the value of each name its condition
reads. *(ADR 0267)*

```ail tool=run,FILE status=1 expect=main() rules=trace.snap.2
fn inner(xs List[int]) int
    xs[5]

fn main()
    print(inner([1]))
```

**trace.snap.3** — The human report keeps the trap's first line
(`FILE:LINE:COL: trap[RULE]: MESSAGE`) and is at most ten lines: the source
line with a caret at the column, the condition's values, then one line per
frame until the limit, then how many frames were left out. `--verbose` shows
every kept frame and each local on its own line, at the long budget. *(ADR 0267)*

```ail tool=run,FILE status=1 expect=^ rules=trace.snap.3
fn inner(xs List[int]) int
    xs[5]

fn main()
    print(inner([1]))
```

**trace.snap.4** — With `--json`, the trap is one JSON object on standard
error with `"severity":"trap"`, the rule, position and message, the source
line, the condition's values, and the stack. Natively `./ail` builds the program with
`--values --verbose` and `tools/trapjson.awk` turns the report into that object, so
the arguments and locals of every frame are given as in the interpreter's. Without
`--trace`, `--json` is the only option that does this; with it, trace.run.1 applies. *(ADR 0267)*

**trace.frame.1** — An executable built by `ail build` reports a trap with
the stack the interpreter reports (trace.snap.2, trace.snap.3), without the
frames' values: the trap's first line and source excerpt, then one line
`  at [FILE:]LINE:COL in NAME(..)` per frame, innermost first (`NAME()` for a
function without parameters; `FILE:` only outside the first file), where
`LINE:COL` is the trap for the innermost frame and the call for each other.
At most seven frames print; with more, the first six and then
`  … N more frames`. A constant's initialiser (mod.const.*, ADR 0604) has no
frame. *(ADRs 0641, 0642)*

**trace.frame.2** — A check that the compiler copied into a caller by inlining
reports the callee's frame, then the caller at the inlined call, as if the
call had been made. A call that the compiler copied in from an inlined function
(a function's own recursive call, ADR 0890, or any call of a function inlined by
native.inline.1) reports the frames it was inlined into after its own, so the
stack lists every source frame. *(ADRs 0643, 0890, 0940)*

**trace.frame.3** — `ail run --values FILE` builds the program so that its
own functions keep their arguments and locals where a trap can print them. The
trap then reports each frame as the interpreter does (trace.snap.2,
trace.snap.3): `  at LINE:COL in NAME(a = 1, b = 2); c = 3` (the arguments,
then `; ` and the locals in scope at the statement that was running, in
declaration order; the values of types the runtime can print), a contract
trap's `  values: ` line (the arguments the condition names, in name order),
and `  … N more frames (--verbose shows every frame and value)`. A build
without `--values` prints no values (trace.frame.1) and costs nothing for
them. `--verbose` implies `--values` and, as trace.snap.3 says, shows every frame, with each
local on its own line: `      NAME = VALUE`. *(ADR 0886, ADR 0641)*

**Note:** a trap inside a task is reported to the task that waits
for it (conc.task.6), whose own stack is the one a snapshot shows.

```ail tool=run,--json,FILE status=1 expect="severity":"trap" rules=trace.snap.4
fn inner(xs List[int]) int
    xs[5]

fn main()
    print(inner([1]))
```

## Run limits

**run.limit.1** — `ail run --timeout MS` and `ail test --timeout MS` stop a
run that has taken longer than `MS` milliseconds of wall-clock time, with
`trap[run.limit.1]` at the statement about to run, and the trap's snapshot
(trace.snap.1). `ail test` gives each test the whole allowance. The check
happens between statements, so a single call that blocks (`sleep`, a read)
ends only once it returns. *(ADR 0288)*

```ail tool=run,--timeout,200,FILE status=1 expect=run.limit.1 rules=run.limit.1
fn main()
    mut n := 0
    while true
        n += 1
```

**run.limit.2** — `--max-memory MB` stops a run whose process uses more than
`MB` megabytes of resident memory, with `trap[run.limit.2]`, as for run.limit.1.
A single statement that allocates past the limit may still exhaust the machine
first. The limit works the same on Linux, macOS and Windows. *(ADR 0288, ADR 0726)*

```ail tool=run,--max-memory,16,FILE status=1 expect=run.limit.2 rules=run.limit.2
fn main()
    mut xs := [0]
    while true
        xs.push(1)
```

**run.limit.3** — `--max-steps N` stops a run that has run more than `N`
steps, with `trap[run.limit.3]` at the statement that passed it. A step is one
statement of the program's own files run; a `while` turn adds none of its own,
and the std's statements are not counted. The program is built with the
counter in it only when the flag is given (ADR 0883). Steps count the same on
every machine, so a limit found once reproduces the same trap. `ail test`
counts per test. *(ADR 0288, ADR 0883)*

```ail tool=run,--max-steps,1000,FILE status=1 expect=run.limit.3 rules=run.limit.3
fn main()
    mut n := 0
    while true
        n += 1
```

**run.limit.4** — The three options take a positive whole number. Any other
value, a missing value, or use with a command other than `run` or `test`, is
an error with exit code 2. With none of them given, a run is not checked.
*(ADR 0288)*

```ail tool=run,--timeout,0,FILE status=2 expect=run.limit.4 rules=run.limit.4
fn main()
    print(1)
```

## Checked examples for the run and value rules

Each block runs `ail run` with the options on its fence (trace.run.1-5, trace.val.2).

```ail tool=run,--trace=calls,FILE expect=call rules=trace.run.1
fn double(n int) int
    n * 2

fn main()
    print(double(2))
    print(double(3))
```

```ail tool=run,--trace=calls,FILE expect=returned rules=trace.run.2
fn double(n int) int
    n * 2

fn main()
    print(double(2))
    print(double(3))
```

```ail tool=run,--trace=effects,FILE expect=effect rules=trace.run.3
use std.time

fn main()
    t := time.now_ms()
    print(t > 0)
```

```ail tool=run,--trace=calls,--trace-limit,2,FILE expect=dropped rules=trace.run.4
fn double(n int) int
    n * 2

fn main()
    print(double(2))
    print(double(3))
    print(double(4))
```

```ail tool=run,--trace=all,FILE expect=done rules=trace.run.5
fn main()
    print("done")
```

```ail tool=run,--trace=calls,FILE expect=… rules=trace.val.2
fn describe(s str) int
    s.len()

fn main()
    print(describe("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz"))
```
**run.limit.5** — A native executable (`ail build`) reads the environment at
start: `AIL_TIMEOUT_MS` and `AIL_MAX_MEMORY_MB`, each a positive whole number.
`ail build --timeout MS --max-memory MB` bakes in defaults that the
environment overrides. An invalid value ends the program with exit code 2
before `main` runs. With neither set, the program is not checked. Windows
executables read the same two variables. *(ADR 0728)*

```ail tool=build,--timeout,1000,FILE rules=run.limit.5
fn main()
    print("hello")
```

**run.limit.6** — A native program that goes past a limit prints its
output so far, then `trap[run.limit.1]` or `trap[run.limit.2]` with the same
message as `ail run`, and exits with status 1. The line has no `FILE:LINE:COL`
prefix and no snapshot, because the native program has no statement to point at.
The memory limit is checked when the heap takes a new chunk and counts the
heap bytes in use. The time limit is checked every 256 allocations and when
the heap takes a chunk. A `while` loop also checks it, but only in an executable
built with `--timeout` (ADR 0727); so without that, a loop that never allocates is not stopped. The
limit is wall-clock time on a clock that doesn't go back. On macOS the clock is read with
`gettimeofday`. *(ADR 0726, ADR 0727)*

```ail tool=build,--max-memory,16,FILE rules=run.limit.6
fn main()
    print("start")
    mut xs := [0]
    for i in 0..100000000
        xs.push(i)
```

## Probes

**probe.run.1** — `ail run --at FILE:LINE --show a,b [FILE]` prints, on standard
error, one line `probe: FILE:LINE a = V, b = W` each time a statement that starts on
line LINE of FILE begins, with the values of the variables `a` and `b` in scope at
that point. It may be combined with `--replay R` (`replay.run.1`), which repeats the
recorded run. The program's output and exit status are unchanged
([ADR 0921](../decisions/0921-replay-probes.md)).
**probe.run.2** — A name that is not a variable in scope at the line shows
`<unset>`, and a value of a type the trap snapshots cannot print (`trace.val.1`) shows
`<type>`. Values are cut and escaped as in a trace (`trace.val.1-2`).
**probe.run.3** — A probe event counts against the event budget and summary of
`trace.run.4`; with `--json` it is the object `{"api":1,"trace":"probe","at":"FILE:LINE",
"NAME":"VALUE",...}`.
**probe.run.4** — `--at` needs `FILE:LINE` and `--show` needs variable names
separated by commas, and each needs the other; otherwise it is a usage error
(exit 2). A program run without them is built and behaves as before.

```ail pkg=probe_demo tool=run,--at,main.ail:3,--show,n,main.ail expect=probe: rules=probe.run.1
```

```ail tool=run,--at,nosuch,--show,a,FILE status=2 expect=probe.run.4 rules=probe.run.1,probe.run.2,probe.run.3,probe.run.4
fn main()
    print("hi")
```

