# 04. Types

Chapter key: `type`

> **Status:** Drafted (partial). Specified: numeric types (integers and
> floats), structs, enums, optional types, strings, characters, the standard
> collections, maps and sets, and grids. Not yet designed: fixed-size arrays, SIMD
> vector types, and const generics.

## Numeric types

**type.num.1** — The signed integer types are `i8`, `i16`, `i32` and `i64`.
The unsigned integer types are `u8`, `u16`, `u32` and `u64`. The
floating-point types are `f32` and `f64`. *(ADR 0055)*

**type.num.2** — `int` is an alias for `i64`, and `float` is an alias for
`f64`. *(ADR 0055)*

**type.num.3** — An integer literal with no other type constraint has type
`int`. A floating-point literal with no other type constraint has type
`float`. *(ADR 0055)*

**type.num.4** — `int` and `i64` are both canonical spellings, as are `float`
and `f64`. The formatter MUST NOT rewrite one into the other. *(ADR 0059)*

**type.num.5** — Integer types support `+ - * / %`, unary `-` and the
comparisons `== != < <= > >=`. Sizes, lengths and indexes have type `int`
(this is the size and index type of `plat.ptr.2`). *(ADR 0116)*

**type.num.6** — `float` (`f64`) supports `+ - * /`, unary `-`, `+= -= *= /=`
and `== != < <= > >=`, all as IEEE 754 binary64 (round to nearest, ties to
even). It has no `%` or `%=`. Nothing traps: overflow gives `inf` or `-inf`,
and `x / 0.0` gives `inf`, `-inf` or `nan`. `nan` is unequal to everything,
itself included. Every `<`, `<=`, `>` and `>=` with a `nan` operand is false,
and `-0.0 == 0.0`. These operators are IEEE wherever they are written,
including in generic code through an `Eq` or `Ord` bound (an amendment of
`trait.std.1–2` for `float` only). There is no implicit conversion between
`int` and `float` (`trait.op.3`): write `n.to_float()` or `x.to_int()`
(`type.num.9`). `float` implements `Eq`, `Ord` and `ToStr`. It isn't `Hash`
(`trait.auto.5`). *(ADR 0130)*

```ail fragment
a := 0.1 + 0.2          # 0.30000000000000004
b := 1.0 / 0.0          # inf: no trap
c := 0.0 / 0.0          # nan; c == c is false
```

**type.num.7** — The text form of a `float` (`ToStr`, interpolation, and
inside other values) is Python's `repr`. The digits are the shortest decimal
that reads back as the same value; if several are equally short, the one
closest to the exact value. Write the value as d.ddd × 10^e. If
`-4 <= e < 16`, use fixed notation with at least one digit after the `.`.
Otherwise use `d[.ddd]e±XX`: the sign is always written, the exponent has at
least two digits, and the mantissa has no `.0`. The special forms are `inf`,
`-inf`, `nan` (never `-nan`) and `-0.0`. *(ADR 0130)*

```ail fragment
"{1000.0}"              # 1000.0
"{0.0001}"              # 0.0001
"{-0.0}"                # -0.0
"{1e16}"                # 1e+16
"{0.00001}"             # 1e-05
"{1.5e300}"             # 1.5e+300
"{5e-324}"              # 5e-324
"{123456789012345678.0}"  # 1.2345678901234568e+17
```

**type.num.8** — `s.to_float()` returns a `float?`. `s` MUST be an optional
`-`, then digits, then an optional `.` and digits, then an optional exponent
(`e` or `E`, an optional sign, digits); or exactly `inf`, `-inf` or `nan`, so
every text form of `type.num.7` reads back. A `+`, a `_`, spaces, and a leading
or trailing `.` make the result `none`. The result is correctly rounded
(nearest, ties to even). Overflow gives `inf` or `-inf` (as Python does), and
underflow gives a subnormal or `±0.0`; `"-0"` gives `-0.0`. Float literals
are correctly rounded too; a literal that overflows is a `lex.num.3` error.
*(ADR 0130)*

```ail fragment
print("2.5E-3".to_float())    # 0.0025
print("1e999".to_float())     # inf
print("-0".to_float())        # -0.0
print("+1".to_float())        # none
print(".5".to_float())        # none
```

**type.num.9** — `x.abs()`, `x.floor()`, `x.ceil()`, `x.round()` and
`x.sqrt()` return a `float`. `round` rounds halves away from zero (Rust, Go
and Swift; Python's `round(2.5)` is 2). `floor`, `ceil`, `round` and `sqrt`
keep the sign of zero, so `(-0.4).round()` is `-0.0`. `sqrt` of a negative
number is `nan`. `x.to_int()` returns an `int`: it truncates toward zero, and
traps on `nan` or when the truncated value is outside `int`. `n.to_float()`
rounds to nearest, ties to even, and never traps. *(ADR 0130)*

**type.num.16** — `x.sin()`, `x.cos()`, `x.tan()`, `x.atan()`, `x.exp()`,
`x.ln()` (the natural logarithm), `x.log10()` and `x.cbrt()` return a
`float`; so do `x.atan2(y)`, `x.pow(y)` and `x.hypot(y)` for a `float` `y`
(`x` is the first argument: `atan2(x, y)` is the angle of `(y, x)`, as in
Rust, and `pow` is `x` raised to `y`). Special arguments give the IEEE 754
and C99 Annex F results: `nan` for `sin` of an infinity and for `ln` of a
negative number; `pow(1, y)` and `pow(x, 0)` are `1` even for `nan`; `atan2`
keeps the sign of zero; `hypot` of an infinity is infinite even beside a
`nan`. For finite arguments the result is within one unit in the last place
of the platform's libm, except that `log10` may differ by two units; the
platforms' libms may differ from each other by the same bound, so the last
bit of these results isn't fixed across operating systems (as for
std.tensor.13). The native backend meets the same bound. *(ADR 0592)*

**type.num.10** — An integer literal whose expected type is `float` is a
float literal. Only literals (and `-` applied to a literal) convert: an
expression of type `int` never does. *(ADR 0130)*

```ail skip=context
struct Stats
    total float

s := Stats(total: 0)    # 0 is the float 0.0
mut t := 0.0
t += 1                  # 1 is the float 1.0
y := t * 2              # 2 is the float 2.0
z := t * n              # compile-time error if n is an int: write n.to_float()
```

**type.num.11** — `cmp` on `float` is a total order: numbers compare in
numeric order, `-0.0` is `Equal` to `0.0`, and `nan` is `Equal` to `nan` and
`Greater` than every number (`nan` sorts last). `sort`, `sorted`, `min`, `max`
(`type.coll.11`), and the ordering of composite types (`List[float]`, and
derived `Ord` on a struct with a `float` field) use this `cmp`, so their
results are deterministic. For `float` alone, `a.cmp(b) == Equal` doesn't
imply `a == b`. *(ADR 0130)*

```ail fragment
n := 0.0 / 0.0
print([2.0, n, 1.0].sorted())  # [1.0, 2.0, nan]
print(n < 1.0)                 # false: the operator is IEEE
```

> **Note:** float literal patterns are an error (`expr.match.7`). The same
> holds for `f32`.

