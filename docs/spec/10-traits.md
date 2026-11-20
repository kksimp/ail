# 10. Methods and traits

Chapter key: `trait`

> **Status:** Drafted (partial). Specified: methods, associated functions,
> placement, traits, default methods, explicit implementations, coherence,
> supertraits, associated functions in traits, call resolution, `Self`,
> operator traits, `any` dynamic dispatch, and the standard traits `Eq`,
> `Ord`, `ToStr`, `Hash` and `Default`, and the arithmetic operator traits.
> Not yet designed: associated types (OQ-037), an indexing operator trait
> (`trait.op.1`), and custom hashing beyond the `uses` form (`trait.hash.1`,
> ADR 0955).

## Methods

**trait.method.1** — A method is declared as `fn Type.name(self, ...)` or
`fn Type.name(mut self, ...)`. With `self`, the receiver is read-only. With
`mut self`, the method may mutate its receiver. *(ADR 0072)*

**trait.method.2** — A method is called as `value.name(arguments)`.
*(ADR 0072)*

**trait.method.3** — The receiver of a method declared with `mut self` MUST be
a mutable place. The call is not marked with `mut` (an exception to
`own.param.2`). Exclusivity (`own.excl.*`) applies between the receiver and
the arguments. *(ADR 0075, ADR 0064)*

```ail skip=context
fn Circle.scale(mut self, k f64)
    self.r = self.r * k

mut c := Circle(r: 1.0)
c.scale(2.0)
```

**trait.method.4** — A declaration `fn Type.name(...)` whose first parameter is
not `self` declares an associated function, called as `Type.name(arguments)`.
*(ADR 0082)*

**trait.method.5** — Methods and associated functions of a type MUST be
declared in the package that declares the type, so `fn int.name(self)` on a
built-in type is an error whose help says to write a free function.
*(ADR 0082, ADR 1169)*

**trait.method.6** — Inside a method declared with `self` or `mut self`, every
field and method of the receiver is in scope by its bare name: `x` means
`self.x` and `f(args)` means `self.f(args)`. The methods are the type's own
and those of the traits it implements; in a trait's default method, they are
the trait's methods. `self.x` stays valid and is the canonical form: `ail fmt`
writes `self.x` and `self.f(args)` for a bare member it can see (fmt.self).
A bare member that is a shorthand argument of a constructor (`Pt(x, y)`) is
written `Pt(x: self.x, y: self.y)`, so it stays named (type.struct.7); one whose
constructor is declared in another file is left bare.
Associated functions see no members by bare name. *(ADR 0141, ADR 1081)*

**trait.method.7** — A parameter or local of a method may have the name of a
field of its receiver: from its declaration to the end of its block the bare
name means the local, and `self.x` the field. It may share a method's name,
since a method is only reached through a call. A bare call whose name is both
a method of the receiver and a top-level or prelude function is a
compile-time error; the help suggests `self.f()` or renaming.
*(ADR 0141, ADR 0044, ADR 1121)*

```ail skip=context
fn Parser.advance(mut self) char?
    c := peek() else return none   # self.peek()
    pos += 1                       # self.pos += 1
    c
```

```ail skip=context
fn Circle.new(r f64) Circle
    Circle(r: r)

c := Circle.new(1.0)
```

## Traits

**trait.decl.1** — A trait is declared with the keyword `trait`, its name, and
an indented block of method signatures. *(ADR 0073)*

**trait.decl.2** — A method in a trait MAY have a body (a default
implementation). Implementations inherit it unless they define the method
themselves. *(ADR 0081)*

**trait.decl.3** — A trait has no type parameters in 0.1: `trait From[T]` is
an error, reported once at the `[`; an `impl From[X] for T` of it and calls
of its methods are not reported again. A generic function `fn f[T](x T)` or one trait per type takes
its place (OQ-037). *(ADR 1111)*

```ail error=trait.decl.3
trait Convert[T]
    fn convert(self) T

fn main()
    print(1)
```

**trait.impl.1** — A type implements a trait only through an
`impl Trait for Type` block containing definitions of the trait's methods.
*(ADR 0073)*

**trait.impl.2** — There is no structural conformance. Having methods whose
signatures match a trait does not make a type implement it. *(ADR 0073)*

**trait.impl.3** — An `impl` block that omits a required method, or defines
one with a mismatched signature, is a compile-time error reported at the
`impl` block. The diagnostic MUST list each missing or mismatched method.
*(ADR 0073)*

