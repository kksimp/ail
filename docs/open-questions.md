# Open questions

Unresolved design questions. Each question lists its options with pros and
cons, plus a recommendation. Since [ADR 0076](decisions/0076-delegated-decisions.md),
routine questions are decided by the AI using the priority order (tokens, AI
understanding, readability, speed). Serious ones go to the designer. When a question is resolved it moves into an ADR
and its entry here is replaced by a link.

IDs (`OQ-NNN`) are never reused.

State on 2026-11-07: 81 entries, 25 open and 56 resolved. Resolved entries stay
here for history; each links the ADR that settled it. Open ones are live design
work, and new ones are welcome as issues (the proposal template has the same shape).

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
**Measured 2026-10-24** (the corpus, `examples/`, `tools/` and `compiler/`, about
148,000 lines): no `match` on a specific error variant follows an `else err`
(errors are tested by field, e.g. `err.kind == "timed_out"`), and about eight
places reach a field through an optional that `a?.b` would shorten. Both stay
open until a program needs them; the ADR 0029 measurement is then rerun.

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
**Resolved** by [ADR 0348](decisions/0348-supertraits.md) (`trait Ord Eq`) and [ADR 0349](decisions/0349-associated-functions-in-traits.md) (`fn zero() Self`, called as `T.zero()`).

## OQ-040 — `else` as an expression
**Resolved** by [ADR 0144](decisions/0144-else-as-an-expression.md) (`e else d` is the loosest operator).

## OQ-041 — Format specifications in interpolation
**Resolved** by [ADR 0173](decisions/0173-format-specifications-in-interpolation.md) (`{x:>8}`, `{r:.2}`, `{n:0>4x}`).

## OQ-042 — Custom hashing
**Resolved** by [ADR 0955](decisions/0955-custom-hash-names-fields.md) (`impl Hash` names the fields that take part).

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

## OQ-049 — A package registry and version ranges
**Resolved** by [ADR 0383](decisions/0383-no-package-registry.md) (no registry; git URLs with tags as versions).

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

## OQ-060 — Loop invariants and static proof for contracts
Items (1) `old(x)`, (2) inherited trait contracts and (3) type invariants are
**resolved** by [ADR 0303](decisions/0303-contract-old.md),
[ADR 0304](decisions/0304-trait-contract-inheritance.md) and
[ADR 0305](decisions/0305-struct-invariants.md); checks stay run-time
([ADRs 0210–0213](decisions/0210-contract-syntax.md)). Still open: (a) loop
invariants (recommended: wait; `assert` inside the loop covers the run-time
case and a prover is the only reason to name one); (b) static proof, and
whether an optimising build keeps the checks it can't prove: **resolved** by
[ADR 1055](decisions/1055-contracts-proved-by-interval-facts.md) (interval facts
prove simple contracts and drop their checks; unproved checks stay); (c) invariants on
enums, and checks for writes through `mut` parameters outside the type's
methods (recommended: revisit with (b), since both need a prover to be cheap).
*Owner: AI under ADR 0076.*

## OQ-071 — Standard modules without `use` lines
**Resolved** by [ADR 0277](decisions/0277-standard-modules-without-use.md)
(option a: in scope unless the program has something else by that name).

