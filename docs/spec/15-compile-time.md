# 15. Compile-time execution and metaprogramming

Chapter key: `ct`

> **Status:** Drafted (partial). Specified: constants computed by calling
> functions, the rules of compile-time evaluation, derive generators and
> generated code. The compiler implements `ct.const.1–3` (with `if`,
> `match` and `?` in a constant left to a function, `bootstrap.unsupported`),
> `ct.derive.1–4` and `ct.gen.1–4` (ADRs 0306–0309). `ct.eval.3`'s step limit is counted for
> constants, in statements run (ADR 0742; natively through `.ct` copies of the
> functions a constant calls, so run-time code is unchanged); its memory limit and a derive's
> limits aren't counted yet. `ct.eval.4` is not implemented yet. Design
> notes: [compile-time-execution.md](../design/compile-time-execution.md),
> [metaprogramming.md](../design/metaprogramming.md).

## Constants

**ct.const.1** — The value of a module constant (`mod.const.1`) MAY be any
expression, including calls of functions and methods, field access,
indexing and ranges. It is computed at compile time. *(ADR 0225, amending
ADR 0132)*

**ct.const.2** — Every function a constant's value may call, directly or
through other functions, MUST be *compile-time-safe*: its effect set
(`eff.infer.1`) contains none of `io`, `unsafe`, `spawn` or a capability.
`fail` and `mutates` are allowed. The diagnostic names the shortest chain
of calls to each forbidden effect. A function is compile-time-safe exactly
when `@pure` (`eff.pure.1`) would accept it. *(ADR 0225, ADR 0164)*

```ail rules=ct.const.1
SQUARES := squares(5)
WORDS := "fn if else".split(" ")
ORIGIN := Point(x: 0, y: 0).moved(3, 4)

struct Point
    x int
    y int

fn Point.moved(self, dx int, dy int) Point
    Point(x: self.x + dx, y: self.y + dy)

fn squares(n int) List[int]
    mut out := List[int]()
    for i in 0..n
        out.push(i * i)
    out

fn main()
    print(SQUARES)
    print(WORDS[1])
    print(ORIGIN)
```

```ail error=ct.const.2
GREETING := greet("ail")

fn greet(name str) str
    print("greeting {name}")
    "hello {name}"

fn main()
    print(GREETING)
```

**ct.const.3** — A constant MUST NOT hold a failure: if its value has a
result type (`T ! E`), it is a compile-time error. `e?` in a constant's
value makes a failure of `e` a compile-time error at the constant; `else`
supplies a fallback value. *(ADR 0225)*

```ail error=ct.const.3
LIMIT := parse_limit("12")

error enum LimitError
    Bad(text str)

fn parse_limit(text str) int ! LimitError
    n := text.to_int() else return LimitError.Bad(text)
    n

fn main()
    print(LIMIT)
```

**ct.const.4** — A constant's type MUST NOT contain `ref` or a `unique`
type: a constant is an immutable value copied into the program. *(ADR 0225)*

```ail error=ct.const.4 rules=ct.const.4
unique struct Token
    id int

TOKEN := Token(id: 1)       # a `unique` value can't be copied into every use

fn main()
    print(1)
```

```ail error=mod.const.2 rules=ct.const.4
struct Box2
    v int

BOX := ref Box2(v: 1)

fn main()
    print(BOX.v)
```

## Evaluation

**ct.eval.1** — Compile-time evaluation is deterministic: it depends only on
the program's text. It reaches no capability (`ct.const.2`), and map and set
iteration follow insertion order (`type.map.1`, `type.map.5`). *(ADR 0226)*

```ail rules=ct.eval.1
LIMIT := 2 * 3

fn main()
    print(LIMIT)
```

