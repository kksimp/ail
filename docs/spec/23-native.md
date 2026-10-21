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
`sorted`/`sort` on a list of `int`, `float` or `str` with a key function of an `int` key (expr.key.1-4), or without one on a list of `int`, and `unreachable(message)` (err.model.2), and function values (type.fn.1-3): a declared function named as a value, a `fn(...)` parameter, result or local, and calls through one. Any other construct is reported at its
position under this rule, all of them at once, and nothing is built.
*(ADRs 0384, 0390, 0392, 0393)*
and the bit operations of expr.bit.1. Also: format specifications on numbers,
`bool` and `str` in `print` (lex.str.7, type.str.11: width, fill, alignment,
`.N`, `x`); `s.to_float()` in the `else` forms; and the calls `fs.exists`,
`fs.read_text`, `fs.write_text`, `fs.append_text`, `fs.list_dir`, `fs.walk`, `fs.glob`, `fs.copy`,
`fs.remove`, `fs.make_dir`, `fs.rename`, `env.get`, `env.home`, `env.cwd`,
`env.program`, `io.read_line`, `io.read_all`, `io.eprint`, `io.exit`,
`time.now_ms` and `time.sleep_ms` (std.fs, std.env, std.io, std.time). A
call that can fail is in the subset only as the value of `else` with a
fallback value, a statement that leaves, or a block without a binding
(err.else.1-2); `else err` and `?` are not yet. Also `sorted`/`sort` on a list of `int`, `float` or `str` with a key function of an `int` key (expr.key.1-4), or without one on a list of `int`; `unreachable(message)` (err.model.2); and function values (type.fn.1-3): a declared function named as a value, a `fn(...)` parameter, result or local, and calls through one. Any other construct is
reported at its position under this rule, all of them at once, and nothing
is built. *(ADRs 0384, 0390, 0392, 0393, 0571-0573, 0580, 0582)*
function is a free function or a method of a struct or enum (no generics or
contracts), in the root module or a module it imports (mod.unit.1-5), over
`int`, `float`, `bool`, `char`, `str`, structs and enums without generics or
invariants, optionals of those and of `char`s (`T?`), and lists of those and of lists, using locals, module constants
of any expression in the subset (computed once before `main` runs),
arithmetic, comparisons (`==` on enums, structs and lists compares their
values), `and`/`or`/`not`, `if` (statement or value), `while`, `for` over a
range or a list, `break`, `continue`, `return`, calls, list comprehensions,
slices `xs[a..b]`, struct construction, enum variants (bare or qualified),
fields (`x.f`, `x.f = v`, and bare field names in a method), method calls,
`match` as a statement or a value (variant patterns and bindings, `_`, `|`,
guards, ranges, and `int`, `bool` and `str` literals), `print` of numbers,
strings and lists of them or an interpolation of them, interpolation as a value, `xs[i]`, `xs[i] = v`,
`len`, `push`, `join`, `env.args()`, `none`, `x == none`, `if v := o`,
`o else d` and `o?` in a function returning an optional (and a `?` chain such as `a?.b.c else d`, err.prop.4), `@allow` and `@deprecated` on functions, functions that can
fail (`T ! E`, `! E`, `!`) whose errors are `error struct`s and `error enum`s
with fields and messages (`return E(...)`, `f()?`, `f() else ...`, `else err`,
typed when the function names one error type, a failing
`main`), the methods `abs`, `min`,
`max`, `to_float`, `to_int`, `sqrt`, `floor`, `ceil`, `round` (type.num.9),
the functions of type.num.16 (`sin`, `cos`, `tan`, `atan`, `atan2`, `exp`, `ln`,
`log10`, `pow`, `hypot`, `cbrt`, ADR 0592), the bit operations of expr.bit.1, and those of native.str.1, and
`Map[K, V]` and `Set[T]` over `int`, `bool`, `char` and `str` keys (`Map[K, V]()`,
`Map[K, V]` and `Set[T]` over `int`, `bool` and `str` keys, with values that
include structs and enums (`Map[K, V]()`,
`Set[T]()`, literals, `m[k]`, `m[k] = v`, `m[k] op= v`, `m.get(k) else ...`,
`Map[K, V]` and `Set[T]` over `int`, `bool` and `str` keys, with values of
any type in the subset, enums included (`Map[K, V]()`,
`Set[T]()`, literals, `m[k]`, `m[k] = v`, `m[k] op= v`, `m.get(k)` (a `V?`, type.map.6) and `m.get(k) else ...`,
`contains`, `remove`, `len`, `keys`, `values`, `push(k, x)` on a
`Map[K, List[T]]`, `s.add(x)`, `for x in s`, printing a table; a local declared from `{}` is typed by its first `m[k] = v`, `m[k] op= v`, `s.add(x)` or `m = e`, and one from `[]` by `xs.push(x)` or `xs = e`, type.coll.13), and constants
that are lists of constants, `join` on lists of strings or characters, and the
list methods `extend`, `remove_at`, `insert`, `clear`, `reverse`, `pop` (as a
statement or with `else`), `contains`, `sort` and `sorted` on lists of `int`,
`float`, `bool` or `str` (not `by:` keys), `swap`, `take` (type.coll.15), `sum`, `min`, `max`, `first`, `last`
and `index_of`, `sorted` on a set of `int` or `str`, `union`, `intersect` and `difference` on
sets and lists of `int` or `str`, `top` on a `Map[K, int]` (`int` or `str` keys), and set and map comprehensions
whose type is expected (a function result, argument or field), in functions with at most eight
`float` parameters (any number of others). Also: format specifications on
numbers, `bool`, `char` and `str` (lex.str.7, type.str.11: width, fill,
alignment, `.N`, `x`); `s.to_int() else ...` (as a binding or inside any expression) and `s.to_float()` (a `float?`, type.num.8);
and the calls `fs.exists`, `fs.read_text`, `fs.write_text`,
`fs.append_text`, `fs.list_dir`, `fs.walk`, `fs.glob`, `fs.copy`, `fs.remove`, `fs.make_dir`, `fs.rename`
(`fs.walk` and the others can fail with `fs.IoError`, std.err.1), `env.get`, `env.home`, `env.cwd`, `env.program`, `io.read_line` (a `str?`),
`io.read_all`, `io.eprint`, `io.exit`, `time.now_ms`, `time.sleep_ms`, and `path.dir`,
`path.base`, `path.ext`, `path.stem` and `path.join`
(std.fs, std.env, std.io, std.time, std.path); `compress.gzip`, `zlib` and `deflate` on a `str` or `bytes`, and `compress.gunzip`, `unzlib` and `inflate` on `bytes` (std.compress.1-4, 7-8: the output is the interpreter's byte for byte, and a failure carries its message); on `bytes`, `+` and `concat`, `to_str`, `find`, `contains`, `starts_with`, `ends_with`, `base64`, `bytes.from_hex`, `bytes.from_base64` and `s.to_bytes()` (type.bytes.6, 8-13); on Linux, `proc.run` (with
`input` and `timeout_ms` left off or given, std.proc.2) and `proc.which`, whose
`Output` has `status`, `out`, `err` and the methods `ok()` and `text()`
(std.proc.1-3, std.proc.9-10; `ProcError.kind` is not
readable yet, and on macOS the program traps at `proc.run`); raw TCP (std.net.12): `net.listen`, `net.connect`, and on a
`net.Listener` `accept`, `addr`, `close`, and on a `net.Conn` `read`, `read_line`, `write`, `set_timeout`, `peer`,
`close`, failing with `net.NetError` (std.net.4), and `net.get` and `net.post` over `http://` (std.net.2, std.net.3: one connection
per request with `Connection: close`, `Content-Length`, chunked or until-close bodies, up to five redirects of a `GET`; an `https://` URL
fails with kind `tls`, and `Accept-Encoding` and connection pooling are not built), `net.request` and `net.options()` (std.net.6, std.net.19: `headers`, `query`, `body`, `timeout_ms`, `max_redirects`, `connect_ms`; `total_ms` is checked but not enforced and `data` is not built); only IPv4 (`A.B.C.D:PORT`, `localhost`, or a name in `/etc/hosts`; any
other host fails with kind `not_found`, as there is no DNS client yet), no TLS, a wait does not let other tasks run, the
`bytes` forms are not built, and a message's wording may differ from the interpreter's (the `kind` is the same); and the standard
modules written in AIL (`std.args`, `std.csv`, `std.log`, `std.net.http`,
`std.net.jar`, `std.regex`, `std.time.date`), which build like a module of the program when
the constructs their source uses are in the subset (so far `std.log` and `std.regex`; `regex.captures` is not built yet). Any other construct is reported at its position under this rule, all of them
at once, and nothing is built. *(ADRs 0384, 0390, 0392, 0393, 0565, 0566, 0532, 0533,
0568-0569, 0574-0576, 0580, 0582, 0591, 0592, 0602–0604, 0605, 0770-0772, 0802-0806, 0818)*
0568-0569, 0574-0576, 0580, 0582, 0591, 0592, 0602–0604, 0605, 0770-0772, 0802-0806, 0830)*

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
width, fill and alignment (type.str.11), of a `List[int]` as `[1, 2]`; `==`, `!=`, `<`, `<=`, `>`, `>=` on
`str` and on `char`; on `str`, `len`, `byte_len`, `chars` (a `List[char]`),
`lines`, `trim`, `trim_start`, `trim_end`, `words`, `to_upper`, `to_lower`, `to_str`, `contains`,
`starts_with`, `ends_with` and `split` (each taking a `str` or a `char`),
`slice`, `replace`, `repeat`, `pad_left`, `pad_right`, `center`, `count`; on `char`, `code`,
`to_str`, `is_digit`, `is_alpha`, `is_alnum`, `is_space`, `is_upper`,
`is_lower`, `to_upper`, `to_lower`; `to_str` on `int`, `float` and `bool`;
`print` of a `char`, a `List[char]` and a `List[str]`. The methods with an
optional result are `find`, `rfind`, `char_at`, `strip_prefix`, `strip_suffix`,
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
before a `push` to `xs` that is on every path to it, or is a constant, or a
variable that only ever holds constants, below the number of `push` calls to
`xs` that are on every path to it. This holds while no operation in the
program shortens a list. *(ADR 0578, ADR 0816)*

```ail rules=native.bce.2
fn main()
    xs := [4, 5]
    a := xs[1]
    print(a + xs[1])
```

**native.bce.3** — A list's capacity is not observable: a native executable
may allocate room for a list ahead of the pushes that fill it, at most 8 MiB
of elements (2^20 elements of 8 bytes, 2^23 for a `List[bool]`) per list. A
list that fills its first block also grows straight to 16 elements before it
doubles. *(ADR 0579, ADR 0816, ADR 0562)*

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

**native.reuse.1** — A list made empty inside a loop may be one object that
each iteration empties (length 0, capacity kept) instead of a new object per
iteration, where no reference to the list can outlive the iteration: it is
not stored, returned, passed to a function value, or passed to a function
whose parameter can keep it, and it isn't carried to the next iteration. A
list bound, assigned or stored from a variable is copied (own.val.1), so the
copy, not the reused list, is what's kept. *(ADR 0824)*

```ail rules=native.reuse.1
fn main()
    mut rows := List[List[int]]()
    for i in 0..3
        mut row := []
        row.push(i)
        rows.push(row)
    print(rows)
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
