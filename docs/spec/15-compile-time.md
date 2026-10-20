# 15. Compile-time execution and metaprogramming

Chapter key: `ct`

> **Status:** Drafted (partial). Specified: constants computed by calling
> functions, the rules of compile-time evaluation, derive generators and
> generated code. The bootstrap implements `ct.const.1–3` (with `if`,
> `match` and `?` in a constant left to a function, `bootstrap.unsupported`);
> `ct.eval.3–4`, `ct.derive.*` and `ct.gen.*` are not implemented yet. Design
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

```ail
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

## Evaluation

**ct.eval.1** — Compile-time evaluation is deterministic: it depends only on
the program's text. It reaches no capability (`ct.const.2`), and map and set
iteration follow insertion order (`type.map.1`, `type.map.5`). *(ADR 0226)*

**ct.eval.2** — Compile-time evaluation has exactly the semantics of run
time, including overflow traps and float arithmetic. Because every target is
64-bit and little-endian (`plat.endian.1`, `plat.ptr.*`), the value doesn't
depend on the target. A trap while computing a constant or a derive is a
compile-time error that names the trap's rule and the compile-time call
stack. The bootstrap computes constants before `main` and reports such a
trap at the constant before `main` starts (`mod.const.3`). *(ADR 0225,
ADR 0226, ADR 0032)*

**ct.eval.3** — Each constant and each derive (`ct.derive.2`) MAY take at
most 100,000,000 evaluation steps (one per expression evaluated) and 1 GiB
of live values, within the run-time call-depth limit. Exceeding a limit is a
compile-time error that names the limit, the constant or `impl`, and the call
stack where evaluation stopped. *(ADR 0226)*

**ct.eval.4** — An implementation MAY reuse a result computed by an earlier
compilation only when the constant's or derive's code and every function the
evaluation entered are unchanged. *(ADR 0226, ADR 0224)*

## Derives

**ct.derive.1** — `@derive(f)` before a `trait` declaration names the
trait's *derive generator*: a top-level function `f` of type
`fn(TypeInfo) str ! DeriveError` (`refl.info.1`). *(ADR 0227)*

**ct.derive.2** — An `impl Trait for T` with no method definitions derives
the trait (`trait.derive.1`). If the trait has a derive generator, the
generator is called at compile time with the `TypeInfo` of `T`, and the text
it returns is the body of the `impl` (`ct.gen.1`). *(ADR 0227)*

```ail skip=bootstrap
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
    print(Version(1, 2).describe())    # Version with major, minor
```

**ct.derive.3** — A derive generator MUST be compile-time-safe
(`ct.const.2`), and its evaluation follows `ct.eval.1–4`. *(ADR 0227)*

**ct.derive.4** — A generator's `DeriveError`, a trap in it, and generated
code that is not valid (`ct.gen.1–2`) are compile-time errors reported at the
deriving `impl`. *(ADR 0227)*

## Generated code

**ct.gen.1** — Generated text MUST consist only of method definitions for the
deriving `impl`. It is parsed, laid out canonically, and checked by every
rule that applies to a hand-written `impl` of the trait. *(ADR 0228)*

**ct.gen.2** — Generated code resolves names in the scope of the deriving
`impl`: its module, that module's imports, and the private members the
module may use (`mod.vis.3`). The trait's module is also reachable by its
qualified name. *(ADR 0228)*

**ct.gen.3** — A diagnostic inside generated code is reported at the
deriving `impl`, with a note at the position in the generated text, written
`gen:impl:TYPE.TRAIT:LINE:COL`. *(ADR 0228)*

**ct.gen.4** — Each generated method has the stable id it would have if
written by hand (`api.id.*`) and an origin naming the `impl` and the
generator. `ail api --expanded` includes generated methods, their origin,
and their text. *(ADR 0228)*