**ct.eval.2** — Compile-time evaluation has exactly the semantics of run
time, including overflow traps and float arithmetic. Because every target is
64-bit and little-endian (`plat.endian.1`, `plat.ptr.*`), the value doesn't
depend on the target. A trap while computing a constant or a derive is a
compile-time error that names the trap's rule and the compile-time call
stack. The compiler computes constants before `main` and reports such a
trap at the constant before `main` starts (`mod.const.3`). *(ADR 0225,
ADR 0226, ADR 0032)*

```ail tool=run,FILE status=1 expect=expr.arith.3 rules=ct.eval.2
Z := 1 / 0

fn main()
    print(Z)
```

**ct.eval.3** — Each constant and each derive (`ct.derive.2`) MAY take at
most 100,000,000 evaluation steps (one per expression evaluated) and 1 GiB
of live values, within the run-time call-depth limit. Exceeding a limit is a
compile-time error that names the limit, the constant or `impl`, and the call
stack where evaluation stopped. *(ADR 0226)*

```ail norun rules=ct.eval.3
# norun: the limit is a run-time trap while the constant is computed before `main`, which the gate
# does not assert. the `misc` gate suite checks the same trap with AIL_CT_MAX_STEPS set low.
fn spin(n int) int
    mut i := 0
    while i < n
        i += 1
    i

BIG := spin(1000000000)     # over 100,000,000 steps: computing `BIG` stops with ct.eval.3

fn main()
    print(BIG)
```

**ct.eval.4** — An implementation MAY reuse a result computed by an earlier
compilation only when the constant's or derive's code and every function the
evaluation entered are unchanged. *(ADR 0226, ADR 0224)*

```ail norun rules=ct.eval.4
SQUARE := square(4)

fn square(n int) int
    n * n

fn main()
    print(SQUARE)
```

## Derives

**ct.derive.1** — `@derive(f)` before a `trait` declaration names the
trait's *derive generator*: a top-level function `f` of type
`fn(TypeInfo) str ! DeriveError` (`refl.info.1`). *(ADR 0227)*

**ct.derive.2** — An `impl Trait for T` with no method definitions derives
the trait (`trait.derive.1`). If the trait has a derive generator, the
generator is called at compile time with the `TypeInfo` of `T`, and the text
it returns is the body of the `impl` (`ct.gen.1`). *(ADR 0227)*

```ail rules=ct.derive.1,ct.derive.2
@derive(derive_describe) trait Describe
    fn describe(self) str

fn derive_describe(t TypeInfo) str ! DeriveError
    mut names := List[str]()
    for f in t.fields
        names.push(f.name)
    "fn describe(self) str\n    \"{t.name} with {names.join(", ")}\"\n"

struct Version
    major int
    minor int

impl Describe for Version

fn main()
    print(Version(major: 1, minor: 2).describe()) # Version with major, minor
```

The compiler runs generators while it loads the program, before checking
it: a generator, and what it calls, can't use what other derives generate.
*(ADR 0306)*

**ct.derive.5** — A `FieldInfo` carries the field's `##` doc comment in
`doc` (`api.doc.1`): the comment lines joined by `\n`, without the `##`
marks, or the empty string when the field has none. A generator can turn the
comments into help text or schema descriptions. *(ADR 0776)*

```ail rules=ct.derive.5
@derive(derive_help) trait Help
    fn help(self) str

fn derive_help(t TypeInfo) str ! DeriveError
    lines := ["{f.name}={f.doc}" for f in t.fields]
    "fn help(self) str\n    \"{lines.join("|")}\"\n"

struct Opts
    ## Input file.
    path str
    verbose bool

impl Help for Opts

fn main()
    print(Opts(path: "a", verbose: false).help()) # path=Input file.|verbose=
```

**ct.derive.6** — Generated code is read with the `use` lines of its
module's files, so a generator can name `args.flag` for a program that has
`use std.args`. *(ADR 0777)*

**ct.derive.3** — A derive generator MUST be compile-time-safe
(`ct.const.2`), and its evaluation follows `ct.eval.1–4`. The diagnostic
names the chain of calls to each forbidden effect. *(ADR 0227, ADR 0306)*

