# Compiler representations: AST, HIR, MIR, machine IR

> **Non-normative design note,** 2026-10-20 (lane 37 of the
> [AI-first plan](ai-first-plan.md)). Topic 19 (AST/IR architecture). The
> decided parts are ADRs [0222](../decisions/0222-four-compiler-representations.md),
> [0223](../decisions/0223-hir-is-the-versioned-api-shape.md) and
> [0224](../decisions/0224-incremental-cache-keys.md). Builds on ADRs 0015
> (whole-program incremental), 0016 (observable optimised code), 0018
> (optimising backend), 0022–0023 (staged self-hosting), 0024 (semantic API
> from Stage 2), 0032 (interpreter as oracle) and 0080 (generic compilation).

## 1. Why four tiers

Each consumer needs a different view of a program, and serving them all from
one structure either leaks backend detail into the API or starves the
optimiser of explicit facts.

| Consumer | Needs | Tier |
|---|---|---|
| `ail fmt`, the parser's diagnostics | the exact tokens, comments and layout | **AST** (with the token stream) |
| `ail api`, `ail check`, `ail effects`, editors, AI agents, the interpreter | names resolved to stable ids, every expression typed, source-level constructs kept, generated code included | **HIR** |
| optimiser, allocation ladder (ADR 0012), parallelisation (lane 39), performance report | control flow, values in SSA form, every allocation, copy, trap check and dispatch explicit | **MIR** |
| instruction selection, register allocation, debug info | ISA-specific instructions with recovery records (ADR 0016) | **machine IR** |

An AI reads at the HIR level: it is the program the author wrote, with every
question about names and types already answered. Nothing below it is needed
to write correct code; MIR facts surface through the API as *reports*
(allocations, copies, decisions), never as a second program to read.

```text
source ──lex/parse──▶ AST ──resolve/type/expand──▶ HIR ──lower──▶ MIR ──optimise──▶ MIR ──select──▶ machine IR ──▶ object
  ▲                    │                            │  ▲                                                     │
  └── ail fmt ─────────┘        ail api / check ◀───┘  └── compile-time evaluation (interpreter, ADR 0225)   └── debug info
```

## 2. AST

What the bootstrap has today (`77592931:bootstrap/src/ast.rs`): per-file declarations
and expressions with positions, produced by `parser.rs` from the lexer's
tokens. `ail fmt` works from the token stream and its comment side table,
not from the AST, so the AST needs no trivia.

- **Shape:** declarations (`FnDecl`, `StructDecl`, `EnumDecl`, `TraitDecl`,
  `ImplDecl`, `ConstDecl`, `Use`) and expression and statement trees, each
  node with its position.
- **Stability:** internal. Not exposed; tools read source text or HIR.
- **Lifetime:** one per file per edit; the parse query (§5) is keyed by the
  file's text hash.

The bootstrap's checker currently attaches its results to the AST through
side tables keyed by position (`Resolved` in `check.rs`). That *is* a HIR in
all but name; §6 says how the bootstrap converges on the shape below without
a rewrite.

## 3. HIR: the program as the API serves it

The HIR is the resolved, typed, expanded program. It is the source of truth
for `ail api`, `ail check`, `ail effects`, rename, the interpreter, and
lowering to MIR.

**Contents.** Items (functions, methods, types, fields, variants, traits,
impls, constants), each named by its **stable id** (`api.id.*`, ADR 0151:
`fn:add`, `method:Point.shift`, `impl:Version.Ord`). Bodies are expression
trees in which:

- every name and call refers to a symbol id or a local, never to text;
- every expression has a type; generic expressions keep type parameters
  (HIR is not monomorphised: instantiation is a MIR query, ADR 0080);
- source-level constructs stay as written: `?`, `else`, `if :=`, `match`,
  `for`, interpolation, implicit `self` (ADR 0141), so any HIR node maps back
  to one source span and a diagnostic can be stated in source terms;
- desugarings that the author can't observe are recorded, not applied:
  positional constructor arguments carry the field they fill, empty literals
  carry their inferred type, operators carry the trait method they call.

**Provenance.** Every item and node has an *origin*: either a **source
span** or a **generated origin** that names what produced it (a derive, for
now: ADR 0228). Generated items are ordinary HIR, checked by the same rules,
with ids like any other (`method:Version.ToJson.to_json`), so references,
effects and diagnostics work on them unchanged. `ail api --expanded` prints
them as AIL text with their origin.

**Stability.** The HIR's *external* shape is what `ail api` prints, and it
is versioned by the `api` key (ADR 0223). Adding an optional key keeps the
version; anything else bumps it. The in-memory HIR may change freely as long
as the printed shape doesn't.

**Facts attached to HIR** (served by the API, computed by queries): effect
sets (ADR 0163), constant values (ADR 0225), automatic and derived trait
implementations, references, holes and their expected types.

## 4. MIR: the unit of the query compiler

MIR is per **body instance**: one function, method or constant initialiser
at one set of type arguments (a generic body may also be lowered once,
shared, per ADR 0080; the instance key then says so).

