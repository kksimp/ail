# Ownership and value semantics: options explained

> **Non-normative design aid,** 2026-10-04. Background for the ownership
> questions (status.md topic 7). The examples use syntax decided so far.
> Anything marked *illustrative* is not decided.

## The core question

When you write `b := a`, or pass `a` to a function, do you get **another
name for the same object** or **an independent value**? Everything else
follows from that answer: aliasing, data races, what the optimizer may assume,
lifetimes, and how often the GC is needed.

## Model A: Mutable value semantics (Hylo/Val-style; Swift structs)

Every value behaves like a number. Assignment and passing give you an
independent value. Mutation happens only through a variable you own, or
through a parameter the caller explicitly lends as `mut`. No two names can
ever see each other's changes, unless you explicitly ask for a shared
reference (`ref`, see below).

```
struct Point
    x f64
    y f64

fn shift(mut p Point, dx f64)       # may change the caller's variable
    p.x = p.x + dx

a := Point(x: 1.0, y: 2.0)
mut b := a                          # independent value
shift(mut b, 5.0)                   # the call site shows b may change
# a.x is still 1.0; b.x is 6.0
```

"Copies" are cheap. Large values (lists, strings, big structs) share their
storage internally, and storage is copied only if someone mutates a shared
copy (copy-on-write). At a variable's last use, the compiler hands over the
storage instead of copying, so in-place reuse (ADR 0012) falls out naturally.

**Why it fits AIL:**
- **Local reasoning:** to know whether something can change, you only look at
  the code in front of you. That's the biggest win for AI and reviewers.
- **No aliasing by default:** the optimizer gets C's `restrict` everywhere,
  automatically. This is one of the main levers for beating C.
- **Data-race freedom for free:** values can't be shared mutably across
  tasks (ADR 0025).
- **Fewer GC objects:** most data are values (stack, in place, or
  uniquely-owned buffers), so the GC heap stays small (ADR 0012).
- **No lifetime annotations** (see "Borrows").

**Cost:** structures that are genuinely shared and mutable (graphs, caches,
observer lists, doubly linked lists) need an explicit shared reference type.

## Model B: Reference semantics with GC (Java, C#, Go pointers)

```
mut b := a          # b and a are the SAME object
b.x = 6.0           # a.x is now 6.0 too
```

- **Pros:** familiar, and arbitrary object graphs are trivial.
- **Cons:** aliasing is everywhere, so any call might change anything you
  passed. Readers and AI must track it, and the optimizer must assume the worst.
  Data-race freedom needs a heavy extra type system (e.g. Pony's reference
  capabilities). More objects end up on the GC heap.

## Model C: Rust-style ownership with lifetimes (plus a GC for sharing)

```
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str   # Rust
```

- **Pros:** the most expressive zero-copy references.
- **Cons:** lifetime annotations, borrow-checker complexity, and many tokens.
  It's the most common source of AI compile-error loops in Rust today.

The same function under Model A with second-class borrows:

```
fn longest(a str, b str) str        # returns a value; storage is shared, not copied
```

## Borrows (only relevant to Model A)

A **borrow** is temporary access to a value without copying it. Parameters
are borrows: read-only by default, or `mut` when the caller lends it.

- **Second-class borrows:** a borrow can be used during the call but can't
  be stored in a struct or returned. So **no lifetime annotations ever
  exist**. Returning or storing data means returning or storing a value,
  which is cheap thanks to shared storage.
- **First-class borrows:** borrows can be stored and returned. That's more
  zero-copy power, but the language then needs lifetimes (inferred, with
  annotations when inference fails).

What second-class borrows rule out, and the replacement:

| Pattern | Replacement |
|---|---|
| Return a reference into a collection (`&list[3]`) | Return the element value (cheap), or mutate in place with `mut list[3].x = 5` |
| A struct holding a reference to another struct | Hold the value, an index or ID, or a `ref` |
| Iterators borrowing a collection | `for x in list` is built in. Custom iterators are designed with this rule in mind. |

## Shared mutable identity

Some things genuinely need identity and sharing: a graph node, a
connection, a cache. Options:

- **Use-site `ref T`:** any type can be put in a GC-managed shared cell.
  Sharing is visible exactly where it happens.
  ```
  struct Node
      value int
      children List[ref Node]       # illustrative syntax
  ```
- **Declaration-site reference types** (Swift's `class` vs `struct`): the
  type itself decides. Clear identity, but there are two kinds of types to
  learn, and the same code means different things depending on a declaration
  elsewhere.
- **Indices and handles only:** no shared references at all. Graphs live in
  arenas or collections and are addressed by ID (the ECS style). The fastest
  and most explicit, but awkward for some designs.

A `ref` is confined to one task unless wrapped in a synchronised type (to be
designed under concurrency), which keeps ADR 0025's data-race freedom.

## Measurements (five tokenizers, ADR 0029 method)

| Example | cl100k | o200k | anthropic-legacy | tekken | spm-v1 |
|---|---|---|---|---|---|
| Rust `longest` with lifetimes | 39 | 39 | 47 | 39 | 47 |
| AIL `longest` (Model A) | **27** | **27** | **26** | **27** | **30** |
| Rust `&mut` parameter plus call | 31 | 31 | 35 | 32 | 38 |
| AIL `mut` parameter plus `mut` at call site | 25 | 25 | 28 | 28 | 31 |
| AIL `mut` parameter, no call-site marker | 24 | 24 | 27 | 26 | 30 |
| Rust struct holding a borrowed `&'a str` | 19 | 19 | 24 | 19 | 25 |
| AIL struct holding a value `str` | **11** | **11** | **9** | **11** | **12** |

Model A with second-class borrows is 20–55% smaller than Rust on these
examples, mostly because lifetimes disappear. A call-site `mut` marker costs
about one token per mutating call.