**type.num.12** — `int` has these methods. *(ADR 0137)*
- `checked_add`, `checked_sub`, `checked_mul`, `checked_div`, `checked_rem`
  and `checked_neg` return an `int?`: `none` on overflow or a zero divisor.
  (`MIN % -1` doesn't overflow: it is `0`, `expr.arith.5`.)
- `wrapping_add`, `wrapping_sub`, `wrapping_mul` and `wrapping_neg` wrap in
  two's complement and return an `int`.
- `saturating_add`, `saturating_sub` and `saturating_mul` clamp to the
  minimum or maximum `int` and return an `int`.
- `abs()` traps on the minimum `int`.
- `to_float()` (`type.num.9`) and `to_hex()` (`type.str.8`).

These satisfy `expr.arith.2` for addition, subtraction, multiplication and
negation. Division and remainder have the checked forms only; wrapping and
saturating division are deferred. `a.min(b)` and `a.max(b)` of two `int`
values are the default methods of `Ord` (ADR 0137).

```ail fragment
x := 9223372036854775807
print(x.checked_add(1))        # none
print(x.wrapping_add(1))       # -9223372036854775808
print(x.saturating_add(1))     # 9223372036854775807
```

**type.num.13** — A sized number is a value of one of the types of
`type.num.1` other than `int` and `float`. Its literals are typed by context: an
integer literal whose expected type is a sized integer type (a parameter, a
field, a return value, or the other operand of an operator, written on either
side) has that type; it MUST fit, or the program is rejected. A `-` applied to
a literal belongs to the literal, so `-128` is an `i8`. An integer or float
literal whose expected type is `f32` is an `f32`. With no expected type a
literal is an `int` or a `float` (`type.num.3`). *(ADR 0194)*

**type.num.14** — Arithmetic, unary `-` and comparison on sized numbers are
defined only between two values of the same type; there is no implicit
widening or conversion (`trait.op.3`). The sized integers support what `int`
does (`type.num.5`), and a result outside the type traps (`expr.arith.1`,
`expr.arith.3`); unary `-` isn't defined for the unsigned types. `f32` follows
`type.num.6` in single precision and has no `%`. Every sized number is `Eq` and
`Ord`; every one but `f32` is `Hash`. A sized integer has the methods of
`type.num.12` (`checked_*`, `wrapping_*`, `saturating_*`, `abs`, `to_hex`,
computed in the type; `checked_neg`, `wrapping_neg` and `abs` for the signed
types only), and `f32` has `abs`, `floor`, `ceil`, `round` and `sqrt` in
single precision. Text forms follow `type.num.7` (an `f32` uses the shortest
digits that read back as an `f32`). *(ADR 0194)*

**type.num.15** — Conversions are explicit methods named for the target:
`to_i8`, `to_i16`, `to_i32`, `to_u8`, `to_u16`, `to_u32`, `to_u64` and `to_f32`
on any integer type (including `int`), `to_f32` on `float`, and `to_int` and
`to_float` back to `int` and `float` from any sized number. An integer
conversion is exact and traps (`type.num.15`) when the value doesn't fit; a
float conversion rounds to nearest, ties to even, and never traps, except
`to_int` of an `f32`, which is `type.num.9`'s. *(ADR 0195)*

```ail fragment
a := 300.to_u16()       # a u16
b := a.to_u32() * 2     # a u32: 600
c := 0.5.to_f32() + 1   # an f32: 1 is a float literal
```

## Structs

**type.struct.1** — A struct type is declared with the keyword `struct`, its
name, and an indented block containing one field declaration per line.
*(ADR 0052)*

**type.struct.2** — A field declaration is written `name Type`. *(ADR 0052,
ADR 0037)*

```ail fragment
struct User
    id int
    name str
    email str?
```

**type.struct.3** — A struct value is constructed by the type name followed
by a parenthesised, comma-separated list of arguments. An argument is named
(`field: value`), shorthand (a bare local name equal to a field's name, which
sets that field), or positional (any other expression). *(ADR 0058,
ADR 0142)*

```ail skip=context
u := User(id: 7, name: "Ada", email: none)
```

**type.struct.4** — A struct literal MUST set every field of the struct
that has no default (`type.struct.8`) exactly once, and no field twice. Named and shorthand arguments may come in any order. Missing,
unknown and repeated fields are compile-time errors. *(ADR 0113, ADR 0142)*

**type.struct.5** — A struct's field names MUST be distinct. Each value in a
struct literal MUST have its field's type. *(ADR 0117)*

**type.struct.6** — `e.f` reads field `f` of `e`. The type of `e` MUST be a
struct that declares `f`. *(ADR 0117)*

**type.struct.7** — Positional arguments fill the fields not set by named or
shorthand arguments, in declaration order. They are allowed only when the
fields they fill have pairwise different declared types, so that a swap is
always a type error; two fields whose types are type parameters count as the
same type. Otherwise it's a compile-time error whose help names the fields.
*(ADR 0142)*

```ail error=type.struct.7
struct Pos
    row int
    col int

Pos(row, col)          # shorthand: locals named row and col
Pos(row: r, col: c)    # named
Pos(r, c)              # error: row and col are both int
```

**type.struct.8** — A struct field may declare a default, written
`name Type = value`. A struct literal that leaves the field out gets that
value; naming the field sets it as usual. The value MUST be a plain constant
of the field's type: literals, `none`, `[]`, `{}`, `List[T]()`, `Map[K, V]()`,
`Set[T]()`, and lists, sets, maps, tuples and a leading `-` of them. It names
no constant, function or type, so it reads the same wherever the literal is
written and has no effect; anything else is a compile-time error. Each
literal gets its own fresh value. A generic struct's default is typed with
its type parameters, so `items List[T] = []` is fine and `x T = 0` is not. A
variant's field has no default. A missing field's error says which fields
may be left out. `impl Default` derivation (`trait.assoc.1`) does not read
field defaults: it uses each field type's default. *(ADR 1076)*

```ail skip=context
struct Config
    name str
    retries int = 3
    tags List[str] = []

c := Config(name: "a")          # retries 3, no tags
d := Config(name: "b", retries: 9)
```

```ail error=type.struct.8
struct Bad
    a int = len("abc")          # error: a default has no calls or names
```

## Enums

**type.enum.1** — An enum (sum) type is declared with the keyword `enum`, its
name, and an indented block containing one variant per line. *(ADR 0053)*

**type.enum.2** — A variant either carries no data or carries a parenthesised
list of fields, each written `name Type`. *(ADR 0053)*

```ail fragment
enum Shape
    Circle(r float)
    Rect(w float, h float)
    Empty
```

**type.enum.3** — Where the expected type of an expression or pattern is a
known enum type, its variants MAY be written without qualification
(`Circle(r)`). Elsewhere they MUST be qualified with the enum name
(`Shape.Circle(r)`). *(ADR 0057)*

```ail error=type.enum.3
enum Shape
    Circle(r int)
    Rect(w int, h int)

fn area(s Shape) int
    match s
        Circle(r) => 3 * r * r
        Rect(w, _) => w * w

fn main()
    c := Circle(2)
    print(area(c))
```

```ail
enum Shape
    Circle(r int)
    Rect(w int, h int)

fn area(s Shape) int
    match s
        Circle(r) => 3 * r * r
        Rect(w, _) => w * w

fn main()
    print(area(Circle(2)))
```

**type.enum.4** — The canonical form omits the qualification wherever
`type.enum.3` permits it. *(ADR 0057)*

```ail rules=type.enum.4
enum Shape
    Circle(r int)
    Rect(w int, h int)

fn area(s Shape) int
    match s
        Circle(r) => 3 * r * r
        Rect(w, _) => w * w

fn main()
    print(area(Shape.Circle(2)))
```

**type.enum.5** — An enum's variant names MUST be distinct. *(ADR 0117)*

**type.enum.6** — A variant with data is constructed like a struct literal
(`type.struct.3–4`, `type.struct.7`): `Circle(r: 1.0)`, or `Circle(1.0)`.
*(ADR 0123, ADR 0142)*

```ail rules=type.enum.6
enum Shape
    Circle(r int)
    Rect(w int, h int)

fn area(s Shape) int
    match s
        Circle(r) => 3 * r * r
        Rect(w, _) => w * w

fn main()
    print(area(Rect(w: 2, h: 3)))
```

**type.enum.7** — A string literal compared with `==` or `!=` to an enum value,
or written as a pattern of a `match` on one, is a compile-time error: an enum
is never equal to a string. When the string is a variant's name in snake case
(`"not_found"` for `NotFound`), the diagnostic carries a machine-applicable
fix that writes the variant (`fix.edit.1`). The kinds of the standard
library's errors are enums (`std.err.1`), so a misspelt kind is a
compile-time error, not a test that is never true. *(ADR 1068)*

```ail error=type.enum.7
fn main()
    text := fs.read_text("notes.txt") else err
        if err.kind == "not_found"
            print("missing")
        ""
    print(text)
```

**type.enum.20** — An enum whose variants have no fields, and that declares no
function of the same name, has three functions without being written.
`Color.variants()` returns a `List[Color]` of every variant in declaration
order. `Color.from_str(s)` returns the `Color?` whose variant name is exactly
`s`, and `none` for any other text. `c.name()` returns the variant's name as a
`str`, the text `print(c)` shows. A call with the wrong number of arguments, or
a `from_str` argument that isn't a `str`, is a compile-time error. *(ADR 1087)*

```ail rules=type.enum.20
enum Color
    Red
    Green

fn main()
    for c in Color.variants()
        print(c.name())              # Red, then Green
    print(Color.from_str("Green"))   # Green
    print(Color.from_str("green"))   # none
```

```ail rules=type.enum.7
fn main()
    text := fs.read_text("notes.txt") else err
        if err.kind == NotFound
            print("missing")
        ""
    print(text)
```

## Optional types

**type.opt.1** — [definitional] AIL has no null value. *(ADR 0048)*

**type.opt.2** — `T?` denotes an optional value of type `T`. It is exactly
equivalent to `Option[T]`. *(ADR 0048)*

**type.opt.3** — The canonical form of an optional type is `T?`.
*(ADR 0048)*

```ail skip=context
fn find(id i64) User?
    ...
```

> **Note:** see also `plat.ptr.2` (the 64-bit size and index type) and
> `expr.arith.*` (integer arithmetic).

**type.opt.4** — `none` is the absent value of every optional type.
*(ADR 0093)*

**type.opt.5** — Where a value of type `T?` is expected, a value of type `T`
converts implicitly to a present optional. *(ADR 0093)*

**type.opt.6** — `T??` is an optional whose content is an optional `T?`; it
is a different type from `T?`. Any number of `?` may follow a type. A value
of type `T??` arises from collection methods on optional elements, for
example `m.get(k)` on a `Map[K, V?]` and `xs.last()` on a `List[T?]`.
*(ADR 0209)*

**type.opt.7** — `none` is the absent value of the outermost level. `e else
d`, `e?` and `if v := e` act on that level only and give an optional of the
inner type: `m.get(k) else none` flattens a `V??` to a `V?`. A present value
whose content is `none` is not `none`: `==` compares every level, and the
text form of the present-but-empty value of a `T??` is `some(none)`. A `T`
converts to a `T??` (`type.opt.5`, twice). *(ADR 0209)*

**type.opt.8** — A `T?` does not convert to a `T??`, because its `none` would
be ambiguous between the two levels. The error is the ordinary mismatch with
a help that says so. *(ADR 0209)*

```ail fragment
fn lookup(m Map[str, int?], k str) int??
    m.get(k)

fn flat(m Map[str, int?], k str) int?
    lookup(m, k) else none
```

```ail error=expr.block.2
fn lift(v int?) int??
    v

fn main()
    print(lift(none))
```

## Function types

**type.fn.1** — `fn(A, B) R` is the type of a function value that takes
arguments of types `A` and `B` and returns `R`; with no result it is
`fn(A, B)`. A function type is a type like any other: it can be a parameter,
a result, a local, a struct field or a list element. It has no equality, hash
or text form. *(ADR 0455)*

**type.fn.2** — The name of a declared function, used where a value is
expected and not called, is a function value of its type: `f := double`. The
function MUST be a top-level function that is not generic, not a test, and has
no `mut` or `move` parameter; anything else is an error. A function that fails
is a value of a failing function type (`type.fn.6`). A local of the same name
hides the function. Naming a function as a value counts as calling it for
effect inference (`eff.infer.3`, `eff.pure.1`). *(ADR 0455, ADR 1067)*

**type.fn.3** — A value of function type is called like a function:
`f(x)`, or `(s.run)(x)` for a field and `pick(name)(x)` for a result. The
arguments are passed by position, `mut` is not allowed, and each must fit the
parameter type (`expr.call.1`, `expr.call.2`). Calling a function value adds no
effect of its own. A function literal (`type.fn.4`) makes a function value
without declaring a function. *(ADR 0456, ADR 1067)*

```ail rules=type.fn.1,type.fn.2,type.fn.3
fn double(n int) int
    n * 2

fn apply(f fn(int) int, x int) int
    f(x)

fn main()
    print("{apply(double, 4)}")
```

**type.fn.4** — A function literal `|a, b| e` is a function value whose
parameters are `a` and `b` and whose result is the value of the expression
`e`; `|| e` takes no parameters. The body is one expression and MUST start on
the line of the parameters. Each parameter's type is the matching parameter
type of the function type the context expects (an annotation, a parameter, a
field, a result); where none is expected it MUST be written after the name,
`|x int| x + 1`, and a written type MUST equal an expected one. The number of
parameters MUST equal the expected type's. The result type is the expected
one, or else the body's. `return`, `break` and `continue` can't leave a
literal. A `|` where an operand is expected opens a literal: when no parameter
name or `|` follows it, the error names both readings (a literal, or bit-or
`a | b`, which goes between operands, ADR 1070). *(ADR 1067)*

```ail rules=type.fn.4
fn apply(f fn(int) int, x int) int
    f(x)

fn main()
    print("{apply(|x| x * 3, 4)}")        # 12
    sq := |n int| n * n
    print("{sq(5)}")                      # 25
    hello := || "hi"
    print(hello())                        # hi
```

**type.fn.5** — A function literal copies each local of the enclosing
function that its body names, when the literal is evaluated: later changes to
the local don't reach the copy. Inside the body the copies are read-only:
assigning one, calling a `mut` method on it, passing it `mut` or moving it is
an error. A value of a `unique` type can't be copied in (`own.uniq.1`). To
share state the literal can change, capture a `ref` cell (`own.ref.1`): the
copy is the same cell. *(ADR 1067)*

```ail rules=type.fn.5
struct Counter
    n int

fn Counter.bump(mut self)
    n += 1

fn adder(n int) fn(int) int
    |x| x + n                             # copies `n`

fn main()
    mut base := 10
    add := |x int| x + base
    base = 100
    print("{add(1)} {adder(2)(3)}")       # 11 5
    c := ref Counter(n: 0)
    tick := || c.bump()                   # the copy is the same cell
    tick()
    print("{c.n}")                        # 1
```

**type.fn.6** — `fn(A) R ! E` is the type of a function value that can fail
with the errors `E` (`err.type.1`); the `!` after the result belongs to the
function type. A call of such a value can fail and must be handled like any
failing call (`err.use.1`). `?` in a literal's body leaves the literal, not
the enclosing function: the literal then fails with the errors `?` passes on,
as do the errors of a failing body. A function value fits a function type with
the same parameter types when its result fits that type's result: one that
can't fail, or fails with fewer errors, fits where a failing one is expected.
*(ADR 1067)*

```ail rules=type.fn.6
error struct BadPort
    text str

fn port(text str) int ! BadPort
    n := text.to_int() else return BadPort(text: text)
    n

fn sum(texts List[str], parse fn(str) int ! BadPort) int ! BadPort
    mut total := 0
    for t in texts
        total += parse(t)?
    total

fn main()
    print("{sum(["1", "2"], port) else -1}")          # 3
    print("{sum(["ab"], |t| t.len()) else -1}")       # 2
    next := |t str| port(t)? + 1
    print("{next("x") else -1}")                      # -1
```

**type.fn.7** — A function literal's body is checked for effects as part of
the function that contains the literal (`eff.infer.3`): writing a literal that
prints makes the enclosing function do output, whether or not the literal is
called. *(ADR 1067)*

```ail rules=type.fn.7
fn main()
    say := |s str| print(s)
    say("hi")                             # hi
```

**type.fn.8** — A generic function can take function values: in
`fn map_all[T, U](xs List[T], f fn(T) U) List[U]` the arguments are checked in
order, so a literal's parameter types come from the type arguments the
arguments before it settle, and its result settles the rest. *(ADR 1067)*

```ail rules=type.fn.8
fn map_all[T, U](xs List[T], f fn(T) U) List[U]
    mut out := List[U]()
    for x in xs
        out.push(f(x))
    out

fn main()
    print(map_all([1, 2], |x| "n{x}"))    # ["n1", "n2"]
```

**type.fn.9** — A statement `fn name(params) R` followed by a block, inside a
function body, declares a local function: a local `name` of the function type
`fn(A, B) R`, visible from its declaration to the end of the enclosing block,
including inside its own body, so it can call itself. Its body is a function
body: `return` leaves it, its result is its last expression, and `R` may fail
(`T ! E`, `!T`). It MUST NOT be generic, a method, or have contracts, and its
parameters MUST NOT be `mut` or `move`: it is called as a function value
(`type.fn.3`), by position. `break` and `continue` can't leave it. The name
follows `expr.bind.4`. *(ADR 1094)*

```ail rules=type.fn.9
fn main()
    fn fact(n int) int
        if n <= 1
            return 1
        n * fact(n - 1)
    print("{fact(5)}")                    # 120
    print([1, 2, 3].map(fact))            # [1, 2, 6]
```

**type.fn.10** — A local function copies each outer local its body names
when the declaration is reached, under `type.fn.5`: the copies are read-only,
a `unique` value can't be copied, and a captured `ref` cell is shared. Its
body's effects count for the enclosing function (`type.fn.7`). In traces and
trap stacks it is named `outer.name`. *(ADR 1094)*

```ail rules=type.fn.10
fn main()
    base := 10
    fn add(x int) int
        x + base                          # copies `base`
    print("{add(5)}")                     # 15
```

## Tuples

**type.tuple.1** — `(A, B, ...)` is the type of a tuple: an anonymous value
of two or more elements, each with its own type. A tuple is a value like a
struct: assignment copies it, it is `mut` where its holder is, and it is never
shared. A one-element or empty tuple type is an error; a value of a unique
type can't be an element (`own.uniq.5`). *(ADR 0791, ADR 0792)*

**type.tuple.2** — `(a, b, ...)` is a tuple whose elements are the values of
`a`, `b`, .... Each element is checked against the matching element of the
expected type, so `(1, none)` can be an `(int, str?)`. A parenthesised single
expression is not a tuple, and `(a,)` is an error. *(ADR 0791, ADR 0792)*

**type.tuple.3** — `t.0`, `t.1`, ... read element 0, 1, ... of the tuple `t`,
and assign it when `t` is a `mut` place. The index is a decimal literal
smaller than the number of elements; anything else is a compile-time error.
`t.0.1` is element 1 of element 0. *(ADR 0791, ADR 0792)*

**type.tuple.4** — `a, b := e` declares one local per element of the tuple
`e`, in order. The number of names MUST equal the number of elements. `_`
skips an element, and `mut` before a name makes that local mutable:
`a, mut b := e`. *(ADR 0791, ADR 0792)*

**type.tuple.5** — `for a, b in xs` takes each element of `xs`, which must be
a tuple of as many elements, apart as `type.tuple.4` does; so does a
comprehension clause `for a, b in xs`. `for k, v in m` iterates the entries of
a map `m` as `(key, value)` pairs, in insertion order (`type.map.5`). `for
mut` takes one name. *(ADR 0791, ADR 0792)*

**type.tuple.6** — A `match` on a tuple takes tuple patterns `(p, q, ...)`
with one pattern per element. An element pattern is `_`, a name, which binds
the element, a literal, a range, a variant, or a nested tuple pattern. A name
that is a variant of the element's enum is an error: write `Enum.Name` to
match the variant. A `match` on a tuple MUST have a `_` arm or an arm that
matches every tuple, such as `(a, _)` (`expr.match.2`). *(ADR 0791, ADR 0793)*

**type.tuple.7** — A tuple is `Eq`, `Ord` and `Hash` when every element is.
`==` compares elementwise, `<` and `sorted` compare lexicographically, element
0 first, and a tuple can be a map key or set element. *(ADR 0791)*

**type.tuple.8** — The text form of a tuple is its elements' text forms,
strings quoted, in parentheses: `(1, "a")`. A derived `ToJson` writes a
tuple-typed field as a JSON array of its elements, and a derived `FromJson`
reads one back, failing with `expected N elements` when the length differs
(`json.derive.1-2`). *(ADR 0791, ADR 0795)*

**type.tuple.9** — `xs.enumerate()` returns a `List[(int, T)]` of each element
of the list `xs` with its index. `zip(a, b)` returns a `List[(A, B)]` pairing
the elements of the lists `a` and `b` in order, as long as the shorter. A
program's own function named `zip` replaces it. *(ADR 0791, ADR 0794)*

```ail rules=type.tuple.1,type.tuple.2,type.tuple.3,type.tuple.4
fn divmod(a int, b int) (int, int)
    (a / b, a % b)

fn main()
    q, r := divmod(17, 5)
    print("{q} {r}")                              # 3 2
    mut p := (1, "a")
    p.0 = 10
    print(p.0)                                    # 10
    mut n, _ := p
    n += 1
    print(n)                                      # 11
```

```ail rules=type.tuple.5,type.tuple.9
fn main()
    ages := {"ann": 31, "bob": 25}
    for name, age in ages
        print("{name} is {age}")
    for i, x in ["a", "b"].enumerate()
        print("{i}: {x}")
    print([n * 2 for n, _ in zip([1, 2, 3], ["x", "y"])])   # [2, 4]
```

```ail rules=type.tuple.6
fn describe(t (int, str)) str
    match t
        (0, _) => "zero"
        (n, "") => "{n} and nothing"
        (n, s) => "{n} {s}"

fn main()
    print(describe((0, "x")))                     # zero
    print(describe((2, "y")))                     # 2 y
```

```ail rules=type.tuple.7,type.tuple.8
struct Edge
    ends (str, str)

impl ToJson for Edge

fn main()
    pairs := [(2, "b"), (1, "z"), (1, "a")]
    print(pairs.sorted())                         # [(1, "a"), (1, "z"), (2, "b")]
    seen := {(0, 0): "origin"}
    print(seen[(0, 0)])                           # origin
    print(Edge(ends: ("a", "b")).to_json().to_text())   # {"ends":["a","b"]}
```

```ail error=type.tuple.3
fn main()
    t := (1, "a")
    print(t.2)                                    # error: no element 2
```

```ail error=type.tuple.4
fn main()
    a, b, c := (1, 2)                             # error: 3 names, 2 elements
    print(a)
```

```ail error=expr.match.2
fn main()
    match (1, "a")
        (0, _) => print("zero")                   # error: no arm covers (1, "a")
```

## Booleans

**type.opt.9** — A `mut` local declared with `:=` from `none` has no type
until its first assignment `x = e`: it takes the type `T?`, where `T` is the
type of `e` (`type.opt.5`). Any other use before then, or none at all, is a
compile-time error whose help asks for an assignment. The local is not a
`none`-typed value in between. *(ADR 0468)*

```ail fragment
fn first_word(s str) str?
    mut owner := none
    for w in s.split(" ")
        owner = w
        break
    owner
```

**type.opt.10** — An `if` with no expected type, whose branch is a bare `none`
and whose other branches have a type `T`, has the type `T?`: each `none`
branch is `none` of that type, and each other branch converts by
`type.opt.5`. When every branch is `none`, the `if` has no type and the
error is `type.opt.4`. *(ADR 0468)*

```ail fragment
fn half(n int) int?
    x := if n % 2 == 0 then n / 2 else none
    x
```

**type.bool.1** — `bool` is the type of the literals `true` and `false`.
Logical operators take and produce `bool` (`expr.op.1`). *(ADR 0110)*

```ail rules=type.bool.1
fn main()
    print(true and not false)
```

## Strings and characters

**type.str.1** — `str` is the type of immutable UTF-8 text. `char` is the type
of a single Unicode scalar value. *(ADR 0089)*

**type.str.2** — `str` doesn't support the indexing operator or `for`
iteration. Character access goes through the explicit view `s.chars()`
(`type.str.7`); the UTF-8 bytes are `s.to_bytes()` (`type.bytes.10`). The diagnostic for `s[i]`
or `for c in s` suggests `s.chars()`. Length is `s.len()` (`type.str.12`).
*(ADR 0089, ADR 0135, ADR 0208)*

**type.str.3** — The text form of an `int` is its decimal representation,
with a leading `-` when negative. The text form of a `bool` is `true` or
`false`, and of a `str` is the string itself; those of a `float` and a `char`
are given by `type.num.7` and `type.char.1`. `bool` and `str` support `==`
and `!=`; `str` equality compares UTF-8 bytes. `str` also supports `<`, `<=`,
`>` and `>=` (`trait.std.4`). *(ADR 0116, ADR 0127, ADR 0130, ADR 0134)*

**type.str.4** — `s.split(sep)` returns a `List[str]` of the pieces of `s`
between occurrences of `sep`, keeping empty pieces, so `n` occurrences give
`n + 1` pieces. An empty `sep` traps. `sep` is a `str` or a `char`
(`type.str.6`). `s.split(sep, limit)` makes at most `limit` pieces and the
last keeps the rest of `s`, so `"a=b=c".split("=", 2)` is `["a", "b=c"]`;
`limit` is an `int` of at least 1, otherwise it traps. *(ADR 0124, ADR 0135,
ADR 0758)*

**type.str.5** — `s.to_int()` returns an `int?`. It holds the value of `s`
when `s` is an optional `-` followed by one or more decimal digits and the
value fits in `int`; otherwise it is `none`. *(ADR 0124)*

**type.str.6** — The `str` methods that search for text (`contains`,
`starts_with`, `ends_with`, `split`, `find`, `rfind` and `count`) accept a
`char` as well as a `str`, and `replace` accepts one for each of its two
arguments; a `char` means the one-character string. `strip_prefix` and
`strip_suffix` take a `str` only. *(ADR 0135, ADR 1087)*

**type.str.7** — A `str` has these methods. *(ADR 0135)*
- `s.chars()` returns a new `List[char]`. Strings stay unindexable
  (`type.str.2`).
- `s.lines()` returns a `List[str]`: `s` split on `\n`, with one trailing
  empty piece dropped and a `\r` before each `\n` removed (Python's
  `splitlines` for `\n` and `\r\n`).
- `s.trim()` removes the leading and trailing characters for which
  `is_space()` is true (`type.char.2`).
- `s.contains(sub)`, `s.starts_with(p)` and `s.ends_with(p)` return a `bool`.
- `s.strip_prefix(p)` and `s.strip_suffix(p)` return the rest of `s` as a
  `str?`, or `none` when `s` doesn't start (end) with `p`.

```ail fragment
print("a\r\nb\n".lines())      # ["a", "b"]
print("a\n\n".lines())         # ["a", ""]
print("  hi ".trim())         # hi
print("key=v".strip_prefix("key=")) # v
print("h\u{e9}".chars())       # ['h', 'é']
```

**type.str.8** — `s.to_float()` returns a `float?` (`type.num.8`).
`n.to_hex()` on an `int` returns its hexadecimal digits, lowercase and without
a prefix, with a `-` before the magnitude of a negative value, so the minimum
`int` is `-8000000000000000`. *(ADR 0135, ADR 0137)*

**type.str.9** — `s.pad_left(width, fill)` and `s.pad_right(width, fill)`
add copies of the `char` `fill` on the left (right) until `s` has `width`
characters (scalar values). A string already that wide, or any string when
`width <= 0`, is returned unchanged. The fill is required: AIL has no default
arguments. *(ADR 0135)*

```ail fragment
print("7".pad_left(3, '0')) # 007
print("ab".pad_right(4, ' ')) # "ab  "
```

**type.str.16** — `s.center(width, fill)` pads with copies of the `char` `fill`
on both sides until `s` has `width` characters; when the padding is odd, the
extra copy goes on the right. A string already that wide is returned
unchanged. *(ADR 0629)*

```ail rules=type.str.16
fn main()
    print("[{"ab".center(7, '*')}]")         # [**ab***]
```

**type.str.17** — `s.words()` returns the pieces of `s` between runs of
whitespace, with no empty pieces (an empty list for blank text).
`s.trim_start()` and `s.trim_end()` remove whitespace from one end only.
*(ADR 0629)*

**type.str.18** — `s.count(sub)` returns the number of non-overlapping
occurrences of `sub`; it traps when `sub` is empty. `s.rfind(sub)` is the
character index of the last occurrence, or `none` (compare `type.str.14`).
*(ADR 0629)*

```ail rules=type.str.17,type.str.18
fn main()
    print("  a  b ".words())                # ["a", "b"]
    print("[{"  x ".trim_end()}]")          # [  x]
    print("abababa".count("aba"))           # 2
    print("a/b/c".rfind("/"))               # 3
```

**type.str.19** — `s.is_empty()` is `s.len() == 0`. *(ADR 0630)*

**type.str.20** — `s.to_int(radix)` returns an `int?` like `to_int()` (`type.str.5`),
reading an optional `-` and then digits `0-9`, `a-z` or `A-Z` whose values are below
`radix`; anything else, or a value that doesn't fit, is `none`. `n.to_str(radix)` on an
integer is its digits in `radix`, lowercase, with a `-` before the magnitude of a negative
value (as `n.to_hex()` for 16); `n.to_str()` is the decimal text. A `radix` outside 2 to 36
is a compile-time error for a literal and otherwise traps with this rule. *(ADR 1087)*

```ail rules=type.str.20
fn main()
    print("ff".to_int(16))        # 255
    print("102".to_int(2))        # none
    print(255.to_str(2))          # 11111111
    print((0 - 255).to_str(16))      # -ff
```

**type.coll.20** — `xs.is_empty()` on a list, map or set is `xs.len() == 0`.
*(ADR 0630)*

```ail rules=type.str.19,type.coll.20
fn main()
    print("".is_empty())                    # true
    print([1].is_empty())                   # false
    print({"a": 1}.is_empty())              # false
```

**type.coll.21** — A list has methods that take a function of its element
(`type.fn.1`), a function literal (`type.fn.4`) or any other function value:
`any`, `all`, `find`, `filter`, `map` and `group_by` (`type.coll.22`-`26`).
Each takes exactly one, which MUST take one parameter of the element type and
MUST NOT fail (`type.fn.6`): to handle errors per element, use a `for` loop.
The function is called on the elements in order, at most once each.
*(ADR 1067)*

**type.coll.22** — `xs.any(f)` is `true` when `f` returns `true` for some
element, and `xs.all(f)` when it does for every element (`true` for an empty
list). Both stop at the first element that settles the answer. `f` returns
`bool`. `any` is a keyword (`lex.kw.1`); after `.` it names this method
(`lex.kw.3`). *(ADR 1067, ADR 1085)*

**type.coll.23** — `xs.find(f)` returns the first element for which `f`
returns `true`, or `none` (`T?`). *(ADR 1067)*

**type.coll.24** — `xs.filter(f)` returns a new list of the elements for
which `f` returns `true`, in order. *(ADR 1067)*

**type.coll.25** — `xs.map(f)` returns a new list of `f`'s results, one per
element, in order: a `List[U]` for `f` of type `fn(T) U`. `f` MUST return a
value. *(ADR 1067)*

**type.coll.26** — `xs.group_by(f)` returns a `Map[K, List[T]]` from each
key `f` returns to the elements that gave it, keys in order of first
appearance and elements in order. `K` MUST be hashable (`gen.bound.3`).
*(ADR 1067)*

```ail rules=type.coll.21,type.coll.22,type.coll.23,type.coll.24,type.coll.25,type.coll.26
fn is_even(n int) bool
    n % 2 == 0

fn main()
    xs := [3, 8, 5, 12]
    print("{xs.any(|x| x > 10)} {xs.all(is_even)}")   # true false
    print(xs.find(|x| x > 4) else 0)                  # 8
    print(xs.filter(is_even))                         # [8, 12]
    print(xs.map(|x| x * 2))                          # [6, 16, 10, 24]
    print(xs.group_by(|x| x % 2))                     # {1: [3, 5], 0: [8, 12]}
```

**type.str.13** — `s.slice(a, b)` returns the characters of `s` from index
`a` up to but excluding `b` (indexes count characters, as `len()` does:
`type.str.12`). It traps unless `0 <= a <= b <= s.len()`; bounds are never
clamped. `s.char_at(i)` returns the character at `i` as a `char?`, `none` if
`i` is out of range or negative. *(ADR 0215)*

**type.str.14** — `s.find(sub)` returns the character index of the first
occurrence of `sub` as an `int?`, or `none`. `s.replace(from, to)` returns a
copy with every non-overlapping occurrence of `from` replaced by `to`, left
to right; it traps if `from` is empty. `s.to_lower()` and `s.to_upper()`
return the Unicode lower- and upper-case forms. `sub`, `from` and `to` are
`str`s. *(ADR 0215)*

```ail fragment
s := "Héllo"
a := s.slice(1, 3)       # "él"
i := s.find("llo")       # 2
t := s.replace("l", "L") # "HéLLo"
```

**type.str.10** — `s.repeat(n)` returns `n` copies of `s` joined together.
It traps if `n < 0`; AIL doesn't clamp. *(ADR 0135)*

**type.str.15** — `repeat` and `pad_left`/`pad_right` MUST trap, rather than
exhaust memory, when the text they would build is over 2^30 bytes. *(ADR 0271)*

**type.str.12** — `s.len()` returns the number of characters (Unicode scalar
values) of `s`, the same as `s.chars().len()` without building the list; it
takes time proportional to the length. `s.byte_len()` returns the number of
UTF-8 bytes. Grapheme clusters are not counted: `"e\u{301}".len()` is 2.
*(ADR 0208)*

```ail fragment
n := "héllo".len()        # 5
b := "héllo".byte_len()   # 6
```

**type.str.11** — In `{x:spec}` (`lex.str.7`), `.N` formats a `float` with
exactly `N` decimal places, rounding the exact value half to even (`{2.5:.0}`
is `2`); `nan` and `inf` print as `{x}` does. `x` formats an `int` in lowercase
hex without `0x`, with a leading `-` for a negative number (as `n.to_hex()`);
`b` does the same in binary and `{n:08b}` pads it. `e` formats a `float` as
`d.ddde` and a decimal exponent, from the shortest digits that read back as `x`
(`{1234.5:e}` is `1.2345e3`, `{0.00123:e}` is `1.23e-3`); `{x:.2e}` keeps two
decimals of the mantissa, rounding those digits half up. `+` forces a sign on
an `int` or `float`, `,` groups the digits of its integer part
(`{1234567:,}` is `1,234,567`), and `0` with a width pads with zeros after the
sign (`{-5:04}` is `-005`). Using `.N` or `e` on a non-float, `x` or `b` on a
non-int, or `+`, `,` or `0` on a non-number is a compile-time error. Width,
fill and alignment apply to the text form of any value. *(ADR 0173, ADR 1087)*

```ail error=type.str.11
n := 1
print("{n:.2}")      # compile-time error: write {n.to_float():.2}
```

> **Note:** joining is `xs.join(sep)` on any list (`type.coll.9`). There is
> no `str.from_chars`. `s.len()` counts characters (`type.str.12`).

## Bytes

**type.bytes.1** — `bytes` is the type of an immutable sequence of `u8`. Its
literal is `b"..."` (`lex.bytes.1`). It holds any bytes, valid UTF-8 or not;
`str` stays text (`type.str.1`). Copies share storage. *(ADR 0412)*

**type.bytes.2** — `bytes` values never change in place: `b[i] = x` is an
error, and no method changes its receiver. To change one, build a new value
(`b[..i] + bytes.from_list([x]) + b[i + 1..]`) or edit a `List[u8]` and
convert it with `bytes.from_list`. `==` and `!=` compare the bytes; `<`, `<=`,
`>` and `>=` order them lexicographically by byte value, a shorter prefix
first; `bytes` is `Hash` and so can be a map key or set element (`trait.std.2`,
`trait.auto.5`). The text form of `bytes`, in `print` and in interpolation, is
its literal form `b"..."`: printable ASCII as itself, `\" \\ \n \t \r` as
escapes, and every other byte as `\xHH`. *(ADR 0412)*

**type.bytes.3** — `b[i]` is the `u8` at index `i`. `i` is an `int`; it traps
if `i` is negative or not less than `b.len()`. *(ADR 0412)*

**type.bytes.4** — `b[lo..hi]` and `b[lo..=hi]` are the `bytes` between the
bounds, as for lists (`type.coll.4`); a missing bound is the start or the end.
It traps if `lo > hi` or a bound is outside `0..=b.len()`. *(ADR 0412)*

**type.bytes.5** — `for x in b` and comprehensions over `b` give each byte as
a `u8`, in order. *(ADR 0412)*

**type.bytes.6** — `a + b`, for two `bytes`, and `a.concat(b)` are a new
`bytes`: the bytes of `a` then those of `b`. *(ADR 0412)*

**type.bytes.7** — `b.len()` is the number of bytes, an `int`. `b.to_list()`
is a new `List[u8]` of them. *(ADR 0412)*

**type.bytes.8** — `b.to_str()` is a `str?`: the bytes as UTF-8 text, or `none`
if they aren't valid UTF-8. `b.text()` is a `str`: the same, with each invalid
sequence as U+FFFD. `to_str` on `bytes` is this fallible conversion, not the
text form of `trait.std.3`; the text form is reached by interpolation
(`"{b}"`) or `ToStr.to_str(b)`. *(ADR 0414)*

**type.bytes.9** — `b.find(sub)` is the index of the first occurrence of
`sub` in `b` as an `int?` (`0` for an empty `sub`). `b.contains(sub)`,
`b.starts_with(p)` and `b.ends_with(p)` return a `bool`. Each argument is a
`bytes`. *(ADR 0414)*

**type.bytes.10** — `s.to_bytes()` is the UTF-8 bytes of the `str` `s`, a
`bytes` of length `s.byte_len()`. *(ADR 0414)*

**type.bytes.11** — `b.hex()` is a `str` of two lowercase hexadecimal digits
for each byte. *(ADR 0414)*

**type.bytes.12** — `b.base64()` is a `str`: the bytes in standard base64
(RFC 4648, the `+` and `/` alphabet), padded with `=`. *(ADR 0414)*

**type.bytes.13** — `bytes` has these associated functions. *(ADR 0414)*
- `bytes.from_list(xs)` is the `bytes` of a `List[u8]`.
- `bytes.from_hex(s)` is a `bytes?`: `none` unless `s` is an even number of
  hexadecimal digits, either case.
- `bytes.from_base64(s)` is a `bytes?`: `none` unless `s` is standard base64
  with its padding and no other character.
- `bytes.zeros(n)` is `n` zero bytes; it traps if `n` is negative or above 2^32.

**type.bytes.14** — `b.hex(sep)` is `b.hex()` with the `str` `sep` between the byte
pairs: `b"\x00\x0f".hex(" ")` is `"00 0f"`. An empty `sep` gives `b.hex()`, and
empty bytes give `""`. *(ADR 0775)*

```ail fragment rules=type.bytes.1,type.bytes.2,type.bytes.3,type.bytes.4,type.bytes.5,type.bytes.6,type.bytes.7,type.bytes.8,type.bytes.9,type.bytes.10,type.bytes.11,type.bytes.12,type.bytes.13,type.bytes.14
b := b"hello\x00\n"
print(b.len())                       # 7
print(b[1])                          # 101
print(b[1..3])                       # b"el"
print(b[..2] + b[3..])               # b"helo\x00\n"
print(b.hex())                       # 68656c6c6f000a
print(b[..3].hex(" "))               # 68 65 6c
print("hi".to_bytes().to_list())     # [104, 105]
print(bytes.from_base64("aGk=") else b"")
for x in b"abc"
    print(x)                         # 97 98 99
print(b.to_str() else "bad")         # bad: the NUL byte is valid UTF-8, so this prints the text
print(b"\xff".text().len())          # 1: one U+FFFD
print(b.find(b"ll") else -1)         # 2
print(b.contains(b"lo"))             # true
print(b.starts_with(b"he"))          # true
print(b.ends_with(b"\n"))            # true
print("é".to_bytes().len())          # 2
print(bytes.zeros(3).hex())          # 000000
print(bytes.from_list([104, 105]))   # b"hi"
print(bytes.from_hex("ff") else b"") # b"\xff"
```

```ail error=type.bytes.2 rules=type.bytes.2
fn main()
    b := b"ab"
    b[0] = 0x63                          # bytes are immutable: no element assignment
```

```ail norun rules=type.bytes.3,type.bytes.4
fn main()
    b := b"abc"
    print(b[3])                          # traps: index 3 is not below b.len()
```

## Characters

**type.char.1** — `char` is a Unicode scalar value (`type.str.1`). It
implements `Eq`, `Ord` (by code point), `Hash` and `ToStr`. Its text form at
the top level is the character itself. Inside another value it is a character
literal in single quotes, escaped as in `trait.auto.2`. *(ADR 0134)*

**type.char.2** — A `char` has these predicates and mappings. *(ADR 0134)*
- `is_digit()` is true for ASCII `0` to `9` only, so it implies that
  `to_digit(10)` is some value.
- `is_alpha()` is true for the Unicode `Alphabetic` property, and `is_alnum()`
  for `is_alpha()` or `is_digit()`.
- `is_space()` is true for the Unicode `White_Space` property.
- `is_upper()` and `is_lower()` test the Unicode `Uppercase` and `Lowercase`
  properties.
- `to_upper()` and `to_lower()` apply the single-character mapping. A
  character whose mapping is several characters (such as `ß`) is unchanged.

The Unicode version is the implementation's (the bootstrap's is that of the
pinned Rust toolchain).

