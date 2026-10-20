# Open questions

Unresolved design questions. Each question lists its options with pros and
cons, plus a recommendation. Since [ADR 0076](decisions/0076-delegated-decisions.md),
routine questions are decided by the AI using the priority order (tokens, AI
understanding, readability, speed). Serious ones go to the designer. When a question is resolved it moves into an ADR
and its entry here is replaced by a link.

IDs (`OQ-NNN`) are never reused.

---

## OQ-001 — ISA priority policy
**Resolved** by [ADR 0008](decisions/0008-isa-priority-policy.md) (strict lockstep).

## OQ-002 — Default atomic memory ordering
**Resolved** by [ADR 0025](decisions/0025-memory-model-atomics.md).

## OQ-003 — ISA-specific hardware features in the GC and runtime
**Resolved** by [ADR 0026](decisions/0026-hardware-feature-acceleration.md).

## OQ-004 — CI merge gating
**Resolved** by [ADR 0027](decisions/0027-ci-gating-all-targets.md).

## OQ-005 — Compiler host platforms
**Resolved** by [ADR 0031](decisions/0031-hosts-and-cross-compilation.md).

## OQ-006 — Spec chapter outline
Approve, reorder, or replace the provisional chapter list in
[spec/README.md](spec/README.md). *Recommendation:* approve it as a starting
point. Chapters can be added later without renumbering existing rule IDs.

## OQ-007 — Token-efficiency benchmark methodology
**Resolved** by [ADR 0029](decisions/0029-token-efficiency-benchmarks.md).

## OQ-008 — Performance benchmark methodology
**Resolved** by [ADR 0030](decisions/0030-performance-benchmarks.md) (initial scope; three-bar suite deferred).

## OQ-009 — Integer overflow semantics
**Resolved** by [ADR 0035](decisions/0035-integer-overflow-traps.md).

## OQ-010 — Implementation language of the bootstrap compiler
**Resolved** by [ADR 0023](decisions/0023-rust-bootstrap.md).

