# 0536. Generics and traits in native code, by monomorphisation

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated the native self-compile lanes on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), gen.decl.1-2, gen.infer.1, gen.bound.1, trait.decl.2, trait.impl.1
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR), [ADR 0393](0393-native-modules.md) (module keys), [ADR 0602](0602-native-enum-layout.md) (enums), lane 120's struct objects and `Lower.user_call`

## Context

The native subset had no generic functions, generic structs or enums, and
no traits. The compiler's own code uses one generic function
(`absent[T]() T?` in `syntax/parse.ail`, 7 calls) and many `impl` blocks;
AIL programs written by agents use `List`-shaped helpers (`first[T]`,
`largest[T Ord]`) and small generic containers routinely. The interpreter
treats generics dynamically, which a native backend can't.

## Decision

1. **Monomorphisation.** Each use of a generic item with concrete types is
   its own instance, lowered as a plain function or type with each type
   parameter replaced. Instances are named with their types, joined by
   `, `: `first[int]`, `pick[str]`, `Box[str]`, `Pair[int, str]`,
   `Box[str].get`, `Tree[int]`; a module item keeps its module prefix
   (`m.f[int]`). An instance type is an ordinary struct or enum under that
   name, so layout, `==`, copies and printing need nothing new.
2. **Type arguments** come, in order, from the explicit `f[T](...)` or
   `G[T](...)`, from unifying the parameters' written types with the
   arguments' types (or the fields' with the given values), and from the
   type the context expects (`make[int]()` or `x: Box[int] := Box(...)`).
   A parameter none of these fixes is reported ("can't tell `T` ...").
3. **Bodies are lowered after the program's functions**, from a queue: an
   instance's body may ask for more instances, which join the queue, so
   recursive and nested instances (`wrap(Pair(left: [1], right: wrap("x")))`)
   work. Instances are declared on first use and only used ones are built.
4. **Methods of a generic type** are instantiated per type instance on
   first call (`Box[int].bump`); inside them the type parameters map to
   the instance's types.
5. **Traits.** The methods of an `impl Trait for T` block are methods of
   `T`, keyed `T.m` like any method; each default method of the trait that
   the block doesn't define is copied to `T`. A trait bound needs nothing
   at run time: an instance's body calls the concrete type's method, and
   operators on a bound type (`x > best` under `Ord`) are those of the
   concrete type. The checker has already proved the bounds.
6. **Built-in types.** An `impl` for `int`, `float`, `bool` or `str` gives
   methods keyed `int.m` (no module prefix: the types are everyone's), and
   `impl Trait for List[T]` is instantiated per element type
   (`List[int].m`), so `[1, 2].size()` and the generic code that calls it
   work as for a struct. An associated call `T.f(...)` on a type parameter
   (trait.assoc.1-2) calls the concrete type's `f`.
7. **Not yet in the subset:** generic methods of `int`, `float`, `bool`, `str` and `List` owners (those of structs and enums work, gen.method.3), `any Trait`
   values (dynamic dispatch, ADR 0616), built-in derives (`impl Ord for S`
   with no body), and optionals of a type parameter (`T?`, with lane 175's
   optionals). Generator derives (`ToJson`, `FromJson`, `Default`) reach
   the backend as written methods (ADR 0807).

## Rejected alternatives

- **One shared body with boxed values** (uniform representation, as the
  interpreter has). Every `T` value would be a pointer, floats boxed, and
  each operator a runtime dispatch: slower code in exactly the helpers
  agents write most, and a second value layout for the collector. Code
  size from monomorphisation is bounded by use: only instances reached are
  built, and the compiler itself has one generic.
- **Dictionary passing for trait bounds** (a table of methods per bound as
  a hidden argument). Needed for `any Trait`, not for static bounds;
  monomorphised calls are direct and can be inlined. `any Trait` will get
  its own decision when it enters the subset.
- **Mangled instance names** (`first$i`). The written form is what a
  trap, a profile and a MIR dump show to a reader, human or agent, and it
  can't clash with a user name: `[` isn't allowed in identifiers.

## Consequences

- `tests/native/run/generic_fns.ail`, `generic_types.ail` and
  `generic_traits.ail` and `impl_builtin.ail` match the interpreter on x86-64 and aarch64;
  `tests/native/generics_out.ail` holds the edges.
- Every new generic use costs a body's worth of code; a later size pass can
  share instances whose types have the same layout.
