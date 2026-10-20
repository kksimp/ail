# 07. Ownership and value semantics

Chapter key: `own`

> **Status:** Drafted (partial). Specified: the value model, parameter modes,
> exclusivity, copy reporting, borrows, unique resource types and their
> cleanup, `defer`, shared references, and `move` parameters, with map entries
> as places. Not yet designed: closures, element projections, `move self`, and
> unique values in collections and generic code.

## Values

**own.val.1** — Binding, assignment, passing an argument without `mut`, and
returning each produce a value that is observably independent of its source.
A mutation through one name MUST NOT be observable through any other name,
except through a `ref` (`own.ref.1`). *(ADR 0060)*

**own.val.2** — An implementation MAY share storage between values (e.g.
copy-on-write), provided `own.val.1` holds. *(ADR 0060)*

```ail skip=context
a := Point(x: 1.0, y: 2.0)
mut b := a
b.x = 6.0
# a.x is still 1.0
```

## Places

**own.place.1** — A place is a local binding, a parameter or `self`, or a
field (`p.f`) or element (`p[i]`) of a place, or an entry (`p[k]`) of a map
place. A place is mutable when its root is a `mut` binding, a `mut` parameter
or `mut self`. A map entry is a place with the added rules of `type.map.3`:
only when `m[k]` is the whole target of `=` does it insert a missing key or
replace the value. Every other use of an entry as a place needs `k` to be
present and traps if it isn't. *(ADR 0117, ADR 0131)*

```ail fragment
mut counts := Map[str, int]()
counts["a"] = 1      # inserts
counts["a"] += 1     # fine: "a" is present
counts["b"] += 1     # traps: "b" is missing
```

## Parameters

**own.param.1** — A parameter declared without `mut` is read-only within the
function. *(ADR 0061)*

**own.param.2** — A parameter declared `mut p T` may be mutated by the
function, and those mutations are visible in the caller's argument after the
call. The argument MUST be a mutable place and MUST be written with a leading
`mut` at the call site. *(ADR 0061)* Method receivers are the exception: see
`trait.method.3`.

```ail skip=context
fn shift(mut p Point, dx f64)
    p.x = p.x + dx

mut b := Point(x: 1.0, y: 2.0)
shift(mut b, 5.0)
```

## Exclusivity

**own.excl.1** — During a call, a place passed as a `mut` argument, including
a `mut self` receiver, MUST NOT overlap another `mut` argument of the same
call. *(ADR 0064, ADR 0119)*

**own.excl.2** — A violation of `own.excl.1` that is statically certain is a
compile-time error. *(ADR 0064)*

**own.excl.3** — If whether two `mut` arguments overlap depends on run-time
values, the implementation MUST check before the call and trap on overlap.
Each such check MUST be listed in the performance report. Two entries `m[j]`
and `m[k]` of the same map place overlap when their keys are equal, like list
elements with equal indexes. *(ADR 0064, ADR 0131)*

**own.excl.4** — A read-only argument or receiver that overlaps a `mut`
argument of the same call is passed as a copy of its value, made before the
call. Each such copy site MUST be listed in the performance report.
*(ADR 0119)*

**own.excl.5** — The body of `for mut x in e` MUST NOT use `e`, or any place
that may overlap `e`, except through `x`. An overlap that depends on index
values counts as an overlap, and so does one that depends on map keys:
inside `for mut x in m[k]`, `m[j]` may overlap `m[k]`. Violations are
compile-time errors. *(ADR 0121, ADR 0131)*

```ail skip=context
xs.push(xs[0])               # xs[0] is copied before the call
f(mut x, mut x)              # compile-time error
swap(mut a[i], mut a[j])     # traps if i == j; a.swap(i, j) doesn't
swap(mut m[a], mut m[b])     # traps if a == b (equal keys overlap)
```

## Copies

**own.copy.1** — Copies of values are never written explicitly. The
implementation MUST report every site where a deep storage copy may occur,
in the performance report and through the semantic API. *(ADR 0065)*

**own.move.1** — A parameter declared `move p T` takes ownership of its
argument. *(ADR 0068)*