## OQ-072 — Display text written on error variants
**Resolved** by [ADR 0278](decisions/0278-messages-on-error-types.md) (option a:
a message after each variant or the struct's name).

## OQ-090 — May accelerator drivers compile AIL-generated code?
**Resolved** by [ADR 0381](decisions/0381-driver-compilers-are-platform.md) (yes: the driver's compiler is platform; AIL generates the device program itself).

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
**Resolved** by [ADR 0877](decisions/0877-no-downcast-of-any-trait.md) (option a: no downcast; an enum is the checked way).

## OQ-095 — Compile-time evaluation limits
`ct.eval.3` sets 100,000,000 steps and 1 GiB per constant or derive.
Options: (a) keep these and let a package manifest raise the step limit
(recommended: a constant that needs more is rare, and the limit is
deterministic); (b) a per-program total budget instead of per item (bounds
the whole build, but one large constant starves the rest); (c) no memory
limit (simpler, but a runaway allocation takes the machine down). *Owner: AI
under ADR 0076, measured once the evaluator counts steps.*

## OQ-096 — Showing built-in derives in `ail api --expanded`

**Resolved** by [ADR 0309](decisions/0309-api-expanded-output.md): option (b), `"derived_by": "built-in"`.

## OQ-097 — Weaker atomic orderings, cancel trees, and send under cancellation

*Resolved:* orderings by [ADR 0352](decisions/0352-weaker-atomic-orderings.md); cancel trees and a cancelled `send` by [ADR 0351](decisions/0351-cancel-trees-and-cancelled-send.md).

## OQ-113 — A matrix product operator
**Resolved** by [ADR 0382](decisions/0382-no-matrix-product-operator.md) (keep `a.matmul(b)`).

## OQ-114 — A mutable local that starts as `none`

**Resolved** by [ADR 0468](decisions/0468-optional-none-locals-and-if-branches.md) (option a).

## OQ-115 — `if c then x else none` where `x` is a plain `T`

**Resolved** by [ADR 0468](decisions/0468-optional-none-locals-and-if-branches.md) (option a).

## OQ-116 — Locals named like an item of their module

**Resolved** by [ADR 0469](decisions/0469-locals-may-shadow-functions.md) (option a).

## OQ-117 — `if` as an operand

**Resolved** by [ADR 0470](decisions/0470-parenthesised-if-operand.md) (option a).

## OQ-118 — Which transcendental functions the language and the native backend need

**Resolved** by [ADR 0592](decisions/0592-std-math-float-methods.md): the full `std.math` set, within one ulp of the platform's libm (option c, the designer's choice on 2026-10-21).

## OQ-125 — Unicode tables for the native `char` predicates and case mapping

**Resolved** by [ADR 0567](decisions/0567-native-text-of-numbers-and-unicode-tables.md)
(option a: range tables generated from the interpreter by
`tools/native/gen_unicode.ail`).

## OQ-119 — A Python-syntax front end ("pail") after the core language (post-1.0)
The designer asked (2026-10-20) whether a `pail` module could let developers
write Python in the AIL editor while the code is really AIL, to help adoption.
Not before the core language and the native toolchain are complete.
- **a.** A Python-syntax front end that compiles directly: Python source in,
  AIL semantics out. Every tool (`ail check`, `ail api`, diagnostics, the
  card) would need a second syntax, and the parts of Python the semantics
  can't express (untyped code, `eval`, monkey patching, shared-reference
  aliasing where `b = a; b.append(1)` changes `a`, which AIL's mutable value
  semantics forbids, ADRs 0060–0063) would have to be rejected with
  Python-shaped diagnostics.
- **b.** A one-way translator, `ail port app.py`, from typed Python to
  readable AIL. Developers keep one language for the tooling and get an
  on-ramp; the translator rejects the dynamic parts and the aliasing cases
  with a message that names the AIL form. Precedents: Codon, Mojo and Nim
  (each kept one language and offered a bridge).
- **c.** Neither: AI agents already write AIL directly, and the editor's
  cards and diagnostics are the on-ramp.
