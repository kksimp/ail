# Standard library: GPU compute

Chapter key: `std`

> **Status:** Drafted. Only the eligibility report (layer 1 of ADR 0995) is
> implemented; kernels, device hosts and offload are planned and have no rule
> IDs until they ship. Design: [gpu-compute.md](../design/gpu-compute.md).

**std.gpu.1** — `ail effects --gpu FILE` prints one line per `for` loop in
source order: `FILE:LINE:COL ID for VAR: VERDICT (RULE: REASON)`, with ` work N`
(the operations one iteration costs, `conc.par.14`) after an `eligible` loop. A loop
is `eligible` when its iterations are independent (`conc.par.2-4`), it iterates a
range `a..b`, and its body has only scalar arithmetic, `if`, element reads and writes
and loops, with no call, field access, `match`, string or collection construction. It
reports what could become a kernel; no program runs differently because of it.
With `--json` it prints `{"ail_api":1,"loops":[{"at","fn","var","verdict","rule","reason","work"},...]}`. *(ADR 0995)*

**std.gpu.2** — Every other loop has verdict `cpu`, and the reason names the first
thing that keeps it there: the loop is sequential (with the `conc.par` rule that
says so), it iterates a collection, its body has a construct the parallel scan
refuses, or it calls a function or method (the report names the callee and says to
inline it). *(ADR 0995)*

```ail tool=effects,--gpu,FILE expect=eligible rules=std.gpu.1
fn main()
    mut out := [0, 0, 0, 0]
    par for i in 0..4
        out[i] = i * i + 1
    print(out[3])
```

```ail tool=effects,--gpu,FILE expect=cpu rules=std.gpu.2
fn main()
    mut total := 0
    for i in 0..10
        total = total + i
    print(total)
```
