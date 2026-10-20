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
contracts), in the root module or a module it imports (mod.unit.1-5), over
`int`, `float`, `bool`, `str`, structs without generics or invariants, and
lists of those and of lists, using locals, constants whose values are
literals, other constants and operators on them, arithmetic, comparisons,
`and`/`or`/`not`, `if` (statement or value), `while`, `for` over a range or
a list, `break`, `continue`, `return`, calls, list comprehensions, slices
`xs[a..b]`, struct construction, fields (`x.f`, `x.f = v`, and bare field
names in a method), method calls, `print` of numbers, strings and lists of
them or an interpolation of them, `xs[i]`, `xs[i] = v`, `len`, `push`,
`join`, `chars`, `env.args()`, the methods `abs`, `min`, `max`, `to_float`,
`to_int`, `sqrt`, `floor`, `ceil`, `round` (type.num.9), and the bit
operations of expr.bit.1. Also: format specifications on numbers, `bool`
and `str` in `print` (lex.str.7, type.str.11: width, fill, alignment, `.N`,
`x`); `s.to_int()` and `s.to_float()` in the `else` forms; and the calls
`fs.exists`, `fs.read_text`, `fs.write_text`, `fs.append_text`,
`fs.list_dir`, `fs.remove`, `fs.make_dir`, `fs.rename`, `env.get`,
`env.home`, `env.cwd`, `env.program`, `io.read_line`, `io.read_all`,
`io.eprint`, `io.exit`, `time.now_ms` and `time.sleep_ms` (std.fs, std.env,
std.io, std.time). A call that can fail is in the subset only as the value
of `else` with a fallback value, a statement that leaves, or a block without
a binding (err.else.1-2); `else err` and `?` are not yet. Any other
construct is reported at its position under this rule, all of them at once,
and nothing is built. *(ADRs 0384, 0390, 0392, 0393, 0532, 0580, 0582, 0591)*

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

**native.build.4** — A Windows x86-64 executable is a PE32+ image with a fixed base, four sections (`.text`, `.rdata`, `.idata`, `.bss`) and an import directory naming each imported function from its DLL. Calls to an import go through its address-table slot (ADR 0597, ADR 0598). The writer is in `compiler/obj/pe.ail`; `ail build` does not yet select it.

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

**native.bce.2** — An access `xs[i]` has no bounds check when `i` was
already used to index `xs` on every path to it, or is `xs.len()` taken
before a `push` to `xs` that is on every path to it. This holds while no
operation in the program shortens a list. *(ADR 0578)*

**native.bce.3** — A list's capacity is not observable: a native executable
may allocate room for a list ahead of the pushes that fill it. *(ADR 0579)*