## OQ-011 — Self-hosting and roadmap order
**Resolved** by [ADR 0022](decisions/0022-staged-self-hosting.md) (self-hosting) and [ADR 0024](decisions/0024-semantic-api-from-stage-2.md) (semantic API timing). The remaining roadmap refinements (#2 and #4) are still open in [roadmap.md](roadmap.md).

## OQ-012 — May the compiler emit C as a temporary path to native code?
**Resolved** by [ADR 0017](decisions/0017-no-c-emission.md).

## OQ-013 — Strategy for the first native backend
**Resolved** by [ADR 0018](decisions/0018-optimizing-backend-from-start.md).

## OQ-014 — Linux runtime without libc?
**Resolved** by [ADR 0020](decisions/0020-linux-no-libc.md).

## OQ-015 — Default data layout
**Resolved** by [ADR 0013](decisions/0013-compiler-chosen-layout.md).

## OQ-016 — Compressed managed references
**Resolved** by [ADR 0014](decisions/0014-compressed-managed-references.md).

## OQ-017 — Whole-program optimisation by default
**Resolved** by [ADR 0015](decisions/0015-always-whole-program-incremental.md), [ADR 0016](decisions/0016-optimized-and-observable-debugging.md).

## OQ-018 — Which memory-management R&D tracks to pursue
**Resolved** by [ADR 0012](decisions/0012-memory-rd-tracks.md).

## OQ-019 — Debug-info formats
**Resolved** by [ADR 0021](decisions/0021-debug-info-formats.md).

---

## OQ-020 — Correctness oracles for the optimising backend
**Resolved** by [ADR 0019](decisions/0019-miscompilation-defence.md).

## OQ-021 — How do Linux programs that call C libraries get a libc?
**Resolved** by [ADR 0105](decisions/0105-libc-only-with-c.md).

## OQ-022 — Keep the Rust interpreter after self-hosting?
**Resolved** by [ADR 0032](decisions/0032-keep-rust-interpreter-as-oracle.md).

## OQ-023 — Backend-ready core subset
[ADR 0022](decisions/0022-staged-self-hosting.md) requires the language features
the backend uses to stabilise early. Which features make up the "core
subset"? To be decided once the type system and syntax are drafted.

## OQ-024 — Git workflow once code exists
**Resolved** by [ADR 0028](decisions/0028-git-workflow.md).

## OQ-025 — What kind of C do the drag races compare against?
**Resolved** by [ADR 0036](decisions/0036-drag-race-c-styles.md).

## OQ-026 — Indentation unit
**Resolved** by [ADR 0040](decisions/0040-four-space-indentation.md).

## OQ-027 — Attribute and annotation syntax
**Resolved** by [ADR 0083](decisions/0083-attribute-syntax.md).

## OQ-028 — Local names vs. module-level, imported and built-in names
**Resolved** by [ADR 0084](decisions/0084-no-shadowing-module-names.md).

## OQ-029 — Error and absence handling forms
**Partly resolved:** fallbacks and early exit ([ADR 0049](decisions/0049-else-for-fallback-and-early-exit.md)),
context ([ADR 0050](decisions/0050-error-return-traces.md)), `?` on optionals
([ADR 0051](decisions/0051-no-implicit-optional-to-error.md)), and handling
success *or* failure as a statement ([ADR 0125](decisions/0125-if-binding.md):
`if x := e` … `else err`).
**Still open:** safe member access (`a?.b`), and matching on specific errors,
including on a set of error types (`IoError | ParseError`). Options for the
latter:
- qualified variant patterns from every member type (`ParseError.Empty =>`),
  plus a pattern for a whole error type;
- `match` with success and error arms.

## OQ-030 — What is an error type?
**Partly resolved:** error types are declared with `error`
([ADR 0120](decisions/0120-error-keyword.md)). Unions, propagation and
recursion are covered by [ADR 0122](decisions/0122-error-unions-and-reports.md).
**Still open:** how an inferred set converts to a public API's error type,
and inference through generic functions.

## OQ-031 — Canonical spelling of the default numeric types
**Resolved** by [ADR 0059](decisions/0059-default-numeric-spelling.md).

## OQ-032 — Exclusivity of `mut` arguments
**Resolved** by [ADR 0064](decisions/0064-exclusivity.md).

## OQ-033 — Visibility of copy-on-write copies
**Resolved** by [ADR 0065](decisions/0065-copy-reporting.md).

## OQ-034 — Deterministic cleanup of resources
**Resolved** by [ADR 0066](decisions/0066-unique-resource-types.md).

## OQ-035 — Parameter form that takes ownership
**Resolved** by [ADR 0068](decisions/0068-move-parameters.md) (`move`) and [ADR 0069](decisions/0069-unique-keyword.md) (`unique`).

## OQ-036 — Method placement, associated functions, and coherence
**Resolved** by [ADR 0082](decisions/0082-method-placement-coherence.md).

## OQ-037 — Associated types vs. trait type parameters
Should traits declare associated types (`type Item` inside `trait Iterator`;
one per implementing type), take type parameters (`trait Iterator[Item]`), or
both, as in Rust (`From[T]` vs. `Iterator`)? To be decided with collections
and iteration, measured on a real corpus. *Owner: AI under ADR 0076, unless
it proves hard to reverse.*

## OQ-038 — In-place iteration over list elements
**Resolved** by [ADR 0121](decisions/0121-for-mut.md) (`for mut x in xs`).

---

### Not yet opened
The proposals in [design/ai-author-wishlist.md](design/ai-author-wishlist.md)
will be raised as questions under their respective topics.

The 28 design topics in [status.md](status.md) each still need their own
questions. They will be opened topic by topic during Stage 1.

## OQ-039 — Supertraits and associated functions in traits
How does a user trait require another (`Ord` requires `Eq`, ADR 0127), and
can a trait declare functions without `self`, called as `T.zero()` in generic
code? Options for supertraits:
- `trait Ord Eq`, mirroring constraints (`[T Eq]`): consistent, but reads
  oddly;
- `trait Ord: Eq` (Rust, Swift): familiar, but AIL writes no `:` in
  declarations (ADR 0037).

Associated functions in traits need the concrete type at run time in shared
generic code (ADR 0080), as Swift's witness tables provide. To be decided
with the standard library (`Default`, numeric `zero`). `Hash` already
behaves as if it required `Eq` (a `[T Hash]` bound allows `==`,
[ADR 0133](decisions/0133-automatic-hash.md)), so the answer must cover it.
*Owner: AI under ADR 0076.*

## OQ-040 — `else` as an expression
**Resolved** by [ADR 0144](decisions/0144-else-as-an-expression.md) (`e else d` is the loosest operator).

## OQ-041 — Format specifications in interpolation
**Resolved** by [ADR 0173](decisions/0173-format-specifications-in-interpolation.md) (`{x:>8}`, `{r:.2}`, `{n:0>4x}`).

## OQ-042 — Custom hashing
`Hash` is automatic and can't be implemented by hand
([ADR 0133](decisions/0133-automatic-hash.md)), so a type with its own
`fn eq` can't be a map key or set element. Options: let `impl Hash` declare
which fields take part (consistent with a custom `eq` by construction), a
`fn hash(self, mut h Hasher)` method as in Rust (flexible, but can disagree
with `eq`), or keep the restriction. Hash values must stay invisible to
programs, so output never depends on the hash function. *Owner: AI under
ADR 0076; decide when a real program needs it.*

## OQ-043 — `s.len()`: bytes or characters
**Resolved** by [ADR 0208](decisions/0208-str-len-counts-characters.md) (`len()` counts characters, `byte_len()` bytes).

## OQ-044 — Nested optionals
**Resolved** by [ADR 0209](decisions/0209-nested-optionals.md) (distinct nested types, `none` is the outermost).

## OQ-045 — Capability manifests for packages
**Resolved** by [ADR 0232](decisions/0232-capabilities-per-dependency.md) (`caps LIST` per `dep` line in `ail.pkg`, default none, checked at compile time by the `--deny` inference).

## OQ-046 — Deterministic execution mode
**Resolved** by [ADR 0207](decisions/0207-deterministic-mode.md) (a guarantee for programs without `clock`, `random`, `proc` and `net`, plus `--deterministic`).

## OQ-047 — Inserting into and removing from lists
**Resolved** by [ADR 0214](decisions/0214-list-editing-methods.md) (`insert`, `remove_at`, `extend`, `clear`, `reverse`, `first`).

## OQ-048 — Editing text without string indexing
**Resolved** by [ADR 0215](decisions/0215-str-slice-and-search-methods.md) (`slice`, `char_at`, `find`, `replace`, `to_lower`, `to_upper`).

## OQ-060 — Loop invariants and static proof for contracts
Items (1) `old(x)`, (2) inherited trait contracts and (3) type invariants are
**resolved** by [ADR 0303](decisions/0303-contract-old.md),
[ADR 0304](decisions/0304-trait-contract-inheritance.md) and
[ADR 0305](decisions/0305-struct-invariants.md); checks stay run-time
([ADRs 0210–0213](decisions/0210-contract-syntax.md)). Still open: (a) loop
invariants (recommended: wait; `assert` inside the loop covers the run-time
case and a prover is the only reason to name one); (b) static proof, and
whether an optimising build keeps the checks it can't prove (recommended:
keep them, so `--no-contracts` stays an explicit choice); (c) invariants on
enums, and checks for writes through `mut` parameters outside the type's
methods (recommended: revisit with (b), since both need a prover to be cheap).
*Owner: AI under ADR 0076.*

