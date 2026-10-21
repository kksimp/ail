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
`int`, `float`, `bool`, `char`, `str`, structs and enums without generics or
invariants, optionals of those and of `char`s (`T?`), and lists of those and of lists, using locals, module constants
of any expression in the subset (computed once before `main` runs),
arithmetic, comparisons (`==` on enums, structs and lists compares their
values), `and`/`or`/`not`, `if` (statement or value), `while`, `for` over a
range or a list, `break`, `continue`, `return`, calls (with `mut` arguments of locals, own.param.2), list comprehensions,
slices `xs[a..b]`, struct construction, enum variants (bare or qualified),
fields (`x.f`, `x.f = v`, and bare field names in a method), method calls,
`match` as a statement or a value (variant patterns and bindings, `_`, `|`,
guards, ranges, and `int`, `bool` and `str` literals), `print` of numbers,
strings and lists of them or an interpolation of them, interpolation as a value, `xs[i]`, `xs[i] = v`,
`len`, `push`, `join`, `env.args()`, `none`, `x == none`, `if v := o`,
`o else d` and `o?` in a function returning an optional, functions that can
fail (`T ! E`, `! E`, `!`) whose errors are message-only `error struct`s
without fields (`return E()`, `f()?`, `f() else ...`, `else err`, a failing
`main`), the methods `abs`, `min`,
`max`, `to_float`, `to_int`, `sqrt`, `floor`, `ceil`, `round` (type.num.9),
the functions of type.num.16 (`sin`, `cos`, `tan`, `atan`, `atan2`, `exp`, `ln`,
`log10`, `pow`, `hypot`, `cbrt`, ADR 0592), the bit operations of expr.bit.1, and those of native.str.1, and
`Map[K, V]` and `Set[T]` over `int`, `bool` and `str` keys (`Map[K, V]()`,
`Set[T]()`, literals, `m[k]`, `m[k] = v`, `m[k] op= v`, `m.get(k) else ...`,
`contains`, `remove`, `len`, `keys`, `values`, `push(k, x)` on a
`Map[K, List[T]]`, `s.add(x)`, `for x in s`, printing a table), and constants
that are lists of constants, `join` on lists of strings or characters, and the
list methods `extend`, `remove_at`, `insert`, `clear`, `reverse`, `pop` (as a
statement or with `else`), `contains`, `sort` and `sorted` on lists of `int`,
`float`, `bool` or `str` (not `by:` keys), in functions with at most eight
`float` parameters (any number of others). Also: format specifications on
numbers, `bool`, `char` and `str` (lex.str.7, type.str.11: width, fill,
alignment, `.N`, `x`); `s.to_int()` in an `else` form and `s.to_float()` (a `float?`, type.num.8);
and the calls `fs.exists`, `fs.read_text`, `fs.write_text`,
`fs.append_text`, `fs.list_dir`, `fs.remove`, `fs.make_dir`, `fs.rename`
(they can fail with `fs.IoError`, std.err.1), `env.get`, `env.home`, `env.cwd`, `env.program`, `io.read_line` (a `str?`),
`io.read_all`, `io.eprint`, `io.exit`, `time.now_ms`, `time.sleep_ms`, and `path.dir`,
`path.base`, `path.ext`, `path.stem` and `path.join`
(std.fs, std.env, std.io, std.time, std.path). Any other construct is reported at its position under this rule, all of them
at once, and nothing is built. *(ADRs 0384, 0390, 0392, 0393, 0565, 0566, 0532, 0533, 0537,
0568-0569, 0574-0576, 0580, 0582, 0591, 0592, 0602–0604, 0605)*

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
may be absent. Recursion is the exception: the interpreter traps after a
fixed number of nested calls, a native executable when its stack is nearly
exhausted. It traps under err.trap.1 at the call, with `call depth exceeded
the native stack in `NAME``, so it may recurse deeper than the interpreter
allows. *(ADRs 0384, 0385, 0535)*

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

**native.build.5** — `ail build` keeps the executables it makes, under a
SHA-256 key of everything that decides their bytes: the path as given, the
target, the `.ail` sources of the backend and of the program's directory tree
(a tree over 4,000 files or 32 MB is not cached), and the build of `ail`
itself. Building again with the same key writes the stored bytes to OUT and
runs nothing else; the check of native.build.3 still runs first. A build
with problems is never stored. `--no-cache` or `AIL_NO_CACHE` reads and
writes nothing. The cache is in `$AIL_CACHE_DIR`, else
`$XDG_CACHE_HOME/ail/build`, else `%LOCALAPPDATA%\ail\build` or `~/.cache/ail/build`, and keeps the 64 most
recently used entries. A cached and an uncached build are the same bytes.
*(ADR 0655)*

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

