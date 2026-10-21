# AI-first buildout plan (2026-10-20)

> **Non-normative design note.** The designer asked for three things on
> 2026-10-20: plan the deeper intricacies of the language, find ways to be
> more efficient, and build the bootstrap out more thoroughly, with aspects
> of the language built around **how an AI uses a computer**, so that an AI
> can understand more. This note is the plan. It is carried out by parallel
> implementation lanes (§6), each landing on the branch that holds this note,
> and integrated into one pull request.
>
> The designer delegated tonight's decisions, serious ones included, under
> [ADR 0076](../decisions/0076-delegated-decisions.md); every one is an
> Accepted ADR that the designer can override.

## 1. The idea: the computer as the AI sees it

An AI agent operates a computer through a narrow loop: **read structured
state, decide, act through a tool, read the result.** It pays for every
token it reads and for every wrong guess it has to recover from. AIL was
designed for the AI as *author* (token-efficient syntax, local reasoning,
precise diagnostics: [ai-author-wishlist.md](ai-author-wishlist.md)). This
plan extends that to the AI as *operator*: the language and toolchain should
expose the program, and the program should expose the machine, in the same
structured, queryable, deterministic way that an agent already works in.

Four principles follow. Each one maps the agent's loop onto the language.

| Agent's loop | Language principle | Where it lands |
|---|---|---|
| **Read state** | Everything about a program is a *query* with a stable, versioned, machine-readable answer: symbols, types, effects, diagnostics, tests, generated code. Never a wall of text to parse. | Semantic API (§2) |
| **Decide** | Everything a call *does* to the machine is visible at the call site or in the signature: effects (I/O, failure, mutation), capabilities (which ambient resources it may touch). No hidden control flow or ambient state. | Effects and capabilities (§3) |
| **Act** | The operating system is reached through small, typed, failure-explicit primitives that return structured values: files, environment, arguments, processes, clock, JSON. Not through a string-typed shell. | Standard modules (§4) |
| **Read the result** | Output of tools and programs is structured, deterministic and byte-stable across runs, so a diff shows only real change. Tests and holes report *what is still needed*, typed. | Testing, holes, determinism (§2, §5) |

## 2. Semantic API: the program as a queryable object (ADR 0024)

ADR 0024 decided the semantic API ships with the Stage 2 front end. Today
only `ail check --json` exists. This is the AI's primary feedback loop, so it
comes first.

### 2.1 Diagnostics schema v1

`ail check --json` becomes a **versioned** schema (`{"ail_api": 1, ...}` per
line or a header line). Each diagnostic gains:

- `end` position (line, col) so an edit can target the exact span;
- `fixes`: zero or more machine-applicable edits, each
  `{title, edits: [{line, col, end_line, end_col, text}]}`;
- `notes`: related positions (e.g. "first declared here");
- `symbol`: the stable symbol identity involved, when there is one (4.4 of
  the wishlist: identities, not line numbers).

Documented in `docs/design/semantic-api.md` with the exact JSON. Byte-stable
order (sorted by position, then rule).

### 2.2 `ail explain RULE`

Prints the spec rule's text, its ADR, a minimal example and a typical fix,
from `docs/spec/` (wishlist 6.4). Repairing an error then needs no spec
search. With `--json`, the same as structured data.

### 2.3 `ail api FILE`

An outline of a program *without bodies* (wishlist 4.1): every type, field,
function, method and constant with its signature, doc comment, position,
stable id, and (once §3 lands) its effects. A dependency then costs a few
hundred tokens to understand instead of thousands. JSON by default, with
`--text` for a human.

### 2.4 Symbols and references

`ail api FILE --symbols` lists every symbol with its definition and all
references, keyed by stable id. This is the basis of rename, "where is this
used", and the later query compiler (ADR 0015).

### 2.5 Typed holes

`todo` (reserved under ADR 0114? no: it needs a new keyword, decided in a
delegated ADR with a token measurement of `todo` vs `???`) is an expression of
any type. The checker accepts the program, and `ail check` reports a
**warning** per hole stating the expected type and the names in scope with
their types (wishlist 2.2). `ail run` traps at a hole with `err.trap`. An AI
can write a program top-down and let the compiler say what each gap needs.

## 3. Effects and capabilities: what a call does to the machine

### 3.1 Inferred effects, shown everywhere (ADR)

