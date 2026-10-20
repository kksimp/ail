# 0384. MIR: flat instructions, block parameters, built on the fly; the native subset

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture; 19 — IR architecture
- **Spec rules:** [native.subset.1, native.build.1–2](../spec/23-native.md)
- **Builds on:** [ADR 0022](0022-staged-self-hosting.md) (backend in AIL),
  [ADR 0222](0222-four-compiler-representations.md) (MIR as the unit),
  [ADR 0032](0032-keep-rust-interpreter-as-oracle.md) (the interpreter is the reference)

## Context

The finish plan's lane 120 starts the native backend (ADR 0018) in AIL. The
first target is the pure subset that the drag race needs: numbers, loops,
calls and lists of numbers. ADR 0222 fixes MIR as a CFG of blocks in SSA
form with block parameters; this ADR fixes how it is represented in AIL,
how it is built from the syntax tree, and what the first subset is.

## Options considered

### Representation

#### Option A — one enum variant per operation (`Add(a int, b int, site int)`)
- **Pros:** each operation's operands are named and typed.
- **Cons:** AIL's positional construction is refused when two fields share
  a type (type.struct.7), so every construction names every field; every
  pass needs a 20-arm `match` to read operands or rename them. Measured on
  the first draft: about 30% more tokens in the lowering and the passes.

#### Option B — one flat `Inst` struct: `op` name, `args`, `imm`, `fimm`, `name`, `site` (chosen)
- **Pros:** operands are one list, so liveness, renaming, the verifier and
  register allocation treat every instruction alike; a new operation is a
  new name, not a new variant threaded through every pass. Terminators are
  likewise one `Term` with parallel `targets` and `args`, so an edge is an
  index and every pass handles jumps and branches the same way.
- **Cons:** the operation's arity is a convention checked by the verifier,
  not the type checker.

### SSA construction

#### Option A — lower to load/store of local slots, then promote ("mem2reg")
- **Cons:** two passes and a dominance-frontier computation before any
  optimisation, and twice the MIR in between.

#### Option B — Braun et al. 2013, on the fly (chosen)
Variables are looked up through predecessors as code is lowered; a block
that can still gain predecessors (a loop header) is unsealed and gets
placeholder parameters, completed when it is sealed. Parameters receiving
one value are removed afterwards.
- **Pros:** one pass from the syntax tree; structured control flow says
  exactly when each block is sealed; no dominance frontiers.
- **Cons:** the clean-up pass is needed to drop trivial parameters.

### Input

#### Option A — read `ail api --ast --program` JSON from the Rust front end
- **Cons:** the native compiler would depend on the Rust front end that
  Stage 3 is replacing.

#### Option B — lex and parse with `compiler/syntax` (chosen)
The backend reads source text with the AIL front end, which is byte-identical
to the Rust one (ADR 0371). The lowering infers the subset's few types
itself (literals, declared parameter and result types, a list typed by its
first `push`). `ail build` runs the Rust checker first, so the lowering
sees only programs that check.

## Decision

- **MIR** (`compiler/mir/mir.ail`): `Func` has blocks; `Block` has
  parameters, flat `Inst`s and one `Term`. Integer `add`, `sub`, `mul`,
  `div`, `rem`, `neg` and `ftoi` carry a trap site (`Site`: rule, kind,
  operator, position, function); list `get` and `set` carry one for
  bounds. A site of -1 means the check is proved unnecessary (a loop's own
  counter, a `for` over a list). The table of operations is in the source.
- **Lowering** (`compiler/mir/lower.ail`): Braun et al.'s construction over
  the AIL syntax tree; variables get a unique key per declaration, so
  sibling blocks reusing a name stay apart; `simplify.ail` removes trivial
  parameters, unreachable blocks and unused pure instructions;
  `verify.ail` checks single definition, dominance of every use, edge
  arities and types, and operand types after every pass (native.build.2).
- **Lists keep value semantics** by copying when a list is bound or assigned
  from a variable or element (`rt.list_copy`), and when a `for` body may
  change the list it iterates. A fresh list is never copied.
- **The subset** (native.subset.1) is what the drag-race programs need.
  Everything else is reported, all at once, with its position.
- **Debugging:** `ail run compiler mir FILE` prints the MIR (unstable).

## What the subset lacks, and the plan

| Missing | Plan |
|---|---|
| strings beyond literals, interpolation outside `print`, `char` | runtime strings (lane 122): a heap record and UTF-8 builders |
| structs, enums, `match`, optionals, errors | value layouts in native-abi.md; `match` lowers to branches |
| methods, generics | monomorphised instances (ADR 0080), keyed `(id, type args)` |
| maps, sets, comprehensions | runtime tables; comprehensions lower to loops |
| closures, tasks, `par for` | `par for` runs sequentially until the runtime has threads |
| constants | evaluated at compile time (ADR 0225) and emitted as data |
| GC | a precise collector with stack maps from MIR's recovery records (ADR 0016) |
| more than 6 int or 8 float parameters | stack arguments |

## Rejected alternatives

- **Sea of nodes:** harder to keep recovery records (ADR 0016) and to print
  per statement; ADR 0222 already chose a CFG.
- **Writing the first backend in Rust and porting it later:** contradicts
  ADR 0022 and would be thrown away.
- **Lowering untyped and typing MIR afterwards:** the subset's types are
  local enough to infer while lowering; a separate inference pass would
  duplicate the checker.
