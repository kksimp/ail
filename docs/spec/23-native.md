# 23. Native compilation

Chapter key: `native`

> **Status:** Drafted (partial). Specified: what the native subset is, how a
> program outside it is reported, what a native executable must match, and
> the `ail build` command. Implemented for Linux x86-64, Linux ARM64 and
> macOS ARM64.
> The backend's representations and the ABI are non-normative and live in
> [ir-architecture.md](../design/ir-architecture.md) and
> [native-abi.md](../design/native-abi.md).

The interpreter is the reference (ADR 0032): a native executable behaves as
`ail run` does on the same program, or the program is rejected before any
code is produced. The backend is written in AIL under `compiler/` (ADR 0022)
with its own encoders and object writers (ADR 0010).

## The native subset

**native.subset.1** — The native backend compiles programs whose every
function is a free function or a method of a struct (no generics or
contracts), in the root module or a module it imports (mod.unit.1-5), over `int`, `float`, `bool`, `str`, structs without generics or
invariants, and lists of those and of lists, using
locals, constants whose values are literals, other constants and operators
on them, arithmetic, comparisons, `and`/`or`/`not`, `if` (statement or
value), `while`, `for` over a range or a list, `break`, `continue`,
`return`, calls, struct construction, fields (`x.f`, `x.f = v`, and bare
field names in a method), method calls, `print` of numbers, strings and
lists of them or an interpolation of them (with
`{x:.N}` on a float), `xs[i]`, `xs[i] = v`, `len`, `push`, `join`,
`chars`, `env.args()`, `s.to_int() else ...` as a binding's value, and the
methods `abs`, `min`, `max`, `to_float`, `to_int`, `sqrt`, `floor`, `ceil`,
`round` (type.num.9, ADR 0591), and the bit operations of expr.bit.1. Any other
construct is reported at its position under this rule, all of them at once,
and nothing is built. *(ADRs 0384, 0390, 0591)*
and the bit operations of expr.bit.1. Any other construct is reported at its
position under this rule, all of them at once, and nothing is built.
*(ADRs 0384, 0390, 0392, 0393)*

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
may be absent. *(ADRs 0384, 0385)*

**native.build.2** — An error inside the backend itself (MIR that fails the
verifier, ir-architecture.md §4) is reported under this rule as an internal
error, never turned into code. *(ADR 0384)*

**native.build.3** — `ail build FILE [-o OUT] [--target T]` checks the
program as `ail check` does and stops on its errors, then reports
native.subset.1 problems, then writes an executable to OUT (default: FILE's
name without `.ail`, in the current directory). T is `x86_64-linux`,
`aarch64-linux` or `aarch64-macos` (a signed Mach-O); the default is the
host's. A host with no target of its own and no `--target`, or a failure to
write the file, is reported under this rule. *(ADRs 0387, 0388, 0389)*

**native.build.4** — A Windows x86-64 executable is a PE32+ image with a fixed base, four sections (`.text`, `.rdata`, `.idata`, `.bss`) and an import directory naming each imported function from its DLL. Calls to an import go through its address-table slot (ADR 0597, ADR 0598). A `libsys.NAME` call passes its arguments in Win64 positions and calls the import through its slot (ADR 0606). The writer is in `compiler/obj/pe.ail`; `ail build` does not yet select it.

```text
$ ail build examples/hello.ail
$ ./hello
Hello, world!
```