Every function has an inferred **effect set**, a subset of
`{io, fail, mutates, unsafe, spawn}` for now (`alloc` arrives with the
backend). `io` is "touches the world outside the program" (standard modules
in §4 and `print`). The set is never written in source by default, so tokens
don't rise, but it is:

- shown by `ail api` and in diagnostics;
- checkable: `@pure fn f(...)` is a compile-time error if `f` has `io`,
  `unsafe` or `spawn`, with the diagnostic naming the call chain that adds
  the effect.

This gives the AI local reasoning about a callee without reading its body
(wishlist 1.1), at zero syntax cost, and is the foundation for sandboxing.

### 3.2 Capabilities for ambient resources (ADR)

Ambient things (filesystem, environment, processes, clock, network,
randomness) are reached through standard modules (§4). The proposal: each
module's functions carry a capability effect (`fs`, `env`, `proc`, `clock`,
`net`), and `ail run --deny fs,proc FILE` (later, a package manifest) refuses
at *compile time* any program whose `main` reaches a denied capability. An
AI-written tool is then sandboxed by its signature, not by a container.
Nothing changes for a program that uses no capabilities. This resolves
wishlist 5.1 and 6.6 together. The `--deny` flag lands tonight; the package
manifest half waits for packages (Topic 13), which the designer hasn't
decided.

### 3.3 Deterministic execution

`ail run --deterministic` is the same as denying `clock` and `random` and
seeding any remaining source of nondeterminism. Byte-identical output across
runs is the goal for everything the toolchain prints (wishlist 6.3); the
diagnostics and API already sort.

## 4. Standard modules: the machine through typed primitives

`use` imports are specified (`mod.use.1–2`) but not implemented. Standard
modules are **built in**: `use std.fs` makes `fs` available, and
`fs.read_text(path)` calls it. User modules (directories, `pub`, qualified
access, ADRs 0097–0099) are implemented by their own lane; packages and
manifests stay design-only.

| Module | Functions (first cut) | Effect |
|---|---|---|
| `std.fs` | `read_text(path) str ! IoError`, `write_text(path, text) ! IoError`, `append_text`, `exists(path) bool`, `list_dir(path) List[str] ! IoError`, `remove(path) ! IoError` | `fs` |
| `std.env` | `args() List[str]`, `get(name) str?`, `cwd() str` | `env` |
| `std.io` | `read_line() str?`, `read_all() str`, `eprint(value)`, `exit(code int)` | `io` |
| `std.proc` | `run(cmd str, args List[str]) Output ! ProcError` with `Output { status int, out str, err str }` | `proc` |
| `std.time` | `now_ms() int`, `sleep_ms(ms int)` | `clock` |
| `std.json` | `enum Json { Null, Bool(b), Num(n float), Str(s), Arr(items), Obj(members) }`, `parse(text) Json ! JsonError`, `Json.to_text()`, `Json.get(key) Json?`, `Json.at(i) Json?` | none |

Every failure is a typed error (ADR 0045), every result is a value an AI can
pattern-match. `IoError` carries `path` and `kind`. These are enough to write
real agent tools in AIL: read a file, run a command, parse its JSON, write a
report.

## 5. Efficiency

### 5.1 Tokens (criterion 1)

Three open questions are resolved by measurement with `tools/tokbench`:
OQ-041 format specifications in interpolation (`"{x:>8}"`, `"{r:.2f}"`),
OQ-043 `str.len()`, OQ-044 nested optionals. Format specs matter most: table
and money printing is common in agent tools and is unwritable today.

A **language card** (wishlist 4.6): `docs/language-card.md`, the whole
language under 4,000 tokens as measured by tokbench, versioned, so any AI can
load AIL in one read. The quick reference stays as the longer form.

### 5.2 Interpreter speed (criterion 4, and the Stage 3 oracle)

The interpreter looks locals up by name and clones values freely. A
benchmark harness (`tools/perfbench/run.sh`, one command per ADR 0071) with
three programs from `examples/` measures wall time, and targeted changes
(slot-resolved locals, fewer clones on reads, `Rc` strings) are accepted only
with a measured win and identical output. The interpreter is the permanent
oracle (ADR 0032), so it must stay simple and obviously correct.

### 5.3 The AI's own loop

`ail check` on the 800-line examples is already well under a second. The
semantic API (§2) removes the fix-one-error-recompile loop and the
read-the-whole-file loop, which are the real costs.

## 6. Buildout