**own.move.2** — An argument passed to a `move` parameter MUST be written with a
leading `move` at the call site. Any later use of the moved-from binding is a
compile-time error. *(ADR 0068)* An argument written `move` to a parameter
that isn't `move` is a compile-time error. *(ADR 0190)*

**own.move.3** — Only a value the current scope owns can be moved: a local
binding, a `move` parameter, or a new value such as a call's result. Moving a
read-only or `mut` parameter, `self`, a loop variable, or a field or element
of another value is a compile-time error. *(ADR 0190)*

**own.move.4** — "Later use" is decided on every path through the function.
A use that some path reaches after a move is a compile-time error, and so is a
move inside a loop of a binding declared outside it, unless every path from
the move leaves the loop or gives the binding a new value first. Assigning a
whole binding a new value (`x = e`) makes it usable again. *(ADR 0190)*

**own.move.5** — A `move` argument of a copyable type moves too: the binding
can't be used afterwards, and no copy is made. *(ADR 0190)*

**own.move.6** — A method declared `fn T.m(move self, ...)` takes ownership
of its receiver. The call is written `x.m(...)`, with no mark: calling it on
a local binding or `move` parameter moves it (`own.move.2`, `own.move.4`),
and calling it on a part of another value (`w.f.m()`, `xs[i].m()`) is a
compile-time error (`own.move.3`). Inside the method, `self` is owned like a
`move` parameter: it can be moved, and it is released when the method
returns unless it was moved (`own.uniq.7`). `move self` is not allowed in a
`trait` or `impl` block. *(ADR 0317)*

```ail fragment
unique struct File
    fd int

fn close(move f File)
    print("closing {f.fd}")

f := File(fd: 3)
close(move f)
```

```ail fragment
unique struct Conn
    id int

fn Conn.finish(move self) int
    print("finishing {id}")
    id

c := Conn(id: 1)
n := c.finish()
```

```ail error=own.move.6
unique struct Conn
    id int

fn Conn.finish(move self) int
    id

fn main()
    c := Conn(id: 1)
    n := c.finish()
    print(c.id)
```

```ail
unique struct Conn
    id int

fn Conn.finish(move self) int
    id

fn main()
    c := Conn(id: 1)
    print(c.id)
    n := c.finish()
```

```ail error=own.move.2
unique struct File
    fd int

fn close(move f File)
    print("closing {f.fd}")

fn main()
    f := File(fd: 3)
    close(move f)
    print(f.fd)
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

```ail fragment
struct Node
    value int
    children List[ref Node]
```

**own.ref.3** — Field access and method calls on a `ref T` value apply to the
`T` held in the cell, with no explicit dereference. *(ADR 0067)*

**own.ref.4** — The contents of a `ref` cell MAY be mutated through any copy
of the reference, including one held in an immutable binding. *(ADR 0067)*

```ail fragment
struct Node
    value int
    children List[ref Node]

root := ref Node(value: 1, children: [])
root.value = 2
```

**own.ref.5** — `ref e` creates a new cell holding the value of `e`, which is
then independent of `e` (`own.val.1`). `ref T?` is an optional reference. A
constant's value can't contain a `ref` (`mod.const.2`). *(ADR 0067, ADR 0192)*

**own.ref.6** — Two `ref` values are equal (`==`) when they refer to the same
cell. A `ref` type isn't `Ord` or `Hash`, so it can't be a map key or set
element. The text form of a `ref` is `ref` followed by its contents' text
form; a cell met again inside its own text form is shown as `ref ...`.
*(ADR 0192)*

**own.ref.7** — While a `mut` access to a cell's contents is in progress (a
`mut self` method called on a `ref`, a `mut` argument or `for mut` loop that
reaches into a cell), using that cell through any other copy of the reference
traps. *(ADR 0193, ADR 0064)*

```ail fragment
struct Counter
    n int

fn Counter.bump(mut self)
    n += 1

a := ref Counter(n: 0)
b := a
b.bump()
assert(a.n == 1)
assert(a == b)
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

**own.uniq.5** — A field of a unique type makes its struct or enum unique:
declaring it in a type that isn't `unique` is a compile-time error. A unique
type can't be a map key or set element, the contents of a `ref`, or a type
argument of a generic type or function. Optionals and results of a unique
type are unique. *(ADR 0191)* It can be a list element or a map value
(`own.uniq.8`). *(ADR 0318)*

