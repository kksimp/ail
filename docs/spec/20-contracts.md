# 20. Contracts

Chapter key: `contract`

> **Status:** Drafted (partial). Specified: `requires` and `ensures` lines,
> their typing and purity, `result`, `old(e)`, contracts on trait methods and
> their inheritance, struct `invariant` lines, the runtime checks and their
> traps, `--no-contracts`, static proof of simple contracts, and contracts in
> `ail api`. Not yet designed: loop invariants (see
> [open-questions.md](../open-questions.md), OQ-060).

A contract states what a function needs and what it promises, in the
function's signature, as ordinary `bool` expressions. The bootstrap checks
them at run time. The compiler proves some contracts statically and drops
their checks (`contract.prove.1`); that changes nothing observable except that
the trap can't occur.

## Declaration

**contract.decl.1** — Contract lines come directly under a function's
signature, one per line and indented as its body, before the body's first
statement: `requires COND` or `ensures COND`. A function may have any number
of each, in any order. A `requires` or `ensures` line anywhere else is a
compile-time error. *(ADR 0210)*

```ail fragment
fn divide(a int, b int) int
    requires b != 0
    ensures result * b + a % b == a
    a / b
```

```ail error=contract.decl.1
fn h(x int)
    print(x)
    requires x > 0
```

**contract.decl.2** — A condition has type `bool`. It sees the function's
parameters and, in a method, `self` and the receiver's members by bare name
(`trait.method.6`). It doesn't see the body's locals. *(ADR 0210)*

```ail error=contract.decl.2
fn f(x int) int
    requires x + 1
    x
```

**contract.decl.3** — A condition is pure: it, and every function it may
call, has no effect but `fail` (`eff.infer.1`). It can't print, reach a
capability, spawn, or change a `mut` argument. The diagnostic names each
effect and the chain of calls that gives it. *(ADR 0212)*

```ail error=contract.decl.3
fn noisy(x int) bool
    print("checking {x}")
    x > 0

fn f(x int) int
    requires noisy(x)
    x
```

## `requires`

**contract.req.1** — Each `requires` condition is evaluated in written
order when the function is called, after its parameters are bound and before
its body runs. If one is `false`, the program traps with
`contract.req.1` at the call. The message names the function, the condition
as written, every argument (`name = value`, values in their quoted text form,
`trait.auto.2`), and where the contract is:
`` `divide` requires `b != 0`, but it was called with a = 7, b = 0 (contract at 3:5) ``.
A failed `requires` is the caller's error. *(ADR 0211)*

```ail tool=run,FILE status=1 expect=trap[contract.req.1] rules=contract.req.1
fn half(n int) int
    requires n >= 0
    n / 2

fn main()
    print(half(-4))
```

## `ensures`

**contract.ens.1** — When the function returns a value (not a failure of a
`T ! E` result, `err.model.1`), each `ensures` condition is evaluated in
written order. If one is `false`, the program traps with `contract.ens.1` at
the `ensures` line. The message names the function, the condition, the
returned value, where the function returned, and the parameters' values at
that point: `` `divide` ensures `result * b == a`, but it returned 3 at 5:5 with a = 7, b = 2 ``.
A failed `ensures` is the function's error. *(ADR 0211)*

**contract.ens.2** — In an `ensures` condition, `result` names the returned
value; for `T ! E` it is the `T`. A function that returns nothing has no
`result`, and its `ensures` lines describe its `mut` parameters. `result` is
not a keyword: it is a name the `ensures` lines bind, so a function with an
`ensures` line can't have a parameter named `result`. In a `requires`
condition, `result` is an error, since nothing has returned yet.
*(ADR 0210, ADR 0358)*

```ail error=contract.ens.2
fn g(result int) int
    ensures result > 0
    result
```

```ail fragment
fn log(mut lines List[str], line str)
    ensures lines.len() > 0
    lines.push(line)
```

