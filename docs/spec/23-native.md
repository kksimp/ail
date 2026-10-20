# 23. Native compilation

Chapter key: `native`

> **Status:** Drafted (partial). Specified: what the native subset is, how a
> program outside it is reported, and what a native executable must match.
> The backend's representations and the ABI are non-normative and live in
> [ir-architecture.md](../design/ir-architecture.md) and
> [native-abi.md](../design/native-abi.md).

The interpreter is the reference (ADR 0032): a native executable behaves as
`ail run` does on the same program, or the program is rejected before any
code is produced. The backend is written in AIL under `compiler/` (ADR 0022)
with its own encoders and object writers (ADR 0010).

## The native subset

**native.subset.1** — The native backend compiles programs whose every
function is a free function (no methods, generics or contracts) over
`int`, `float`, `bool`, string literals (`str` values that are literals or
copies of them) and lists of those numbers, using locals, arithmetic,
comparisons, `and`/`or`/`not`, `if` (statement or value), `while`, `for`
over a range or a list, `break`, `continue`, `return`, calls, `print` of
those values or an interpolation of them, `xs[i]`, `xs[i] = v`, `len`,
`push`, and the methods `abs`, `min`, `max`, `to_float`, `to_int`, `sqrt`,
`floor`, `ceil`. Any other construct is reported at its position under this
rule, all of them at once, and nothing is built. *(ADR 0384)*

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

**native.build.1** — A native executable's standard output and exit status
equal the interpreter's on the same program and input, and a trap's first
line on standard error is the interpreter's (`PATH:LINE:COL:
trap[RULE]: MESSAGE`, err.trap.*); the interpreter's frame lines after it
may be absent. *(ADR 0384)*

**native.build.2** — An error inside the backend itself (MIR that fails the
verifier, ir-architecture.md §4) is reported under this rule as an internal
error, never turned into code. *(ADR 0384)*
