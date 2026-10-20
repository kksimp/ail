# Glossary

Shared terminology for AIL. Use these terms consistently in docs, code,
diagnostics, and AI interfaces. Entries are alphabetical.

| Term | Definition |
|---|---|
| **ABI** | Application Binary Interface: calling conventions, data layout, and symbol rules that let separately compiled code interoperate. Windows x64 and the System V ABI (Linux, macOS) differ. |
| **Absent** | The prelude error `Absent` ("a value is absent") that `?` on a `none` fails with in a function whose errors are inferred, or in a test (ADR 0279, `err.prop.5`). |
| **ADR** | Architecture Decision Record: one file per design decision in `docs/decisions/`. See ADR 0001. |
| **AEAD** | Authenticated encryption with associated data: a cipher that encrypts and authenticates in one step and refuses to decrypt on a bad tag. TLS 1.3 uses ChaCha20-Poly1305 and AES-GCM (ADR 0366). |
| **Agent-tool corpus** | The benchmark programs of the kind an AI agent writes when operating a computer (`logscan`, `tasks`, `build`, and since ADR 0280 `snapdiff` and `csvreport`), each with a Python twin, measured for tokens and speed. See ADRs 0217 and 0218. |
| **Allocation ladder** | The ordered list of allocation strategies the compiler tries per site: eliminate > stack > in-place reuse > inferred arena > task-local heap > shared GC heap. See ADR 0012. |
| **`any Trait`** | A type holding any value whose type implements `Trait`. Calls dispatch dynamically (ADR 0079). |
| **AoS / SoA** | Array of Structs vs. Struct of Arrays: two memory layouts for collections of records. SoA often vectorises and caches better. |
| **Arena (scoped allocation)** | A region where many objects with a shared lifetime are allocated, then freed together in one operation. |
| **Asserted loop** | A `par for` loop that the checker can't prove independent but finds no conflict in; it relies on the programmer's assertion (`conc.par.5`, ADR 0237). |
| **Associated function** | A function declared `fn Type.name(...)` without `self`, called `Type.name(...)` (ADR 0082). In a trait, `fn zero() Self`, called `T.zero()` on a bounded type parameter (ADR 0349). |
| **Atomic** | `Atomic[int]` or `Atomic[bool]`, a cell shared by every copy, so tasks and `par for` iterations update it without a lock: `load`, `store`, `cas`, and `add` on an `int`, each one indivisible step, sequentially consistent unless `order:` names a weaker one (`conc.atomic.3–7`, ADRs 0320, 0352). |
| **Attribute** | `@name` or `@name(args)` metadata placed before a declaration (ADR 0083). |
| **Automatic implementation** | `Eq`, `ToStr` and `Hash`, which every type implements without an `impl` when its parts do; an explicit `impl Eq` or `impl ToStr` replaces it (ADR 0126, ADR 0133). |
| **Axis reduction** | `t.sum(axis)`, `t.max(axis)`, `t.mean(axis)`: a tensor reduced along one axis, whose shape drops that axis (`std.tensor.11`, ADR 0345). |
| **Bare variant pattern** | A variant with data written without parentheses in a pattern, `Rect =>`: it matches any fields and binds none (ADR 0183). |
| **Baseline (ISA)** | The minimum CPU feature level the compiler may assume without runtime checks. See ADR 0005 and ADR 0006. |
| **Bootstrap backend** | An existing code generator (e.g. LLVM) used temporarily, before AIL's custom backend exists. |
| **Bytecode tier** | The second evaluator in the bootstrap: each function's body compiled on its first call to register code that works on the tree walker's environment; the walker stays the oracle, and `--no-bytecode` turns the tier off (ADRs 0359, 0361). |
| **Cancellation** | Asking a task to stop with `t.cancel()` or `g.cancel()`: the task sees `task.cancelled()`, `task.check()` fails with `Cancelled`, and blocking calls return early (a `send` on a full channel drops its value); a cancel reaches every task the task started; the scope still waits for it (`conc.cancel.1–4`, ADRs 0321, 0351). |
| **Canonical layout** | The one layout `ail fmt` writes for a given token sequence; examples and docs use it (ADR 0169, [formatting.md](design/formatting.md)). |
| **Capability** | An effect that reaches an ambient resource: `fs`, `env`, `proc`, `clock`, `random`, `net`, `ui`. `ail run --deny` refuses a program whose `main` reaches a denied one (ADR 0163, ADR 0165). |
| **Capability grant** | `caps LIST` on a manifest line: the capabilities a dependency (`dep` line) or the package itself (`package` line) may reach, checked at compile time; a `dep` line without one grants none (ADR 0232). |
| **Budget (token budget)** | `--budget N` on any command: about N estimated tokens of its output, most severe records first, then a note counting the rest with a `--from K` handle to continue (ADR 0394). |
| **Card (program card)** | `ail api --card`: one Markdown page of a program or package (entry, tests, manifest, examples, each module's signatures), trimmed to a token budget. The language card is the same idea for the language. See ADRs 0310 and 0311. |
| **Channel** | A bounded queue between tasks, `Channel[T](capacity)`: `send` waits while it is full, `recv` waits while it is empty and gives `none` once it is closed and empty; `for v in ch` receives until then (ADR 0100, ADR 0188, ADR 0300). |
| **Closing line** | In a multi-line string, the line holding the closing `"""`. When only spaces precede the delimiter, they count toward the common indentation and the line adds no text, not even a final newline (ADR 0135). |
| **Coherence** | The guarantee that a type has at most one implementation of a given trait across a whole program (OQ-036). |
| **Colorless tasks** | Concurrency where any function may block and the runtime suspends the task; no async/await marking (ADR 0095). |
| **Compile-time-safe** | A function whose effect set has none of `io`, `unsafe`, `spawn` or a capability, so constants and derive generators may call it; the same functions `@pure` accepts (`ct.const.2`, ADR 0225). |
| **Comprehension** | An expression that builds a list, set or map from `for` clauses: `[f(x) for x in xs if c]`, `{x for ...}`, `{k: v for ...}` (ADR 0182). |
| **Compressed reference** | A managed reference stored in fewer than 64 bits (e.g. a 32-bit offset into a heap region). See ADR 0014. |
| **Constant time (cryptography)** | Code whose branches and memory indexes never depend on secret data, so its timing reveals nothing about keys or plaintext. Required of the TLS primitives (ADRs 0360, 0366). |
| **Conformance test** | A test, usually extracted from a spec example, that checks an implementation follows a normative rule. |
| **Constant** | A top-level `NAME := expr` whose value is a constant expression, computed at compile time (before `main` in the bootstrap); visible in the whole module (ADR 0132, ADR 0225). |
| **Constant expression** | A constant's value: literals, other constants, collection and struct literals, operators, and calls of compile-time-safe functions; it can't hold a failure (`mod.const.2`, `ct.const.*`, ADR 0132, ADR 0225). |
| **Constraint (bound)** | A trait a generic type parameter must implement, written inline as `[T Trait]` (ADR 0074). |
| **Contract** | `requires` and `ensures` lines under a function's signature: what callers must satisfy and what the function promises, checked at run time (ADR 0210). |
| **Copy-on-write (COW)** | Values share storage until one of them is mutated, at which point that one gets its own copy. Unobservable to programs (`own.val.2`). |
| **Deadlock** | Every unfinished task is blocked, so none can continue. The program traps with `conc.task.7` instead of hanging (ADR 0186). |
| **`defer`** | `defer S` runs statement `S` when the enclosing block exits by any path, including a trap; with the block's cleanups, last first (ADR 0066, ADR 0191). |
| **Delegated decision** | A routine design decision made by the AI using the designer's priority order (ADR 0076). |
| **Dependency** | A package another package declares with a `dep` line in its `ail.pkg` and imports by that name (`use geo.shapes`) (ADR 0231). |
| **Derive generator** | A compile-time function `fn(TypeInfo) str ! DeriveError` that a trait names with `@derive(f)`; an empty `impl` of the trait runs it and checks the AIL text it returns as the impl's body, plus associated functions of the type (`ct.derive.*`, ADRs 0227, 0306, 0307). `ToJson` and `FromJson` are prelude examples (ADR 0308). |
| **Derived implementation** | An `impl` of `Eq`, `Ord` or `ToStr` with no methods; the language supplies it field by field (ADR 0127). An empty `impl` of a trait with a derive generator is derived by the generator instead. |
| **Deterministic mode** | `ail run --deterministic`: denies `clock`, `random`, `proc` and `net` so a program gives the same output for the same inputs (`eff.det.1–3`, ADR 0207). |
| **Diagnostics schema** | The versioned JSON form of `ail check --json`: one object per line, tagged `"api":1`, with span, notes and fixes; see [semantic-api.md](design/semantic-api.md) (ADR 0145). |
| **Diverging statement** | A statement that never completes normally: `return`, `break`, `continue`, `unreachable(msg)`, a `while true` with no `break`, or a branch whose arms all diverge. A block ending in one fits any expected type (ADR 0139). |
| **Doc comment** | A run of `##` lines directly above a declaration, with no blank line between; it documents that declaration (lex.comment.2, ADR 0152). |
| **Doc example** | A block fenced `ail` in a `##` doc comment, run by `ail test` as the test `doc:NAME` (ADR 0200). |
| **Drag race** | A head-to-head benchmark: the same small program in AIL, idiomatic C and tuned C, built and run automatically with identical outputs checked (ADR 0030, ADR 0036). |
| **`Drop`** | The prelude trait whose `fn drop(mut self)` is a unique type's cleanup; it runs once when the value's last owner goes out of scope and is never called directly (ADR 0191). |
| **Effect set** | What a function may do to the world outside its arguments: `io`, `fail`, `mutates`, `unsafe`, `spawn`, plus capabilities. Inferred over the call graph, never written; shown by `ail effects` (ADR 0163). |
| **Error message** | A string written after an error struct's name or an error variant (`BadName(text str) "bad name {text}"`); it is the type's display text and defines its `impl ToStr` (ADR 0278, `err.msg.1`). |
| **Error return trace** | The list of source locations an error passed through via `?`, recorded automatically and shown in error reports (ADR 0050). |
| **Error set** | The closed union of error types a function can fail with. Inferred from the body for non-public functions (`!T`), explicit at package APIs (`T ! E`). See ADR 0046. |
| **Error type** | A struct or enum declared with `error` (`error enum ParseError`). Only error types can be errors (ADR 0120). |
| **Escape analysis** | Compiler analysis that determines whether a value can outlive or leave its creating scope. A value that does not escape can be stack-allocated. |
| **Exclusivity** | The rule that two `mut` arguments of one call can't overlap. A read-only argument that overlaps a `mut` one is passed as a copy (ADR 0064, ADR 0119). |
| **Failure path** | A place in a function where it fails, passes a failure on or handles one: `?` on a result, `else` on a result, or a `return` of an error. `ail check --errors` lists them (`err.paths.1`, ADR 0286). |
| **Fix** | A machine-applicable edit a diagnostic carries (ADR 0145). `ail fix` applies the first fix of each diagnostic and re-checks (ADR 0203). |
| **Fix anchor** | How a fix names a position the checker doesn't know by coordinates: a word on the line, the end of the line, after a block, the call `f(...)` or bracket form `T[...]` that starts there, a chained comparison, or a character with its blanks (ADRs 0145, 0264). |
| **Fix (diagnostic)** | A machine-applicable edit attached to a diagnostic: a title and text replacements by line and column, applied all or nothing (ADR 0145). |
| **Foreign syntax** | A construct from Python, Rust, Go or TypeScript that AIL doesn't have (`&&`, `def`, `try`, `x++`, `len(xs)`). Reported once with the AIL spelling (`gram.foreign.1`, ADR 0262). |
| **Format specification** | The `[[fill]align][width][.decimals][x]` after the `:` in `"{x:spec}"` (ADR 0173). |
| **Front-end allocator** | The global allocator `ail check`, `ail api` and `ail effects` use: small blocks reused by 16-byte size class, larger ones bumped from one reserved region with huge pages; other commands use the system allocator. `AIL_SYSTEM_ALLOC=1` turns it off. See ADR 0332. |
| **Fuzz harness** | `tools/fuzz/run.sh`: seeded, well-typed programs, edge-value probes and token-level mutants, run through `ail` with crash, determinism, `fmt`, `api` and contract-differential checks. A crash becomes a regression case (ADRs 0269–0271). |
| **Generated code** | The text a derive generator returned, checked as if written in its `impl`; addressed `gen:impl:TYPE.TRAIT:LINE:COL` and shown by `ail api --expanded` (`ct.gen.*`, ADRs 0228, 0309). |
| **Generated origin** | The provenance of generated code: the deriving `impl` and the generator, shown by `ail api --expanded` (`ct.gen.4`, ADR 0228). |
| **Glob** | A path pattern with `*` and `?` inside a component and `**` for any number of directories; `fs.glob` returns the matching paths, sorted (`std.fs.9`, ADR 0281). |
| **`Hash`** | The standard trait of map keys and set elements. Automatic for every type whose `==` is automatic or derived and whose parts are `Hash`; has no methods (ADR 0133). |
| **HIR** | High-level IR: the resolved, typed, expanded program, named by stable ids; what `ail api` serves and the interpreter runs ([ir-architecture.md](design/ir-architecture.md), ADR 0222). |
| **Hand-off (bytecode tier)** | An expression or statement a compiled body leaves to the tree walker, which runs it on the same environment (ADR 0361). A method call whose receiver fails its guard is handed off with the receiver already evaluated (ADR 0363). |
| **Hot/cold splitting** | Placing rarely used fields (or code) away from frequently used ones to improve cache use. |
| **Hugging** | In `ail fmt`, splitting a too-long statement inside its last argument's list, so `f(a, [` stays on the first line and `])` closes the last (fmt.wrap, ADR 0169). |
| **Import name** | The name `use a.b` binds in its file: `b`, or `c` with `use a.b as c`. Items are reached through it as `b.item` (ADR 0099, ADR 0154). |
| **INDENT / DEDENT** | Tokens the lexer produces when indentation increases or decreases, giving indentation-based blocks a structure the parser can use (ADR 0033). |
| **Independent iteration** | A `for` loop whose iterations can run in any order or at once with the same result: no ordering effect, no early exit, and writes only to iteration locals, to `r[i]` by the loop variable, or to `int` accumulators (`conc.par.2–4`, ADR 0236). |
| **In-place reuse** | Reusing a uniquely owned value's memory at its last use for a new value, avoiding the allocator (Perceus-style). See ADR 0012. |
| **Insertion order** | The iteration order of maps and sets: the order keys were first added. Updating a key keeps its place (ADR 0131). |
| **Invariant** | An `invariant COND` line in a struct: a pure `bool` over its fields that holds after construction and after every `mut self` method, checked at run time (ADR 0305). |
| **ISA** | Instruction Set Architecture, e.g. x86-64 or ARM64 (AArch64). |
| **JSON path** | The place of a failure in a document a derived `from_json` reads, from the root: `items[2].kind`. Messages read `missing PATH` or `PATH: problem` (`json.derive.3`, ADR 0343). |
| **JSON value (`Json`)** | The standard `enum Json` (`Null`, `Bool`, `Num`, `Str`, `Arr`, `Obj`) that `parse_json` returns and `Json.to_text` prints. Numbers are `float`; objects keep document order (`json.type.1`, ADR 0161). |
| **Key function** | The `by: \|x\| key` argument of `sorted`, `sort`, `min` and `max`: an expression evaluated once per element to order by. It is an argument form, not a value (ADR 0250). |
| **Keyword** | A reserved word with fixed meaning, such as `fn` or `match`. Keywords can never be used as names (`lex.kw.1`, ADR 0114). |
| **Lazy binary splitting** | Splitting a parallel loop's range in half only when another worker is idle, so small or nested loops cost almost nothing (ADR 0238). |
| **Literal pattern** | A `match` pattern that is an integer, character, string or `bool` literal (ADR 0138). |
| **Lockfile** | `ail.lock`: the exact commit of every git dependency of the program, written by `ail run`/`ail test` when stale, refreshed by `ail pkg update` (`mod.pkg.8–9`, ADRs 0233, 0356). |
| **Dependency cache** | `$AIL_HOME/deps` (default `~/.ail/deps`): one read-only tree per fetched git dependency and commit, `NAME-COMMIT` (`mod.pkg.7`, ADR 0355). |
| **LSE** | Large System Extensions (ARMv8.1+): single-instruction atomic operations (CAS, LDADD, SWP…). Guaranteed by AIL's ARMv8.2-A baseline. |
| **Machine IR** | The per-ISA representation after instruction selection, carrying recovery records into debug info (ADR 0222). |
| **Managed reference** | A reference to an object whose storage is managed by the compiler or runtime, as opposed to a raw pointer. See spec `plat.ptr.3`. |
| **Manifest** | A package's `ail.pkg` file: its `package` line and one `dep` line per dependency (ADR 0230). |
| **MIR** | Mid-level IR: per body instance, a control-flow graph in SSA form with explicit allocations, copies, traps, dispatch and effects; the unit of the query compiler; internal and unversioned (ADR 0222, ADR 0223). |
| **Mistakes corpus** | `bootstrap/tests/mistakes/`: small programs that each hold one plausible AI mistake, with the expected rules and what `ail fix` makes of them (`gram.help.2`, ADR 0262, [diagnostics-audit.md](design/diagnostics-audit.md)). |
| **Module** | A directory of `.ail` files sharing one namespace (ADR 0097). |
| **Move** | Handing a value to a new owner; the old binding can't be used afterwards on any path. Unique values move on binding, assignment, return and into fields; any value moves as a `move` argument (ADRs 0068, 0190); a `move self` method moves its receiver (ADR 0317). |
| **`move self`** | A method receiver that takes ownership: `fn Conn.close(move self)`; `c.close()` moves `c`, with no mark at the call (ADR 0317). |
| **MTE** | Memory Tagging Extension (ARMv8.5+): hardware tags on memory and pointers that detect memory-safety errors. |
| **Mutable value semantics** | The model where every assignment or pass yields an observably independent value, and mutation happens only through owned variables or `mut` parameters (ADR 0060). |
| **Mutex** | `Mutex[T]`, a value shared between tasks: `get`, `set` and `swap` are indivisible, and `with m as mut x` (or `lock`/`unlock`) brackets a read-modify-write (ADR 0189, ADR 0301). |
| **Nested optional** | `T??`: an optional whose content is an optional. `none` is the outermost level; `else`, `?` and `if v := e` peel one level (`type.opt.6–8`, ADR 0209). |
| **Normative** | Part of the binding specification. Contrast with non-normative notes and rationale. |
| **`old(e)`** | In an `ensures` condition, the value `e` had when the function was called (ADR 0303). |
| **One-line `if`** | `if c then a else b`: the single-line form of an `if` with one condition and an `else`; `then` is a keyword only there (ADR 0280, `expr.if.8`). |
| **Or-pattern** | `p1 | p2` in a `match` arm: matches when any alternative does; binds no names (ADR 0138). |
| **Outline** | A program's declarations with signatures, doc comments, positions and ids but no bodies, printed by `ail api` (ADR 0150). |
| **PAC** | Pointer Authentication (ARMv8.3+): cryptographic signatures stored in a pointer's unused bits. |
| **Package** | The code under one package root: its root module and the modules below it, optionally described by an `ail.pkg` manifest (ADRs 0153, 0230). |
| **Package root** | The directory import paths are relative to: the directory given to `ail run DIR`, or the directory of the file given to `ail run FILE` (ADR 0153). |
| **`par for`** | A `for` loop with an assertion that its iterations are independent, for what the checker can't prove; provable conflicts are still errors (`conc.par.5`, ADR 0237). |
| **`par for` comprehension** | `[f(x) par for x in xs]`: a list whose elements are computed on the thread pool, in order, with the sequential result (`conc.par.12`, ADR 0302). |
| **PGO** | Profile-Guided Optimisation: optimising using measurements from real runs. |
| **Pinning** | Preventing the GC from moving an object, e.g. while foreign code holds its address. |
| **Place** | A local binding, parameter or `self`, or a field or element of a place: something that can be assigned to or passed `mut`. It is *mutable* when its root is a `mut` binding, `mut` parameter or `mut self` (`own.place.1`, ADR 0117). |
| **Poisoned lock** | A synchronisation primitive whose holder trapped. Later users get an error instead of possibly inconsistent state (ADR 0047). |
| **Pooled task** | A spawned task whose function touches no handle and has no ordering effect; the interpreter runs it on its own thread outside the scheduler (`conc.par.10`, ADR 0273). |
| **Position query** | `ail api --locals`, `--type` or `--refs` with `FILE LINE COL`: the names in scope, the type, or the references at one position (ADRs 0258–0261). |
| **Probe** | A one-to-three statement program that feeds edge values (extremes, zero, empty, non-ASCII) to an operation that may trap; a trap is correct, an interpreter panic is the finding (ADR 0269). |
| **Property test** | A `@test` with parameters: `ail test` runs it on generated inputs from a replayable seed and shrinks a failure to the smallest failing input (ADR 0198–0199). |
| **Pure function** | A function marked `@pure`: it has no `io`, `unsafe`, `spawn` or capability, though it may fail or take `mut` parameters (ADR 0164). |
| **Qualified name** | An item named through its module, `geo.Point`; the bootstrap renames every imported item to its module path plus its name (ADR 0156). |
| **Random schedule** | `--schedule=random:SEED` for `ail test` and `ail run`: the interpreter switches tasks at points picked from `SEED`, so a race shows up under some seeds and replays exactly under the same one (`conc.sched.1–2`, ADR 0322). |
| **Raw string** | `r"..."` or `r"""..."""`: a string literal with no escapes or interpolation (ADR 0135). |
| **Receiver** | The value a method is called on, written `self` (read-only) or `mut self` (mutable) in the method's declaration (ADR 0072). |
| **Reduction** | Combining every iteration's value into one, like `total += x`. AIL parallelises `int` `+=` reductions and `xs.sum()` without changing the result (`conc.par.4`, ADR 0239). |
| **`ref` cell** | A GC-managed shared mutable cell (`ref T`). The only way two names can observe each other's mutations (ADR 0063). |
| **Reference (semantic API)** | A use of a declared name, listed by `ail api --symbols` as the declaration's symbol id and a position (ADR 0151). |
| **Region inference** | Compiler inference of arenas (regions) for groups of allocations with a shared lifetime. See ADR 0012. |
| **Root module** | The module that declares `main`: the `.ail` files directly in the package root for `ail run DIR`, or the single file for `ail run FILE` (ADR 0153). |
| **Root store** | The set of root certificates a TLS client trusts as the end of a certificate chain: the platform's (or `SSL_CERT_FILE`'s) for `std.net` (ADR 0368). |
| **Rule ID** | A stable identifier for a normative spec rule, formatted `<chapter>.<section>.<n>` (e.g. `plat.ptr.1`). See ADR 0002. |
| **Run limit** | `--timeout`, `--max-memory` or `--max-steps`: a run that exceeds one traps with `run.limit.*` and a snapshot (ADR 0288). |
| **Runtime dispatch** | Choosing among code paths at run time based on detected CPU features. |
| **Safepoint** | A point where a thread can pause for the GC. Polls cost a little. Loops that provably don't allocate can omit them. |
| **Second-class borrow** | Temporary access to an argument that lasts only for a call and can't be stored or returned, so no lifetimes are needed (ADR 0062). |
| **`Self`** | In a trait, the implementing type; in a method or `impl`, the type it belongs to, with its type parameters (ADR 0128). |
| **SeqCst** | Sequentially consistent ordering: all threads observe atomic operations in a single global order. |
| **Self-hosted front end** | The lexer, parser and checker written in AIL under `compiler/`, run by the bootstrap and compared with the Rust ones on every program (Stage 3, ADR 0022). Its syntax tree is `compiler/ast/` (ADR 0376). |
| **SIMD** | Single Instruction, Multiple Data: vector instructions (NEON and SVE on ARM; SSE, AVX2 and AVX-512 on x86). |
| **Site cache** | A per-AST-node cache in the bootstrap interpreter (keyed by the node's address, valid because the program is immutable) that remembers a resolved call target, method, string literal or field slot. It never changes behaviour (ADR 0220). |
| **Sized number** | One of `i8 i16 i32 u8 u16 u32 u64 f32`: the numeric types besides `int` (`i64`) and `float` (`f64`). Literals take their type from context, and nothing converts implicitly (ADRs 0194–0195). |
| **Span-free fingerprint** | The hash of an item's HIR with spans replaced by offsets relative to the item, so moving code doesn't invalidate cached results (ADR 0224). |
| **Standard module** | A built-in module imported with `use std.NAME` (`fs`, `env`, `io`, `proc`, `time`) whose functions reach the operating system with typed failures (ADR 0157). |
| **Standard traits** | `Eq` (`==`), `Ord` (`<`, via `cmp` returning `Ordering`) and `ToStr` (text form), declared by the language (ADR 0127). |
| **Structured concurrency** | Tasks can't outlive the block that spawned them unless placed in an explicit `TaskGroup` (ADR 0096). |
| **Supertrait** | A trait another trait requires: `trait Ord Eq`. Implementing the subtrait needs the supertrait's `impl`, and a bound on the subtrait provides it (ADR 0348). |
| **SVE / SVE2** | Scalable Vector Extension (ARM): vector instructions whose width is chosen by the hardware. Optional for AIL (ADR 0006). |
| **Symbol id** | A declaration's stable name in the semantic API, `kind:path`, e.g. `method:Point.shift` or `field:Point.x`; it doesn't change when lines move (ADR 0151). |
| **Target** | An OS + ISA combination the compiler generates code for, e.g. `linux-arm64`. See spec `plat.target.1`. |
| **Task** | A function call running concurrently, started by `spawn f(x)`. Its handle, `Task[T]`, must be consumed once by `.wait()` or `g.add(t)` (ADRs 0095-0096, 0187). |
| **Task group** | `TaskGroup()`, which owns tasks that return nothing and waits for them when its block ends, or at `wait_all()` (ADR 0096, ADR 0187). |
| **Task-local heap** | A heap private to one task, collected independently with no synchronisation or write barriers. See ADR 0012. |
| **TBI** | Top Byte Ignore (ARM64): the hardware ignores the top 8 bits of an address, so software can store tags there for free. |
| **Tensor** | `tensor.Tensor[T]`: a value holding a shape and a contiguous buffer of `int` or `float`, whose methods and `+ - *` operators run as vector kernels (`std.tensor.*`, ADRs 0328–0330, 0344–0345). |
| **Test** | A function marked `@test`, run by `ail test`: it passes unless an assertion fails, it traps, or it returns an error (ADR 0167). |
| **Text cell** | One character position in a `std.ui` window, with a foreground and background color; the window is a grid of them, `cols` by `rows` (ADR 0177). |
| **Text form** | The `str` a value is shown as by `print`, interpolation and error reports: its `to_str()` (ADR 0126). |
| **Thread pool** | The interpreter's `min(cores, 8)` worker threads (`AIL_THREADS=N` overrides) that run a `par for` in chunks on copied values, with the sequential result (`conc.par.8`, ADR 0272). |
| **Tier 1 target** | A fully supported target: it gates CI, receives full optimisation, and ships official releases. |
| **Token listing** | The output of `ail api --tokens FILE`: one `LINE:COL KIND VALUE` line per token, then comments and diagnostics; the AIL lexer must print it byte for byte (ADR 0371, `api.tok.1–3`). |
| **Total order (float)** | Float's `cmp`: numeric order with NaN after every number and `-0.0` equal to `0.0`, used for sorting; the operators `<` and `==` stay IEEE (ADR 0130). |
| **Trace** | The events `ail run --trace` writes to stderr: calls, returns, effects, tasks and contract checks, with values, up to a budget (ADR 0265–0266). `ail test --trace` keeps them per failing test (ADR 0326). |
| **Trait** | A named set of method signatures that types implement explicitly with `impl Trait for Type` (ADR 0073). |
| **Trap** | The response to a bug (overflow, out-of-bounds access, failed assertion). It stops the current task, runs its deferred cleanup, and is reported to the parent task. In the main task it ends the program (ADR 0035, ADR 0047). |
| **Trap snapshot** | The source line and the stack, each frame with its arguments and locals, reported with a trap that ends `ail run` (ADR 0267). |
| **Trapped** | The prelude error `Trapped { rule, message }` that `Task.wait()` fails with when the task trapped (ADR 0187). |
| **TSO** | Total Store Order: x86's relatively strong memory-ordering model. |
| **Type argument inference** | Working out a generic call's type arguments from its arguments and expected type (ADR 0129). |
| **Typed hole** | `todo` written where code isn't finished: it takes the expected type, the checker warns with what it needs and the names in scope, and running it traps (ADR 0147). |
| **TypeInfo** | The prelude value describing a struct or enum as declared (name, id, fields, variants), passed only to derive generators; AIL has no run-time reflection (`refl.*`, ADR 0229). |
| **Unique type** | A `unique struct` or `unique enum`: its values can't be copied, only moved, and are released (their `Drop` run) exactly once (ADRs 0066, 0069, 0191). A list or map of a unique type is unique too (ADR 0318). |
| **Unit result** | A result with no success value, `! E` or bare `!`: the function either completes or fails (ADR 0139). |
| **Walk** | `fs.walk(dir)`: every file below a directory at any depth, as paths, sorted (`std.fs.8`, ADR 0281). |
| **Warning** | A diagnostic that doesn't stop a program from building or running, such as a `_` arm that hides enum variants (`expr.match.3`). Errors stop it. |
| **Watch** | `ail run --watch NAME`: each assignment to a `mut` variable `NAME`, with its new value and position; follows it into `mut` arguments and `mut self` calls, naming the frame that wrote (ADRs 0268, 0327). |
| **Weak memory model** | A model (as on ARM) in which loads and stores can be observed out of order unless barriers or ordered atomics are used. |
| **Whole-program optimisation** | Optimising across all modules at once (specialisation, devirtualisation, cross-module constant propagation). See OQ-017. |
| **Window host** | In the bootstrap, the main thread that owns `std.ui`'s window and serves the interpreter thread's requests, because macOS allows windows only on the main thread (ADR 0178). |
| **`with` block** | `with m as mut x` and a block: holds mutex `m`, with its value in `x`, and stores `x` back however the block ends; a trap keeps the old value (`conc.share.6`, ADR 0301). |
| **Work stealing** | A scheduler in which each worker thread has its own queue of work and idle workers take work from the others' queues; the runtime uses one such pool for tasks and parallel loops (ADR 0238). |
| **Write barrier** | Code the GC requires on certain pointer stores so it can track references between heap regions or generations. |
| **x86-64-v3** | x86-64 microarchitecture level including AVX2, FMA and BMI1/2. AIL's x86 baseline (ADR 0005). |