## OQ-071 — Standard modules without `use` lines
**Resolved** by [ADR 0277](decisions/0277-standard-modules-without-use.md)
(option a: in scope unless the program has something else by that name).

## OQ-072 — Display text written on error variants
**Resolved** by [ADR 0278](decisions/0278-messages-on-error-types.md) (option a:
a message after each variant or the struct's name).

## OQ-049 — A package registry and version ranges
Dependencies come from a path or a pinned git commit
([ADR 0231](decisions/0231-path-and-git-dependencies.md)). Options: (a) a
central registry with names and semver ranges (`dep json 1.4`), resolved
into `ail.lock` (recommended eventually: short `dep` lines and discovery for
agents, but it needs hosting, accounts, a naming policy and a solver);
(b) git URLs only, with tags as versions (no service to run; long URLs in
every manifest); (c) a registry that only maps names to git URLs and commits
(no solver; small service). Involves hosting and publishing, so it is the
designer's call (AGENTS.md escalation criterion 4). *Owner: designer.*

## OQ-050 — Unique values in generic code, sets and traits
**Partly resolved:** `move self` receivers by
[ADR 0317](decisions/0317-move-self-receivers.md); lists and maps of unique
values by [ADR 0318](decisions/0318-unique-values-in-lists-and-maps.md) and
[ADR 0319](decisions/0319-copying-methods-on-unique-collections.md).
Still open: unique values as type arguments of user generics, `ref`
contents, map keys and set elements; `move self` in traits; `List[Task[T]]`
(a `TaskGroup` covers it today); and moving out of a field
(destructuring, [ADR 0190](decisions/0190-moves-and-use-after-move.md)).
Options for generics: (a) a `Copy`-like bound every generic parameter has
unless it opts out (`[T unique]`), which also opens `move self` in traits;
(b) keep the restriction. Recommendation: (a), designed with the standard
library's resource types (files, sockets). *Owner: AI under ADR 0076, with
the standard-library resource types.*

## OQ-090 — May accelerator drivers compile AIL-generated code?
ADR 0010 forbids external backends. GPUs and NPUs can't be reached without
the OS driver compiling what AIL hands it: SPIR-V (Vulkan), MSL (Metal), a
Core ML model, an ONNX graph. Options: (a) AIL generates the SPIR-V, MSL or
graph with its own code generator, and the driver's compiler counts as part
of the platform, like CPU microcode (recommended: it is the only way to
reach these devices at all, and nothing of LLVM's ships with AIL);
(b) CPU only, no GPU or NPU ever; (c) allow vendor libraries (cuDNN, MPS) as
opaque kernels. See [automatic-parallelism.md](design/automatic-parallelism.md)
section 6 and [ADR 0247](decisions/0247-std-tensor-and-accel-capability.md).
The CPU slice exists (`std.tensor`, ADRs 0328–0329); [ADR 0330](decisions/0330-tensor-accelerator-backends-plug-in.md)
lists what a device backend needs decided here. Still missing from the CPU
slice: `conv2d`, activations, `f32`, indexing, `/` and unary `-`, broadcasting
beyond a scalar, and fusing chains of elementwise calls (axis reductions,
`transpose` and `+ - *` landed in ADRs 0344–0345).
*Owner: designer (it reads ADR 0010, a designer decision).*

## OQ-091 — Field attributes for derive generators
A `ToJson` derive (ADR 0227) wants per-field options (`@json("user_id")`,
skip a field). Today an unknown attribute is an error (`test.attr.1`), and
`FieldInfo` (ADR 0229) has no attributes. Options: (a) a trait declares the
attribute names its generator reads (`@derive(derive_json, attrs: [json])`),
and `FieldInfo` gains `attrs`; unknown names stay errors (recommended: still
one place that says what an attribute means, and typos are caught);
(b) any `@name(...)` on a field is allowed and passed through (shortest, but
a typo is silently ignored); (c) no field options; a type that needs them
writes the `impl` by hand. *Owner: AI under ADR 0076.* **Update
2026-10-20:** the `ToJson`/`FromJson` derives (ADR 0308) use field names as
member names, which `examples/` needs; still open until a program needs a
rename, with (a) recommended.

## OQ-092 — Type information in constants and compile-time assertions
`TypeInfo` reaches only derive generators (`refl.info.2`). A compile-time
check such as "every variant of `Cmd` has a help line" needs it in a
constant. Options: (a) a prelude `type_info[T]()` callable only in
compile-time code, plus `assert` in constants as a compile error
(recommended when a use appears: small, and still no run-time reflection);
(b) only through derives (status quo); (c) Zig-style `@typeInfo` anywhere at
compile time (rejected in ADR 0225 and ADR 0229). *Owner: AI under ADR 0076.*
**Update 2026-10-20:** generators now receive `TypeInfo` (ADR 0306); no
program needs it in a constant yet, so this stays open with (a) recommended.

## OQ-093 — Generating items other than an impl body
Generated code is limited to the methods of the deriving `impl` and
associated functions of its type (`ct.gen.1`, ADR 0307). Bindings generated from a schema, or a type per table, would
need generators that add types or functions to a module. Options: (a) defer
until a real use appears (recommended: C headers already have ADR 0103, and
every current need is an impl); (b) a module-level `@generate(f)` whose items
appear in `ail api --expanded` like derives. *Owner: AI under ADR 0076.*

## OQ-094 — Downcasting `any Trait`
`refl.run.1` gives no way to recover the concrete type of an `any Trait`
value. Options: (a) none (recommended for now: an enum is the checked way to
hold one of several known types, and `match` is exhaustive); (b) a checked
`x as T?` returning `none` on a mismatch (needs a type tag per value, which
`any Trait` already has as its method table); (c) a trait method the author
writes. *Owner: designer (it changes what `any Trait` promises).*

## OQ-095 — Compile-time evaluation limits
`ct.eval.3` sets 100,000,000 steps and 1 GiB per constant or derive.
Options: (a) keep these and let a package manifest raise the step limit
(recommended: a constant that needs more is rare, and the limit is
deterministic); (b) a per-program total budget instead of per item (bounds
the whole build, but one large constant starves the rest); (c) no memory
limit (simpler, but a runaway allocation takes the machine down). *Owner: AI
under ADR 0076, measured once the evaluator counts steps.*

## OQ-096 — Showing built-in derives in `ail api --expanded`
**Resolved** by [ADR 0309](decisions/0309-api-expanded-output.md): option (b),
`"derived_by": "built-in"`; template text would show code that doesn't run.
`trait.derive.1`'s `Eq`, `Ord` and `ToStr` derives are specified as if a
prelude generator wrote them, but the bootstrap implements them natively.
Options: (a) `--expanded` prints their equivalent AIL text from a fixed
template (recommended: one rule for every derive, and an agent sees what
`cmp` compares); (b) list them by name only, like automatic implementations
(cheapest); (c) write the prelude generators in AIL and drop the native code
(slower in the interpreter). *Owner: AI under ADR 0076.*

## OQ-110 — `Set.union` and friends
Round 8 (lane 83, [program-measurements-r8.md](design/program-measurements-r8.md)):
`snapdiff` costs 14.4 tokens (4.6%) more than its Python twin, and 9.6 of those
are `{n for keys in [old.keys(), new.keys()] for n in keys}`, where Python
writes `old.keys() | new.keys()`. `Set` has no `union`, `intersect` or
`difference`. Options: (a) methods `a.union(b)` (also on `keys()`), measured
−9.6 tokens on `snapdiff` (recommended); (b) an `|`
operator on sets (similar saving, a new operator); (c) leave it. *Owner: AI
under ADR 0076.*

## OQ-111 — Appending to a map entry that may be missing
`m[k] += 1` starts from zero (ADR 0251), but a list-valued entry has no
equivalent, so the shortest form is `m[k] = (m.get(k) else []) + [x]`, which
also needs `+` on lists (it does not exist), and Python has
`defaultdict(list)` or `setdefault`. Not measured; `scan` avoids the construct
with a second pass over the list. Options: (a) `m.push(k, x)`,
appending to the list at `k` and starting an empty list when absent
(recommended: one call, the type follows from the map); (b) `+` on lists;
(c) leave it. The panic that exposed this is fixed; the diagnostic for the
write above still says "a list index must be `int`" when `m` was an untyped
`{}`, which a better message would improve. *Owner: AI under ADR 0076.*

## OQ-112 — Optional keyword arguments for `proc.run`
`proc.run("git", ["hash-object", "--stdin"], text)` is fine, but a time limit
without input is `proc.run(cmd, args, none, 200)` (`runner`, `none` costs 1
token and a surprise). Python's `timeout=0.2` names itself. Estimated
saving: 1–2 tokens per call, under 0.5% of the corpus. Options: (a) accept
named arguments for the optional parameters of std functions
(`proc.run(cmd, args, timeout_ms: 200)`; recommended if other std functions
grow optionals); (b) leave it. *Owner: AI under ADR 0076.*
## OQ-097 — Weaker atomic orderings, cancel trees, and send under cancellation
Lane 72 (ADRs 0320–0322) left three follow-ups. (1) `conc.atomic.2` requires
acquire, release, acquire-release and relaxed orderings; the bootstrap has
only SeqCst. Options: (a) a trailing named argument, `a.load(order: Relaxed)`
(recommended: rare, so the token cost lands only where it's used); (b)
separate methods (`load_relaxed`); (c) wait for the native backend. (2)
Cancelling a task doesn't reach tasks it already started (only tasks it
starts later). Options: (a) cancel the whole tree, as structured concurrency
suggests (recommended once tasks record their parent); (b) keep it per
task. (3) A cancelled task blocked in `send` on a full channel still
waits. Options: (a) `send` returns `! Cancelled` (breaks every existing
call's type); (b) keep waiting, so a cancelled producer is released by its
consumer (current); (c) drop the value and return. *Owner: AI under ADR 0076.*

## OQ-113 — A matrix product operator
`a.matmul(b)` is the only spelling of the matrix product (ADR 0344). Options:
(a) keep the method (recommended: `matmul` is rare in agent code, so the
token gain is under the 2% tie, and the operator set is fixed by ADR 0077);
(b) `a @ b` as in Python, which needs a new operator against `trait.op.2`,
and `@` already starts attributes (`@inline`), so a line that starts with an
operand-less `@` reads two ways; (c) a named operator such as `a matmul b`,
which no other part of AIL has. *Owner: designer (amends ADR 0077).*
