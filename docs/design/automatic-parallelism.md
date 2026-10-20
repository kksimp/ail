# Automatic parallelism and modern hardware

> **Design note,** 2026-10-20 (lane 39). Decisions are in ADRs 0236–0239 and 0246–0247;
> normative rules are `conc.par.1–7` in [11-concurrency.md](../spec/11-concurrency.md)
> and the amended `type.coll.12`. This note explains them and records what a
> native runtime and backend will build on.

The designer's direction (2026-10-20): AIL should use modern CPUs and NPUs
**automatically**, optimised for at least four cores. The shape chosen:
**automatic where provable, plus a `par` keyword** for loops the compiler
can't prove, with a compile-time error when they are provably unsafe.

## 1. The one rule: parallel never changes the answer

`conc.par.1`: a loop the implementation runs in parallel gives **the same
observable result, bit for bit, as running it in order**, including which
trap happens. That makes parallelism a pure optimisation, like ADR 0026's
accelerators: a program means the same on 1 core or 64, under
`--deterministic` (ADR 0207) or not, in the interpreter or natively. Tests
never flake because of scheduling, and an AI reading a loop reasons about
it sequentially.

Two consequences shape everything else:

- **Float reductions keep their order.** `total += x` over floats is a left
  fold; any other grouping rounds differently. So a hand-written float
  accumulation is not auto-parallel (the checker says why and points to
  `xs.sum()`), and `xs.sum()` itself is now *defined* as a fixed tree whose
  shape depends only on the length (ADR 0239). Sequential, SIMD and
  parallel implementations all compute that tree.
- **Integer reductions are exact.** `+` on `int` is associative except where
  it traps (`expr.arith.1`). The runtime keeps each chunk's sum and the
  running minimum and maximum of its prefix sums in 128-bit integers. A
  sequential prefix overflowed exactly when some chunk's offset (the sum of
  the chunks before it) plus its prefix minimum or maximum leaves the `int`
  range; that chunk is re-run in order to find the trapping iteration. So
  the parallel sum traps where, and only where, the sequential one would.

## 2. What the compiler parallelises without being asked

### 2.1 Independent loops (`conc.par.2–4`)

A `for` loop's iterations are **independent** when all of these hold:

| Obligation | How the checker discharges it | Rule |
|---|---|---|
| No ordering effect | The body's effect set (lane 6's inference, `eff.infer.*`, through every call) has no `io`, `unsafe`, `spawn` or capability (`fs env proc clock random net`) | conc.par.2 |
| No early exit | No `break` (of this loop), `return`, `?`, or `else return` in the body; `continue` is fine | conc.par.2 |
| No synchronisation | No method call on a task, `TaskGroup`, `Channel` or `Mutex` handle (lane 22's types), directly or through a called function (a fixed point over the call graph) | conc.par.2 |
| Writes stay in the iteration | Every written place is rooted at a name declared in the body (or the loop variable), **or** is an element `r[i]...` where `i` is the variable of a range loop and the steps before `[i]` are fields, literals or outer names, **or** is an `int` accumulator (below). Method calls with `mut self` and `mut` arguments count as writes | conc.par.2–3 |
| Element reads don't cross | A root written at `r[i]` is read only through `r[i]...` or as `r.len()` | conc.par.3 |
| Accumulators are reductions | A root written only by `acc += e`, never read in the body, declared as an `int` | conc.par.4 |

Exclusivity (`own.excl.*`) is what makes this cheap: values are independent
(`own.val.1`), there are no aliases except `ref T`, and `for mut x in xs`
already forbids the body from touching `xs` (`own.excl.5`). So the analysis
is local to the loop body plus the effect sets; there is no pointer
analysis. `for mut x in xs` is independent whenever its body is, since each
iteration owns its element.

Not yet handled (each is a conservative "sequential", never a wrong
"parallel"): `acc = acc + e`, `-=`, `*=`, min/max patterns, writes through
`ref T` (lane 23), and mutation through trait calls whose receiver
mutability isn't known statically.

### 2.2 Collection operations

Built-in operations whose elements are independent are parallel and SIMD
candidates in the native runtime with no analysis beyond their element
type's trait implementations being free of ordering effects:

- `xs.sum()`: int exact (as above), float in the fixed tree (ADR 0239).
- `xs.min()` / `xs.max()`: the *first* least or greatest element
  (`type.coll.11`), so chunks combine left to right, keeping the earlier
  winner on ties.
- `xs.sorted()` / `xs.sort()`: stable, so the result is unique for a given
  `cmp`; a parallel merge sort gives the same list.
- `contains`, `index_of` (lowest index wins), `join` (chunked, then
  concatenated in order).
- `map`/`filter`-like methods, when they exist, follow the loop rules for
  the function they apply.

Each needs the element type's `Eq`/`Ord`/`ToStr`/`Hash` implementations to
be free of ordering effects, which `eff.infer.5` already tracks.

### 2.3 Independent calls in one expression

`f(a) + g(b)`: operands are evaluated left to right (`expr.*`), but if both
calls are free of ordering effects and neither can trap, the order isn't
observable and the backend may run them as a fork-join pair. Traps make
this rarely provable today (`+` itself can trap), so it's a backend
optimisation with a cost threshold, not a language rule.

## 3. `par for` (`conc.par.5`)

`par for x in e` means exactly what `for x in e` means (the interpreter
runs it in order). It adds an **assertion** that the iterations are
independent, for what the analysis can't prove:

- writes to `r[e]` where `e` isn't the loop variable (a permutation,
  `out[perm[i]]`);
