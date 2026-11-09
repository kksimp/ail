# 09. Generics

Chapter key: `gen`

> **Status:** Drafted (partial). Specified: bracket syntax (`gram.generic.1`),
> generic types, constraints, methods on generic types, inference, anonymous
> generic parameters, and the compilation-strategy freedom. Not yet designed:
> variance, const generics beyond `gen.const.1-3`, and methods for one instantiation only. Because
> user code can't declare such methods, `xs.sum()` (`List[int]` and
> `List[float]` only) is a built-in special case (`type.coll.12`, ADR 0136).
> `Hash` is now a usable bound (`trait.auto.5`, ADR 0133).

## Generic declarations

**gen.decl.1** — Functions, `struct`, `enum` and `error` type declarations MAY
take type parameters in brackets after their name: `fn largest[T Ord](...)`,
`struct Stack[T]`. Every use of a generic type supplies its type arguments
(`Stack[int]`), except where `gen.method.1` and `gen.infer.1` allow them to be
left out. *(ADR 0041, ADR 0128)*

```ail rules=gen.decl.1
fn pick[T Ord](a T, b T) T
    a.max(b)

fn main()
    print(pick(1, 2))
```

**gen.decl.2** — A type parameter MUST NOT reuse the name of a type, a trait,
or another type parameter in scope. *(ADR 0128)*

```ail error=gen.decl.2
struct Box[Box]
    v int

fn main()
    print(1)
```

```ail
struct Box[T]
    v T

fn main()
    print(Box(v: 1).v)
```

## Constraints

**gen.bound.1** — A generic type parameter MAY be followed by one or more
constraints, joined with `+`, inside the generic parameter brackets:
`[T Shape]`, `[K Hash + ToStr, V]`. *(ADR 0074)*

**gen.bound.2** — The body of a generic declaration is type-checked using only
the operations its declared constraints provide. Using any other operation on a
value of a type parameter is a compile-time error at the declaration.
*(ADR 0074)*

```ail fragment
fn largest[T Ord](items List[T]) T
    mut best := items[0]
    for item in items
        if item > best
            best = item
    best
```

> **Note:** `Hash` has no methods and implies `Eq` (`trait.auto.5`), so a body
> constrained by `[K Hash]` can use `==` on `K`. Writing `Hash + Eq` is
> allowed but redundant. *(ADR 0133)*

**gen.bound.3** — Every type argument MUST satisfy its parameter's
constraints. The diagnostic names the type and the trait it doesn't
implement. *(ADR 0129)*

```ail error=gen.bound.3
struct Foo
    v int

fn pick[T Ord](a T) T
    a

fn main()
    print(pick[Foo](Foo(v: 1)).v)
```

```ail
fn pick[T Ord](a T) T
    a

fn main()
    print(pick[int](1))
```

**gen.bound.4** — The constraints declared on a generic type's parameters
apply wherever the type is used, and are assumed in its methods and `impl`
blocks. *(ADR 0128)*

```ail rules=gen.bound.4
struct Top[T Ord]
    v T

fn main()
    t := Top(v: 3)
    print(t.v)
```

## Methods on generic types

**gen.method.1** — A method or associated function of a generic type is
declared with the type's bare name, `fn Stack.push(mut self, x T)`. The type's
parameters are in scope under their declared names, with their declared
constraints. The same holds for `impl Trait for Stack`. *(ADR 0128)*

**gen.method.2** — To add constraints for one method or `impl`, the type's
parameters are written after its name, in order, under their declared names,
with the added constraints: `fn Stack[T Ord].max(self) T?`. Such a method or
implementation exists only for instances whose type arguments satisfy them.
*(ADR 0128)*

**gen.method.3** — A method MAY declare type parameters of its own, after its
name: `fn Pair.with_first[C](self, c C) Pair[C, B]`. *(ADR 0128)*

```ail fragment
struct Stack[T]
    items List[T]

fn Stack.new() Self
    Stack(items: [])

fn Stack.push(mut self, x T)
    self.items.push(x)
```

## Inference

**gen.infer.1** — The type arguments of a call to a generic function or
associated function, of a struct literal, and of a variant are inferred from
the arguments and, failing that, from the expected type. If any remains
unknown, it is a compile-time error, and the diagnostic SHOULD suggest
explicit type arguments. *(ADR 0129)*

```ail rules=gen.infer.1
fn pick[T Ord](a T, b T) T
    a.max(b)

fn main()
    print(pick(1, 2))
```

**gen.infer.2** — Explicit type arguments are written in brackets after the
name of a generic function or type: `parse[int](s)`, `Stack[int].new()`,
`Pair[int, str](first: 1, second: "a")`. The generic function or type may be
declared in another file of the same module. *(ADR 0041, ADR 0129)*

```ail rules=gen.infer.2
fn pick[T Ord](a T, b T) T
    a.max(b)

fn main()
    print(pick[int](1, 2))
```

## Anonymous generic parameters

**gen.anon.1** — A parameter whose type is written as a trait name declares an
anonymous type parameter constrained by that trait: `fn draw(s Shape)` is
equivalent to `fn draw[T Shape](s T)`. *(ADR 0078)*

