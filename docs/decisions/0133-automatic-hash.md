# 0133. `Hash` is implemented automatically, like `Eq` and `ToStr`

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** Designer (2026-10-05)
- **Topic:** 10 — Traits; 4 — Type system
- **Spec rules:** [trait.auto.5](../spec/10-traits.md)
- **Amends:** [ADR 0126](0126-automatic-equality-and-text.md) (its exception to [ADR 0073](0073-explicit-trait-impls.md), "limited to two built-in traits", now covers three: `Eq`, `ToStr` and `Hash`)
- **Extends:** [ADR 0074](0074-inline-generic-bounds.md) (`Hash` is a bound), [ADR 0131](0131-maps-and-sets.md) (keys and set elements must be `Hash`)
- **Evidence:** `examples/sheet.ail` (tokbench, mean of five tokenizers)

## Context

Maps and sets need their keys hashed (ADR 0131). ADR 0126 made `Eq` and
`ToStr` automatic and said the exception to ADR 0073's explicit `impl` was
limited to two built-in traits. A third widens a limit the designer stated, so
under AGENTS.md (escalation rule 1) it went to the designer, with a
recommendation for automatic. The designer chose automatic.

Hashing has no user-written meaning yet: custom hashing isn't designed
(OQ-042), so an `impl Hash` would be an empty line that can't say anything. The
spreadsheet example has one user-defined key type, `Pos`; the other two
programs key by `str`.

## Options considered

Tokens are the mean of five tokenizers on `examples/sheet.ail`.

| Option | Tokens | Key type needs |
|---|---|---|
| **Automatic (chosen)** | 6019.4 | nothing |
| Empty `impl Hash for Pos` | 6024.8 (+5.4, +0.09%) | one line per key type |

The cost is a tie on tokens: one line in sheet, none in interp and json. The
difference is in first attempts. `Map[Pos, V]` fails until the line is added,
the failure ADR 0126 rejected for `Eq`. The sheet's drafter wrote no
`impl Hash for Pos`, assuming keys hash wherever `Eq` is automatic.

## Rationale

The same argument as ADR 0126: the exception is limited to traits whose
meaning the language fixes, so `x == y`, `"{x}"` and a key lookup never depend
on which declarations happen to exist. Hash has even less to declare than the
others, since there is nothing a user could write in the `impl`.

## Decision

1. `Hash` is a prelude trait with no methods. Map keys and set elements must be
   `Hash` (ADR 0131).
2. A type is `Hash` when its `Eq` is automatic (trait.auto.1) or derived by an
   empty `impl Eq` (trait.derive.1), and all its parts are `Hash`.
3. `Hash` types: `int`, `bool`, `str`, `char`, `T?` and `List[T]` when `T` is
   `Hash`, `Ordering`, and structs, enums and error types meeting rule 2.
   Generic types are checked per instance: `Box[int]` is `Hash`, `Box[float]`
   isn't. Not `Hash`: `float`, `Map`, `Set` and `any Trait`.
4. A hand-written `fn eq` (an `impl Eq` with a body) removes `Hash`, from that
   type and from every type that contains it.
5. `impl Ord` doesn't change `Eq`: `==` never calls `cmp`. A type with
   `impl Ord` (derived or written) and an automatic `Eq` stays `Hash`; the
   sheet's `Pos` is one.
6. `impl Hash` is an error, with the help that `Hash` is automatic.
7. `[T Hash]` is a bound, and `Hash` implies `Eq`: a `Hash` bound provides
   `==`, as an `Ord` bound does (trait.std.2). `Map` and `Set` are declared
   with the bound (`Map[K Hash, V]`, `Set[T Hash]`), so a type that wraps one
   of them writes it too (gen.bound.4).
8. Hash values are never visible to programs: there's no `hash()` method.
   Output can't depend on the hash function.

```ail
struct Pos
    col int
    row int

impl Ord for Pos              # derived cmp; Eq stays automatic, so Pos is Hash

struct User
    id int
    name str

impl Eq for User              # hand-written eq: User isn't Hash
    fn eq(self, other Self) bool
        self.id == other.id

fn uniq[T Hash](xs List[T]) Set[T]
    mut seen := Set[T]()
    for x in xs
        seen.add(x)
    seen

ok := Set[Pos]()              # fine
bad := Set[User]()            # error: User isn't Hash: it has its own eq
```

## Consequences

- A key type costs no lines, so `Map[Pos, V]` works at once.
- A type with its own `eq` can't be a key, and the diagnostic names the
  cause, such as the type's own `eq` or a `float` field. OQ-042 covers custom
  hashing.
- ADR 0126's exception to ADR 0073 now covers three built-in traits. Each is
  fixed by the language; widening it again needs the designer.
- `Hash` implying `Eq` is built in, like `Ord` implying `Eq`. How a user trait
  would declare a supertrait stays OQ-039, which must cover `Hash`.
- The example `[K Hash + Eq, V]` in ADR 0074 stays valid, but `+ Eq` is
  redundant.

## Rejected alternatives

- **Opt in with `impl Hash for T`:** +5.4 tokens (0.09%) in sheet, a tie, but a
  first-attempt failure ADR 0126 already rejected for `Eq`, and a line that
  can't say anything.
- **`@derive(Hash)`:** the same cost, plus an attribute (rejected for `Eq` in
  ADR 0126).
- **A hand-written `fn hash(self, mut h Hasher)` now:** not designed, and it can
  disagree with `eq` (OQ-042).
- **Hashing `float`:** `-0.0 == 0.0` and `nan != nan` break the rule that equal
  keys hash alike and a key finds itself, so `float` isn't `Hash`.
- **Sorted maps keyed by `Ord`:** rejected in ADR 0131.