- reads of other elements of a written root;
- an accumulator whose type the analysis can't see.

Everything the analysis *can* prove is still checked: an ordering effect,
an early exit, synchronisation, a whole-variable write, a float
accumulation, reading an accumulator. Each is a `conc.par.5` error naming
the first offending write or effect, with a help line.

A future checked build verifies the assertion at run time: each parallel
`par for` records the elements it writes per chunk and traps
(`conc.par.5`) on overlap. Release builds trust it, as with `unsafe`
contracts.

`par` is a reserved word (`lex.kw.1`), not a contextual one: one meaning
per word is easier for a model to predict, and no example or test used it
as a name. Measured with tokbench on a five-line loop (mean of five
tokenizers): `for` 59.2, `par for` 60.2, `parallel for` 60.2,
`for i in par(0..4)` 60.8, `@par for` 61.2. The corpus has no `par` loops, so
its totals don't change.

Rejected: an attribute `@par for` (attributes are declaration metadata,
ADR 0083, and `@` costs a token); `parallel for` (ties on tokens, so the
designer's `par` stands); `for ... in par(xs)` (looks like a function call that changes
the collection); OpenMP-style clauses (`par(schedule: dynamic)`): the
runtime chooses scheduling, and a clause would let a program depend on it.

## 4. Runtime model (native runtime; ADR 0238)

- **One pool.** One worker thread per hardware thread, created lazily, shared
  by tasks (`conc.task.3`) and parallel loops, so nesting never
  oversubscribes. The cost model assumes **at least four cores** when it
  decides statically; at run time it uses the real count (and the
  `AIL_THREADS` cap).
- **Work stealing.** Each worker has a Chase–Lev deque. A parallel loop is
  split lazily (*lazy binary splitting*): a worker runs its range from the
  front and splits off the back half only when another worker is idle and
  the remainder is above the grain. Heterogeneous cores (Apple P/E, Intel
  P/E) balance themselves because slow workers get stolen from.
- **Grain and threshold.** The compiler estimates a body's cost from its
  IR. A loop whose `iterations × cost` is below about 20 µs (four thread
  wakes) runs sequentially without touching the pool. Unknown costs (calls
  through `any Trait`, data-dependent inner loops) start sequentially and
  split after the first grain has been timed.
- **Chunks are contiguous** so each chunk vectorises (§5) and walks memory
  forwards, and chunk boundaries of written lists fall on 64-byte lines to
  avoid false sharing.
- **Fixed combination order.** Reductions combine per-chunk partials in an
  order fixed by iteration ranges, never by which worker finished first, so
  results don't depend on the machine (§1).
- **Traps.** A trapping iteration stops the loop. If several iterations trap,
  the loop reports the one with the lowest index (chunks below it finish
  first to make sure none of them traps earlier). Writes made by the other
  iterations are not observable: a trap ends the task (`conc.task.6`).
- **Composition with tasks.** A parallel loop is fork-join inside the
  current task: it ends before the statement after it starts, so it is
  scope-bound like `conc.task.4` without a `TaskGroup`. Its chunks aren't
  tasks in the language (no handles, no `Trapped`); a loop's trap is its
  task's trap. Inside a spawned task, a parallel loop uses the same pool.