Recommendation: **b.** One language in the tooling (the AI-first goal), a
real on-ramp for Python developers, and no second syntax to keep in step
with every rule. Decide after milestone 8; see the roadmap.
## OQ-120 — Must a unique value be consumed, or does scope-end cleanup suffice?
Lane 125 (streaming processes) kills a child process left running when the
program ends (`std.proc.7`). A general rule that a unique value MUST be
consumed (waited, closed, or moved) before its scope ends would replace that
exit-time kill, but it changes `own.uniq.3` (ADR 0066, designer's decision):
today a unique value's cleanup runs at scope end by any path, and nothing
needs to be consumed. Task handles already must be waited (ADR 0096).
- **a.** Keep scope-end cleanup (ADR 0066). A stray child is killed by its
  `Drop`, as `std.proc.7` says. Nothing changes.
- **b.** Require consumption for every unique value: a unique local that is
  neither moved nor consumed is a compile-time error. Precise, but
  every unique value then needs a consuming call, and `defer`-style cleanup
  stops being automatic.
- **c.** Require it only for handles declared with a marker (for example
  `unique(consume)`), keeping ADR 0066 for the rest.
Recommendation: **a.** The model stays affine, not linear, which is simpler
for agents; the exit-time kill is the `Drop` of the child and needs no new
rule. Reopen only if stray handles cause real bugs.
## OQ-121 — Closures as `std.net.http` handlers
**Resolved** by [ADR 1067](decisions/1067-closures.md): function literals copy
the locals they name (`type.fn.4`-`5`), so a handler is `|req| reply(req, store)`
with shared state in a captured `ref`.

## OQ-122 — What `t[i][j]` on a one-dimensional tensor means

`std.tensor.15` gives `t[i]` the tensor of the rows after the first, so a
one-dimensional `t[i]` is a tensor with no axes (ADR 0328). The checker
accepts `t[0][1]` and the program prints a 0-dimensional tensor, with no
error. Round two's mistakes corpus (ADR 0646) caught it. Options: (a) keep
the spec and make a second index on a 0-dimensional tensor a compile-time
error (`expr.index.*`, a new rule); (b) make `t[i]` on a one-dimensional
tensor an element, which changes std.tensor.15. Recommendation: (a), since
the spec's shape rule stays true and an agent gets a diagnostic at the second
index. The decision belongs to the designer, because it changes a spec rule
(std.tensor.15). The corpus omits the program until then (ADR 0647).
## OQ-123 — Compare `bytes` with `str` directly?
Round 4 ([measurements](design/program-measurements-agent-r4.md)) found
`gz_check` and `frames` comparing a file's bytes with its text through
`to_bytes()`, `text()` and `len()`. Options: (a) keep `==` between `bytes`
and `str` a type error, and write `b.text() == s` (status quo, an extra call
per comparison); (b) allow `bytes == str` as the UTF-8 comparison, with
`bytes == bytes` unchanged. Pros of (b): about 3 tokens per comparison on the
round-4 programs. Cons: two kinds of equality, and a mixed `==` that a reader
could take for a conversion. Recommendation: (a) until a larger corpus shows
the cost; the saving is small and the rule is simpler. Decided by the
integrator with a measurement, not by this lane.
## OQ-124 — Pairs: tuples, `zip`, `enumerate` and map `items`
Resolved by [ADR 0791](decisions/0791-tuples.md): full tuples.

## OQ-135 — Spec rules that need a tool the bootstrap doesn't have (lane 238)

Lane 238 implemented the rules of the lane 223 and lane 225 lists that a checker or interpreter change could carry (own.ref.2, trait.op.2, ct.const.4, err.type.3, ct.eval.3's step limit, ADRs 0741–0743). Six remain, each because the rule names something that doesn't exist yet. The spec is unchanged.

| Rule | What's missing | Options | Recommendation |
|---|---|---|---|
| `gen.anon.2`, `type.enum.4` | They define a *canonical form* that rewrites tokens (`fn f[T Shape](s T)` to `fn f(s Shape)`, `Shape.Circle(r)` to `Circle(r)`). `ail fmt` only changes whitespace and refuses output whose tokens differ (fmt.safe, ADR 0169); `type.enum.4` also needs expected types. | (a) an `ail fix` rewrite with a lint-style diagnostic (new rule IDs); (b) let `ail fmt` rewrite tokens; (c) reword the rules as advice | (a): `ail fix` already rewrites tokens from checker information, and fmt.safe stays true. |
| `err.type.4` | Matching on an inferred error set (OQ-029, "not designed yet"). | Decide OQ-029 first. | Wait for OQ-029. |
| `ct.eval.4` | A MAY rule (reuse of compile-time results); nothing observable to test. | (a) implement with the build cache (lane 200); (b) keep it untested | (a) once `ail build` has its cache; no work before. |
| `own.copy.1` | The "performance report" (ADR 0026) doesn't exist in any tool, so there is nothing to list copy sites in. | (a) a new `ail effects --copies FILE` listing each site where a list, map, set or struct is copied by assignment, argument or return; (b) wait for the report | (a), as the first line of the report: it follows the existing `--parallel` listing. |
| `plat.isa.3` | A native start-up check of the CPU level needs the code generator's entry sequence (`compiler/mir/lower.ail`, the merge queue's critical path) or a start-up stub in the object writers. | (a) a stub in `compiler/obj/` that runs `cpuid` before `main`, x86-64 only (ARM64's baseline is checked by the OS); (b) wait for the queue to empty | (a) after the queue empties, so it lands without conflicts. |

## OQ-136 — A typed buffer for `mem.alloc` and C strings (day-3 brief)

Raw FFI code is the most expensive shape in the corpus. In `ffi_buffers`, each
buffer is an `extern` declaration, a `mem.alloc`/`free` pair, an `unsafe`
block and index arithmetic on raw pointers. Measured on 079c6d8 (five-tokenizer
mean): `ffi_buffers` 366.2 tokens against 254.2 for the Python ctypes twin
(+112.0); `ffi_libc` 208.8 against 189.4 (+19.4). Both are rare constructs:
they sum to 131.4 tokens on a 23,004-token corpus (0.6%), so under
[ADR 0076](decisions/0076-delegated-decisions.md) the token criterion is close
to a tie. The Python side is itself library ceremony (ctypes), so the gap is a
ceiling, not a target. Not yet measured: a prototype of any option below.

- **a.** Keep `mem.alloc`/`free` and `mem.cstr` as they are. Nothing changes;
  FFI programs stay about 45% longer than the twin.
- **b.** A typed buffer `mem.buffer[int](n)` with bounds-checked indexing, `len()`,
  and release at scope end (no `free` pair), plus the C-string helpers in c.
  Largest saving on `ffi_buffers`; largest surface: a new `std.mem` type,
  spec rules for ownership and bounds, and checker and native parity for each.
- **c.** Only C-string helpers: `mem.cstr(s)` and `cstr.text()` (no new buffer
  type). Covers the `ffi_libc` gap and the `from_cstr` lines; `ffi_buffers`
  keeps its raw pointer code.

Recommendation: **a** now, because the gap sits in rare constructs and the
criteria tie. Revisit with **c** when a third program needs C strings, and
measure **b** with a prototype of `ffi_buffers` before adding a new type to the
card. Both options change the card, so the designer should decide the
ownership rule for **b** (release at scope end versus an explicit `free`).

## OQ-137 — Free-form inline assembly in `driver` packages

[ADR 0820](decisions/0820-driver-level-unsafe.md) gives `unsafe driver`
packages CPU intrinsics (`hw.x64.*`, `hw.a64.*`) but no free-form assembly.

- **a.** Intrinsics only. Each one is typed, checked and explained by
  `ail explain`, and emitted by the in-house assemblers. Con: an instruction
  without an intrinsic needs a compiler change.
- **b.** An `asm` block with AIL's own assembler syntax and a register
  constraint list. It covers everything. Con: a constraint language to design,
  per-ISA text that agents can't check, and the optimiser must treat the block
  as opaque.
- **c.** Whole functions in assembly (`asm fn`), with the calling convention
  fixed by the native ABI (`native-abi.md`). No constraint language. Con: a
  call per use.

Recommendation: **a** now. Revisit **c** when a driver needs an instruction
sequence that intrinsics can't express. Measure tokens on a real driver before
adding anything to the card.

## OQ-138 — An LLM runner written in AIL (Kaleb's idea, 2026-10-22)

Run a language model's inference in AIL itself, since native AIL may reach C
speed. Not for now: it follows two milestones, `matmul` at C speed (it is 2.0x
C in instructions today, from layout and register spills) and a GPU path
([ADR 0381](decisions/0381-driver-compilers-are-platform.md), AIL-generated SPIR-V
and MSL).

- **a.** A CPU-only runner first: load quantised weights (`bytes` and sized
  integers, [ADR 0834](decisions/0834-native-sized-numbers.md)), tensor kernels
  on `Tensor[f32]`, tokeniser and sampler in AIL. Pro: a real benchmark of the
  claim "faster than C" on a workload people care about, and a stress test of
  tensors, sized numbers and mmap. Con: CPU inference is memory-bound, so the
  win over C is small until kernels are vectorised.
- **b.** Wait for the GPU path, then write the runner against it. Pro: where
  the speed is. Con: no early feedback on missing tensor or I/O features.
- **c.** Do not do it; leave inference to existing runners and call them
  through FFI. Con: gives up the showcase.

Recommendation: **a**, as a small tool in `tools/` once `matmul` is within
1.2x of C and sized numbers and `mem` are native, then **b** when the GPU path
exists. Measure with `tools/perfbench` (tokens per second against a C twin).

## OQ-139 — Copy-on-write or last-use moves for whole-value copies (day-4 profile)

Value semantics make the native compiler copy a `Func`, `Block` or `Module`
whenever a pass binds one to a new name or takes it as a value argument.
Stage 1 compiling `tools/garlic/garlic.ail` (callgrind, 2026-10-23, after
making the MIR passes edit in place): about 12% of instructions are
`copy.mir.*` and `rt.list_copy` (still `bce`, `fill_loops`, `reserve_lists`,
`duplicate_returns`, and `Module(funcs: l.funcs)`), and the collector is about
30% (`gc_find`, `gc_sweep`, `gc_mark`). Raising the collection threshold from
3x to 8x live cut user time 7.1 to 5.7 s but added 2.3 s of system time and
doubled memory, so it was reverted.

- **a.** Keep rewriting passes to take `mut` and edit in place. Pro: no
  compiler change, byte-identical output, about -13% so far. Con: each pass is
  hand work, and a pass that reads the old value while editing needs care.
- **b.** Last-use moves for fields and call arguments (`Module(funcs: l.funcs)`
  where `l` is not used again; `f = pass(f)`): extend `elide_moves` beyond
  owned locals. Pro: removes copies everywhere, including user code. Con: the
  analysis must prove no alias or later read; a miss is a wrong result.
- **c.** Copy-on-write lists (a reference count in the header). Pro: copies
  become free until a write. Con: a count update on every share and write, and
  every list store checks it; slower for the common unshared case.

Recommendation: **a** for the compiler now, then **b**, which helps user
programs too. Do **c** only if measurement shows shared-then-read lists
dominating. Measure with callgrind on stage 1 and `tools/perfbench`.

## OQ-140 — Computed values for `@observe` (ADR 0884)

`@observe` marks a struct field (ADR 0879, ADR 0884), so an agent can read what
the program stores but not what it computes (a cart total from its lines, a
count of selected items). A program that wants such a value today stores it in
a field and keeps it up to date by hand.

- **a.** Leave it: fields only. Pro: smallest form, nothing to check beyond the
  field's type, stale values are the program's own bug. Con: duplicated state
  that can drift.
- **b.** The accessor form: `@observe fn Cart.total(self) int`, a method with
  only `self` and no effects, observed under its name. Pro: derived values,
  read-only by type, no grammar beyond `gram.attr.1-2`. Con: three lines per
  value, and a second way to observe.
- **c.** `@observe` on a field with a computing expression (`@observe total int
  = lines.sum()`). Pro: short. Con: new field grammar, and the expression runs
  on every query.

Recommendation: **b**, added next to the field form when a real program needs a
computed value; **a** until then. Needs the designer, because it extends the
approved syntax (ADR 0879 item 1).

## OQ-141 — Reading an exact byte count from standard input (ADR 0966)

**Resolved** by [ADR 1071](decisions/1071-streaming-file-handles.md): `io.read_exact(n int) bytes?` (std.io.7); `ail lsp` drops its line trick after the next seed.

## OQ-142 — ADR 0076's order after the 2026-11-11 goals (ADR 1065)

Resolved 2026-11-12 by the designer: [ADR 1077](decisions/1077-decision-order-ai-understanding-first.md).

## OQ-143 — Input hashes for the explain golden (tools/golden/golden.py)

Every `explain` row's input hash covers all of `docs/spec/*.md` and `compiler/explain/*`.
So any spec edit makes every row stale. Stale rows are skipped unless `--strict` is set, so
from 2026-11-10 to 2026-11-12 the explain golden checked nothing: the refreeze at batch 4
added 60 rules and changed 439 outputs. Until this is settled, each batch's fix-forward
refreezes it.

- **A. Hash per rule.** A rule's row hashes its own spec paragraph, its example blocks and
  `compiler/explain/*`. The bare list hashes the rule IDs. Pro: rows stay checked through
  unrelated edits. Con: a rule's output can depend on its neighbours (related rules,
  chapter headers), so the hash has to follow what `explain` actually reads.
- **B. Hash per chapter.** Pro: simple, and stale rows stay local to a chapter. Con: an edit
  still turns a whole chapter stale.
- **C. Run `check --strict` in the gate and refreeze in every batch.** Pro: no code change.
  Con: a refreeze blesses whatever the binary prints, so a real regression can slip through.

**Recommendation:** A, starting with B if A's dependencies are unclear. Neither option is
needed before v0.1.0, because the per-batch refreeze plus a `golden.py compare` against
main covers the gap.
**Decision needed from:** AI (delegated, ADR 0076).
