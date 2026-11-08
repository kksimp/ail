# GPU compute: Metal first, then Vulkan (design note)

> **Status:** Proposed, non-normative. Follows [ADR 0381](../decisions/0381-driver-compilers-are-platform.md)
> (the OS driver's compiler is platform), [ADR 0958](../decisions/0958-gpu-compute-first.md)
> (compute first, then rendering) and [automatic-parallelism.md §6](automatic-parallelism.md).
> Goal from the designer: AIL uses modern hardware very well, automatically where
> it is efficient, never forcing a device that would be slower.

## What runs on a GPU

A GPU wins on large, regular, independent work with little data per unit of
compute. AIL already finds that work: `par for` and the compiler-chosen
loops (`conc.par.1`-`conc.par.16`), `Tensor` operations and fused elementwise
chains (ADR 0524), and `matmul`. First candidates, in order:

1. **Output-only or one-pass loops** over a range whose body is scalar `int`
   and `float` arithmetic, `if`, bounded inner loops and reads of
   `Tensor[f32]` or flat `List[float]` inputs, with writes `r[i]` (`conc.par.3`).
   Mandelbrot, n-body force sums, image filters, particle updates.
2. **`Tensor[f32]` elementwise chains** and **`matmul`** past a size.
3. **Reductions** (`sum`, `max`) as a second pass.

Everything else stays on the CPU. A loop that allocates, calls a function the
analysis cannot inline, traps in a way it cannot report, or touches `str`,
`Map` or a user type is *not eligible*, and the report says which of these.

## Four layers, each testable without a GPU

| Layer | Content | Needs hardware |
|---|---|---|
| 1. **Eligibility** | `ail effects --gpu FILE` prints, per loop and tensor op, `eligible` or the reason code why not, plus an estimate of work and bytes moved | no |
| 2. **Kernel IR and emitters** | a small SSA kernel IR from MIR; `MSL` and `SPIR-V` text/binary emitters; golden files | no |
| 3. **Device host** | runs a kernel on a device and moves buffers | yes |
| 4. **Offload policy** | a cost model decides CPU or GPU per call; `--deny accel` forces CPU | cost data only |

Layers 1 and 2 are pure compiler work in AIL and gate on any machine, so the
lane can land them and test them before any Metal code exists. Agents benefit
from layer 1 alone: the report says what to change to make a hot loop eligible,
with the same fix-suggestion shape as other diagnostics.

## The device host

A native AIL binary has no Objective-C runtime. Metal is an Objective-C API;
`objc_msgSend` needs a per-signature function type that `extern c fn`, which has
one signature per symbol, cannot provide (`ffi.decl.2`). Vulkan is plain C and can
be called straight from AIL.

**Decision for Metal: a small helper process, the same pattern as the app host.**
`tools/gpu-host/macos` (Swift, built by `run.sh`) speaks the line protocol of
[ADR 0957](../decisions/0957-shared-surface-transport.md): one JSON line per
command on stdin/stdout, bulk data in **shared mappings** made with the same
mkstemp/mmap recipe (mode 0600, random name, unlinked after mapping). On Apple
silicon memory is unified: the host wraps a page-aligned shared mapping in an
`MTLBuffer` with `bytesNoCopy`, so a buffer the program fills is the buffer the
GPU reads, with no copy in either direction.

Commands: `buffer` (map a file as buffer N), `kernel` (MSL source, entry name),
`dispatch` (kernel, buffer bindings, grid), `wait` (completion, returns the GPU
time), `drop`. Reply lines carry a status and error text (the Metal compiler's
message, which the program shows as a diagnostic). Vulkan on Linux and Windows
starts the same way for symmetry, then moves into the program through FFI when
the C-struct building is worth it: SPIR-V and the API calls are all pointers and
integers.

## Codegen

MIR is typed and has explicit loads and stores, so the kernel IR is a straight
subset translation: one kernel per eligible loop, the loop index becomes the
thread id, `int` is 64-bit only where needed (MSL has `long`), `float` is `f32`
on the device. Because device `float` is narrower than AIL `float`, **a loop
over `float` is eligible only under approximate precision** (below). Integer
kernels are exact and must match the CPU bit for bit (ADR 0958).

Bounds checks stay: an out-of-range index writes a flag word that the host
returns, and the program traps with the rule ID it would have on the CPU, naming
the iteration. A kernel never silently reads or writes out of range.

## Precision and determinism

Exactness is part of the program, not of the device ([automatic-parallelism.md
§6.2](automatic-parallelism.md), ADR 0247). So:

- Integer kernels and `f32` kernels using only `+ - * /` and comparisons, with no
  fast-math and one fixed association, are bit-exact against the CPU and may be
  offloaded without being asked.
- Anything else (`f64` narrowed to `f32`, `sin`/`exp` from the device's library,
  reductions with a different order) is offloaded only when the call asks:
  `precision: approx`, or an `f32` tensor program, as in §6.2. `--deterministic`
  denies `accel`.
- Every offload decision is reported by `ail explain` and by the layer 1 report
  with the reason ("moved 48 MB, 2.1 ms; CPU estimate 9 ms").

## Offload policy ("never force more than is efficient")

Per candidate loop or op, the runtime compares estimates, measured once per
device and cached: `t_gpu = launch + bytes_moved / bandwidth + work / throughput`
against `t_cpu = work / (cores_used * per_core_throughput)`, where the CPU side
already uses as many cores as help (`conc.par.9`). GPU only when `t_gpu` is
smaller by a margin (default 1.5x), and the data is not so small that launch
dominates. Small or branchy work stays on the CPU; a program on a machine with
no usable device behaves exactly as today. Shared mappings make `bytes_moved`
near zero on Apple silicon, so the margin is met earlier there than on a
discrete GPU.

## Rendering on the same device host

The device host also draws: sprite and rect batches (`std.gfx.13`) are
instanced draws into a `CAMetalLayer` that the app host shows, fed from the same
shared command buffer layout as `Frame`. That path replaces the CPU surface for
games while keeping `host.pixels` as the headless oracle (a software rasteriser
the GPU result is compared against within a tolerance in tests).

## Plan

1. **Layer 1**: `ail effects --gpu` and its reason codes (spec `std.gpu.*`,
   headless tests, `ail explain` entries).
2. **Layer 2a**: kernel IR from MIR for the output-only loop shape; MSL emitter;
   golden MSL for mandelbrot, a stencil, and an `f32` elementwise chain.
3. **Layer 3a**: `tools/gpu-host/macos`, buffers by shared mapping, `dispatch`;
   an `std.gpu` low-level module (`gpu.kernel(msl)`, `gpu.buffer`, `gpu.run`) so
   agents and tests can run a hand-written kernel before automatic offload exists.
4. **Layer 4**: automatic offload of the mandelbrot shape on the Mac, with the cost
   model and the report; benchmark against the CPU path (`tools/perfbench`).
5. **Tensor ops and `matmul`**; reductions.
6. **SPIR-V emitter and Vulkan** (Linux, Windows) through FFI.
7. Render batches (above), then NPUs through Core ML and ONNX (ADR 0381).

Each step ships with rule IDs in `docs/spec/std-gpu.md`, headless tests, an
`ail explain` entry, and a timing table in `tools/bench`.

## Rejected alternatives

- **`objc_msgSend` through FFI for Metal.** One symbol, many signatures; the FFI
  would need per-call-site typing and ObjC block/selector support first.
- **MPS / MPSGraph / Core ML as the compute path.** Closed kernels (ADR 0381
  rejects vendor kernel libraries); only for the NPU step.
- **Always offload eligible loops.** Violates "never force more than is
  efficient"; launch and transfer cost make small loops slower.
- **Silent `f64` to `f32`.** Changes results without being asked (ADR 0026).
- **A new keyword or dialect for GPU code.** `par for` already says
  "independent iterations"; eligibility is analysis, not syntax.

## Open points for the designer

- Helper process for Metal first (recommended) versus waiting for Objective-C
  FFI support. The helper also serves rendering, so the work is not wasted.
- Default offload margin (1.5x) and whether `precision: approx` should be a
  per-call argument or a program-level switch.
