# 07. Ownership and value semantics

Chapter key: `own`

> **Status:** Drafted (partial). Specified: the value model, parameter modes,
> borrows, and shared references. Not yet designed: exclusivity details
> (OQ-032), copy reporting (OQ-033), resource cleanup (OQ-034), closures, and
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
`mut` at the call site. *(ADR 0061)*

```ail
fn shift(mut p Point, dx f64)
    p.x = p.x + dx

mut b := Point(x: 1.0, y: 2.0)
shift(mut b, 5.0)
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