**type.char.3** — `c.to_digit(radix)` returns an `int?`. The characters `0`
to `9`, `a` to `z` and `A` to `Z` are worth 0 to 35. The result is `none` if
`c` isn't one of those or its value is `>= radix`. A `radix` outside `2..36`
traps. *(ADR 0134)*

**type.char.4** — `c.code()` returns the scalar value of `c` as an `int`.
`char.from_code(n)` returns a `char?`: `none` for surrogates and values
outside the scalar range. The diagnostics for `ord(c)`, `chr(n)` and
`c.to_int()` suggest `c.code()`, `char.from_code(n)` and "`c.code()` or
`c.to_digit(10)`". *(ADR 0134)*

```ail fragment
c := 'f'
print(c.is_alpha())     # true
print(c.to_digit(16))   # 15
print(c.to_digit(10))   # none
print(c.code())         # 102
print(char.from_code(0x41)) # 'A'
```

## Collections

**type.coll.1** — The standard library provides `List[T]`, `Map[K, V]` and
`Set[T]`, all with value semantics (`own.val.1`). *(ADR 0090)*

**type.coll.2** — `[a, b, ...]` is a list literal. `{k: v, ...}` is a map
literal. `{a, b, ...}` is a set literal. *(ADR 0090)*

```ail fragment
ports := {"http": 80, "https": 443}
primes := [2, 3, 5, 7]
```

