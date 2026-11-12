# Glossary

Shared terminology for AIL. Use these terms consistently in docs, code,
diagnostics, and AI interfaces. Entries are alphabetical.

| Term | Definition |
|---|---|
| **Task ledger** | `.ail/task-log.jsonl`: one line per `ail task verify` (stage, result, tokens printed, fixes, rules hit, wall ms); nothing leaves the machine ([ADR 1048](decisions/1048-task-ledger.md)). |
| **Core card** | `docs/language-card-core.md`: the language in under 2,000 tokens; `ail card` prints it, `ail card TOPIC` a section of the full card ([ADR 1051](decisions/1051-core-card-and-std-lookup.md)). |
| **`ail merge`** | A three-way merge that cuts `.ail` files into declarations, so branches that change different declarations never conflict; it works as a git merge driver ([ADR 1019](decisions/1019-ail-merge-driver.md)). |
| **Semantic conflict** | An error the merge of two branches has that neither branch has alone, such as a new caller of a function the other branch changed; `ail merge --check` reports it ([ADR 1019](decisions/1019-ail-merge-driver.md)). |
| **Task (agent)** | One change from `ail task start` to `finish`: a base revision, a goal and the last verify result, kept in `.ail-task` ([ADR 0917](decisions/0917-ail-task-protocol.md)). |
| **`ail mcp`** | The compiler's own tools served over MCP on stdio ([ADR 0918](decisions/0918-ail-mcp-compiler-tools.md)); not `ail serve --mcp`, which serves a program's `@tool` functions. |
| **HPACK** | The header compression of HTTP/2 (RFC 7541): a static and a dynamic table of header fields, integers with prefixes, and an optional Huffman code. Implemented in `compiler/rt/native/net_h2.ail` (ADR 0913). |
| **ALPN** | Application-Layer Protocol Negotiation (RFC 7301): a TLS extension in which the client lists protocols (`h2`, `http/1.1`) and the server names the one it will speak. The native `std.net` offers it on `https://` ([ADR 0913](decisions/0913-http2-client-over-alpn.md)). |
| **ABI** | Application Binary Interface: calling conventions, data layout, and symbol rules that let separately compiled code interoperate. Windows x64 and the System V ABI (Linux, macOS) differ. |
| **Absent** | The prelude error `Absent` ("a value is absent") that `?` on a `none` fails with in a function whose errors are inferred, or in a test (ADR 0279, `err.prop.5`). |
| **Ad hoc signature** | A code signature with no certificate: SHA-256 hashes of the file's pages in a code directory. Apple Silicon runs only signed executables; `ail build` signs Mach-O files this way itself (ADR 0389). |
| **Address folding** | Folding a constant offset into a memory access, so `s[bi + 3]` loads directly instead of copying the index, adding, then loading ([ADR 0625](decisions/0625-address-folding.md)). |
| **Allocation cache** | A thread's own free-list heads and bump page per size class, from which the native allocator serves small objects without a lock once the process has threads; refilled under the allocation lock and emptied at each collection ([ADR 0943](decisions/0943-per-thread-allocation-caches.md)). |
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
| **Bit operation** | An `int` method that works on the 64-bit two's complement bits: `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`, `shr` (expr.bit.1-2, ADR 0465). |
| **Block parameter** | A MIR value defined on entry to a block; each edge into the block passes one argument per parameter, in place of phi nodes (ADR 0222, ADR 0384). |
| **Bootstrap backend** | An existing code generator (e.g. LLVM) used temporarily, before AIL's custom backend exists. Never used (ADR 0010). |
| **Bytecode tier** | (Removed with the Rust bootstrap, ADR 0904.) The second evaluator in the bootstrap: each function's body compiled on its first call to register code that works on the tree walker's environment; the walker stays the oracle, and `--no-bytecode` turns the tier off (ADRs 0359, 0361). |
| **Bytes** | The built-in `bytes` type: an immutable sequence of `u8` for binary data, written `b"..."` (ADR 0412, `type.bytes.1`). Not text: `str` is UTF-8 text, and `b.text()` or `b.to_str()` read bytes as text. |
| **Cancellation** | Asking a task to stop with `t.cancel()` or `g.cancel()`: the task sees `task.cancelled()`, `task.check()` fails with `Cancelled`, and blocking calls return early (a `send` on a full channel drops its value); a cancel reaches every task the task started; the scope still waits for it (`conc.cancel.1–4`, ADRs 0321, 0351). |
| **Canonical layout** | The one layout `ail fmt` writes for a given token sequence; examples and docs use it (ADR 0169, [formatting.md](design/formatting.md)). |
| **Capability** | An effect that reaches an ambient resource: `fs`, `env`, `proc`, `clock`, `random`, `net`, `ui`. `ail run --deny` refuses a program whose `main` reaches a denied one (ADR 0163, ADR 0165). |
| **Capability grant** | `caps LIST` on a manifest line: the capabilities a dependency (`dep` line) or the package itself (`package` line) may reach, checked at compile time; a `dep` line without one grants none (ADR 0232). |
| **Budget (token budget)** | `--budget N` on any command: about N estimated tokens of its output, most severe records first, then a note counting the rest with a `--from K` handle to continue (ADR 0394). |
| **Build cache** | `ail build` keeps each finished executable under a SHA-256 of its inputs and copies it when the same inputs come again; `--no-cache` skips it (native.build.5, ADR 0655). |
| **Card (program card)** | `ail api --card`: one Markdown page of a program or package (entry, tests, manifest, examples, each module's signatures), trimmed to a token budget. The language card is the same idea for the language. See ADRs 0310 and 0311. |
| **Card level** | One of the fixed levels to which `ail api --card` trims a card, from full docs down to names; the middle level keeps signatures without docs ([ADR 0703](decisions/0703-card-middle-trim-level.md)). |
| **Channel** | A bounded queue between tasks, `Channel[T](capacity)`: `send` waits while it is full, `recv` waits while it is empty and gives `none` once it is closed and empty; `for v in ch` receives until then (ADR 0100, ADR 0188, ADR 0300). |
| **Child** | A process started with `proc.start`: a unique value with a process id and three `Channel[str]` streams (`stdin`, `out`, `err`); `proc.wait(move c)` or `proc.kill(move c)` ends it (ADRs 0424–0428). |
| **Chunked transfer** | The HTTP framing that sends a body in sized chunks when its length is not known up front; `std.net` uses it for request bodies ([ADR 0637](decisions/0637-http-persistent-connections-and-timeouts.md)). |
| **Closing line** | In a multi-line string, the line holding the closing `"""`. When only spaces precede the delimiter, they count toward the common indentation and the line adds no text, not even a final newline (ADR 0135). |
| **Coherence** | The guarantee that a type has at most one implementation of a given trait across a whole program (OQ-036). |
| **Colorless tasks** | Concurrency where any function may block and the runtime suspends the task; no async/await marking (ADR 0095). |
| **Compile-time-safe** | A function whose effect set has none of `io`, `unsafe`, `spawn` or a capability, so constants and derive generators may call it; the same functions `@pure` accepts (`ct.const.2`, ADR 0225). |
| **Comprehension** | An expression that builds a list, set or map from `for` clauses: `[f(x) for x in xs if c]`, `{x for ...}`, `{k: v for ...}` (ADR 0182). |
| **Compressed reference** | A managed reference stored in fewer than 64 bits (e.g. a 32-bit offset into a heap region). See ADR 0014. |
| **Conn** | A raw TCP connection, plain or TLS, with `read`, `read_line`, `write`, `set_timeout` and `close` (`std.net.12`, ADR 0421). |
| **Generator** | A function with `yield` that returns `Seq[T]`, a lazy single-use sequence compiled to a state machine. Proposed in [ADR 0965](decisions/0965-generators.md). |
| **Const generic** | A generic parameter that is an `int` value, not a type: `struct Vec[T, const N int]`. Proposed in [ADR 0954](decisions/0954-const-generics.md). |
| **Constant time (cryptography)** | Code whose branches and memory indexes never depend on secret data, so its timing reveals nothing about keys or plaintext. Required of the TLS primitives (ADRs 0360, 0366). |
| **Conformance test** | A test, usually extracted from a spec example, that checks an implementation follows a normative rule. |
| **Constant** | A top-level `NAME := expr` whose value is a constant expression, computed at compile time (before `main` in the bootstrap); visible in the whole module (ADR 0132, ADR 0225). |
| **Constant expression** | A constant's value: literals, other constants, collection and struct literals, operators, and calls of compile-time-safe functions; it can't hold a failure (`mod.const.2`, `ct.const.*`, ADR 0132, ADR 0225). |
| **Constraint (bound)** | A trait a generic type parameter must implement, written inline as `[T Trait]` (ADR 0074). |
| **Content encoding** | The HTTP `Content-Encoding` of a response body (gzip or deflate). `std.net` asks for it with `Accept-Encoding` and decodes the body itself ([ADR 0717](decisions/0717-transparent-content-encoding.md)). |
| **Context message** | The text given to a `?` as `e? "msg"`; a failing run prints it as `context:` lines after `error:`, innermost first ([ADR 0701](decisions/0701-native-error-context.md)). |
| **Contract** | `requires` and `ensures` lines under a function's signature: what callers must satisfy and what the function promises, checked at run time (ADR 0210). |
| **Cooperative wait** | A network call that lets the other ready tasks run while it waits, so a client and a server in one program make progress (`std.net.8`, ADR 0420). |
| **Copy-on-write (COW)** | Values share storage until one of them is mutated, at which point that one gets its own copy. Unobservable to programs (`own.val.2`). |
| **CRC-32** | The 32-bit cyclic redundancy check of gzip, zip and PNG; `hash.crc32` (`std.hash.3`, ADR 0445). |
| **Deadlock** | Every unfinished task is blocked, so none can continue. The program traps with `conc.task.7` instead of hanging (ADR 0186). |
| **`defer`** | `defer S` runs statement `S` when the enclosing block exits by any path, including a trap; with the block's cleanups, last first (ADR 0066, ADR 0191). |
| **Date, DateTime** | `std.time.date` types: a calendar day, and an instant with its zone and calendar fields. A duration is an `int` of milliseconds; a zone is a `str` (`"UTC"`, `"+05:30"` or an IANA name) (`std.date.1–15`, ADRs 0447–0451). |
| **Dispatcher** | A function the native backend generates for a method called on `any Trait` values, `any.T.m`, which branches on the box's tag to the implementing type's method (ADR 0616). |
| **Definitional rule** | A spec rule marked `[definitional]`: it describes the language rather than a behaviour the code must implement, so the rules audit does not require a citation ([ADR 0673](decisions/0673-definitional-rule-marker.md)). |
| **DEFLATE** | The compressed format of RFC 1951 inside gzip and zip, written and read in-house by `std.compress` (ADR 0445). |
| **Delegated decision** | A routine design decision made by the AI using the designer's priority order (ADR 0076). |
| **Dependency** | A package another package declares with a `dep` line in its `ail.pkg` and imports by that name (`use geo.shapes`) (ADR 0231). |
| **Derive generator** | A compile-time function `fn(TypeInfo) str ! DeriveError` that a trait names with `@derive(f)`; an empty `impl` of the trait runs it and checks the AIL text it returns as the impl's body, plus associated functions of the type (`ct.derive.*`, ADRs 0227, 0306, 0307). `ToJson` and `FromJson` are prelude examples (ADR 0308). |
| **Derived implementation** | An `impl` of `Eq`, `Ord` or `ToStr` with no methods; the language supplies it field by field (ADR 0127). An empty `impl` of a trait with a derive generator is derived by the generator instead. |
| **Destructuring** | Binding each element of a tuple to its own name: `a, b := t`, `for k, v in m`, and the first statement the parser adds to the body of `for a, b in xs` (`type.tuple.4-5`, ADR 0792). |
| **Deterministic mode** | `ail run --deterministic`: denies `clock`, `random`, `proc` and `net` so a program gives the same output for the same inputs (`eff.det.1–3`, ADR 0207). |
| **Diagnostics schema** | The versioned JSON form of `ail check --json`: one object per line, tagged `"api":1`, with span, notes and fixes; see [semantic-api.md](design/semantic-api.md) (ADR 0145). |
| **Diverging statement** | A statement that never completes normally: `return`, `break`, `continue`, `unreachable(msg)`, a `while true` with no `break`, or a branch whose arms all diverge. A block ending in one fits any expected type (ADR 0139). |
| **Doc comment** | A run of `##` lines directly above a declaration, with no blank line between; it documents that declaration (lex.comment.2, ADR 0152). |
| **Doc example** | A block fenced `ail` in a `##` doc comment, run by `ail test` as the test `doc:NAME` (ADR 0200). |
| **`Drop`** | The prelude trait whose `fn drop(mut self)` is a unique type's cleanup; it runs once when the value's last owner goes out of scope and is never called directly (ADR 0191). |
| **Effect set** | What a function may do to the world outside its arguments: `io`, `fail`, `mutates`, `unsafe`, `hw`, `spawn`, plus capabilities. Inferred over the call graph, never written; shown by `ail effects` (ADR 0163). |
| **Error message** | A string written after an error struct's name or an error variant (`BadName(text str) "bad name {text}"`); it is the type's display text and defines its `impl ToStr` (ADR 0278, `err.msg.1`). |
| **Error return trace** | The list of source locations an error passed through via `?`, recorded automatically and shown in error reports (ADR 0050). |
| **Error set** | The closed union of error types a function can fail with. Inferred from the body for non-public functions (`!T`), explicit at package APIs (`T ! E`). See ADR 0046. |
| **Error type** | A struct or enum declared with `error` (`error enum ParseError`). Only error types can be errors (ADR 0120). |
| **Escape analysis** | Compiler analysis that determines whether a value can outlive or leave its creating scope. A value that does not escape can be stack-allocated. |
| **Cookie jar** | `std.net.jar`: stores the `Set-Cookie` lines of responses and gives the `Cookie` header for a URL by RFC 6265 domain, path, expiry and `Secure` rules; `Jar.request` carries cookies through redirects (`std.net.20`, `std.net.21`, ADR 0638). |
| **Body** | A `net.Body` from `net.open`: the response head is read and the body is left on the connection, read with `read_bytes(max)` until it gives no bytes (`std.net.25`, ADR 0716). |
| **Exchange** | A request the `std.net` server has received, with the handle to answer it: `ex.request` and `ex.respond(response)` (`std.net.9`, `std.net.11`, ADR 0419). |
| **Exclusivity** | The rule that two `mut` arguments of one call can't overlap. A read-only argument that overlaps a `mut` one is passed as a copy (ADR 0064, ADR 0119). |
| **Failure path** | A place in a function where it fails, passes a failure on or handles one: `?` on a result, `else` on a result, or a `return` of an error. `ail check --errors` lists them (`err.paths.1`, ADR 0286). |
| **Fix** | A machine-applicable edit a diagnostic carries (ADR 0145). `ail fix` applies the first fix of each diagnostic and re-checks (ADR 0203). |
| **Fix anchor** | How a fix names a position the checker doesn't know by coordinates: a word on the line, the end of the line, after a block, the call `f(...)` or bracket form `T[...]` that starts there, a chained comparison, or a character with its blanks (ADRs 0145, 0264). |
| **Fix (diagnostic)** | A machine-applicable edit attached to a diagnostic: a title and text replacements by line and column, applied all or nothing (ADR 0145). |
| **Foreign syntax** | A construct from Python, Rust, Go or TypeScript that AIL doesn't have (`&&`, `def`, `try`, `x++`, `len(xs)`). Reported once with the AIL spelling (`gram.foreign.1`, ADR 0262). |
| **Format specification** | The `[[fill]align][width][.decimals][x]` after the `:` in `"{x:spec}"` (ADR 0173). |
| **Front-end allocator** | The global allocator `ail check`, `ail api` and `ail effects` use: small blocks reused by 16-byte size class, larger ones bumped from one reserved region with huge pages; other commands use the system allocator. `AIL_SYSTEM_ALLOC=1` turns it off. See ADR 0332. |
| **Function value** | A declared function used as a value, of type `fn(A, B) R`, that can be stored, passed and called; no closures (`type.fn.1-3`, ADRs 0455, 0456). |
| **Fused instruction** | A bytecode instruction that does several steps of a loop body at once, cutting dispatch cost ([ADR 0619](decisions/0619-bytecode-calls-loops-and-fusion.md)). |
| **Coverage-guided fuzzing** | `ail test --fuzz NAME`: a property test run on mutated inputs, keeping those that reach a new statement, with an optional corpus directory ([ADR 1015](decisions/1015-user-fuzzing.md)). |
| **Fuzz harness** | `tools/fuzz/run.sh`: seeded, well-typed programs, edge-value probes and token-level mutants, run through `ail` with crash, determinism, `fmt`, `api` and contract-differential checks. A crash becomes a regression case (ADRs 0269–0271). |
| **Gap (checker)** | A construct the AIL checker in `compiler/check/` doesn't type yet. The function that uses it is left out of the comparison with `ail check`, and its value is unknown (ADR 0378). |
| **Garlic** | The web browser written in AIL (own repository kksimp/garlic): text mode first, with a page model for agents. See garlic-browser.md in that repository. |
| **Graceful shutdown** | A server stop that refuses new connections and lets requests already arriving finish ([ADR 0640](decisions/0640-samesite-and-graceful-shutdown.md)). |
| **Grid** | `Grid[T]`, the prelude's 2-D list in one flat block, row by row: `Grid.fill(rows, cols, v)`, `g[i, j]` (`type.grid.*`, [ADR 0937](decisions/0937-grid-flat-2d-list.md)). |
| **Host (std.app)** | The program that draws a `std.app` view tree with the platform's own controls and sends back events over JSON lines; generic, it knows no app (`std.app.7-10`, ADR 0838). The macOS host is `tools/app-host/macos`, the Linux host `tools/app-host/linux`, the Windows host `tools/app-host/windows` (ADR 0888). |
| **Keep-alive** | A connection that stays open for more requests after the first response, instead of sending `Connection: close` ([ADR 0639](decisions/0639-http-server-keep-alive.md)). |
| **Lifetime hole** | A gap in a value's live interval, where it is dead between two uses (for example, across another branch). Linear-scan allocation with holes lets other values use the register there (ADR 0391). |
| **Module fuzzer** | A template program in `tests/fuzz/modules/` that `fuzz_modules.rs` fills with seeded adversarial inputs (text, JSON, hex bytes, regexes, dates, HTTP) and runs on both tiers and, once it exists, natively (ADR 0461). |
| **Drag race** | `tools/perfbench/run.sh --all`: min and median wall time of AIL, native AIL, `clang -O2`, `gcc -O2` and Python on twin programs, with ratios (ADR 0460). |
| **Generated code** | The text a derive generator returned, checked as if written in its `impl`; addressed `gen:impl:TYPE.TRAIT:LINE:COL` and shown by `ail api --expanded` (`ct.gen.*`, ADRs 0228, 0309). |
| **Generated origin** | The provenance of generated code: the deriving `impl` and the generator, shown by `ail api --expanded` (`ct.gen.4`, ADR 0228). |
| **Glob** | A path pattern with `*` and `?` inside a component and `**` for any number of directories; `fs.glob` returns the matching paths, sorted (`std.fs.9`, ADR 0281). |
| **`Hash`** | The standard trait of map keys and set elements. Automatic for every type whose `==` is automatic or derived and whose parts are `Hash`; has no methods (ADR 0133). |
| **hash64** | `hash.hash64(s)`: AIL's fast non-cryptographic 64-bit hash, stable across platforms (`std.hash.4`, ADR 0444). |
| **HIR** | High-level IR: the resolved, typed, expanded program, named by stable ids; what `ail api` serves and the interpreter runs ([ir-architecture.md](design/ir-architecture.md), ADR 0222). |
| **Hand-off (bytecode tier)** | An expression or statement a compiled body leaves to the tree walker, which runs it on the same environment (ADR 0361). A method call whose receiver fails its guard is handed off with the receiver already evaluated (ADR 0363). |
| **Hot/cold splitting** | Placing rarely used fields (or code) away from frequently used ones to improve cache use. |
| **Hugging** | In `ail fmt`, splitting a too-long statement inside its last argument's list, so `f(a, [` stays on the first line and `])` closes the last (fmt.wrap, ADR 0169). |
| **Import name** | The name `use a.b` binds in its file: `b`, or `c` with `use a.b as c`. Items are reached through it as `b.item` (ADR 0099, ADR 0154). |
| **INDENT / DEDENT** | Tokens the lexer produces when indentation increases or decreases, giving indentation-based blocks a structure the parser can use (ADR 0033). |
| **I420** | A picture as three planes: full-size luma (`y`) and half-width, half-height chroma (`u`, `v`), each a byte per sample; what `std.video.vp9` returns (`std.vpx.3`, ADR 0989). |
| **Independent iteration** | A `for` loop whose iterations can run in any order or at once with the same result: no ordering effect, no early exit, and writes only to iteration locals, to `r[i]` by the loop variable, or to `int` accumulators (`conc.par.2–4`, ADR 0236), or pushes one element per iteration to a built list (`conc.par.16`, ADR 0938). |
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
| **Machine model (runtime)** | The byte-addressed `Mem` that `compiler/rt/rt.ail` is written against, with its allocator, stdout buffer and trap record; native executables replace it with real memory and syscalls (ADR 0404, [native-abi.md](design/native-abi.md)). |
| **Managed reference** | A reference to an object whose storage is managed by the compiler or runtime, as opposed to a raw pointer. See spec `plat.ptr.3`. |
| **Manifest** | A package's `ail.pkg` file: its `package` line and one `dep` line per dependency (ADR 0230). |
| **MIR** | Mid-level IR: per body instance, a control-flow graph in SSA form with explicit allocations, copies, traps, dispatch and effects; the unit of the query compiler; internal and unversioned (ADR 0222, ADR 0223). |
| **Mistakes corpus** | `tests/mistakes/`: small programs that each hold one plausible AI mistake, with the expected rules and what `ail fix` makes of them (`gram.help.2`, [ADR 0262](decisions/0262-report-foreign-syntax-once-with-help.md), [diagnostics-audit.md](design/diagnostics-audit.md), [ADR 0646](decisions/0646-mistakes-corpus-round-two.md), [ADR 0648](decisions/0648-round-two-fix-rules-and-the-corpus-target.md)). |
| **Module** | A directory of `.ail` files sharing one namespace (ADR 0097). |
| **Monomorphisation** | Compiling a generic function once per set of type arguments, so each instance runs with concrete types. The compiler chooses per instance between specialising and sharing code (ADR 0080). |
| **Move** | Handing a value to a new owner; the old binding can't be used afterwards on any path. Unique values move on binding, assignment, return and into fields; any value moves as a `move` argument (ADRs 0068, 0190); a `move self` method moves its receiver (ADR 0317). |
| **`move self`** | A method receiver that takes ownership: `fn Conn.close(move self)`; `c.close()` moves `c`, with no mark at the call (ADR 0317). |
| **MTE** | Memory Tagging Extension (ARMv8.5+): hardware tags on memory and pointers that detect memory-safety errors. |
| **Mutation testing** | `ail test --mutate`: changes the reached lines of a program one way at a time, runs the tests on each change and reports the changes (mutants) that no test fails on ([ADR 1022](decisions/1022-mutation-testing.md)). |
| **Mutable value semantics** | The model where every assignment or pass yields an observably independent value, and mutation happens only through owned variables or `mut` parameters (ADR 0060). |
| **Mutex** | `Mutex[T]`, a value shared between tasks: `get`, `set` and `swap` are indivisible, and `with m as mut x` (or `lock`/`unlock`) brackets a read-modify-write (ADR 0189, ADR 0301). |
| **Native subset** | The constructs the native backend compiles (native.subset.1): free functions over numbers, string literals and lists of numbers; anything else is reported before code is made (ADR 0384). |
| **Native test runner** | `ail test`: builds each `@test` function natively and runs it; the comparison with the interpreter was retired ([ADR 0679](decisions/0679-native-test-runner.md), [ADR 0900](decisions/0900-refuse-rust-only-routes.md)). |
| **Nested optional** | `T??`: an optional whose content is an optional. `none` is the outermost level; `else`, `?` and `if v := e` peel one level (`type.opt.6–8`, ADR 0209). |
| **net.serve** | The capability to listen for connections, held in addition to `net` (`std.net.14`, ADR 0422). |
| **Normative** | Part of the binding specification. Contrast with non-normative notes and rationale. |
| **`old(e)`** | In an `ensures` condition, the value `e` had when the function was called (ADR 0303). |
| **One-line `if`** | `if c then a else b`: the single-line form of an `if` with one condition and an `else`; `then` is a keyword only there (ADR 0280, `expr.if.8`). As an operand it is parenthesised (ADR 0470). |
| **Or-pattern** | `p1 | p2` in a `match` arm: matches when any alternative does; binds no names (ADR 0138). |
| **Outline** | A program's declarations with signatures, doc comments, positions and ids but no bodies, printed by `ail api` (ADR 0150). |
| **PAC** | Pointer Authentication (ARMv8.3+): cryptographic signatures stored in a pointer's unused bits. |
| **Package** | The code under one package root: its root module and the modules below it, optionally described by an `ail.pkg` manifest (ADRs 0153, 0230). |
| **Package root** | The directory import paths are relative to: the directory given to `ail run DIR`, or the directory of the file given to `ail run FILE` (ADR 0153). |
| **Packed list** | The storage of a `List[int]`, `List[float]` or `List[bool]` in the interpreter: one machine word (one byte for `bool`) per element instead of a 32-byte `Value`. Invisible to programs; any other list is boxed ([ADR 0760](decisions/0760-packed-list-storage.md)). |
| **Page model** | Garlic's agent-facing view of a page: each perceivable element's role, accessible name and the cells it was drawn in, as `--json` (api 1). See garlic-browser.md in that repository. |
| **Parity probe** | `tools/gate/parity.ail`, the gate's `parity` suite: one line per feature (`FEATURE works`, `missing` or `differs`), found by trying it. The lines must be the same on macOS, Linux and Windows but for those in `tests/parity/expected-differences.txt` ([ADR 0925](decisions/0925-windows-release-gate.md)). |
| **`par for`** | A `for` loop with an assertion that its iterations are independent, for what the checker can't prove; provable conflicts are still errors (`conc.par.5`, ADR 0237). |
| **`par for` comprehension** | `[f(x) par for x in xs]`: a list whose elements are computed on the thread pool, in order, with the sequential result (`conc.par.12`, ADR 0302). |
| **Peephole** | A small rewrite of adjacent machine or bytecode instructions into a cheaper form ([ADR 0623](decisions/0623-branch-and-codegen-peepholes.md)). |
| **PGO** | Profile-Guided Optimisation: optimising using measurements from real runs. |
| **Pike VM** | The matcher of `std.regex`: one thread per program position stepped across the text once, so every pattern runs in time proportional to pattern size times text length (`std.regex.4`, ADR 0438). |
| **Pinning** | Preventing the GC from moving an object, e.g. while foreign code holds its address. |
| **Place** | A local binding, parameter or `self`, or a field or element of a place: something that can be assigned to or passed `mut`. It is *mutable* when its root is a `mut` binding, `mut` parameter or `mut self` (`own.place.1`, ADR 0117). |
| **Poisoned lock** | A synchronisation primitive whose holder trapped. Later users get an error instead of possibly inconsistent state (ADR 0047). |
| **Pooled task** | A spawned task whose function touches no handle and has no ordering effect; the interpreter runs it on its own thread outside the scheduler (`conc.par.10`, ADR 0273). |
| **Position query** | `ail api --locals`, `--type` or `--refs` with `FILE LINE COL`: the names in scope, the type, or the references at one position (ADRs 0258–0261). |
| **Push-builder loop** | A loop whose body names an outer list only in one top-level `out.push(e)`, so iteration `k` pushes element `k`; it runs on the pool with a list per chunk, joined in order (`conc.par.16`, [ADR 0938](decisions/0938-auto-parallel-push-builder-loops.md)). |
| **Renderer** | The part of a host that turns a view tree into controls on screen; one per platform (SwiftUI on macOS), so the same AIL app runs everywhere a host exists (ADR 0838). |
| **SameSite** | The cookie attribute (`Strict`, `Lax` or `None`) that limits which cross-site requests carry the cookie; the cookie jar honours it ([ADR 0640](decisions/0640-samesite-and-graceful-shutdown.md)). |
| **Screen model** | What is on a screen as items (role, text, colour name, box, state) that an agent queries in text within a token budget, without a vision model (`std.screen`, ADR 0878). |
| **Spill** | Storing a value in memory because no register is free for it; the allocator spills the value cheapest to keep in memory ([ADR 0624](decisions/0624-loop-weighted-spill-choice.md)). |
| **`std.args`** | The standard module for flags, options and positional arguments, with generated help ([ADR 0627](decisions/0627-std-args.md)). |
| **`std.file`** | The standard module of streaming file handles: a `unique` `Reader` and a buffered `Writer` that flushes when closed or dropped ([ADR 1071](decisions/1071-streaming-file-handles.md)). |
| **`std.csv`** | The standard module that reads CSV as rows and records, with quoted fields ([ADR 0626](decisions/0626-std-csv.md)). |
| **`std.log`** | The standard module for leveled messages on standard error ([ADR 0628](decisions/0628-std-log.md)). |
| **Stderr capture** | `ail test` collects a test's `io.eprint` output and shows it only under a failing test's `FAIL` line ([ADR 0693](decisions/0693-test-captures-stderr.md)). |
| **Tool method** | A method marked `@tool` that a running app offers an attached agent as a typed action: listed by `tools`, run by `call` (behind `--attach-input`) or through `ail attach mcp` (ADR 0986). |
| **Tuple** | An anonymous value of two or more elements with their own types, written `(1, "a")` and typed `(int, str)`; elements are `t.0`, `t.1`, ... (`type.tuple.1-9`, ADRs 0791-0795). |
| **Typed fast path** | A bytecode instruction specialised to the operand types the compiler knows, which skips the checks the general path makes ([ADR 0618](decisions/0618-bytecode-typed-fast-paths.md)). |
| **Replay** | Running a program again with the same inputs and thread schedule so a failure reproduces exactly. Property tests print the seed that replays a failing case (ADR 0199); random schedule testing reproduces from its seed on every target (ADR 0322). |
| **Stack trace** | The chain of calls from the point of a trap outward. A trap's diagnostic includes it (ADR 0047). |
| **`unsafe driver`** | The `ail.pkg` word (`package` or `dep` line) that lets a package use `std.hw`: fixed addresses, volatile access and barriers. It includes `unsafe allow`; a program without `ail.pkg` uses `--unsafe driver` (`unsafe.hw.1`, ADR 0820). |
| **`unsafe` block** | `unsafe` and an indented block: the only place an unsafe operation (`unsafe fn` call, later C calls and raw pointers) is allowed. A package may contain one only if its `ail.pkg` says `unsafe allow` (`unsafe.block.1–5`, ADR 0430). |
| **`extern c fn`** | A C function declared with a signature and no body, optionally with its library: `extern c "libm" fn hypot(a float, b float) float`. Calling it is unsafe; the interpreter loads the library on first call (`ffi.decl.2`, `ffi.load.1–3`, ADR 0431). |
| **Raw pointer** | `*T`: an address with no ownership, for C interop. Copying one is safe, using one (`read`, `write`, `offset`) is `unsafe` (`unsafe.ptr.1–6`, ADRs 0433, 0435). |
| **Probe** | A one-to-three statement program that feeds edge values (extremes, zero, empty, non-ASCII) to an operation that may trap; a trap is correct, an interpreter panic is the finding (ADR 0269). |
| **Property test** | A `@test` with parameters: `ail test` runs it on generated inputs from a replayable seed and shrinks a failure to the smallest failing input (ADR 0198–0199). |
| **Pure function** | A function marked `@pure`: it has no `io`, `unsafe`, `spawn` or capability, though it may fail or take `mut` parameters (ADR 0164). |
| **Qualified name** | An item named through its module, `geo.Point`; the compiler renames every imported item to its module path plus its name (ADR 0156). |
| **Random schedule** | `--schedule=random:SEED` for `ail test` and `ail run`: the scheduler (`compiler/rt/native/sched.ail`) switches tasks at points picked from `SEED`, so a race shows up under some seeds and replays exactly under the same one (`conc.sched.1–2`, ADR 0322). |
| **Raw string** | `r"..."` or `r"""..."""`: a string literal with no escapes or interpolation (ADR 0135). |
| **Receiver** | The value a method is called on, written `self` (read-only) or `mut self` (mutable) in the method's declaration (ADR 0072). |
| **Recording (replay)** | A JSON-lines file of every effect result of one run (files, environment, input, processes, network, clock), written by `--record R`; `--replay R` answers the program's effects from it so the run repeats exactly (ADR 0398). |
| **Reduction** | Combining every iteration's value into one, like `total += x`. AIL parallelises `int` `+=` reductions and `xs.sum()` without changing the result (`conc.par.4`, ADR 0239). |
| **`ref` cell** | A GC-managed shared mutable cell (`ref T`). The only way two names can observe each other's mutations (ADR 0063). |
| **Reference (semantic API)** | A use of a declared name, listed by `ail api --symbols` as the declaration's symbol id and a position (ADR 0151). |
| **Reference shape** | A character's OCR features (ink and outline-edge grids, size and position) measured from a font drawn by AIL's rasterizer; a scanned glyph is named by its nearest reference shape (ADR 0512). |
| **Region inference** | Compiler inference of arenas (regions) for groups of allocations with a shared lifetime. See ADR 0012. |
| **Rematerialisation** | Rebuilding a constant at each use instead of keeping it in a register; the native allocator gives constants no location (ADR 0386). |
| **Root module** | The module that declares `main`: the `.ail` files directly in the package root for `ail run DIR`, or the single file for `ail run FILE` (ADR 0153). |
| **Root store** | The set of root certificates a TLS client trusts as the end of a certificate chain: the platform's (or `SSL_CERT_FILE`'s) for `std.net` (ADR 0368). |
| **Rust removal** | Step D5 of ADR 0855: deleting `bootstrap/`, the interpreter and the Rust toolchain. `./ail` then runs a pinned native seed (ADR 0904). |
| **Rule ID** | A stable identifier for a normative spec rule, formatted `<chapter>.<section>.<n>` (e.g. `plat.ptr.1`). See ADR 0002. |
| **Run limit** | `--timeout`, `--max-memory` or `--max-steps`: a run that exceeds one traps with `run.limit.*` and a snapshot (ADR 0288). |
| **Runtime dispatch** | Choosing among code paths at run time based on detected CPU features. |
| **Safepoint** | A point where a thread can pause for the GC. Polls cost a little. Loops that provably don't allocate can omit them. |
| **Second-class borrow** | Temporary access to an argument that lasts only for a call and can't be stored or returned, so no lifetimes are needed (ADR 0062). |
| **`Self`** | In a trait, the implementing type; in a method or `impl`, the type it belongs to, with its type parameters (ADR 0128). |
| **Seed** | A native `ail` binary published as a release asset for one host (`ail-seed-ARCH-OS`). `./ail` downloads it, checks its SHA-256 against `tools/release/seeds.sha256` and uses it to build the compiler from source; only the designer publishes seeds ([ADR 0855](decisions/0855-retire-the-rust-bootstrap.md), [ADR 0976](decisions/0976-public-release-and-seed-trust.md)). |
| **Context pack** | `ail context PROGRAM FN --budget N`: the body, callee and caller signatures, types, tests and cited spec rules that an edit to `FN` needs, cut to N tokens (ADR 1020). |
| **Review report** | `ail review BASE..HEAD`: effects, public API, contracts, performance and untested code of a change, cut to a token budget (ADR 1024). |
| **Proved contract** | A `requires` at a call, or an `ensures` at every return, that the checker shows holds from interval facts; its run-time check is left out. `ail prove` lists them (ADR 1055). |
| **Inferred contract** | A `requires` or `ensures` line that `ail infer --contracts FN FILE` proposes from the calls the tests make: observed, not proved, at most 3, applied only with `--apply` (ADR 1023). |
| **Semantic diff** | `ail diff OLD NEW`: what changed between two versions by meaning (declarations by id, diagnostics, test outcomes), not by text (ADR 0396). |
| **SeqCst** | Sequentially consistent ordering: all threads observe atomic operations in a single global order. |
| **Self-hosted front end** | The lexer, parser and checker written in AIL under `compiler/`, now the only front end (the Rust one was removed, ADR 0904; Stage 3, ADR 0022). Its syntax tree is `compiler/ast/` (ADR 0376). |
| **SIMD** | Single Instruction, Multiple Data: vector instructions (NEON and SVE on ARM; SSE, AVX2 and AVX-512 on x86). |
| **Site cache** | (Removed with the Rust interpreter, ADR 0904.) A per-AST-node cache in the bootstrap interpreter (keyed by the node's address, valid because the program is immutable) that remembers a resolved call target, method, string literal or field slot. It never changes behaviour (ADR 0220). |
| **Sized number** | One of `i8 i16 i32 u8 u16 u32 u64 f32`: the numeric types besides `int` (`i64`) and `float` (`f64`). Literals take their type from context, and nothing converts implicitly (ADRs 0194–0195). |
| **Snapshot (expectation)** | An expected value written in a test: a literal in `assert_eq`, a doc example's `# =>` lines, or a `NAME.out` file. `ail test --update` rewrites stale ones and prints the diff (ADR 0395). |
| **Span-free fingerprint** | The hash of an item's HIR with spans replaced by offsets relative to the item, so moving code doesn't invalidate cached results (ADR 0224). |
| **Standard module** | A built-in module imported with `use std.NAME` (`fs`, `env`, `io`, `proc`, `time`) whose functions reach the operating system with typed failures (ADR 0157). |
| **Standard traits** | `Eq` (`==`), `Ord` (`<`, via `cmp` returning `Ordering`) and `ToStr` (text form), declared by the language (ADR 0127). |
| **String object** | The native layout of a `str`: a byte count, a character count and UTF-8 data, so `len()` is O(1) (ADR 0404). |
| **Structured concurrency** | Tasks can't outlive the block that spawned them unless placed in an explicit `TaskGroup` (ADR 0096). |
| **Structured edit** | An edit named by declaration id instead of by text: `ail edit` applies a JSON list of them (replace_body, replace, insert, delete, rename, import, and the refactors change_sig, extract and move of ADR 1054), then formats and checks (ADR 0397). The compact form is hunks under a declaration's name (ADR 0399). |
| **Superframe** | A VP9 packet holding several frames, with an index of their sizes in its last bytes; usually a hidden alternate reference and a shown frame (`std.vpx.2`, ADR 0989). |
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
| **Text layer** | The text a PDF carries as glyph codes plus its fonts' maps to Unicode, read with positions and no OCR; scans have none (ADR 0520). |
| **Thread pool** | The interpreter's `min(cores, 8)` worker threads (`AIL_THREADS=N` overrides) that run a `par for` in chunks on copied values, with the sequential result (`conc.par.8`, ADR 0272). |
| **Tier 1 target** | A fully supported target: it gates CI, receives full optimisation, and ships official releases. |
| **Token listing** | The output of `ail api --tokens FILE`: one `LINE:COL KIND VALUE` line per token, then comments and diagnostics; the AIL lexer must print it byte for byte (ADR 0371, `api.tok.1–3`). |
| **Total order (float)** | Float's `cmp`: numeric order with NaN after every number and `-0.0` equal to `0.0`, used for sorting; the operators `<` and `==` stay IEEE (ADR 0130). |
| **Trace** | The events `ail run --trace` writes to stderr: calls, returns, effects, tasks and contract checks, with values, up to a budget (ADR 0265–0266). `ail test --trace` keeps them per failing test (ADR 0326). |
| **Trait** | A named set of method signatures that types implement explicitly with `impl Trait for Type` (ADR 0073). |
| **Trap** | The response to a bug (overflow, out-of-bounds access, failed assertion). It stops the current task, runs its deferred cleanup, and is reported to the parent task. In the main task it ends the program (ADR 0035, ADR 0047). |
| **Trap record** | A per-site constant the native compiler emits for a runtime check: file, line, column, rule ID and message, which `rt_trap` prints (ADR 0405). |
| **Trap site** | A MIR record of one check that can trap: its rule, kind, operator and position; the code jumps to a stub that prints the interpreter's trap line (ADR 0384). |
| **Trap snapshot** | The source line and the stack, each frame with its arguments and locals, reported with a trap that ends `ail run` (ADR 0267). |
| **Call table** | The rows `trace.tab` holds in a native executable's code, one per user call: return address, call position and function. A trap's stack is printed by walking frame pointers and looking each return address up in it (trace.frame.1, ADR 0641). |
| **Trap stub** | Code at the end of a native function that a failed check branches to; it calls `rt.trap` with the check's kind, operands and constant source text (ADR 0385). |
| **Trapped** | The prelude error `Trapped { rule, message }` that `Task.wait()` fails with when the task trapped (ADR 0187). |
| **TSO** | Total Store Order: x86's relatively strong memory-ordering model. |
| **Table object** | The native runtime's `Map` or `Set`: an insertion-ordered entry array plus an open-addressing hash index, one allocation per table (ADR 0568). |
| **Type argument inference** | Working out a generic call's type arguments from its arguments and expected type (ADR 0129). |
| **Typed hole** | `todo` written where code isn't finished: it takes the expected type, the checker warns with what it needs and the names in scope, and running it traps (ADR 0147). |
| **Uhwhatnow** | `uhwhatnow?`, the typed hole that asks: like `todo`, but its warning ranks what in scope fits, best first, each with a fix (ADR 0872, `expr.hole.3`). |
| **TypeInfo** | The prelude value describing a struct or enum as declared (name, id, fields, variants), passed only to derive generators; AIL has no run-time reflection (`refl.*`, ADR 0229). |
| **Unique type** | A `unique struct` or `unique enum`: its values can't be copied, only moved, and are released (their `Drop` run) exactly once (ADRs 0066, 0069, 0191). A list or map of a unique type is unique too (ADR 0318). |
| **Unit result** | A result with no success value, `! E` or bare `!`: the function either completes or fails (ADR 0139). |
| **View tree** | The `app.Node` value an app's `view` returns: labels, buttons, fields, toggles, rows, columns and canvases, as plain data a host draws (`std.app.2`, ADR 0838). |
| **VP9** | A royalty-free video codec from the WebM Project, what YouTube sends open-source Chromium; decoded by `std.video.vp9` (ADR 0989). |
| **Vtable** | A table of function pointers for one type's implementation of a trait, used to call a method through `any Trait` at run time. Planned by ADR 0616 (lane 120, not yet landed). |
| **Walk** | `fs.walk(dir)`: every file below a directory at any depth, as paths, sorted (`std.fs.8`, ADR 0281). |
| **Warning** | A diagnostic that doesn't stop a program from building or running, such as a `_` arm that hides enum variants (`expr.match.3`). Errors stop it. |
| **Watch** | `ail run --watch NAME`: each assignment to a `mut` variable `NAME`, with its new value and position; follows it into `mut` arguments and `mut self` calls, naming the frame that wrote (ADRs 0268, 0327). |
| **TZif** | The binary file format of the IANA time zone database (RFC 9636) that `std.tz` reads from `/usr/share/zoneinfo`; a footer holds a POSIX rule for instants after the last stored transition (`std.tz.1`, ADR 0448). |
| **Weak memory model** | A model (as on ARM) in which loads and stores can be observed out of order unless barriers or ordered atomics are used. |
| **Whole-program optimisation** | Optimising across all modules at once (specialisation, devirtualisation, cross-module constant propagation). See OQ-017. |
| **Window host** | The main thread that owns the window and serves the program's requests (now the `std.app` host process, ADR 0838), because macOS allows windows only on the main thread (ADR 0178). |
| **`with` block** | `with m as mut x` and a block: holds mutex `m`, with its value in `x`, and stores `x` back however the block ends; a trap keeps the old value (`conc.share.6`, ADR 0301). |
| **Schedule sanitizer** | `ail sanitize`: runs a program on the fixed schedule and under seeded random schedules and reports any run whose result differs (ADR 0967). |
| **Leak report** | `ail leak`: the collector counts live bytes by object kind and by root at each collection, and reports growth and the roots that hold it (ADR 0969). |
| **Work stealing** | A scheduler in which each worker thread has its own queue of work and idle workers take work from the others' queues; the runtime uses one such pool for tasks and parallel loops (ADR 0238). |
| **Frame loop** | The `std.gfx` cycle of reading `Input`, calling `update` and `draw`, and presenting at the display's vsync (ADR 0956). |
| **Surface** | A shared-memory pixel buffer that a `std.gfx` program fills and the window host shows, with no JSON on the path (ADR 0957). |
| **Demuxer** | Reads a media container and splits it into the coded frames of each track, with their times, without decoding them (`std.media.webm`, ADR 0988). |
| **sans_line** | The natural line height of `sans` at 13 points that a `std.app` host reports on `start` and `resize`, so an app centres text without per-platform constants (`std.app.34`, ADR 1034). |
| **Write barrier** | Code the GC requires on certain pointer stores so it can track references between heap regions or generations. |
| **x86-64-v3** | x86-64 microarchitecture level including AVX2, FMA and BMI1/2. AIL's x86 baseline (ADR 0005). |
| **Remark** | One line of `ail perf --explain`: what the optimizer did to a loop (bounds checks, vector kernel, parallel verdict, calls left) with the rule and a fix (ADR 1021). |