```ail rules=gen.anon.1
trait Shape
    fn area(self) float

struct Sq
    s float

impl Shape for Sq
    fn area(self) float
        self.s

fn draw(s Shape) float
    s.area()

fn main()
    print(draw(Sq(s: 2.0)))
```

**gen.anon.2** — The canonical form uses the anonymous form when the type
parameter would be referenced exactly once, and the named form otherwise.
*(ADR 0078)*

```ail rules=gen.anon.2
trait Shape
    fn area(self) float

struct Sq
    s float

impl Shape for Sq
    fn area(self) float
        self.s

fn draw(s Shape) float
    s.area()

fn main()
    print(draw(Sq(s: 2.0)))
```

## Compilation

**gen.impl.1** — The semantics of generic code MUST NOT depend on whether an
instance is specialised or shares code with other instances. The
implementation chooses per instance and MUST report its choices in the
performance report. *(ADR 0080)*

```ail rules=gen.impl.1
fn pick[T Ord](a T, b T) T
    a.max(b)

fn main()
    print(pick(1, 2))
    print(pick(1.5, 2.5))
```

## Runnable examples for the rules above

Each rule ID is cited by a checked example (ADR 0498). `error=` blocks must
report the named rule; `rules=` blocks check and run.

```ail rules=gen.decl.1,gen.bound.1,gen.infer.1
fn largest[T Ord](xs List[T]) T     # a type parameter with a constraint
    mut best := xs[0]
    for x in xs
        if x > best
            best = x
    best

fn main()
    print(largest([3, 9, 2]))       # 9: T is inferred from the list
```

```ail error=gen.decl.2 rules=gen.decl.2
fn pick[T, T](x T) T                # a type parameter is declared once
    x
```

```ail error=trait.op.1 rules=gen.bound.2
fn dup[T](x T) T                    # with no constraint, `+` is not available on T
    x + x
```

```ail error=gen.bound.3 rules=gen.bound.3
trait Shape
    fn area(self) float

struct Sq
    s float

fn show(s Shape) float              # Sq must implement Shape to be passed here
    s.area()

fn main()
    print(show(Sq(s: 2.0)))
```

```ail rules=gen.bound.4,gen.anon.1
trait Shape
    fn area(self) float

struct Sq
    s float

impl Shape for Sq
    fn area(self) float
        self.s * self.s

fn show(s Shape) float              # an anonymous type parameter constrained by Shape
    s.area()

fn main()
    print(show(Sq(s: 2.0)))         # 4.0
```

```ail rules=gen.method.1,gen.method.2
struct Stack[T]
    items List[T]

fn Stack.push(mut self, x T)        # methods are written with the bare type name
    self.items.push(x)

fn Stack[T Ord].max(self) T?        # constraints for one method, under the declared name
    none

fn main()
    mut s := Stack(items: [0])
    s.push(1)
    print(s.items.len())            # 2
```

```ail rules=gen.method.3
struct Pair[A, B]
    first A
    second B

fn Pair.with_first[C](self, c C) Pair[C, B]   # a method with its own type parameter
    Pair(first: c, second: self.second)

fn main()
    p := Pair(first: 1, second: "x")
    q := p.with_first("y")
    print(q.first)                  # y
```

```ail rules=gen.infer.2
fn parse[T](s str) T?
    none

fn main()
    print(parse[int]("3") else 0)   # 0: the explicit type argument selects T
```

```ail skip=context rules=gen.anon.2
fn show(s Shape) float
    s.area()
# The canonical form uses the anonymous form when the parameter is referenced once; the
# formatter enforces it (ail fmt), so there is no output to print.
```

```ail norun rules=gen.impl.1
fn main()
    # Generic code behaves the same whether or not an instance is specialised; the
    # observable behaviour is the same program output shown in the examples above.
    print("instances")
```

## Const generics

**gen.const.1** — A struct or function MAY declare a const parameter as a name
preceded by `const` and followed by `int`: `struct Vec[T, const N int]`, `fn dot[const N int](...)`. A const
argument is an int literal (`Vec[float, 3]`) or a const parameter in scope. A
call that uses one const parameter twice, as `dot` does, needs the same value
in both places; lengths that differ (`Vec[float, 3]` against `Vec[float, 4]`)
are an ordinary argument type mismatch (`expr.call.2`). *(ADR 0954)*

**gen.const.2** — A const parameter MUST get an int literal or a const parameter
in scope, and a type parameter MUST NOT get one. *(ADR 0954)*

```ail error=gen.const.2
struct Vec[T, const N int]
    data List[T]

fn f(v Vec[float, int])
    print(1)

fn main()
    print(1)
```

**gen.const.3** — Inside the declaration a const parameter is an `int` value,
fixed per instance. *(ADR 0954)*

```ail rules=gen.const.1,gen.const.3
struct Vec[T, const N int]
    data List[T]

fn Vec.size(self) int
    N

fn dot[const N int](a Vec[float, N], b Vec[float, N]) float
    mut s := 0.0
    for i in 0..N
        s += a.data[i] * b.data[i]
    s

fn main()
    a := Vec[float, 3](data: [1.0, 2.0, 3.0])
    b := Vec[float, 3](data: [4.0, 5.0, 6.0])
    print(a.size())
    print(dot(a, b))
```