**type.coll.3** — The empty literals `[]` and `{}` require a known expected
type, except when they declare a local (`type.coll.13`). Otherwise they are a
compile-time error, whose help suggests the constructors of `type.coll.8`.
*(ADR 0090, ADR 0131, ADR 0143)*

**type.coll.4** — `xs[i]` traps if `i` is out of bounds. `xs[a..b]` yields
the sub-list from `a` up to but excluding `b`, as a value; `xs[a..=b]` also
includes element `b` (it is `xs[a..b + 1]`). A slice traps unless
`0 <= a <= b <= xs.len()`: bounds are never clamped, unlike Python. A negative
integer literal as an index or a bound is a compile-time error, whose help
suggests `xs[xs.len() - 1]` or `xs.last()`. *(ADR 0090, ADR 0136)*

**type.coll.5** — `xs.len()` returns the number of elements of a list as an
`int`. *(ADR 0116)*

**type.coll.6** — `xs.push(v)` appends `v` to a list. Its receiver is
`mut self` (`trait.method.3`). *(ADR 0116)*

**type.coll.7** — `xs.swap(i, j)` exchanges the elements at indexes `i` and
`j`, and does nothing when `i == j`. It traps if either index is out of
bounds. Its receiver is `mut self`. *(ADR 0119)*

