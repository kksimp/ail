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
function is a free function or a method of a struct or enum (no generics or
contracts), in the root module or a module it imports (mod.unit.1-5), over
`int`, `float`, `bool`, `str`, structs and enums without generics or
invariants, optionals of those (`T?`), and lists of those and of lists, using locals, module constants
of any expression in the subset (computed once before `main` runs),
arithmetic, comparisons (`==` on enums, structs and lists compares their
values), `and`/`or`/`not`, `if` (statement or value), `while`, `for` over a
range or a list, `break`, `continue`, `return`, calls, list comprehensions,
slices `xs[a..b]`, struct construction, enum variants (bare or qualified),
fields (`x.f`, `x.f = v`, and bare field names in a method), method calls,
`match` as a statement or a value (variant patterns and bindings, `_`, `|`,
guards, ranges, and `int`, `bool` and `str` literals), `print` of numbers,
strings and lists of them or an interpolation of them, `xs[i]`, `xs[i] = v`,
`len`, `push`, `join`, `chars`, `env.args()`, `none`, `x == none`, `if v := o`,
`o else d` and `o?` in a function returning an optional, functions that can
fail (`T ! E`, `! E`, `!`) whose errors are message-only `error struct`s
without fields (`return E()`, `f()?`, `f() else ...`, `else err`, a failing
`main`), the methods `abs`, `min`,
`max`, `to_float`, `to_int`, `sqrt`, `floor`, `ceil`, `round` (type.num.9),
the functions of type.num.16 (`sin`, `cos`, `tan`, `atan`, `atan2`, `exp`, `ln`,
`log10`, `pow`, `hypot`, `cbrt`, ADR 0592), and the bit operations of expr.bit.1, in functions with at most eight
`float` parameters (any number of others). Also: format specifications on
numbers, `bool` and `str` in `print` (lex.str.7, type.str.11: width, fill,
alignment, `.N`, `x`); `s.to_int()` in an `else` form and `s.to_float()` (a `float?`, type.num.8);
and the calls `fs.exists`, `fs.read_text`, `fs.write_text`,
`fs.append_text`, `fs.list_dir`, `fs.remove`, `fs.make_dir`, `fs.rename`
(they can fail with `fs.IoError`, std.err.1), `env.get`, `env.home`, `env.cwd`, `env.program`, `io.read_line` (a `str?`),
`io.read_all`, `io.eprint`, `io.exit`, `time.now_ms` and `time.sleep_ms`
(std.fs, std.env, std.io, std.time). Any other construct is reported at its position under this rule, all of them
at once, and nothing is built. *(ADRs 0384, 0390, 0392, 0393, 0532, 0533,
0574-0576, 0580, 0582, 0591, 0592, 0602–0604)*

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
name without `.ail`, in the current directory). T is `x86_64-linux`,
`aarch64-linux` or `aarch64-macos` (a signed Mach-O); the default is the
host's. A host with no target of its own and no `--target`, or a failure to
write the file, is reported under this rule. *(ADRs 0387, 0388, 0389)*

```ail rules=native.build.3
fn main()
    print("built")
```

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
before a `push` to `xs` that is on every path to it. This holds while no
operation in the program shortens a list. *(ADR 0578)*

```ail rules=native.bce.2
fn main()
    xs := [4, 5]
    a := xs[1]
    print(a + xs[1])
```

**native.bce.3** — A list's capacity is not observable: a native executable
may allocate room for a list ahead of the pushes that fill it. *(ADR 0579)*

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

**native.bce.8** — The check on `xs[e]` may be removed where the arithmetic of
the loop bounds `e` between 0 and a constant `hi`, from a constant loop bound
and operands tested before the loop, and `xs` is longer than `hi`. *(ADR 0622)*

**native.bce.9** — A loop that adds an unchanging `s` to its induction value
is treated as a counted loop where `s` is tested, before the loop, to be
between 1 and 2^30. *(ADR 0622)*