```ail skip=context
trait Shape
    fn area(self) f64

impl Shape for Circle
    fn area(self) f64
        pi * self.r * self.r
```

**trait.impl.4** — `impl Trait for Type` MAY appear only in the package that
declares `Trait` or the package that declares `Type`. *(ADR 0082)*

**trait.impl.5** — An `impl` block defines only methods of its trait, each with
the trait's signature, with `Self` replaced by the implementing type.
*(ADR 0129)*

```ail error=trait.impl.5
trait Shape
    fn area(self) float

impl Shape for int
    fn area(self) float
        1.0
    fn grow(self) float
        2.0
```

```ail fragment
trait Shape
    fn area(self) float

impl Shape for int
    fn area(self) float
        1.0
```

**trait.impl.6** — A type implements a trait at most once. *(ADR 0129)*

```ail error=trait.impl.6
trait Shape
    fn area(self) float

impl Shape for int
    fn area(self) float
        1.0

impl Shape for int
    fn area(self) float
        2.0
```

```ail fragment
trait Shape
    fn area(self) float

impl Shape for int
    fn area(self) float
        1.0
```

## Supertraits

**trait.super.1** — A trait MAY name supertraits after its name, joined with
`+` as in a bound: `trait Ord Eq`, `trait Key Hash + ToStr`. A trait with
supertraits MAY have no methods. A trait MUST NOT require itself, directly or
through its supertraits. *(ADR 0348)*

**trait.super.2** — `impl Key for T` is a compile-time error unless `T`
implements every supertrait of `Key`; the help names the missing `impl`.
*(ADR 0348)*

**trait.super.3** — A bound `[T Key]` provides every supertrait of `Key`,
transitively: their methods, and `==` when one of them is `Eq`. The prelude
declares `trait Ord Eq` and `trait Hash Eq` (`trait.std.2`, `trait.auto.5`).
*(ADR 0348)*

```ail fragment
trait Named Eq
    fn name(self) str

fn same_name[T Named](a T, b T) bool
    a == b or a.name() == b.name()
```

## Associated functions in traits

**trait.assoc.1** — A function in a trait without `self` is an associated
function of the trait, such as `fn zero() Self`. An `impl` defines it with
`Self` replaced (`trait.impl.5`), or inherits its default body. The prelude
trait `Default` has `fn default() Self`; an empty `impl Default for T`
derives it for a struct from each field's declared default
(`type.struct.8`), else its type's (`0`, `0.0`, `""`,
`false`, an empty list, map or set, `none`, or the field type's own
`default()`); it is a `ct.derive.4` error for an enum. `int`, `float`,
`str`, `bool` and `List[T]` implement `Default`. *(ADR 0349, ADR 0350)*