- **Reporting.** The performance report (ADR 0026) lists which loops ran
  in parallel, their chunk counts, and the active SIMD level, so speed
  differences between machines are explainable.

The **bootstrap interpreter runs `par for` on a thread pool** (ADRs
0272–0275). Its values are `Rc`, so each worker gets a copy of the captured
values once per run and returns only the list elements it changed and its
partial sums; the main thread applies them in chunk order, and any trap
reruns the loop in order. `ail run --parallel` also pools loops the analysis
proves independent. A compute-bound loop (perfbench `parsum`) runs 3.1×
faster on 4 threads; loops over large captured values gain less, because the
copy is per worker. The analysis half (`ail effects --parallel`) is what the
native backend will consume.

## 5. SIMD (ADR 0246)

The loops that are independent are also the loops that vectorise: no
loop-carried dependence except recognised reductions. The backend
vectorises them inside each chunk:

| Target | Baseline (no check) | Dispatched at run time |
|---|---|---|
| x86-64 | x86-64-v3: AVX2 + FMA, 256-bit (4 × f64, 8 × f32, 4 × i64) (ADR 0005) | AVX-512 (v4) where present |
| ARM64 | ARMv8.2-A: NEON/ASIMD 128-bit (2 × f64, 4 × f32), FP16 arithmetic (ADR 0006) | SVE/SVE2 where present, SME on chips that have it |

Lockstep (ADR 0008) means neither ISA is the reference: the vector width is
a backend parameter, and kernels unroll NEON to two registers so both ISAs
have four f64 lanes in flight.

- **Overflow checks vectorise:** add lanes in 64 bits, OR the per-lane
  overflow flags, and only on a set flag redo the chunk scalar to trap at
  the right iteration.
- **Float `sum()`**: the fixed tree's leaf keeps four running sums (element
  `k` into sum `k % 4`) and returns `(p0 + p1) + (p2 + p3)`. That is exactly
  one AVX2 register or two NEON registers, so the vector code *is* the
  specification, not an approximation of it.
- **`fma` is never introduced** by the compiler: fusing `a * b + c` rounds
  once instead of twice and changes bits. A future `fma(a, b, c)` function
  is the explicit form.
- **Layout control buys the rest.** Compiler-chosen layout (ADR 0013) lets
  the backend store `List[Point]` as struct-of-arrays when the loops that
  use it read fields independently, so `for p in pts: p.x * k` becomes a
  contiguous vector load instead of a strided gather. This is the main way
  AIL can beat C here: C can't change a struct array's layout behind the
  programmer's back.

## 6. NPUs and GPUs (ADR 0247)

### 6.1 What's reachable per target

| Target | NPU | GPU compute | Uniform? |
|---|---|---|---|
| macOS (Apple Silicon) | Apple Neural Engine: **only through Core ML** (an `MLModel` / ML Program); no public direct API. BNNS and Accelerate run on the CPU | Metal compute (MSL compiled by the OS), MPSGraph | One vendor, stable APIs |
| Windows 11 | Windows ML (ONNX Runtime, with vendor execution providers: Qualcomm QNN, Intel OpenVINO, AMD Vitis AI). DirectML is in maintenance and runs on GPUs mainly | Direct3D 12 compute, Vulkan | Yes, through ONNX graphs |
| Linux | Kernel `accel` subsystem (`/dev/accel/*`) with per-vendor user space: Intel NPU (Level Zero / OpenVINO), AMD XDNA (XRT), Qualcomm (QNN) | Vulkan compute | **No** uniform NPU API |

The common shape: **NPUs run operator graphs, not arbitrary code.** Every
reachable NPU API takes a graph of tensor operations (matmul, convolution,
elementwise, reductions, activations) in a vendor or ONNX format, usually at
fp16 or int8. GPUs run arbitrary kernels (SPIR-V for Vulkan, MSL for
Metal).

### 6.2 Proposal: `std.tensor` with pluggable backends

The CPU slice is built (eager, exact, AVX2/NEON, pooled `matmul`; ADRs
0328–0329), and [ADR 0330](../decisions/0330-tensor-accelerator-backends-plug-in.md)
fixes how a device backend plugs in.

- A `Tensor[T]` type (dense, row-major by default, shape known at run time)
  with operations the NPU APIs all share: `matmul`, `conv2d`, elementwise
  arithmetic and activations, `sum`/`max` over axes, `reshape`,
  `transpose`.
