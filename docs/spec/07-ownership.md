# 07. Ownership and value semantics

Chapter key: `own`

> **Status:** Drafted (partial). Specified: the value model, parameter modes,
> exclusivity, copy reporting, borrows, unique resource types, `defer`, and
> shared references, and `move` parameters. Not yet designed: closures, and
> element projections.

## Values

**own.val.1** — Binding, assignment, passing an argument without `mut`, and
returning each produce a value that is observably independent of its source.
A mutation through one name MUST NOT be observable through any other name,
except through a `ref` (`own.ref.1`). *(ADR 0060)*

**own.val.2** — An implementation MAY share storage between values (e.g.
copy-on-write), provided `own.val.1` holds. *(ADR 0060)*

```ail
a := Point(x: 1.0, y: 2.0)
mut b := a
b.x = 6.0
# a.x is still 1.0
```

## Parameters

**own.param.1** — A parameter declared without `mut` is read-only within the
function. *(ADR 0061)*

**own.param.2** — A parameter declared `mut p T` may be mutated by the
function, and those mutations are visible in the caller's argument after the
call. The argument MUST be a mutable place and MUST be written with a leading
`mut` at the call site. *(ADR 0061)* Method receivers are the exception: see
`trait.method.3`.

```ail
fn shift(mut p Point, dx f64)
    p.x = p.x + dx

mut b := Point(x: 1.0, y: 2.0)
shift(mut b, 5.0)
```

## Exclusivity

**own.excl.1** — During a call, a place passed as a `mut` argument MUST NOT
overlap any other argument of the same call. *(ADR 0064)*

**own.excl.2** — A violation of `own.excl.1` that is statically certain is a
compile-time error. *(ADR 0064)*

**own.excl.3** — If whether two arguments overlap depends on run-time values,
the implementation MUST check before the call and trap on overlap. Each such
check MUST be listed in the performance report. *(ADR 0064)*

```ail
f(mut x, x)                  # compile-time error
swap(mut a[i], mut a[j])     # traps if i == j
```

## Copies

**own.copy.1** — Copies of values are never written explicitly. The
implementation MUST report every site where a deep storage copy may occur,
in the performance report and through the semantic API. *(ADR 0065)*

**own.move.1** — A parameter declared `move p T` takes ownership of its
argument. *(ADR 0068)*

**own.move.2** — An argument passed to a `move` parameter MUST be written with a
leading `move` at the call site. Any later use of the moved-from binding is a
compile-time error. *(ADR 0068)*

```ail
unique struct File
    fd int

fn close(move f File)
    ...

close(move f)
```

## Borrows

**own.borrow.1** — A parameter's access to its argument lasts only for the
duration of the call. It cannot be stored in a value, captured beyond the
call, or returned. Storing or returning a parameter stores or returns a value.
*(ADR 0062)*

**own.borrow.2** — The language has no lifetime parameters or lifetime
annotations. *(ADR 0062)*

## Shared references

**own.ref.1** — `ref T` is the type of a shared, mutable cell holding a value
of type `T`. Copies of a `ref` value refer to the same cell, and a mutation
through any copy is observable through all of them. *(ADR 0063)*

**own.ref.2** — A `ref` value MUST NOT be accessible from more than one task,
unless it is wrapped in a synchronised type defined by the concurrency
chapter. *(ADR 0063, ADR 0025)*

```ail
struct Node
    value int
    children List[ref Node]
```

**own.ref.3** — Field access and method calls on a `ref T` value apply to the
`T` held in the cell, with no explicit dereference. *(ADR 0067)*

**own.ref.4** — The contents of a `ref` cell MAY be mutated through any copy
of the reference, including one held in an immutable binding. *(ADR 0067)*

```ail
root := ref Node(value: 1, children: [])
root.value = 2
```

## Unique types and cleanup

**own.uniq.1** — A struct or enum declaration prefixed with `unique` declares
a unique type. A unique type cannot be copied. Its values can only be moved.
*(ADR 0066, ADR 0069)*

**own.uniq.2** — Assigning, returning, or passing a unique value to a
parameter that takes ownership moves it. Any later use of the moved-from
binding is a compile-time error. *(ADR 0066)*

**own.uniq.3** — A unique type MAY define a cleanup operation. It MUST run
exactly once, when the scope that owns the value exits by any path
(including a trap), unless the value was moved out of that scope first.
*(ADR 0066)*

**own.uniq.4** — Cleanup operations for values owned by the same scope run in
reverse order of declaration. *(ADR 0066)*

**own.defer.1** — `defer S` schedules statement `S` to run when the enclosing
block exits by any path, including a trap. *(ADR 0066)*

**own.defer.2** — Deferred statements in the same block run in reverse order
of their `defer` statements. *(ADR 0066)*