```ail error=ct.derive.3
@derive(derive_loud) trait Loud
    fn loud(self) str

fn derive_loud(t TypeInfo) str ! DeriveError
    print("deriving {t.name}")
    "fn loud(self) str\n    \"{t.name}!\"\n"

struct Bell
    size int

impl Loud for Bell

fn main()
    print(Bell(size: 1).loud())
```

**ct.derive.4** — A generator's `DeriveError`, a trap in it, and generated
code that is not valid (`ct.gen.1–2`) are compile-time errors reported at the
deriving `impl`. *(ADR 0227)*

```ail error=ct.derive.4
@derive(derive_size) trait Size
    fn size(self) int

fn derive_size(t TypeInfo) str ! DeriveError
    if t.kind == TypeKind.Enum
        return DeriveError("`Size` needs a struct; `{t.name}` is an enum")
    "fn size(self) int\n    {t.fields.len()}\n"

enum Light
    On
    Off

impl Size for Light

fn main()
    print(Light.On.size())
```

## Generated code

**ct.gen.1** — Generated text MUST consist only of method definitions for the
deriving `impl` and associated functions `fn T.name(...)` of its type `T`,
which are declared at the top level of the `impl`'s module. It is parsed,
laid out canonically, and checked by every rule that applies to a
hand-written `impl` of the trait and to a hand-written associated function.
*(ADR 0228, ADR 0307)*

```ail error=ct.gen.1
@derive(derive_bad) trait Bad
    fn bad(self) str

fn derive_bad(t TypeInfo) str ! DeriveError
    "struct Extra\n    a int\n"

struct Pal
    a int

impl Bad for Pal

fn main()
    print(Pal(a: 1).bad())
```

**ct.gen.2** — Generated code resolves names in the scope of the deriving
`impl`: its module, that module's imports, and the private members the
module may use (`mod.vis.3`). The trait's module is also reachable by its
qualified name. *(ADR 0228)*

```ail rules=ct.gen.2
@derive(derive_hello) trait Hello
    fn hello(self) str

fn greeting() str
    "hi"

fn derive_hello(t TypeInfo) str ! DeriveError
    "fn hello(self) str\n    \"{greeting()} {t.name}\"\n"

struct Pal
    a int

impl Hello for Pal

fn main()
    print(Pal(a: 1).hello())
```

**ct.gen.3** — A diagnostic inside generated code is reported at the
deriving `impl`, with a note at the position in the generated text, written
`gen:impl:TYPE.TRAIT:LINE:COL`. *(ADR 0228)*

```ail error=type.struct.6 rules=ct.gen.3
@derive(derive_first) trait First
    fn first(self) str

fn derive_first(t TypeInfo) str ! DeriveError
    "fn first(self) str\n    {t.name}.nope\n"

struct Pair
    a int

impl First for Pair

fn main()
    print(Pair(a: 1).first())
```

**ct.gen.4** — Each generated method has the stable id it would have if
written by hand (`api.id.*`) and an origin naming the `impl` and the
generator. `ail api --expanded` includes generated methods, their origin,
and their text. *(ADR 0228)* In the compiler, each generated function sits
under its `impl`'s `methods` with `origin` (`kind`, `impl`, `generator`,
`at`) and `text`; the `impl` gets `derived_by`, the generator's id, or
`built-in` for an empty `impl` of `Eq`, `Ord` or `ToStr` (OQ-096). Without
`--expanded` the outline is unchanged. *(ADR 0309)*

```ail tool=api,--expanded,FILE expect=origin rules=ct.gen.4
@derive(derive_describe) trait Describe
    fn describe(self) str

fn derive_describe(t TypeInfo) str ! DeriveError
    mut names := List[str]()
    for f in t.fields
        names.push(f.name)
    "fn describe(self) str\n    \"{t.name} with {names.join(", ")}\"\n"

struct Version
    major int
    minor int

impl Describe for Version

fn main()
    print(Version(major: 1, minor: 2).describe())
```