- Operations build a lazy graph; `t.eval()` (or reading an element) runs
  it on a backend.
- **Backends:** `cpu` (always present; AIL code, multi-core and SIMD under
  §4–5, bit-reproducible), `coreml` (macOS ANE/GPU), `winml` (Windows
  NPUs/GPUs via ONNX), `vulkan` (GPUs on Linux and Windows), `metal`.
- **Precision is part of the request.** The `cpu` backend is exact by
  `conc.par.1`'s standard. NPUs compute in fp16/int8 with their own
  accumulation order, so their results differ. A graph runs on an
  accelerator only when the program asks for approximate results:
  `t.eval(precision: .approx)` (or `f16`/`i8` tensors). Then the runtime
  picks the device automatically (largest graph on the NPU, otherwise GPU,
  otherwise CPU). That is "automatic" in the designer's sense without
  silently changing exact programs.
- **An `accel` capability** (`eff.cap.*` machinery): reaching an
  accelerator device gives `accel`. `--deny accel` forces the CPU backend,
  and `--deterministic` adds `accel` to its denied set, since device
  results vary by driver. ADR 0026's rule (accelerators never change
  semantics) holds because `accel` results are explicitly approximate, like
  reading the clock is explicitly nondeterministic.

### 6.3 Constraint to settle: driver compilers

ADR 0010 forbids external backends. A GPU or NPU can't be reached without
the OS driver compiling *something*: Vulkan drivers compile SPIR-V, Metal
compiles MSL, Core ML compiles its model, ONNX Runtime compiles ONNX. AIL
would generate SPIR-V/MSL/graphs itself (its own code generator), and the
driver's compiler is part of the platform, like the CPU's microcode. This
reading was recorded as OQ-090 and the designer accepted it in
[ADR 0381](../decisions/0381-driver-compilers-are-platform.md).

### 6.4 First prototype and cost

1. **`std.tensor` on the CPU in the bootstrap** (about one lane-night):
   the type, the operations, lazy graph, exact CPU evaluation, tests. This
   fixes the API before any device work.
2. **Core ML backend on macOS** (one to two weeks after FFI to the
   Objective-C runtime exists, `14-ffi.md`): serialise the graph as a Core
   ML ML Program, compile it with `MLModel.compileModel`, run on the ANE.
3. **Vulkan compute** (two to three weeks): an AIL SPIR-V emitter for the
   elementwise and reduction kernels, then matmul.
4. **Windows ML** via ONNX graph export (one to two weeks, after FFI).

## 7. Tooling: `ail effects --parallel` (`conc.par.6`)

```
$ ail effects --parallel examples/parallel.ail
examples/parallel.ail:32:5 fn:main for _: sequential (conc.par.2: writes `rows`, which outlives the iteration at examples/parallel.ail:33:9)
examples/parallel.ail:34:5 fn:main for y: parallel (conc.par.3: writes only `rows[y]`)
examples/parallel.ail:36:9 fn:main for x: sequential (conc.par.2: writes `line`, which outlives the iteration at examples/parallel.ail:39:13)
examples/parallel.ail:41:5 fn:main for row: sequential (conc.par.2: has effect `io` via print)
examples/parallel.ail:44:5 fn:main for y: parallel (conc.par.4: adds to `steps` in a fixed order)
```

`--json` gives `{"ail_api":1,"loops":[{"at","fn","var","par","verdict","rule","reason","reason_at"}]}`.
Verdicts are `parallel`, `asserted` (a `par for` that passes only by its
assertion) and `sequential`. An agent uses it to find the loop that would
parallelise if one write moved: the reason names the write and its position.

## 8. Measurements

- **Tokens:** `par` adds one token per annotated loop and nothing elsewhere;
  automatic parallelism costs zero tokens, which is the point of making it
  automatic. The corpus is unchanged (tokbench shows no difference).
- **Interpreter speed:** the analysis runs only for `ail effects --parallel`
  and, in `check`, past a scan for `par` only when a `par for` exists. Float
  `sum()` now gathers its values before adding. Perfbench on Linux x86-64,
  best of 7 runs, plan branch before → after: `interp` 0.208 → 0.212 s,
  `json` 0.011 → 0.012 s, `sheet` 0.023 → 0.023 s, `loops` 0.686 → 0.705 s
  (three re-runs: 0.699/0.704, 0.700/0.709, 0.791/0.690, so noise),
  `examples/parallel.ail` 0.029 → 0.029 s. No measurable change.