**type.coll.8** — `List[T]()`, `Map[K, V]()` and `Set[T]()` give an empty
collection anywhere, with no expected type needed. `{}` where the expected
type is a `Map` or `Set` is the empty map or set; with no expected type it is
the error of `type.coll.3`. *(ADR 0131)*

```ail fragment
seen := Set[int]()
counts := Map[str, int]()
names := List[str]()
```

**type.coll.9** — A list has these methods. *(ADR 0136)*
- `xs.pop()` returns a `T?`: it removes and returns the last element, or is
  `none` if the list is empty. Its receiver is `mut self`.
- `xs.last()` returns the last element as a `T?`, or `none` if empty.
- `xs.contains(x)` returns a `bool`. `T` MUST be `Eq`.
- `xs.index_of(x)` returns the index of the first element equal to `x` as an
  `int?`, or `none` if absent. `T` MUST be `Eq`. (`xs.index_of(x) == -1` is an
  error: `expr.cmp.3`.)
- `xs.join(sep)` returns a `str`. `T` may be any type: each element
  contributes its text form, with `str` and `char` elements unquoted at the
  top level and nested values in their quoted forms (`trait.auto.2`).

```ail fragment
xs := ["a", "b"]
print(xs.join(", "))    # a, b
print([1, 2, 3].join("")) # 123
print(xs.index_of("b")) # 1
```

**type.coll.14** — A list has these further methods. *(ADR 0214)*
- `xs.insert(i, x)` puts `x` at index `i`, moving later elements up. `i` MAY
  equal `xs.len()` (append). Receiver `mut self`.
- `xs.remove_at(i)` removes and returns the element at `i` as a `T`.
  Receiver `mut self`.
