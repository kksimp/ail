# Standard library: `std.gpu`

Chapter key: `std`

> **Status:** Experimental (ADR TBD-gpu-metal-host, kksimp/ail#1599). Step 1 of
> ADR 0958: the Metal host runtime, written in AIL over the Objective-C runtime
> and the Metal framework. Only macOS has Metal. The MSL generator for `par`
> loops, the offload policy and Vulkan are later steps.

The host side of GPU compute: open the device, make buffers in shared memory,
compile Metal Shading Language (MSL) source at run time, and run a kernel.
Kernel source is a `str` for now; step 2 generates it from `par for` loops.

**std.gpu.1** — `open() Device ! GpuError` opens the system's default GPU and
makes one command queue. On a host without Metal (every target but macOS) it
fails with `GpuError` and a message that starts `std.gpu.1`.
`Device.name() str` is the device's name. *(ADR TBD-gpu-metal-host)*

**std.gpu.2** — A `Device`, `Buffer` and `Pipeline` are plain values holding
Objective-C objects; they are not freed unless the program calls `release`.
*(ADR TBD-gpu-metal-host)*

**std.gpu.3** — `Device.buffer(size) Buffer ! GpuError` makes `size` bytes of
shared memory, zeroed, that the CPU and GPU both see; a non-positive size is a
`GpuError`. `b.size()`, `b.release()`, `b.write_f32(xs)` and `b.read_f32(n)`
(single-precision, `xs` rounded as `to_f32` does), `b.write_i32(xs)` and
`b.read_i32(n)` access the memory from the start of the buffer.
*(ADR TBD-gpu-metal-host)*

**std.gpu.4** — `Device.pipeline(source, entry) Pipeline ! GpuError` compiles
the MSL `source` and makes a pipeline of its kernel `entry`. A compile error,
or a missing kernel, is a `GpuError` that starts `std.gpu.4` and carries the
Metal compiler's text. `p.release()` frees it. *(ADR TBD-gpu-metal-host)*

**std.gpu.5** — `Device.run(p, bufs, consts, n) ! GpuError` runs `p` over `n`
threads and waits for it to finish. `bufs` are bound to buffer indices
0, 1, ... in order and `consts` (32-bit integers) to the indices after them. A
kernel reads its thread number as `uint i [[thread_position_in_grid]]`. A
kernel that does not complete is a `GpuError` that starts `std.gpu.5`.
*(ADR TBD-gpu-metal-host)*

```ail rules=std.gpu.1,std.gpu.3,std.gpu.4,std.gpu.5 norun
use std.gpu

fn main() ! gpu.GpuError
    dev := gpu.open() else
        print("no metal")
        return
    src := "#include <metal_stdlib>\nusing namespace metal;\nkernel void twice(device float* a [[buffer(0)]], uint i [[thread_position_in_grid]]) \{ a[i] = a[i] * 2.0f; }"
    a := dev.buffer(16)?
    a.write_f32([1.0, 2.0, 3.0, 4.0])
    p := dev.pipeline(src, "twice")?
    dev.run(p, [a], [], 4)?
    print(a.read_f32(4))
```

**std.gpu.6** — With `--accel` (`ail run`, `ail build`), a parallel comprehension
`[f(a, b, r) par for r in 0..n]` on macOS, where `f` returns a `float` and its
body is in the subset of the MSL emitter (`compiler/msl`), may run on the GPU as one
kernel over `n` threads: `float` lists become `f32` buffers, `bytes` are copied once and
kept, a struct argument's `int` and `float` fields and the lists' lengths are
constants, and the result is the list of the kernel's results. The result then differs
from the CPU's by `f32` rounding (ADR 0247), and an `int` computation that would trap on
the CPU (overflow, division by zero) is not trapped. The loop is offloaded only when `n`
is at least 256; if Metal is missing, the kernel does not compile or the run fails, the
comprehension runs on the CPU as it was written, and no later loop retries a failed
host. `--deny accel` and `--deterministic` keep every loop on the CPU, whatever `--accel` says.
Without `--accel` nothing is offloaded. *(ADR 1319, ADR 0247, ADR 0958)*

```ail rules=std.gpu.6 norun
struct Lin
    rows int
    cols int

fn dot(w List[float], m Lin, x List[float], r int) float
    mut acc := 0.0
    for c in 0..m.cols
        acc += w[r * m.cols + c] * x[c]
    acc

fn mv(w List[float], m Lin, x List[float]) List[float]
    [dot(w, m, x, r) par for r in 0..m.rows]

fn main()
    m := Lin(rows: 2, cols: 2)
    print(mv([1.0, 2.0, 3.0, 4.0], m, [1.0, 1.0]))
```