- **Form:** a control-flow graph of basic blocks; values in SSA form with
  block parameters (no phi nodes); places (locals, fields, list elements)
  explicit, so exclusivity (`own.excl.*`) and copy reporting (ADR 0065) are
  checks over MIR.
- **Explicit operations:** allocation sites (each with a slot for the
  allocation-ladder decision, ADR 0012); copies; overflow and bounds checks
  as trap edges (ADR 0035, `err.trap.*`); calls tagged static, dynamic
  (`any Trait`) or standard-module; effect operations (the sources of
  `eff.infer.4`) as distinct instructions, so the effect set of a body is
  recomputable from its MIR; task spawn and join (ADR 0095–0096).
- **Decision records:** each optimisation records what it decided and the
  inputs it depended on (ADR 0015): "inlined `fn:g` into `fn:f` because …",
  "specialised `fn:largest` for `int`", "loop at L parallelised" (lane 39).
  Decisions are data, so the performance report and the API serve them and
  the cache invalidates them precisely.
- **Recoverability:** every MIR value that corresponds to a source variable
  carries a recovery record (where it lives or how to recompute it,
  ADR 0016), kept up to date by every pass. A pass that can't keep it records
  why.
- **Verifier:** a MIR verifier runs after every pass (ADR 0018,
  miscompilation defence), checking SSA dominance, types, and recovery
  records.
- **Stability:** internal and unversioned. `ail mir FILE [--fn ID]` (future)
  dumps it for debugging, with a header saying it is unstable. Agents should
  not build on it; they use the API's reports instead.

MIR is written in AIL by the backend (ADR 0022). The bootstrap front end
hands HIR to the backend through a serialised form; the backend lowers it.
The interpreter (the oracle, ADR 0032) runs HIR directly, so a lowering bug
can't hide in both implementations.

## 5. Queries and the incremental cache

Every step is a **query**: a pure function from keys to a result, memoised
with the keys and the fingerprints of everything it read (ADR 0015, ADR 0224).

| Query | Key | Reads |
|---|---|---|
| `parse(file)` | file path | file text hash |
| `item_hir(id)` | stable id | the item's AST, the signatures it names |
| `signature(id)` | stable id | the declaration line, the types it names |
| `effects(id)` | stable id | MIR or HIR of the body, callees' `effects` |
| `const_value(id)` | constant id | `item_hir`, the HIR of every function evaluated (ADR 0226) |
| `derive(impl id)` | impl id | the type's `type_info`, the generator's HIR (ADR 0227) |
| `mir(id, type args)` | body instance | `item_hir`, callee signatures |
| `optimised(id, type args)` | body instance | `mir`, recorded decision inputs |
| `machine(id, type args, isa)` | body instance, ISA | `optimised` |

**Keys are stable ids, never positions.** Results are fingerprinted by a
**span-free hash**: the HIR's content with spans replaced by offsets relative
to the item's start. Moving a function down the file, or editing a comment
above it, changes no fingerprint except the span table, which only debug-info
and diagnostic queries read. This is the "red-green" scheme: when an input
changes but the recomputed result hashes the same, dependants are not
recomputed.

**Determinism.** Every query is deterministic (ADR 0015, wishlist 6.3): no
hash-order iteration, no clock, no environment. That makes the cache safe to
share between machines and lets the build print *why* it rebuilt ("`fn:g`
changed; it was inlined into 14 functions").

## 6. How the bootstrap converges

The Rust bootstrap won't be restructured tonight, and doesn't have to be:

1. `Resolved` already holds the HIR's facts (targets, symbol ids, literal
   types). New facts go there or into new side tables, keyed by position
   *within the bootstrap* only; `ail api` prints ids.
2. Generated code (derives) is parsed into ordinary AST items with a
   generated origin and passed through the same checker (ADR 0228), so the
   bootstrap's HIR-equivalent includes it.
3. When the query compiler starts (Stage 3), the side tables become HIR
   fields and position keys become `(id, relative span)`.

## 7. Rejected alternatives

- **One IR for everything (a "rich AST" all the way to codegen):** the API
  would expose backend churn, and optimisations would work on a tree that
  hides control flow. Rejected for AI understanding (a versioned shape must
  not move with the optimiser) and speed.
- **Exposing MIR through the API as a stable format:** freezes the
  optimiser's internals. Facts from MIR are served as reports instead.
- **A lossless concrete syntax tree as the API shape (as Roslyn does):**
  more than agents need, and much larger answers (tokens, criterion 1). The
  formatter already owns layout; text is the lossless form.
- **Monomorphised HIR:** the API would show many copies of one generic
  function; instantiation belongs to MIR (ADR 0080).
- **Position-keyed cache:** every edit above a function would invalidate it.
- **Sea-of-nodes instead of CFG + SSA:** harder to keep recovery records
  (ADR 0016) and to explain decisions per statement; can be revisited inside
  the backend without changing the tiers.