**contract.ens.3** — In an `ensures` condition, `old(e)` is the value that
`e` had when the function was called, after its parameters were bound. `e` is
an expression over the parameters and `self`, typed and pure like the
condition; it is evaluated once on entry, and only when checks are on
(`contract.run.3`). `old` takes one expression and only means something in an
`ensures` line: anywhere else it is an error. A trap shows each `old(e)` as
its own name among the condition's values. *(ADR 0303)*

```ail fragment
fn bump(mut xs List[int])
    ensures xs.len() == old(xs.len()) + 1
    xs.push(7)
```

```ail error=contract.ens.3
fn f(x int) int
    requires old(x) > 0
    x
```

## Trait contracts

**contract.inh.1** — A trait method, with or without a default body, may have
`requires` and `ensures` lines under its signature (`contract.decl.1`); they
see the trait method's parameters. An implementation of the method inherits
them. For a call, an implementation's own `requires` lines **weaken** the
trait's: the call is accepted when every inherited `requires` holds or every
one of the implementation's own holds. An implementation with no `requires` of
its own accepts exactly what the trait does. Failing both traps with
`contract.req.1`, naming the trait method the condition came from. *(ADR 0304)*

**contract.inh.2** — An implementation's own `ensures` lines **add** to the
trait's: every inherited `ensures` and every one of its own must hold, in that
order. A failure traps with `contract.ens.1` in the implementation, and names
the trait method when the condition is inherited. *(ADR 0304)*

**contract.inh.3** — Inherited conditions are checked on every call of the
implementation, through the trait (`trait.any.1`) and through the concrete
type alike. They read the arguments under the trait method's parameter names,
whatever names the implementation uses. A default method's own contracts check
its body and are inherited by implementations that override it, as above.
*(ADR 0304)*

```ail
trait Shape
    fn scale(self, k int) int
        requires k > 0
        ensures result >= 0

struct Square
    side int

impl Shape for Square
    fn scale(self, k int) int
        requires k == -1
        ensures result < 1000
        self.side * k * k

fn main()
    s := Square(side: 2)
    print(s.scale(3))
    print(s.scale(-1))
```

```ail error=contract.decl.2
trait Shape
    fn area(self) int
        ensures result

struct Square
    side int

impl Shape for Square
    fn area(self) int
        self.side * self.side
```

## Invariants

**contract.inv.1** — A struct's declaration may contain `invariant COND`
lines among its fields. A condition has type `bool`, is pure like any
contract (`contract.decl.3`), and sees the fields by bare name and `self`.
*(ADR 0305)*

**contract.inv.2** — Every value built with the struct's literal form
satisfies each invariant, in written order. If one is `false`, the program
traps with `contract.inv.2` at the construction. The message names the struct,
the condition, the value and where it was built. *(ADR 0305)*

**contract.inv.3** — When a method with a `mut self` receiver returns, each
invariant holds for the receiver, whether the method returned a value or a
failure. If one is `false`, the program traps with `contract.inv.3` at the
invariant line, naming the method, where it returned and the receiver's value.
Other writes (a field assigned in a function that received the struct as a
`mut` parameter) aren't checked until the value passes through one of these
two points. *(ADR 0305)*

```ail
struct Account
    balance int
    limit int
    invariant balance >= 0
    invariant balance <= limit

fn Account.deposit(mut self, n int)
    balance += n

fn main()
    mut a := Account(balance: 5, limit: 10)
    a.deposit(3)
    print(a.balance)
```

```ail error=contract.decl.3
fn noisy(n int) bool
    print("checking {n}")
    n > 0

struct Account
    balance int
    invariant noisy(balance)
```

## Running

**contract.run.1** — Contracts are part of the signature: they are checked
at every call, including calls through a trait (`trait.any.1`), direct
recursion, and calls from tests (`test.run.1`). Invariants are checked at the
points `contract.inv.2` and `contract.inv.3` name. *(ADR 0211, ADR 0305)*

