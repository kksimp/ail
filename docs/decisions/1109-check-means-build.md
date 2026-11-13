# 1109. What `ail check` accepts, the native build builds

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076); the designer approved the AI-first review (item S26) for 0.1.0 on 2026-11-12 and asked on 2026-11-13 that `ail check` be "really really good for 0.1".
- **Topic:** Tools ([22-tools.md](../spec/22-tools.md)), native subset ([23-native.md](../spec/23-native.md))
- **Spec rules:** native.subset.1, type.coll.9-19, type.map.2, type.str.3, expr.key.4, own.excl.5, type.tuple.2

## Context

Native code is the only backend (ADR 0904). A program `ail check` accepts and
the native build then refuses with `native.subset.1` is a mistake an agent
finds one step too late: after it has written the code, believed it, and
moved on. The review probes and the corpus found such programs by chance
(`Grid.fill` beside a program's own `Heap`, `.sorted()` on lists of lists,
`else` on a fallible method called through `any Trait`, a list of `Mutex`).
Chance doesn't cover a vocabulary of about 100 built-in methods over a dozen
element types.

## Decision

1. **A generated matrix.** `tools/native-matrix/matrix.py` writes one program
   per collection or prelude type (`List`, `Map` by key and by value, `Set`,
   `Heap`, `Deque`, `Grid`, `Seq`, plain values and tuples) and element type
   (`int`, `float`, `str`, `char`, `bool`, `u8`, a distinct type, a tuple, a
   struct, an ordered struct, a flat and a payload enum, an optional, a nested
   list, a map, a set, `bytes`), each method call in a function of its own,
   plus the scalar methods and whole-package cases (names across a module's
   files). It drops what `ail check` refuses, builds the rest natively, runs
   it, and names every snippet the build refuses (`HOLE`) or that traps.
2. **A gate suite.** The programs that build, with their output, are frozen
   in `tests/native_matrix/` and the `native_matrix` suite checks, builds and
   runs each one on every host. A new method or element type that check
   accepts and native code doesn't is red there.
3. **Close holes in the backend first.** Where native support is possible it
   is added, by a loop over the operations every element type already has
   (`==`, `<`, text, copy). Only where it isn't possible for 0.1.0 does the
   checker refuse the program, with the diagnostic the build would give.

The first run (batch 10) built 1,952 snippets and found 65 holes. They were
closed in the backend: `counts` of any hashable element; `chunks`, `windows`
and `flatten` of composite elements (the pieces are copied, own.val.1);
`index_of` by the element's own `==`; `join` and `"{x}"` of any value with a
text (lists of lists, optionals, maps); sort keys of `char` and sized
integers; `bool` sets and comprehensions; `top` on narrow keys; `sorted` on
sets of `u8` and `bytes`; `for mut` over every element changed by whole
assignment; maps and sets inside tuples; `else` on a fallible method called
through `any Trait`; lists of `Mutex`, `Atomic` and `Channel` handles. One
was a checker hole: a `{}` typed by its first use (`m[k] += 1`) now needs a
hashable key, as `Map[K, V]()` does (type.map.2).

## Rejected alternatives

- **Make `ail check` run the native lowering.** It would make check and build
  agree by construction, but check would take the build's time and report
  lowering messages for programs with type errors still in them. The matrix
  keeps check fast and the agreement tested.
- **Refuse in the checker whatever native code lacks.** Cheaper, but each
  refusal is a language feature an agent reads about and can't use; the
  priority order (ADR 1077) puts AI understanding first.
- **Hand-written cases only.** They cover what someone thought of; the
  matrix covers the product of methods and element types.
