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