```ail rules=contract.run.1
fn half(n int) int
    requires n >= 0
    n / 2

fn main()
    print(half(4))
```

**contract.run.2** — Calls made while evaluating a condition don't check
their own contracts, so a condition may call the function it describes:
`ensures n == 0 or result == n * fact(n - 1)`. *(ADR 0211)*

**contract.run.3** — `ail run --no-contracts` and `ail test --no-contracts`
evaluate no condition, including `old(e)` and invariants. Because conditions
are pure (`contract.decl.3`), the only difference a program can show is that a
contract trap doesn't occur.
Checks are on by default. *(ADR 0212)*

```ail tool=run,--no-contracts,FILE expect=-2 rules=contract.run.3
fn half(n int) int
    requires n >= 0
    n / 2

fn main()
    print(half(-4))
```

## Static proof

**contract.prove.1** — The checker proves a `requires` of a top-level function
at a call that names it, `f(args)`, in the same file, and an `ensures` of a
top-level function that is not generic and can't fail, at every return of its
body. A proved check is not evaluated at run time. `ail run --trace` evaluates
every check, so its contract events don't change, and `AIL_OPT_OFF=prove`
keeps every check. Because a proved condition is true and can't trap, leaving
it out changes nothing a program can show. `ail prove` lists what was proved
(`prove.cmd.1`). *(ADR 1055)*

```ail rules=contract.prove.1
fn at(xs List[int], i int) int
    requires i >= 0
    requires i < xs.len()
    xs[i]

fn main()
    xs := [3, 1, 4]
    mut sum := 0
    for i in 0..xs.len()
        sum += at(xs, i)
    print(sum)
```

**contract.prove.2** — A condition is proved when it holds for every value
these facts allow: integer literals; the function's own `requires`, which held
at the call (`contract.run.1`); an immutable `x := e` of an `int`; the
condition of an `if` or `elif` in its block, the negation of the conditions
before it in later branches and `else`, and that negation after an `if` whose
every branch leaves (`return`, `break`, `continue`); `for i in a..b` and
`a..=b`, for an immutable `i`; `len()` of a parameter or binding that is a
`List`, `Map`, `Set` or `str`, which is between 0 and 2^57; the `ensures` of a
top-level function called in `x := f(args)`; and `+`, `-`, `*`, `/` and `%`
(the last two by a positive constant), unary `-`, `and`, `or` and `not`. A
`mut` name, a field, an element, a method call other than `len()`, a call in a
condition, a float and a sized number carry no facts. In a condition an
arithmetic term counts only when its bounds are known, so a proved condition
can't overflow. *(ADR 1055)*

```ail tool=prove,FILE expect=proved rules=contract.prove.2
fn digit(n int) int
    requires n >= 0
    ensures result >= 0
    ensures result < 10
    n % 10

fn main()
    print(digit(42))
```

**contract.prove.3** — There is no solver: the result depends only on the
file, and is the same on every run. A check that isn't proved stays, evaluated
at run time (`contract.req.1`, `contract.ens.1`) and fuzzed by
`ail test --contracts` (`contract.fuzz.1`). *(ADR 1055)*

```ail tool=prove,FILE expect=checked rules=contract.prove.3
fn half(n int) int
    requires n >= 0
    n / 2

fn main()
    mut k := 4
    k -= 6
    print(half(k))
```

## Tools

**contract.api.1** — `ail api` lists a function's conditions, as written,
under the keys `requires` and `ensures` (arrays of strings, present only when
non-empty), and `ail api --text` prints them under the signature.
*(ADR 0213)*

```ail tool=api,FILE expect="requires" rules=contract.api.1
fn half(n int) int
    requires n >= 0
    n / 2

fn main()
    print(half(4))
```