- **`ail test FILE`**: `@test fn name()` functions (attributes per ADR 0083)
  run in file order; `assert(cond)` and `assert_eq(a, b)` in the prelude;
  results as text or `--json` (name, pass/fail, position, message). Tests
  live next to the code (wishlist 4.3). Serious decision: delegated, since
  ADR 0083 anticipates `@test`.
- **Spec conformance harness** (ADR 0002): every fenced `ail` block in
  `docs/spec/` is extracted and checked by the selftest, with an annotation
  for blocks that are expected to fail or that use unimplemented features.
  The spec then can't drift from the bootstrap.
- **Doc comments**: `##` comments are attached to declarations and surfaced
  by `ail api`.
- **More tests and examples**: an agent-tool example (`examples/tool.ail`)
  that reads a file, runs a command, parses JSON and writes a report, as the
  milestone 7 program.

## 7. Workstreams

Ten lanes, each self-contained, each branching from `claude/project-thread-ipafhh`
and opening a pull request against it. The integrator reviews, resolves
conflicts in `check.rs`, `parser.rs` and `interp.rs` (the shared hot spots),
runs `./ail selftest` on Linux and `tools/verify.sh` on macOS, and opens the
final pull request against `main`.

| # | Lane | Size | Touches | Depends on |
|---|---|---|---|---|
| 1 | Diagnostics schema v1, `ail explain`, typed holes | large | `diag.rs`, `main.rs`, new `explain.rs`, `lexer.rs`, `parser.rs`, `check.rs`, `docs/design/semantic-api.md`, ADRs | none |
| 2 | `ail api`: outline, symbols, references, doc comments | large | new `api.rs`, `lexer.rs` (doc comments), `main.rs` | none |
| 3 | User modules: directories, `pub`, `use a.b`, qualified access (ADRs 0097–0099) | large | `main.rs`, `parser.rs`, `check.rs`, `interp.rs`, tests with module directories | none; shares `use` with 4 |
| 4 | Standard modules `std.fs`, `std.env`, `std.io`, `std.proc`, `std.time` behind `use std.*` | large | `parser.rs` (`use std.x` only), `check.rs`, `interp.rs`, builtins, ADR | none; shares `use` with 3 |
| 5 | `std.json` in the prelude: `Json`, `parse`, `to_text`, accessors | medium | `prelude.ail`, builtins | none (prelude-level until 4 lands) |
| 6 | Effects inference, `@pure`, capability denial (`ail run --deny`) | large | new `check/effects.rs`, `main.rs`, ADRs | none; capability names come from the §4 table; 2 integrates the display |
| 7 | `ail test`, `@test`, `assert`, `assert_eq` | medium | `parser.rs` (attributes), `main.rs`, `prelude.ail`, `interp.rs` | none |
| 8 | Formatter `ail fmt` (roadmap refinement 4) | large | new `fmt.rs`, `lexer.rs` (comment tokens), `main.rs`, ADR | none |
| 9 | Spec conformance harness; OQ-041 format specs by measurement; language card | medium | `bootstrap/tests/spec.rs`, `docs/spec/*`, `lexer.rs`, `check.rs`, `interp.rs`, ADRs, `docs/language-card.md` | none |
| 10 | Interpreter performance: perfbench and slot-resolved locals | medium | `tools/perfbench/`, `interp.rs` | none |

The designer chose "modules and formatter too" for the buildout, so lanes 3
and 8 exist.

### 7.1 Later rounds (added 2026-10-20 as the night went on)

Each round started once the previous one was mostly merged, with up to ten
implementation lanes open at a time plus two editor lanes (the designer's
stretch goal: a windowed code editor written in AIL, ADR 0180).

