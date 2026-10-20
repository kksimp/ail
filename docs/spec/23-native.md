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
`int`, `float`, `bool`, `char`, `str`, structs without generics or
invariants, and lists of those and of lists, using locals, constants whose
values are literals, other constants and operators on them, arithmetic,
comparisons, `and`/`or`/`not`, `if` (statement or value), `while`, `for` over
a range or a list, `break`, `continue`, `return`, calls, struct construction,
fields (`x.f`, `x.f = v`, and bare field names in a method), method calls,
`print` of numbers, strings and lists of them or an interpolation of them,
interpolation as a value, format specifications on numbers, `bool`, `char`
and `str` (lex.str.7, type.str.11: width, fill, alignment, `.N`, `x`),
`xs[i]`, `xs[i] = v`, `len`, `push`, `join`, `env.args()`, `s.to_int() else
...` as a binding's value, optionals of numbers, `char`s, strings and lists (`none`,
`x == none`, `if v := o`, `o else d`, `o?` in a function returning an optional),
functions that can fail (`T ! E`, `! E`, `!`) whose errors are message-only
`error struct`s without fields (`return E()`, `f()?`, `f() else ...`,
`else err`, a failing `main`), the methods `abs`, `min`, `max`, `to_float`,
`to_int`, `sqrt`, `floor`, `ceil`, `round` (type.num.9, ADR 0591), the bit
operations of expr.bit.1, and those of native.str.1. Also `s.to_float()` in
the `else` forms, and the calls `fs.exists`, `fs.read_text`, `fs.write_text`,
`fs.append_text`, `fs.list_dir`, `fs.remove`, `fs.make_dir`, `fs.rename`,
`env.get`, `env.home`, `env.cwd`, `env.program`, `io.read_line`,
`io.read_all`, `io.eprint`, `io.exit`, `time.now_ms` and `time.sleep_ms`
(std.fs, std.env, std.io, std.time). A call that can fail is in the subset
only as the value of `else` with a fallback value, a statement that leaves,
or a block without a binding (err.else.1-2); `else err` and `?` are not yet.
Any other construct is reported at its position under this rule, all of them
at once, and nothing is built. *(ADRs 0384, 0390, 0392, 0393, 0565, 0566,
0574, 0575, 0576, 0580, 0582, 0591)*

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

**native.str.1** — In a native program `str` and `char` behave as
type.str.3-14 and type.char.1-4 say, with the interpreter's results and
traps. The subset has: interpolation as a value, with `{x:.N}`, `{x:x}`,
width, fill and alignment (type.str.11); `==`, `!=`, `<`, `<=`, `>`, `>=` on
`str` and on `char`; on `str`, `len`, `byte_len`, `chars` (a `List[char]`),
`lines`, `trim`, `to_upper`, `to_lower`, `to_str`, `contains`,
`starts_with`, `ends_with` and `split` (each taking a `str` or a `char`),
`slice`, `replace`, `repeat`, `pad_left`, `pad_right`; on `char`, `code`,
`to_str`, `is_digit`, `is_alpha`, `is_alnum`, `is_space`, `is_upper`,
`is_lower`, `to_upper`, `to_lower`; `to_str` on `int`, `float` and `bool`;
`print` of a `char`, a `List[char]` and a `List[str]`. The methods with an
optional result are `find`, `char_at`, `strip_prefix`, `strip_suffix`,
`to_digit` and `char.from_code`, used with the optional forms of
native.subset.1.
The character predicates and case mappings, and `to_upper` and `to_lower`
on a `str`, are those of the interpreter on every scalar value, read from
tables generated from it (a capital sigma becomes the final form by the
Unicode rule, with its case-ignorable set approximated by the apostrophes,
dots, accents and combining marks). *(ADRs 0565, 0566, 0567)*

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