```ail rules=native.bce.7
fn main()
    xs := [1, 2, 3, 4]
    mut total := 0
    mut i := 1
    while i < 3
        total += xs[i]
        i += 1
    print(total)
```

**native.bce.8** — The check on `xs[e]` may be removed where the arithmetic of
the loop bounds `e` between 0 and a constant `hi`, from a constant loop bound
and operands tested before the loop, and `xs` is longer than `hi`. *(ADR 0622)*

```ail rules=native.bce.8
fn main()
    xs := [1, 2, 3]
    mut total := 0
    for i in 0..xs.len()
        total += xs[i] + xs[i]
    print(total)
```

**native.bce.9** — A loop that adds an unchanging `s` to its induction value
is treated as a counted loop where `s` is tested, before the loop, to be
between 1 and 2^30. *(ADR 0622)*

```ail rules=native.bce.9
fn main()
    mut total := 0
    mut s := 2
    for i in 0..4
        total += i + s
    print(total)
```
**native.test.1** — `ail test --native [PATH...]` takes files and directories
(by default `examples`, `bootstrap/tests/cases` and `tools`) and, for every
`@test` function with no parameters in a file that has one, builds the
function natively (native.build.3), runs it as its own process, and compares
the result with `ail test` in the interpreter. A passing native run exits 0.
*(ADR 0679)*
**native.test.2** — The native build is of a rewritten copy of the file: `@test`
marks go, `assert(c)` and `assert_eq(a, b)` become a check of `c` and of
`(a) == (b)` that prints the original line and exits with status 1, and the
copy's `main` is a dispatcher. The rewrite keeps every line number, so a
construct the backend refuses is reported at its place in the original file.
*(ADR 0679)*
**native.test.3** — A test the backend refuses (native.subset.1) is reported
as not in the native subset yet, with the first blocking construct and its
line. The same holds for a test whose file has a refused construct outside any
test; the report says so. A property test, which takes arguments, is skipped.
*(ADR 0679)*
**native.test.4** — Each test is `match` (native and interpreter agree on pass
or fail), `DIFF` (they disagree), `not-native` or `skipped`, and the command
ends with the counts. The exit status is 1 if any test is `DIFF`, otherwise 0.
`--json` writes one object per test and a final summary. *(ADR 0679)*

```ail rules=native.bce.9
fn main()
    mut total := 0
    mut s := 2
    for i in 0..4
        total += i + s
    print(total)
```
**native.test.5** — A file inside a package (a directory under an `ail.pkg`) is
built in a staging directory that holds the package's other module directories
and the rewritten copy as the root module's only file, so its `use` lines
resolve. A package that has a dependency (`dep NAME path DIR`) isn't staged
yet: the dependency's own imports resolve against the wrong root, and its tests
are reported as not in the native subset yet. *(ADR 0679)*

## Checked examples for the native rules

Each block runs `ail test --native` or `ail build` with the options on its fence. The rules below have no observable output the harness can check (a Windows image, the cache, a backend fault, a package staging run), so their blocks are `norun`: checked, not run (native.build.2, native.build.4, native.build.5, native.test.5).

```ail tool=test,--native,FILE expect=match rules=native.str.1,native.test.1
@test fn strings()
    s := "hello"
    assert(s.len() == 5)
    assert("{3:>4}" == "   3")
    assert("ab" < "b")

fn main()
    print("ok")
```

```ail tool=test,--native,FILE expect=assertion rules=native.test.2
@test fn wrong()
    assert(1 == 2)

fn main()
    print("ok")
```

```ail tool=test,--native,FILE expect=not-native rules=native.test.3
fn first[T](xs List[T]) T
    xs[0]

@test fn picks()
    assert(first([1, 2]) == 1)

fn main()
    print("ok")
```

```ail tool=test,--native,--json,FILE expect=summary rules=native.test.4
@test fn strings()
    s := "hello"
    assert(s.len() == 5)
    assert("{3:>4}" == "   3")
    assert("ab" < "b")

fn main()
    print("ok")
```

```ail tool=build,FILE status=1 expect=subset rules=native.subset.1
fn first[T](xs List[T]) T
    xs[0]

fn main()
    print(first([1, 2]))
```

```ail norun rules=native.build.2
fn main()
    print("ok")
```

```ail norun rules=native.build.4
fn main()
    print("ok")
```

```ail norun rules=native.build.5
fn main()
    print("ok")
```

```ail norun rules=native.test.5
fn main()
    print("ok")
```
