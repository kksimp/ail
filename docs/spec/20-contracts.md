# 20. Contracts

Chapter key: `contract`

> **Status:** Drafted (partial). Specified: `requires` and `ensures` lines,
> their typing and purity, `result`, the runtime checks and their traps,
> `--no-contracts`, and contracts in `ail api`. Not yet designed: `old(x)`
> (the value of a parameter on entry), invariants on types, loop invariants,
> contracts on trait methods without a body and whether implementations
> inherit them, and static proof (see [open-questions.md](../open-questions.md),
> OQ-060).

A contract states what a function needs and what it promises, in the
function's signature, as ordinary `bool` expressions. The bootstrap checks
them at run time. A later compiler may prove a contract statically and drop
its check; that changes nothing observable except that the trap can't occur.

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
`ensures` line can't have a parameter named `result`.
*(ADR 0210)*

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

## Running

**contract.run.1** — Contracts are part of the signature: they are checked
at every call, including calls through a trait (`trait.any.1`), direct
recursion, and calls from tests (`test.run.1`). *(ADR 0211)*

**contract.run.2** — Calls made while evaluating a condition don't check
their own contracts, so a condition may call the function it describes:
`ensures n == 0 or result == n * fact(n - 1)`. *(ADR 0211)*

**contract.run.3** — `ail run --no-contracts` and `ail test --no-contracts`
evaluate no condition. Because conditions are pure (`contract.decl.3`), the
only difference a program can show is that a contract trap doesn't occur.
Checks are on by default. *(ADR 0212)*

## Tools

**contract.api.1** — `ail api` lists a function's conditions, as written,
under the keys `requires` and `ensures` (arrays of strings, present only when
non-empty), and `ail api --text` prints them under the signature.
*(ADR 0213)*