**contract.api.2** — For a method that implements a trait method with
contracts, `ail api` adds `"inherits"` (the trait method's name) and the
trait's conditions as written under `"inherited_requires"` and
`"inherited_ensures"`, each present only when non-empty; `ail api --text`
prints them after the method's own, marked `(inherited)`. A struct's
invariants are listed under `"invariants"`, and `--text` prints each as an
`invariant` line under the struct. *(ADR 0303, ADR 0304, ADR 0305)*

```ail tool=api,FILE expect=inherited_ensures rules=contract.api.2
trait Shape
    fn area(s Self) int
        ensures result >= 0

struct Sq
    side int

impl Shape for Sq
    fn area(s Sq) int
        ensures result >= 0
        s.side * s.side

fn main()
    print(Sq.area(Sq(side: 2)))
```

**contract.fuzz.1** — `ail test --contracts FILE` checks contracts by running
generated inputs ([ADR 0868](../decisions/0868-ai-first-tooling-after-rust-removal.md)
item 5, [ADR 0885](../decisions/0885-contracts-checked-by-fuzzing.md)). For each function that has `requires` or `ensures` lines it makes a
property of the function's parameters (`test.prop.1-5`): an input that fails
a `requires` is discarded, any other calls the function. A failing `ensures`,
or a contract of anything the function calls, ends the property with the
smallest failing input, reported as `input: n = 1` with the failing condition,
the number of cases and the seed. Only these checks run: the file's own
`@test` functions don't. The exit status and `--json` form are those of
`test.run.3-4`, and `--cases` and `--seed` apply.

**contract.fuzz.2** — A free function whose parameters are all property-test
types (`test.prop.2`), without a generic, `mut` or `move` parameter, is
checked; a method, or a function whose parameter type can't be generated, is
not (it is reported as `skip` with the reason when it has contracts and its
type is the cause).

**contract.fuzz.3** — A function with effects (`eff.infer.1`) is reported as
`skip` and is not called with generated inputs, so checking contracts does
no I/O. With no contracts in the file, the output is `no contracts to check`
and the status 0.

```ail tool=test,--contracts,FILE status=1 expect=input: rules=contract.fuzz.1,contract.fuzz.2
fn half(n int) int
    requires n >= 0
    ensures result * 2 == n
    n / 2

fn main()
    print(half(4))
```

```ail tool=test,--contracts,FILE expect=skip rules=contract.fuzz.3
fn shout(n int) int
    ensures result == n
    print(n)
    n

fn main()
    print(shout(1))
```

## Checked examples for the rules without one

Each block is checked; the blocks marked `norun` trap on purpose, so they are not run (contract.ens.1, contract.inv.2, contract.inv.3).

```ail norun rules=contract.ens.1
fn half(n int) int
    ensures result * 2 == n
    n / 2

fn main()
    print(half(8))
    print(half(7))
```

```ail rules=contract.inv.1
struct Range
    lo int
    hi int
    invariant lo <= hi

fn main()
    r := Range(lo: 1, hi: 5)
    print(r.hi - r.lo)
```

```ail norun rules=contract.inv.2
struct Range
    lo int
    hi int
    invariant lo <= hi

fn main()
    r := Range(lo: 5, hi: 1)
    print(r.hi)
```

```ail norun rules=contract.inv.3
struct Counter
    n int
    invariant n >= 0

fn Counter.drop_to(mut self, k int)
    self.n = k - 10

fn main()
    mut c := Counter(n: 3)
    c.drop_to(2)
    print(c.n)
```

```ail rules=contract.inh.1,contract.inh.2,contract.inh.3
trait Sized
    fn size(self) int
        ensures result >= 0

struct Box
    w int

impl Sized for Box
    fn size(self) int
        ensures result < 100
        self.w * self.w

fn main()
    print(Box(w: 4).size())
```

```ail rules=contract.run.2
fn fact(n int) int
    requires n >= 0
    ensures n == 0 or result == n * fact(n - 1)
    if n <= 1
        return 1
    n * fact(n - 1)

fn main()
    print(fact(5))
```