| Round | Lanes | Outcome |
|---|---|---|
| Editor | 11 `std.ui` text-grid windows (ADR 0177), mouse, clipboard, tabs; 12 `tools/edit/edit.ail`: editing, undo, highlighting, diagnostics with fixes, completion, go to definition, hover, find/replace, format on save, run and test keys | merged |
| 2 | 21 tokens round 3: comprehensions, bare variant patterns (ADRs 0182–0184); 22 tasks, channels, mutexes on a deterministic scheduler (0186–0189); 23 ownership: `unique`, `ref`, moves (0190–0193); 24 sized numerics and attributes (0194–0197); 25 property and doc tests (0198–0201); 26 `ail fix` (0202–0205); 27 `std.random` and `--deterministic` (0206–0209); 28 `requires`/`ensures` contracts (0210–0213) | merged |
| 3 | 31 list and `str` editing methods (0214–0215); 33 agent-tool corpus as the standing benchmark (0217–0218); 36 interpreter speed versus Python (0219–0221); 37 compile-time execution and the four-representation IR design (0222–0229); 38 packages, `ail.pkg`, per-dependency capability grants (0230–0233); 39 automatic parallelism, `par for`, SIMD, `std.tensor` design (0236–0239, 0246–0247) | merged |
| 4 | 41 tokens round 4 on the agent corpus (0250–0255); 43 documentation consistency sweep; 44 semantic API: locals, references, hover (0258–0261); 45 diagnostics audit and `ail fix` coverage (0262–0264); 46 runtime trace and trap snapshots (0265–0268); 47 fuzz and differential harness (0269–0271) | merged |
| 5 | 51 token parity on the agent corpus: implicit standard modules, error messages, `?` on `none`, one-line `if` (0277–0280); 54 `std.net`, `std.fs` walking, `std.proc` control (0281–0284); 55 error ergonomics (0285–0288); 56 `ail test --coverage` and run limits (0289–0290); 57 editor project features (0291–0293); 58 editor themes, `~/.ail-edit`, `env.home()` (0294–0296) | merged |
| 6 | 61 interpreter speed round three: no bytecode tier in the oracle (0297); 63 `for v in ch`, `with m as mut x`, `[f(x) par for x in xs]` (0300–0302); 64 `old()`, inherited contracts, struct `invariant` (0303–0305); 65 derive generators, `ToJson`/`FromJson` in AIL, `ail api --expanded` (0306–0309); 67 `ail api --card` (0310–0311); 69 editor agent panel (0312–0313); 70 `std.ui.widgets`, the first standard module in AIL (0314–0316) | merged |
| 7 | 71 `move self`, unique values in collections (0317–0319); 72 `Atomic`, cancellation, `--schedule=random` (0320–0322); 73 formatter chain splitting, attributes on their own line (0323–0324); 74 `ail api` on packages, `ail test --trace`, `--watch` through calls (0325–0327); 75 `std.tensor` CPU prototype with AVX2/NEON kernels and pooled `matmul` (0328–0330); 76 editor program card view (0331) | merged |
| 8 | 81 front-end speed: fast hasher, allocation-free keyword lexing, front-end allocator (0332–0334); 82 `ail explain` and `ail fix` coverage, spec chapter 22 (0335–0336); 83 agent corpus round two, five programs with Python twins (0337); 84 card positions, problems pane, replace in files, remembered tabs (0338–0339) | merged |
| 9 | 91 `Set` operations, `m.push(k, x)`, named optional arguments for std functions (0340–0342); 92 `FromJson` for generics, maps and enum payloads (0343); 93 tensor transpose, axis reductions, operators (0344–0345); 94 formatter splitting at operators, `--watch` on `push` (0346–0347) | merged (PR #85) |
| 10 | 101 supertraits, trait functions without `self`, generic `FromJson`, `Default` (0348–0350); 102 cancel trees, `send` under cancellation, atomic orderings (0351–0352); 103 editor live problems, reviewable replace, tests on save (0353–0354); 104 git dependencies and `ail.lock` (0355–0357); 105 mistakes corpus for the new features (0358) | merged |

Design-only items without a lane tonight: a package registry (OQ-049, the
designer's call; decided 2026-10-20 as ADR 0383, no registry), accelerator drivers (OQ-090, reads ADR 0010; decided 2026-10-20 as ADR 0381), TLS for
`std.net` (ADR 0283, a dependency and licence choice) and a bytecode tier
in the interpreter (ADR 0297 keeps the tree walker; ADR 0032 makes a second
evaluator the designer's call).

## 8. Assumptions recorded tonight

- The deadline is 8am US Eastern (12:00 UTC), per the designer.
- The designer chose to weight the semantic API, effects and capabilities,
  and standard modules **equally**, so each has implementation lanes tonight.
  Capabilities therefore get a first implementation (`ail run --deny`) in
  lane 6, keyed to the module table in §4, and packages stay design-only.
- The designer asked that all decisions tonight be made and recorded as
  Accepted (delegated, ADR 0076), with the designer overriding later if
  needed. ADRs written tonight therefore say
  `Decided by: AI (delegated, ADR 0076; the designer delegated tonight's
  serious decisions on 2026-10-20)`.
- Nothing lands on `main` without the designer's `tools/verify.sh` run.