- `xs.extend(ys)` appends the elements of the `List[T]` `ys`. Receiver
  `mut self`.
- `xs.clear()` removes every element and `xs.reverse()` reverses in place.
  Receivers are `mut self`.
- `xs.first()` returns the first element as a `T?`, or `none` if empty.

`insert` traps unless `0 <= i <= xs.len()` and `remove_at` traps unless
`0 <= i < xs.len()`, both under rule `type.coll.4`. There is no `xs[a..b]`
method: slicing is `type.coll.4`.

```ail fragment
mut xs := [1, 2, 3]
xs.insert(1, 9)       # [1, 9, 2, 3]
n := xs.remove_at(0)  # 1
xs.extend([4, 5])     # [9, 2, 3, 4, 5]
```

**type.coll.10** — Open-ended slices `xs[a..]`, `xs[..b]` and `xs[..=b]` follow
the bounds rule of `type.coll.4`. Open ranges exist only inside `[]`;
`xs[..]` is a compile-time error (write `xs`). *(ADR 0136)*

**type.coll.11** — `xs.sorted()` returns a new `List[T]` and `xs.sort()`
(`mut self`) sorts in place. `T` MUST be `Ord`, both are stable, and both use
`cmp` (so `float` follows `type.num.11`). `xs.min()` and `xs.max()` return a
`T?`: the first least (greatest) element, or `none` if the list is empty. An
inherent method takes precedence over a method of the same name from `Ord`
(`trait.res.2`), so `xs.min()` isn't `Ord`'s `min(other)`. A binding of
`xs.sort()`'s result gets a help suggesting `sorted()`. Each also takes a key
function, `by: |x| key` (`expr.key.1`); `s.sorted(by: ...)` works on a set.
*(ADR 0136, ADR 0137, ADR 0250)*

**type.coll.15** — `xs.take(n)` returns a new list of the first `n`
elements, or all of them when the list is shorter. A negative `n` traps.
*(ADR 0250)*

**type.coll.17** — `xs.chunks(n)` returns the list's elements as lists of `n`
(the last may be shorter); `xs.windows(n)` returns every run of `n`
neighbouring elements (none when the list is shorter than `n`). Both give a
`List[List[T]]`, and trap when `n < 1`. *(ADR 0629)*

**type.coll.18** — `xs.counts()` returns a `Map[T, int]` of how many times each
element occurs, keys in order of first appearance. `T` must be hashable.
*(ADR 0629)*

```ail rules=type.coll.17,type.coll.18
fn main()
    xs := [1, 2, 3, 4, 5]
    print(xs.chunks(2))                      # [[1, 2], [3, 4], [5]]
    print(xs.windows(3))                     # [[1, 2, 3], [2, 3, 4], [3, 4, 5]]
    print("a b a".split(" ").counts())       # {"a": 2, "b": 1}
```

**type.coll.19** — `xs.flatten()` on a `List[List[T]]` returns one `List[T]`
with the inner lists' elements in order. *(ADR 0629)*

```ail rules=type.coll.19
fn main()
    print([[1, 2], [], [3]].flatten())      # [1, 2, 3]
```

**type.coll.12** — `xs.sum()` exists only for `List[int]`, returning an `int`,
and `List[float]`, returning a `float`. It is a built-in special case that user
code can't declare (ADR 0128 is unchanged for user code). An empty list
gives `0` or `0.0`. For `int` it adds left to right from `0`, and an overflow
of any partial sum traps. For `float` the sum `S(xs)` is a fixed tree that
depends only on the length: if it has at most 32 elements, four partial sums
`p0`–`p3`, where `pk` adds the elements whose index is `k` modulo 4 left to
right from `0.0`, and `S = (p0 + p1) + (p2 + p3)`; otherwise
`S(xs[..m]) + S(xs[m..])` with `m = 32 × ⌈len / 64⌉`. Vector and parallel
implementations compute the same tree (`conc.par.1`). *(ADR 0136,
ADR 0239)*

**type.coll.13** — A local declared with `:=` from `[]` or `{}` with no
expected type takes its type from the first statement that determines it:
`x.push(e)` (a `List`), `x.add(e)` (a `Set`), `x[k] = v` or `x[k] op= v`
with a number `v` (a `Map`, ADR 0251, ADR 0280), `x.push(k, e)` (a
`Map[K, List[E]]`, ADR 0341), `x = e`,
passing `x` where a parameter's type is known, returning `x`, or using it as
a field's value. Any other use before then, or none at all, is a
compile-time error whose help suggests `List[T]()`, `Map[K, V]()` or
`Set[T]()`. *(ADR 0143, ADR 0280)*

```ail fragment
mut out := []
out.push("a")          # out is List[str]
```

## Maps and sets

**type.map.1** — Iteration, `keys()`, `values()` and the text form of a map or
set follow insertion order, and `s.sorted()` is a stable sort of a set's
elements in that order. Assigning to an existing key keeps its
position. `remove` keeps the order of the other entries, and a key added again
later goes last. In a literal, a later duplicate replaces the value and keeps
the first position; duplicate keys or elements that are equal constant values
are a compile-time error. *(ADR 0131)*

**type.map.2** — Map keys and set elements MUST be `Hash` (`trait.auto.5`).
This is checked where the type is written or inferred: `Map[float, int]` is a
compile-time error that names the cause. *(ADR 0131, ADR 0133)*

**type.map.3** — `m[k]` reads the value for `k` and traps if `k` is missing;
the message shows the key's text form. An entry `m[k]` of a map place is a
place (`own.place.1` amended), but only when `m[k]` is the whole target of `=`
does it insert or replace, and when it is the whole target of `op=` a missing
`k` starts from zero (`type.map.7`). Every other use traps if `k` is missing:
- a step before the last one in a longer place (`m[k].f = v`, `m[k][j] = v`);
  `m[a][b] = v` needs `m[a]` to exist and inserts `b`;
- the receiver of a `mut self` method (`m[k].push(x)`);
- a `mut` argument, and `for mut x in m[k]`.

`k` is evaluated once, before the right-hand side. *(ADR 0131, ADR 0251)*

```ail fragment norun
mut m := Map[str, int]()
m["a"] = 1              # inserts
m["a"] = 2              # replaces; "a" keeps its position
m["a"] += 1             # 3
x := m["z"]             # traps: key "z" is missing
mut g := Map[int, Map[int, str]]()
g[1] = Map[int, str]()
g[1][2] = "x"           # inserts key 2 into g[1]
g[3][4] = "y"           # traps: g has no key 3
```