**own.uniq.6** — A unique type's cleanup is its implementation of the
prelude trait `Drop`, whose one method is `fn drop(mut self)`. Only a unique
type may implement `Drop`, and its `drop` MUST NOT be called directly.
*(ADR 0191)*

**own.uniq.7** — When a unique value is released, its `drop` runs first, then
its unique fields are released, last field first. Assigning a new value to a
place releases the unique value it held. A call's unique result that a
statement doesn't use is released at the end of that statement. The binding
of `if x := e` owns its value until the branch ends. A function's `move`
parameters are released when it returns. *(ADR 0191)*

```ail fragment
unique struct Lock
    id int

impl Drop for Lock
    fn drop(mut self)
        print("unlock {id}")

fn release(move l Lock)
    print("releasing {l.id}")

a := Lock(id: 1)
b := Lock(id: 2)
release(move a)
# prints "releasing 1", "unlock 1", then "unlock 2" when the block ends
```

```ail error=own.uniq.5
unique struct File
    fd int

struct Holder
    f File
```

## Unique values in lists and maps

**own.uniq.8** — `List[T]` and `Map[K, T]` of a unique `T` are unique: they
move instead of being copied (`own.uniq.2`), and a struct with such a field
is unique (`own.uniq.5`). The elements of a list literal, and the values of a
map literal, move into it. When the collection is released, its elements are
released, last first; `xs.clear()` releases the elements it removes, and
assigning to `xs[i]` or `m[k]` releases the value it replaces. *(ADR 0318)*

**own.uniq.9** — For a list or map of a unique type: `xs.push(move x)`,
`xs.insert(i, move x)` and `xs.extend(move ys)` take ownership of their
argument, which is marked `move` (`own.move.2`); `xs.pop()`,
`xs.remove_at(i)` and `m.remove(k)` move an element out to the caller.
Indexing (`xs[i]`, `m[k]`) and iteration (`for x in xs`) borrow an element in
place: it can be read, passed to a parameter without `move`, and changed
through `mut` (`xs[i].f = v`, `for mut x in xs`), but binding, returning or
moving it is a compile-time error (`own.uniq.1`, `own.move.3`). *(ADR 0318)*

**own.uniq.10** — A built-in method that would copy an element of a list or
map of a unique type is a compile-time error: `first`, `last`, `min`, `max`,
`sorted` and `take` of a list, and `get` and `values` of a map.
*(ADR 0319)* A task handle is not a unique value in this sense: it can't be
stored in a list (`conc.task.5`); a `TaskGroup` holds many tasks. *(ADR 0319)*

```ail fragment
unique struct Conn
    id int

impl Drop for Conn
    fn drop(mut self)
        print("close {id}")

mut pool := List[Conn]()
pool.push(move Conn(id: 1))
pool.push(move Conn(id: 2))
for c in pool
    print("open {c.id}")
if c := pool.pop()
    print("took {c.id}")
# prints "open 1", "open 2", "took 2", "close 2", then "close 1" when the block ends
```

```ail error=own.uniq.10
unique struct Conn
    id int

fn main()
    pool := [Conn(id: 1)]
    first := pool.first()
```

```ail
unique struct Conn
    id int

fn main()
    pool := [Conn(id: 1)]
    print(pool[0].id)
```

**own.defer.1** — `defer S` schedules statement `S` to run when the enclosing
block exits by any path, including a trap. *(ADR 0066)*

**own.defer.2** — Deferred statements in the same block run in reverse order
of their `defer` statements. *(ADR 0066)* Deferred statements and the cleanup
of values owned by the block run together in reverse order of the `defer`
statements and declarations. *(ADR 0191)*

**own.defer.3** — A deferred statement is a call, another expression, or an
assignment. It can't be `return`, `break`, `continue`, a binding, a loop or
another `defer`; it can't move a value; and a binding it uses MUST NOT be
moved before the block ends. *(ADR 0191)*

```ail fragment
defer print("second")
defer print("first")
print("body")
```