**trait.assoc.2** — `T.name(args)`, with `T` a type parameter in scope (or
`Self` in a trait's default body), calls the associated function `name` of
a trait among `T`'s bounds and their supertraits. With no such trait it is
a compile-time error; the help names a trait that declares `name`. Calling
an associated function on a value, `x.name()`, is the same error.
*(ADR 0349)*

**trait.assoc.3** — `T.name(args)` runs the implementation of the type that
`T` stands for in the running call. A generic function's type arguments are
inferred as usual (`gen.infer.1`) or written (`f[int]()`); one that appears
only in the result needs the expected type or written arguments. Implementations
MAY pass the concrete types at run time (as the compiler does for run-time
type arguments) or monomorphise; the result is the same. *(ADR 0349)*

**trait.assoc.4** — `Type.name(args)` and `Type[A].name(args)` call the
associated function `name` of a trait that `Type` implements, when `Type`
declares no associated function of that name itself. *(ADR 0349)*

```ail fragment
trait Zero
    fn zero() Self

impl Zero for int
    fn zero() int
        0

fn zeros[T Zero](n int) List[T]
    [T.zero() for _ in 0..n]

print(zeros[int](3))     # [0, 0, 0]
```

## `Self`

**trait.self.1** — In a trait, `Self` is the implementing type. In a method,
associated function or `impl` block, `Self` is the type it belongs to, with
its type parameters. `Self` is not a type anywhere else. *(ADR 0081,
ADR 0128)*

```ail rules=trait.self.1
trait Zero
    fn zero() Self

impl Zero for int
    fn zero() int
        0

fn zeros[T Zero](n int) List[T]
    [T.zero() for _ in 0..n]

fn main()
    print(zeros[int](3))
```

## Call resolution

**trait.res.1** — If a method call `x.f(...)` could refer to more than one
method (inherent or from any implemented trait), it is a compile-time error.
A qualified call such as `Shape.area(x)` selects one explicitly. `trait.res.2`
makes an exception for standard traits' default methods. *(ADR 0082, ADR 0137)*

```ail error=trait.res.1
trait Shape
    fn area(self) float

trait Area
    fn area(self) float

impl Shape for int
    fn area(self) float
        1.0

impl Area for int
    fn area(self) float
        2.0

fn main()
    n := 1
    print(n.area())
```

```ail fragment
trait Shape
    fn area(self) float

impl Shape for int
    fn area(self) float
        1.0

fn main()
    n := 1
    print(n.area())
```

**trait.res.2** — A built-in or inherent method takes precedence over a
default method with the same name of a standard (prelude) trait, so
`trait.res.1` reports no ambiguity between them. A list's `xs.min()`
(`type.coll.11`) therefore hides `Ord`'s `min(other)`, and a user type's own
`min` keeps working when the type implements `Ord`. Methods of user traits
stay ambiguous under `trait.res.1`. *(ADR 0137)*

```ail rules=trait.res.2
fn main()
    xs := [3, 1, 2]
    print(xs.min())
```

## Operators

**trait.op.1** — The operators `+ - * / %`, unary `-` and `== != < <= > >=`
MAY be implemented for a struct or enum type by implementing the
corresponding prelude trait: `Eq` for `==` and `!=`, `Ord` for `< <= > >=`
(`trait.std.1–2`), and for both operands of type `T`:

| Operator | Trait | Method |
|---|---|---|
| `a + b` | `Add` | `fn add(self, other Self) Self` |
| `a - b` | `Sub` | `fn sub(self, other Self) Self` |
| `a * b` | `Mul` | `fn mul(self, other Self) Self` |
| `a / b` | `Div` | `fn div(self, other Self) Self` |
| `a % b` | `Rem` | `fn rem(self, other Self) Self` |
| `-a` | `Neg` | `fn neg(self) Self` |

`a + b` calls `a.add(b)` and has type `T`; `a += b` is `a = a + b`. A bound
`[T Add]` provides `+` in generic code. The built-in numbers keep their own
operators and implement none of these traits. Without the `impl`, the
operator is a `trait.op.1` error. A type MUST NOT be named like a prelude
trait (`mod.unit.1`). Indexing has no trait yet. *(ADR 0077, ADR 0127,
ADR 1124)*

```ail rules=trait.op.1
struct V
    x int

impl Add for V
    fn add(self, o V) V
        V(x: self.x + o.x)

impl Neg for V
    fn neg(self) V
        V(x: -self.x)

fn main()
    mut v := V(x: 1) + V(x: 2)
    v += V(x: 10)
    print((-v).x)   # -13
```

```ail error=trait.op.1
struct V
    x int

fn main()
    print((V(x: 1) * V(x: 2)).x)    # V has no `impl Mul`
```

```ail
struct V
    x int

fn main()
    print(V(x: 1) == V(x: 1))
```

**trait.op.2** — No other operator can be defined or implemented. *(ADR 0077)*

```ail error=trait.op.2 rules=trait.op.2
struct Vec2
    x int

impl Pow for Vec2           # there is no `**`, so there is no trait to implement for it
    fn pow(self, other Vec2) Vec2
        self

fn main()
    print(1)
```

**trait.op.3** — Operator calls dispatch statically and perform no implicit
conversion of their operands. *(ADR 0077)*

## Dynamic dispatch

**trait.any.1** — `any Trait` is a type whose values may hold a value of any
type implementing `Trait`. Method calls on it dispatch dynamically.
*(ADR 0079)*

**trait.any.2** — A bare trait name is not a type, except as a parameter's
top-level type (see `gen.anon.1`). *(ADR 0079, ADR 0078)*

```ail skip=context
struct Scene
    shapes List[any Shape]
```

**trait.any.3** — Where a value of type `any Trait` is expected, a value of any
type implementing `Trait` converts to it implicitly. *(ADR 0129)*

```ail rules=trait.any.3
trait Shape
    fn area(self) float

impl Shape for int
    fn area(self) float
        1.0

fn total(x any Shape) float
    x.area()

fn main()
    print(total(1))
```

**trait.any.4** — A trait method can be called on an `any Trait` value only if
`Self` appears in its signature only as the receiver. Calling any other method
is a compile-time error at the call. *(ADR 0129)*

```ail error=trait.any.4
trait Shape
    fn same(self, other Self) bool

impl Shape for int
    fn same(self, other int) bool
        true

fn show(x any Shape) bool
    x.same(x)

fn main()
    print(show(1))
```

```ail
trait Shape
    fn area(self) float

impl Shape for int
    fn area(self) float
        1.0

fn show(x any Shape) float
    x.area()

fn main()
    print(show(1))
```

**trait.any.5** — `any Trait` satisfies a constraint `Trait` only when every
method of `Trait` can be called through it (`trait.any.4`). *(ADR 0129)*

```ail rules=trait.any.5
trait Shape
    fn area(self) float

impl Shape for int
    fn area(self) float
        1.0

fn total[T Shape](x T) float
    x.area()

fn show(x any Shape) float
    total(x)

fn main()
    print(show(1))
```

## Standard traits

**trait.std.1** — `trait Eq` has the method `fn eq(self, other Self) bool`.
`a == b` calls `a.eq(b)`, and `a != b` is `not a.eq(b)`. For `float`, `==` and
`!=` are the IEEE 754 operators of `type.num.6` wherever they are written,
including in generic code through an `Eq` bound: NaN is unequal to itself and
`-0.0 == 0.0`, so `float` equality isn't reflexive. *(ADR 0127, ADR 0130)*

**trait.std.2** — `trait Ord` has the method `fn cmp(self, other Self)
Ordering`, where `enum Ordering` has the variants `Less`, `Equal` and
`Greater`. `a < b` is true when `a.cmp(b)` is `Less`, `a <= b` when it isn't
`Greater`, and likewise for `>` and `>=`. A type that implements `Ord` also
implements `Eq`, and a constraint `Ord` provides `==`: the prelude declares
`trait Ord Eq` (`trait.super.1`). For `float`, the four
operators are instead the IEEE comparisons of `type.num.6`, also through an
`Ord` bound, so each is false when an operand is NaN and they don't follow
`cmp`; `float`'s `cmp` is the total order of `type.num.11`. `Ord` also has
the default methods `a.min(b)` and `a.max(b)`: `a.min(b)` is `b` if
`b.cmp(a)` is `Less`, else `a`; `a.max(b)` is `b` if `b.cmp(a)` is `Greater`,
else `a`, so ties give `a`. Every `Ord` type has them, in generic code too.
For `float` they use `cmp`, so NaN counts as the greatest value and
`x.min(nan)` is `x`. *(ADR 0127, ADR 0130, ADR 0137)*

```ail fragment
fn clamp[T Ord](x T, lo T, hi T) T
    x.max(lo).min(hi)
```

**trait.std.3** — `trait ToStr` has the method `fn to_str(self) str`. A value's
text form, used by `print`, interpolation and error reports, is its
`to_str()`. *(ADR 0127)*

**trait.std.4** — `int`, `str`, `float` and `char` implement `Ord`. `str`
compares UTF-8 bytes lexicographically, and `char` compares by code point.
`float`'s `cmp` is the total order of `type.num.11`: numbers in numeric order,
`-0.0` equal to `0.0`, and NaN equal to NaN and greater than every number.
`List[T]` implements `Ord` when `T` does, comparing lexicographically with
`T`'s `cmp`, so a `List[float]` compares by the total order; a derived `Ord`
(`trait.derive.1`) likewise compares `float` fields by `cmp`. *(ADR 0127,
ADR 0130, ADR 0134)*

**trait.derive.1** — An `impl` of `Eq`, `Ord` or `ToStr` with no method
definitions derives the implementation: structs compare field by field in
declaration order; enums compare by variant declaration order, then field by
field. Every field's type MUST implement the trait. *(ADR 0127)*

```ail fragment
struct Version
    major int
    minor int

impl Ord for Version     # compares major, then minor
```

**trait.auto.1** — Every type implements `Eq` automatically when its parts do:
`int`, `bool`, `str`, `char` and `float` (whose `==` is IEEE, `trait.std.1`);
`List[T]`, `Set[T]` and `T?` when `T` does; `Map[K, V]` when `V` does (`K` is
always `Hash`, so `Eq`); and struct, enum and `error` types whose fields all
do. Two values are equal when they are the same variant with equal fields;
lists are equal element by element; maps are equal when they have the same
keys with equal values, and sets when they have the same elements, in any
order (`type.map.4`). Parts are compared with their own `==`, so a value
holding a NaN is unequal to itself. *(ADR 0126, ADR 0130, ADR 0131,
ADR 0134)*

**trait.auto.2** — Every type implements `ToStr` automatically. The text form
of an `int`, `bool` or `str` is given by `type.str.3`, of a `float` by
`type.num.7`, and of a `char` by `type.char.1`. A struct's is its name and
fields, `Point(x: 1, y: 2)`; a variant's is its enum and variant names and
fields, `Dir.North` or `Shape.Circle(r: 1)`; a list's is `[1, 2]`; a map's is
`{"a": 1, "b": 2}`; a set's is `{1, 2}`; an empty map and an empty set are
both `{}`. Maps and sets follow insertion order (`type.map.1`). An absent
optional's is `none`, and a present one's is its value's. Inside these forms,
a `str` is quoted with `"` and a `char` with `'`, and other values use their
own text form (a `float` as at top level). Within the quotes, `\\`, the
delimiting quote, and the control characters `\n`, `\t`, `\r` and `\0` are
escaped with a backslash, every other control character (category Cc) is
written `\u{hex}` in lowercase hex without padding, and everything else is
kept as is, including `{` and the other kind of quote. *(ADR 0126,
ADR 0135, ADR 0130, ADR 0131, ADR 0134)*

```ail fragment
print(["a\"b", "x\ty"])     # ["a\"b", "x\ty"]
print(['x', '\'', '{'])      # ['x', '\'', '{']
print({"k": [1.5, 2.0]})     # {"k": [1.5, 2.0]}
```

**trait.auto.3** — An explicit `impl Eq for T` or `impl ToStr for T` replaces the
automatic implementation. For the effect on `Hash`, see `trait.auto.5`.
*(ADR 0126, ADR 0133)*

**trait.auto.4** — A method named `eq` or `to_str` MUST NOT be declared outside
an `impl` of `Eq` or `ToStr`. The diagnostic SHOULD suggest the `impl`.
*(ADR 0126)*

```ail skip=context
impl ToStr for Dir
    fn to_str(self) str
        match self
            North => "north"
            South => "south"
```

**trait.auto.5** — `Hash` is a prelude trait with no methods. A type is
`Hash` when its `Eq` is automatic (`trait.auto.1`) or derived by an empty
`impl Eq` (`trait.derive.1`), and all its parts are `Hash`: `int`, `bool`,
`str`, `char`, `Ordering`, `T?` and `List[T]` when `T` is, and struct, enum
and `error` types whose fields all are. A generic type is `Hash` per instance
(`Box[int]` is, `Box[float]` isn't). `float`, `Map`, `Set` and `any Trait`
aren't `Hash`. A hand-written `fn eq` (`trait.auto.3`) removes `Hash`. An
`impl Ord` doesn't change `Eq` (`==` never calls `cmp`), so a type with
`impl Ord` and an automatic `Eq` stays `Hash`. `impl Hash` is a compile-time
error except the `uses` form of `trait.hash.1`. A constraint `[T Hash]` implies
`Eq`: the prelude declares `trait Hash Eq` (`trait.super.3`). Map keys and set elements MUST be `Hash` (`type.map.2`). Hash values are
never visible to programs, so output can't depend on the hash function.
*(ADR 0133)*

```ail rules=trait.auto.5
struct Sq
    s int

fn main()
    mut m := Map[Sq, int]()
    m[Sq(s: 1)] = 5
    print(m[Sq(s: 1)])
```

**trait.hash.1** — A struct with its own `fn eq` and no type parameters MAY
write `impl Hash for T` whose body is one line `uses a, b`, naming fields of
`T`. `T` is then `Hash`: a map or set of `T` identifies a key by those fields
alone, in hash and in comparison, so the other fields (even a `float`) are
ignored. A hand-written `fn eq` SHOULD compare at least the named fields; the
table never calls it. Any other `impl Hash` (no `uses`, methods, an enum, a
generic struct) is `trait.auto.5`. *(ADR 0955)*

```ail rules=trait.hash.1
struct Key
    id int
    noise float

impl Eq for Key
    fn eq(self, other Self) bool
        self.id == other.id

impl Hash for Key
    uses id

fn main()
    mut m := Map[Key, int]()
    m[Key(id: 1, noise: 1.5)] = 7
    print(m[Key(id: 1, noise: 9.0)])
```

**trait.hash.2** — In `uses`, `T` MUST have its own `fn eq` (otherwise `==`
and the automatic hash already agree), and each name MUST be a distinct field
of `T` whose type is `Hash`. *(ADR 0955)*

```ail error=trait.hash.2
struct Key
    id int
    f float

impl Eq for Key
    fn eq(self, other Self) bool
        self.id == other.id

impl Hash for Key
    uses f

fn main()
    print(1)
```

## Runnable examples for the rules above

Each rule ID is cited by a checked example (ADR 0498). `error=` blocks must
report the named rule; `rules=` blocks check and run. Where the compiler
disagrees with the rule, the block is `skip=bootstrap` (ADR 0498).

```ail rules=trait.method.1,trait.method.2,trait.method.3
struct Counter
    n int

fn Counter.bump(mut self)       # a mut receiver changes the value
    self.n = self.n + 1

fn Counter.get(self) int
    self.n

fn main()
    mut c := Counter(n: 1)      # the receiver must be mutable
    c.bump()                    # no `mut` marker at the call
    print(c.get())              # 2
```

```ail rules=trait.method.4
struct Pt
    x int

fn Pt.origin() Pt               # no `self`: an associated function
    Pt(x: 0)

fn main()
    print(Pt.origin().x)        # 0
```

```ail norun rules=trait.method.5
# Methods are declared in the package that declares the type; a single file is one package.
fn main()
    print("package")
```

```ail rules=trait.method.6
struct Pt
    x int

fn Pt.double(self) int
    x * 2                       # bare `x` is `self.x`; ail fmt writes `self.x * 2`

fn main()
    print(Pt(x: 4).double())    # 8
```

```ail error=trait.method.7 rules=trait.method.7
struct Pt
    x int

fn grow(n int) int
    n + 1

fn Pt.grow(self) int
    x + 1

fn Pt.twice(self) int
    grow() * 2                  # error: `grow` is both a method of `Pt` and a function
```

```ail rules=trait.method.7
struct Acc
    d int

fn Acc.bump(self) int
    d := self.d + 1             # a local may take a field's name; `self.d` is the field
    d * 10

fn main()
    print(Acc(d: 4).bump())     # 50
```

```ail rules=trait.decl.1,trait.decl.2
trait Greet
    fn name(self) str
    fn hello(self) str          # a default body: implementations inherit it
        "hi {self.name()}"

struct P
    who str

impl Greet for P
    fn name(self) str
        self.who

fn main()
    print(P(who: "ann").hello())    # hi ann
```

```ail rules=trait.impl.1,trait.impl.5
trait Named
    fn name(self) str

struct P
    who str

impl Named for P
    fn name(self) str           # the impl defines the trait's method, with Self as P
        self.who

fn main()
    print(P(who: "cy").name())  # cy
```

```ail error=gen.bound.3 rules=trait.impl.2
trait Area
    fn area(self) int

struct Sq
    s int

fn Sq.area(self) int            # the same signature is not enough: there is no impl
    self.s * self.s

fn show(a Area) int
    a.area()

fn main()
    print(show(Sq(s: 2)))
```

```ail error=trait.impl.3 rules=trait.impl.3
trait Area
    fn area(self) int

struct Sq
    s int

impl Area for Sq
    fn area(self) str           # a mismatched signature
        "x"
```

```ail error=trait.impl.6 rules=trait.impl.6
trait Named
    fn name(self) str

struct P
    who str

impl Named for P
    fn name(self) str
        self.who

impl Named for P               # a second impl of the same trait
    fn name(self) str
        "again"
```

```ail rules=trait.auto.4
# A method named `to_str` declared outside an impl of ToStr should be rejected, and the
# diagnostic should suggest the impl. The compiler accepts `fn to_str(p P) str`: a known bug.
fn main()
    print("auto")
```

```ail rules=trait.super.1,trait.super.3
trait Named
    fn name(self) str

trait Greeter Named             # Greeter names Named as its supertrait
    fn greet(self) str
        "hi {self.name()}"

struct P
    who str

impl Named for P
    fn name(self) str
        self.who

impl Greeter for P

fn main()
    print(P(who: "bo").greet())     # hi bo
```

```ail error=trait.super.2 rules=trait.super.2
trait Named
    fn name(self) str

trait Greeter Named
    fn greet(self) str

struct P
    who str

impl Greeter for P              # P must implement Named first
    fn greet(self) str
        "x"
```

```ail rules=trait.assoc.1,trait.assoc.2,trait.assoc.3,trait.assoc.4,trait.self.1
trait Zero
    fn zero() Self              # an associated function of the trait

struct N
    v int

impl Zero for N
    fn zero() N
        N(v: 0)

fn make[T Zero]() T
    T.zero()                    # called on the type parameter, run for the type it stands for

fn main()
    n := make[N]()
    print(n.v)                  # 0
    print(N.zero().v)           # 0
```

```ail error=trait.res.1 rules=trait.res.1
trait A
    fn go(self) int

trait B
    fn go(self) int

struct S
    v int

impl A for S
    fn go(self) int
        1

impl B for S
    fn go(self) int
        2

fn main()
    print(S(v: 1).go())         # two methods named go: ambiguous
```

```ail norun rules=trait.res.2
# A built-in or inherent method wins over a default method of a prelude trait with the same
# name, so the call below uses the struct's own method.
fn main()
    print("inherent")
```

```ail error=trait.op.2 rules=trait.op.2
# Only the operators of trait.op.1 may be implemented: `Shl` would define `<<`, which no trait exists for.
struct V
    n int

impl Shl for V
    fn shl(self, other int) V
        self

fn main()
    print(1)
```

```ail rules=trait.op.1,trait.std.2
struct V
    n int

impl Ord for V                  # `<` comes from Ord's cmp
    fn cmp(self, other V) Ordering
        if self.n < other.n
            return Less
        if self.n > other.n
            return Greater
        Equal

fn main()
    print(V(n: 1) < V(n: 2))    # true
```

```ail rules=trait.any.1,trait.any.3
trait Shape
    fn area(self) int

struct Sq
    s int

impl Shape for Sq
    fn area(self) int
        self.s * self.s

fn show(s any Shape) int        # any value that implements Shape converts to it
    s.area()

fn main()
    print(show(Sq(s: 3)))      # 9
```

```ail error=expr.name.1 rules=trait.any.2
trait Shape
    fn area(self) int

fn main()
    mut s := Shape              # a bare trait name is not a value or a type here
    print(s)
```

```ail error=trait.assoc.2 rules=trait.any.4
trait Shape
    fn area(self) int
    fn make() Self

struct Sq
    s int

impl Shape for Sq
    fn area(self) int
        self.s * self.s
    fn make() Sq
        Sq(s: 1)

fn show(s any Shape) int
    s.make().s                  # make doesn't take self: not callable through any Shape
```

```ail norun rules=trait.any.5
# `any Shape` satisfies the constraint Shape only when every method can be called through it
# (trait.any.4); the rule is checked by the example for trait.any.4 above.
fn main()
    print("any")
```

```ail rules=trait.std.1,trait.auto.1
struct P
    x int
    y int

fn main()
    print(P(x: 1, y: 2) == P(x: 1, y: 2))   # Eq is automatic: true
```

```ail rules=trait.std.3,trait.auto.3
struct V
    n int

impl ToStr for V                # an explicit impl replaces the automatic one
    fn to_str(self) str
        "V{self.n}"

fn main()
    print(V(n: 5))              # V5
```

```ail rules=trait.std.4
fn main()
    print("b" > "a")            # strings compare bytewise: true
    print('b' > 'a')            # chars by code point: true
    print(2 < 3)                # ints: true
```

```ail rules=trait.derive.1
struct P
    x int

impl ToStr for P                # no methods: the implementation is derived

fn main()
    print(P(x: 3))              # P(x: 3)
```

```ail rules=trait.auto.2
struct P
    x int

fn main()
    print(P(x: 3))              # every type is ToStr automatically
```

```ail skip=context rules=trait.auto.5
# Hash has no methods; a type is Hash when its Eq is automatic or derived by an empty impl.
# The map and set examples in chapter 17 use it as a key.
```

```ail skip=context rules=trait.impl.4
# An impl outside the package that declares the trait or the type is rejected. An example
# needs two packages; the compiler's package tests cover it (tests/cases/packages).
```