**type.map.7** — When `m[k]` is the whole target of an arithmetic `op=`
(`expr.op.2`) and `k` is missing, the operation starts from zero (`0`, `0.0`,
or the sized type's zero) and inserts `k` at the end. Only the last step
defaults: `m[a][b] += 1` still needs `m[a]`. `V` is numeric, since `op=` needs
an arithmetic type. *(ADR 0251)*

```ail fragment
mut counts := Map[str, int]()
for w in ["a", "b", "a"]
    counts[w] += 1      # {"a": 2, "b": 1}
```

**type.map.4** — Two maps are `==` when they have the same keys with equal
values (`V` MUST be `Eq`), and two sets when they have the same elements. Order
is ignored, so equal maps can print differently. Maps and sets aren't `Ord` or
`Hash`. The text form is `{"a": 1, "b": 2}` for a map, `{1, 2}` for a set, and
`{}` for an empty map or set. *(ADR 0131)*

**type.map.5** — `for x in s` iterates a set in insertion order, over the
value `s` had when the loop started (`expr.loop.4`, amended to allow a
`Set`). `for k, v in m` iterates a map's entries as pairs
(`type.tuple.5`); `for k in m`, with one name, is a compile-time error whose
help says to write `for k, v in m` or iterate `m.keys()`. `for mut` over a set, or over `m.keys()` or
`m.values()`, is a compile-time error. *(ADR 0131, ADR 0792)*

**type.map.6** — Maps and sets have these methods. *(ADR 0131)*
- Map: `m.len()` returns an `int`; `m.get(k)` returns a `V?`;
  `m.contains(k)` returns a `bool`; `m.remove(k)` (`mut self`) removes the
  entry and returns its value as a `V?`; `m.keys()` returns a `List[K]` and
  `m.values()` a `List[V]`, both new lists in insertion order.
- Set: `s.len()` returns an `int`; `s.add(x)` (`mut self`) returns whether `x`
  was added; `s.remove(x)` (`mut self`) returns whether `x` was there;
  `s.contains(x)` returns a `bool`; `s.sorted()` returns a `List[T]` (`T` MUST
  be `Ord`); `s.is_subset(t)` returns a `bool`.

**type.coll.16** — `s.union(t)`, `s.intersect(t)` and `s.difference(t)`
return a new `Set[T]` (`T` MUST be `Hash`): the elements in either set, in
both, or in `s` but not `t`. The result keeps `s`'s order, then (for `union`)
the new elements of `t` in order. A `List[T]` has the same three methods
taking a `List[T]`, so `m.keys().union(n.keys())` works; the result is still a
set. *(ADR 0340)*

**type.map.8** — On a `Map[K, List[T]]`, `m.push(k, x)` (`mut self`) appends
`x` to the list at `k`, and starts an empty list first when `k` is missing. On
a map with any other value type it is a compile-time error. *(ADR 0341)*

**type.map.12** — `m.get_or(k, d)` returns the value stored under `k`, or `d` when
`k` is missing; `d` has the value type and is always evaluated. It is the same
as `m.get(k) else d`. *(ADR 1087)*

**type.coll.27** — `xs.reversed()` returns a new list with the elements of `xs`
in reverse order and leaves `xs` as it is; `xs.reverse()` (`mut self`) reverses
in place. A list in descending order is `xs.sorted().reversed()`. *(ADR 1087)*

```ail rules=type.map.12,type.coll.27
fn main()
    m := {"a": 1}
    print(m.get_or("a", 0))                 # 1
    print(m.get_or("b", 0))                 # 0
    xs := [3, 1, 2]
    print(xs.reversed())                    # [2, 1, 3]
    print(xs.sorted().reversed())           # [3, 2, 1]
```

The diagnostics for `m.get(k, d)`, `contains_key` and `items()` suggest
`m.get_or(k, d)`, `contains`, and iterating `keys()` and indexing.

```ail fragment
mut seen := Set[str]()
first := seen.add("a")  # true
again := seen.add("a")  # false: already there
mut ages := {"ann": 31, "bob": 27}
print(ages.get("cy"))   # none
for k in ages.keys()
    print("{k}: {ages[k]}")
```

## Grids

**type.grid.1** — `Grid[T]` is a 2-D list held in one flat block, row by
row. `Grid.fill(rows, cols, v)` gives a `rows` by `cols` grid of `v`, and
`Grid.from(xs)` the grid of the lists `xs`; a negative size, or rows of
different lengths, trap. The type is the prelude's, read when a program names
`Grid` and doesn't declare it. *(ADR 0937)*

**type.grid.2** — `g.rows`, `g.cols` and `g.cells` (the elements, row by row,
as a `List[T]`) are read like fields. Assigning any of them is a
compile-time error. *(ADR 0937)*

**type.grid.3** — `g[i, j]` is the element at row `i`, column `j`; both are
`int`. It is a place: `g[i, j] = v` and `g[i, j] += v` change it when `g` is
`mut`. An index outside `0..g.rows` or `0..g.cols` traps (type.coll.4). Any
other index form on a grid is a compile-time error. *(ADR 0937)*

**type.grid.4** — `g.row(i)` is row `i` as a list, and `g.to_list()` the
rows as a list of lists. *(ADR 0937)*

**type.grid.5** — A grid is a value (own.val.1): `h := g` copies it. `==`
compares sizes and elements; printing shows its fields. *(ADR 0937)*

**type.grid.6** — On any other value, `x[a, b]` is `x[(a, b)]`: a map with
tuple keys reads `m[1, 2]`. *(ADR 0937)*

```ail rules=type.grid.1,type.grid.3,type.grid.4
fn main()
    mut dp := Grid.fill(3, 3, 1)
    for i in 1..3
        for j in 1..3
            dp[i, j] = dp[i - 1, j] + dp[i, j - 1]
    print(dp[2, 2])         # 6
    print(dp.row(2))        # [1, 3, 6]
```

```ail error=type.grid.2
fn main()
    mut g := Grid.fill(2, 2, 0)
    g.rows = 3                  # error: a grid's size fields are never assigned
```

```ail rules=type.grid.5,type.grid.6
fn main()
    g := Grid.fill(2, 2, 0)
    mut h := g
    h[0, 0] = 9
    print(g[0, 0])          # 0: `h := g` copied the grid
    print(g == Grid.fill(2, 2, 0))   # true
    mut m := Map[(int, int), str]()
    m[1, 2] = "x"
    print(m[1, 2])          # x: the same as m[(1, 2)]
```

**type.heap.1** — `Heap[T]` is a priority queue of elements of an `Ord` type
`T`. `Heap[int]()` is empty and gives the smallest element first;
`Heap[int](max_first: true)` gives the largest first. The type is the
prelude's, read when a program names `Heap` and doesn't declare it. *(ADR 1086)*

**type.heap.2** — `Heap.from(xs)` is the min-first heap of the list `xs`, and
`Heap.from_max(xs)` the max-first one, built in linear time. *(ADR 1086)*

**type.heap.3** — `h.push(x)` adds `x`. `h.pop()` removes and returns the
element that comes first, and `h.peek()` returns it without removing; both
are `T?` and `none` when the heap is empty. Both ends of an unordered pair of
equal elements may come out in either order. *(ADR 1086)*

**type.heap.4** — `h.len()` and `h.is_empty()` count the elements.
`h.to_sorted()` is the list of all elements in the order `pop` returns them,
and leaves `h` unchanged. *(ADR 1086)*

**type.heap.5** — A heap is a value (own.val.1): `g := h` copies it. Its
`items` field holds the elements in heap order, which is no particular
order; use `to_sorted()` to read them in order. *(ADR 1086)*

```ail rules=type.heap.1,type.heap.2,type.heap.3,type.heap.4,type.heap.5
fn main()
    mut h := Heap[int]()
    h.push(5)
    h.push(1)
    h.push(3)
    print(h.peek())              # 1
    print(h.pop())               # 1
    print(h.to_sorted())         # [3, 5]
    mut m := Heap[int](max_first: true)
    m.push(2)
    m.push(9)
    print(m.pop())               # 9
    print(Heap.from([4, 2, 8]).to_sorted())      # [2, 4, 8]
    print(Heap.from_max([4, 2, 8]).to_sorted())  # [8, 4, 2]
    g := h
    h.push(0)
    print(g.len())               # 2: `g := h` copied the heap
```

**type.deque.1** — `Deque[T]` is a double-ended queue. `Deque[int]()` is
empty and `Deque.from(xs)` holds the elements of `xs` in order. The type is
the prelude's, read when a program names `Deque` and doesn't declare it.
*(ADR 1086)*

**type.deque.2** — `d.push_back(x)` and `d.push_front(x)` add an element at
an end. `d.pop_back()` and `d.pop_front()` remove and return the element at
that end as a `T?`, `none` when empty. Each is O(1) amortised. *(ADR 1086)*

**type.deque.3** — `d.first()` and `d.last()` return the end elements as a
`T?` without removing them. `d.len()` and `d.is_empty()` count the elements.
*(ADR 1086)*

**type.deque.4** — `d.at(i)` is the element at position `i` counted from the
front, in O(1); a position outside `0..d.len()` traps. `d.to_list()` is the
elements, first to last, as a list. *(ADR 1086)*

**type.deque.5** — A deque is a value (own.val.1): `e := d` copies it. Its
`front` and `back` fields are the representation and aren't read. *(ADR 1086)*

```ail rules=type.deque.1,type.deque.2,type.deque.3,type.deque.4,type.deque.5
fn main()
    mut d := Deque[int]()
    d.push_back(2)
    d.push_back(3)
    d.push_front(1)
    print(d.to_list())           # [1, 2, 3]
    print(d.first())             # 1
    print(d.last())              # 3
    print(d.at(1))               # 2
    print(d.pop_front())         # 1
    print(d.pop_back())          # 3
    e := d
    d.push_back(9)
    print(e.len())               # 1: `e := d` copied the deque
    print(Deque.from([7, 8]).len())  # 2
```

```ail norun rules=type.deque.4
fn main()
    d := Deque.from([1])
    print(d.at(5))               # traps: position 5 is outside 0..1
```

## Examples

Each block below is a conformance example (ADR 0002). Its `rules=` list names
the rules it demonstrates; `norun` and `error=` blocks are checked, not run.

```ail rules=type.num.1,type.num.2,type.num.3,type.num.4
fn signed() i8
    -128

fn unsigned() u64
    4000000000

fn main()
    print(signed())          # -128
    print(unsigned())        # 4000000000
    print(7)                 # an int literal
    print(1.5)               # a float literal
```

```ail rules=type.num.5
fn main()
    print(7 / 2)             # 3
    print(7 % 3)             # 1
    print(-(2 * 3))          # -6
    print(2 < 3)             # true
```

```ail rules=type.num.6,type.num.7
fn main()
    print(0.1 + 0.2)         # 0.30000000000000004
    print(1.0 / 0.0)         # inf: no trap
    print(0.0 / 0.0 == 0.0 / 0.0)   # false: nan is unequal to itself
    print("{1000.0} {0.0001} {-0.0} {1e16} {1e-5}")  # 1000.0 0.0001 -0.0 1e+16 1e-05
```

```ail rules=type.num.8
fn main()
    print("2.5E-3".to_float())   # 0.0025
    print("1e999".to_float())    # inf
    print("+1".to_float())       # none
    print(".5".to_float())       # none
```

```ail rules=type.num.9
fn main()
    x := -0.4
    print(x.round())             # -0.0: floor, ceil, round, sqrt keep the sign of zero
    print(2.5.round())           # 3.0: halves round away from zero
    print(9.0.sqrt())            # 3.0
    print(3.9.to_int())          # 3: truncates toward zero
    print(7.to_float())          # 7.0
```

```ail rules=type.num.10
fn one() float
    1                            # an int literal in a float position is a float literal

fn main()
    print(one())                 # 1.0
```

```ail rules=type.num.11
fn main()
    n := 0.0 / 0.0
    print([2.0, n, 1.0].sorted())  # [1.0, 2.0, nan]: nan sorts last
    print(n < 1.0)                 # false: the operator is IEEE
```

```ail rules=type.num.12
fn main()
    x := 9223372036854775807
    print(x.checked_add(1))        # none
    print(x.wrapping_add(1))       # -9223372036854775808
    print(x.saturating_add(1))     # 9223372036854775807
    print((-5).checked_div(0))     # none: a zero divisor
    print(255.to_hex())            # ff
```

```ail rules=type.num.13
fn low() i8
    -128                           # the literal is an i8 because the return type is i8

fn byte(n u8) u8
    n

fn main()
    print(low())                   # -128
    print(byte(255))               # 255
```

```ail error=type.num.13
fn too_big() u8
    256                            # error: 256 doesn't fit in u8
```

```ail rules=type.num.14
fn add8(a i8, b i8) i8
    a + b

fn main()
    print(add8(100, 27))           # 127
    print(-(5.to_i8()))            # -5: unary minus on a signed size
    print(f32_half(3.0))           # 1.5
    print(200.to_hex())            # c8: same methods as int, in the type

fn f32_half(x f32) f32
    x.sqrt() * x.sqrt() / 2
```

```ail error=trait.op.3
fn mixed(a i8, b i64) i64
    a + b                          # error: no implicit widening between i8 and i64
```

```ail rules=type.num.15
fn main()
    a := 300.to_u16()
    b := a.to_u32() * 2
    print(b)                       # 600
    c := 0.5.to_f32() + 1
    print(c)                       # 1.5
    print(3.to_float())            # 3.0
```

```ail rules=type.num.16
fn main()
    print(0.0.sin())                # 0.0
    print(1.0.exp())                # 2.718281828459045
    print(100.0.log10())            # 2.0
    print(2.0.pow(10.0))            # 1024.0
    print(3.0.hypot(4.0))           # 5.0
    print(27.0.cbrt())              # 3.0
    print(1.0.atan2(1.0) * 4.0)     # 3.141592653589793
    print((0.0 / 0.0).pow(0.0))     # 1.0: pow(x, 0) is 1 even for nan
```

```ail rules=type.struct.1,type.struct.2,type.struct.3,type.struct.4,type.struct.6
struct User
    id int
    name str
    email str?

fn main()
    u := User(id: 7, name: "Ada", email: none)
    v := User(name: "Lin", email: "l@x", id: 8)   # named arguments in any order
    id := 9
    name := "Kim"
    w := User(id, name, email: none)              # shorthand and named together
    print(u.name)                                 # Ada
    print(v.id + w.id)                            # 17
```

```ail error=type.struct.4
struct User
    id int
    name str

fn main()
    u := User(id: 1)                              # error: `name` is not set
    print(u.id)
```

```ail error=type.struct.5
struct Pair
    a int
    a str                                         # error: duplicate field name
```

```ail rules=type.struct.5,type.struct.7
struct Pos
    row int
    col int

fn main()
    row := 2
    col := 5
    p := Pos(row, col)                            # positional: both fields are int
    print(p.col)                                  # 5
```

```ail rules=type.enum.1,type.enum.2,type.enum.3,type.enum.4,type.enum.6
enum Shape
    Circle(r float)
    Rect(w float, h float)
    Empty

fn area(s Shape) float
    match s
        Circle(r) => 3.0 * r * r                  # unqualified: the type is known
        Rect(w, h) => w * h
        Empty => 0.0

fn main()
    c := Shape.Circle(r: 1.0)                     # qualified: no context yet
    r := Shape.Rect(w: 2.0, h: 3.0)               # named: both fields are float
    print(area(c))                                # 3.0
    print(area(r))                                # 6.0
    print(area(Empty))                            # 0.0
```

```ail error=type.enum.5
enum Dir
    Up
    Down
    Up                                            # error: duplicate variant name
```

```ail rules=type.opt.1,type.opt.2,type.opt.3,type.opt.4,type.opt.5
fn find(id int) int?
    if id > 0
        return id * 10
    none

fn main()
    print(find(2))                                # 20
    print(find(-1))                               # none: none is the absent value
    x := find(1) else 0                           # else gives a plain int
    print(x)                                      # 10
```

```ail rules=type.opt.6,type.opt.7
fn lookup(m Map[str, int?], k str) int??
    m.get(k)

fn flat(m Map[str, int?], k str) int?
    lookup(m, k) else none

fn find(id int) int?
    if id > 0
        return id
    none

fn main()
    mut m := Map[str, int?]()
    m["a"] = 1
    m["b"] = none
    print(flat(m, "a"))                           # 1
    print(flat(m, "b"))                           # none: a present but empty value
    print(flat(m, "c"))                           # none: missing key
    print(lookup(m, "b") == none)                 # false: the present-but-empty value is not none
```

```ail error=expr.call.2 rules=type.opt.8
fn show(x int??)
    print(x)

fn find(id int) int?
    if id > 0
        return id
    none

fn main()
    v := find(1)                                  # an int?
    show(v)                                       # error: int? does not convert to int??
```

```ail rules=type.bool.1
fn main()
    t := true
    f := false
    print(t and f)                                # false
    print(t or f)                                 # true
```

```ail rules=type.str.1,type.str.2,type.str.3
fn main()
    s := "héllo"
    n := 42
    ok := true
    print(s.len())                                # 5 characters
    print(s.chars().len())                        # 5
    print("{n} {ok} {s}")                         # 42 true héllo
    print(s == "héllo")                           # true: equality compares UTF-8 bytes
    print("abc" < "abd")                          # true
    c := 'é'
    print(c)                                      # é
```

```ail error=type.str.2
fn main()
    s := "abc"
    print(s[0])                                   # error: str has no indexing
```

```ail rules=type.str.4,type.str.5,type.str.6
fn main()
    parts := "a,,b".split(",")
    print(parts.len())                            # 3: empty pieces are kept
    print("a/b/c".split('/').len())               # 3: a char separator
    print("k=v=w".split("=", 2))                  # ["k", "v=w"]: at most 2 pieces
    print("-42".to_int())                         # -42
    print("4x".to_int())                          # none
    print("x".to_int() else 0)                    # 0
    print("path/to".contains('/'))                # true
    print("pre-body".starts_with("pre-"))         # true
```

```ail rules=type.str.7,type.str.8,type.str.9,type.str.10
fn main()
    print("a\r\nb\n".lines().len())               # 2
    print("  hi ".trim())                         # hi
    print("key=v".strip_prefix("key=") else "")   # v
    print("file.txt".strip_suffix(".md") else "") # none: the empty string here
    print("h\u{e9}".chars().len())                # 2
    print("3.5".to_float() else 0.0)              # 3.5
    print(255.to_hex())                           # ff
    print("7".pad_left(3, '0'))                   # 007
    p := "ab".pad_right(4, ' ')
    print("{p}|")                                 # ab  |
    print("ab".repeat(3))                         # ababab
```

```ail rules=type.str.11
fn main()
    x := 2.5
    print("{x:.0} {x:.2}")                        # 2 2.50: half to even
    n := 255
    print("{n:x}")                                # ff
```

```ail rules=type.str.12,type.str.13,type.str.14
fn main()
    s := "Héllo"
    print(s.len())                                # 5
    print("héllo".byte_len())                     # 6
    print("e\u{301}".len())                       # 2: grapheme clusters are not counted
    print(s.slice(1, 3))                          # él
    print(s.char_at(1) else '?')                  # é
    print(s.find("llo") else -1)                  # 2
    print(s.replace("l", "L"))                    # HéLLo
    print("Ünï".to_lower())                       # ünï
    print("straße".to_upper())                    # STRASSE is not produced: ß is unchanged
```

```ail norun rules=type.str.15
fn main()
    # A trap on purpose: building a 2 GiB string exceeds the 2^30-byte limit.
    s := "x".repeat(2147483648)
    print(s.len())
```

```ail rules=type.char.1,type.char.2
fn main()
    c := 'f'
    print(c.is_alpha())                           # true
    print('7'.is_digit())                         # true
    print(' '.is_space())                         # true
    print('A'.is_upper())                         # true
    print('a'.to_upper())                         # A
    print('ß'.to_upper())                         # ß: a multi-character mapping is unchanged
    print(c == 'f')                               # true
```

```ail rules=type.char.3,type.char.4
fn main()
    c := 'f'
    print(c.to_digit(16) else 0)                  # 15
    print(c.to_digit(10))                         # none
    print(c.code())                               # 102
    print(char.from_code(0x41))                   # A
    print(char.from_code(0xD800))                 # none: a surrogate
```

```ail rules=type.coll.1,type.coll.2,type.coll.3,type.coll.8
fn main()
    ports := {"http": 80, "https": 443}
    primes := [2, 3, 5, 7]
    seen := Set[int]()
    counts := Map[str, int]()
    names := List[str]()
    empty := Map[str, int]()
    print(primes.len())                           # 4
    print(ports["https"])                         # 443
    print(seen.len() + counts.len() + names.len() + empty.len())   # 0
```

```ail rules=type.coll.4,type.coll.5,type.coll.10
fn main()
    xs := [10, 20, 30, 40]
    print(xs[1])                                  # 20
    print(xs[1..3])                               # [20, 30]
    print(xs[1..=2])                              # [20, 30]
    print(xs[2..])                                # [30, 40]
    print(xs[..2])                                # [10, 20]
    print(xs.len())                               # 4
```

```ail error=type.coll.4
fn main()
    xs := [1, 2, 3]
    print(xs[-1])                                 # error: a negative index literal
```

```ail rules=type.coll.6,type.coll.7,type.coll.13
fn main()
    mut out := []
    out.push("a")                                 # out is List[str] from this use
    out.push("b")
    mut ids := [3, 1, 2]
    ids.swap(0, 2)                                # [2, 1, 3]
    ids.swap(1, 1)                                # does nothing
    print(out.join(","))                          # a,b
    print(ids)                                    # [2, 1, 3]
```

```ail error=type.coll.13
fn main()
    xs := []
    print(xs.len())                               # error: the element type is never determined
```

```ail rules=type.coll.9
fn main()
    mut xs := ["a", "b"]
    print(xs.join(", "))                          # a, b
    print([1, 2, 3].join(""))                     # 123
    print(xs.index_of("b") else -1)               # 1
    print(xs.contains("z"))                       # false
    print(xs.pop() else "")                       # b
    print(xs.last() else "")                      # a
```

```ail rules=type.coll.11,type.coll.12,type.coll.15
fn main()
    xs := [3, 1, 2]
    print(xs.sorted())                            # [1, 2, 3]
    print(xs.min() else 0)                        # 1
    print(xs.max() else 0)                        # 3
    print(xs.sum())                               # 6
    print([0.5, 0.25].sum())                      # 0.75
    print(xs.take(2))                             # [3, 1]
    mut ys := [2.0, 1.0]
    ys.sort()
    print(ys)                                     # [1.0, 2.0]
```

```ail rules=type.coll.14
fn main()
    mut xs := [1, 2, 3]
    xs.insert(1, 9)                               # [1, 9, 2, 3]
    xs.insert(xs.len(), 4)                        # append
    n := xs.remove_at(0)                          # 1
    xs.extend([5, 6])
    print(n)                                      # 1
    print(xs.first() else 0)                      # 9
    xs.reverse()
    print(xs)
    xs.clear()
    print(xs.len())                               # 0
```

```ail rules=type.coll.16
fn main()
    a := {1, 2, 3}
    b := {3, 4}
    print(a.union(b))                             # {1, 2, 3, 4}
    print(a.intersect(b))                         # {3}
    print(a.difference(b))                        # {1, 2}
    m := {"x": 1, "y": 2}
    print(m.keys().union(["z"]))                  # {"x", "y", "z"}: the result is a set
```

```ail rules=type.map.1,type.map.2,type.map.3,type.map.7
fn main()
    mut m := Map[str, int]()
    m["a"] = 1
    m["b"] = 2
    m["a"] = 5                                    # keeps the first position
    m["a"] += 1                                   # 6
    m["c"] += 1                                   # a missing key starts from zero
    print(m.keys())                               # [a, b, c]
    mut g := Map[int, Map[int, str]]()
    g[1] = Map[int, str]()
    g[1][2] = "x"                                 # inserts key 2 into g[1]
    print(g[1][2])                                # x
    mut counts := Map[str, int]()
    for w in ["a", "b", "a"]
        counts[w] += 1
    print(counts["a"])                            # 2
```

```ail error=type.map.2
fn main()
    m := Map[float, int]()                        # error: float is not Hash
    print(m.len())
```

```ail norun rules=type.map.3
fn main()
    m := {"a": 3}
    print(m["z"])                                 # traps: the key "z" is missing
```

```ail rules=type.map.4,type.map.5,type.map.6
fn main()
    a := {"a": 1, "b": 2}
    b := {"b": 2, "a": 1}
    print(a == b)                                 # true: order is ignored
    print({1, 2} == {2, 1})                       # true
    e := Map[str, int]()
    print(e)                                      # {}
    print(a.get("cy") else 0)                     # 0
    print(a.contains("a"))                        # true
    print(a.len())                                # 2
    for k in a.keys()
        print("{k}: {a[k]}")
    s := {3, 1, 2}
    for x in s
        print(x)                                  # 3, 1, 2: insertion order
    print(s.is_subset({1, 2, 3, 4}))              # true
    print(s.sorted())                             # [1, 2, 3]
```

```ail error=type.map.5
fn main()
    m := {"a": 1}
    for k in m                                    # error: write `for k, v in m` or m.keys()
        print(k)
```

```ail rules=type.map.8
fn main()
    mut groups := Map[str, List[int]]()
    groups.push("odd", 1)
    groups.push("odd", 3)
    groups.push("even", 2)
    print(groups["odd"])                          # [1, 3]
    mut seen := Set[str]()
    print(seen.add("a"))                          # true
    print(seen.add("a"))                          # false: already there
    print(seen.remove("a"))                       # true
```
