# What would make AIL easiest and most efficient for an AI author

> **Non-normative design note,** 2026-10-04. The designer asked the AI
> implementer what it would want from a language it will write every day.
> These are **proposals**, not decisions. Each one will be raised as a design
> question, with pros and cons, when its topic comes up.

The theme is simple. **An AI's main costs are reading context and recovering
from wrong guesses.** Every feature that lets an AI understand a piece of code
*locally* (without opening other files), or that turns a silent mistake into a
precise compiler error, saves more tokens than shorter syntax does.

## 1. Local reasoning: everything important is visible at the use site

| Proposal | Why it helps an AI | Possible cost |
|---|---|---|
| **1.1 Function effects in signatures**: whether a function allocates on the GC heap, mutates its arguments, can fail, is async, does IO, or uses unsafe. Inferred, but shown in signatures, docs and the API. | The AI can call a function correctly without reading its body or its callees. | Signature noise. Mitigated by inference plus display only, or by marking only the exceptions. |
| **1.2 One name, one meaning per scope**: no overloading by argument type, no implicit conversions, limited or no shadowing. | No need to resolve overloads mentally, so fewer wrong guesses. | Slightly longer names (`parse_int` vs `parse`). |
| **1.3 Explicit imports, no wildcards** | Where every name comes from is visible in the file itself. | A few more lines per file. |
| **1.4 No hidden control flow**: no exceptions, no implicit conversions that run code, no invisible destructors with side effects (use an explicit `defer`). | What you see is what runs. Easier to verify by reading. | Some convenience lost. |
| **1.5 Named arguments**, at least required when a function has two or more parameters of the same type | `copy(src: a, dst: b)` can't be silently swapped. That is a common AI (and human) bug. | More tokens per call. Could apply only when ambiguous. |

## 2. Make the compiler catch AI mistakes immediately

| Proposal | Why it helps | Possible cost |
|---|---|---|
| **2.1 Sum types plus exhaustive matching** | Add an enum variant and the compiler lists *every* place to update. That is the most reliable multi-file edit an AI can make. | None significant. |
| **2.2 Typed holes**: `todo` (or similar) compiles, and the compiler reports the expected type and the names in scope at that point | The AI can write a program top-down, and the compiler says exactly what each gap needs. Ideal for step-by-step generation. | Must be blocked in release builds. |
| **2.3 No undefined behaviour in safe code; defined evaluation order** | A program means exactly one thing. The AI never has to reason about "works on my machine". | Small optimisation limits (largely recoverable). |
| **2.4 Independent diagnostics**: all errors in one pass, no cascades, a stable order, each with machine-applicable fixes and a spec rule ID | One compile tells the AI everything to fix. No fix-one-error-recompile loop. | Error-recovery engineering in the type checker. |
| **2.5 Fast, deterministic `ail check`** with structured output (sub-second on edits) | The AI's main feedback loop. The faster and more structured it is, the fewer tokens wasted. | Incremental compiler architecture (already planned). |

## 3. Make code cheap to read and edit as text

| Proposal | Why it helps | Possible cost |
|---|---|---|
| **3.1 Edits stay local**: trailing separators allowed, one item per line in canonical format, declarations in any order, no forward declarations or headers | Adding a field, argument or case is a one-line change. Diffs and patches stay small and reliable. | None significant. |
| **3.2 Careful with significant indentation** | The brief's example uses indentation for blocks. That saves tokens, but AI text-editing tools match exact whitespace, and changing nesting means re-indenting many lines. Explicit block delimiters are more robust for edits. | **A real trade-off for the core-syntax topic.** It should be measured, not assumed. |
| **3.3 Keywords should be ordinary words** | Tokenizers usually encode common English words (`function`, `return`, `match`) as **one token**, the same as `fn`. Abbreviations rarely save tokens, but unusual punctuation combinations often cost *more*. | None. It just needs measuring with real tokenizers (OQ-007). |
| **3.4 Unambiguous brackets**: `[]` for generics (as in the brief's `Result[T, E]`), never `<>` | Removes the `a < b > c` parsing ambiguity, so humans, AI and the parser all read it one way. | `[]` can't also mean indexing in the same position. That needs a grammar decision. |
| **3.5 Only one obvious way** to loop, format strings, handle errors and declare things | Less variation means more predictable generated code and smaller context. | Some expressiveness given up deliberately. |

## 4. Context efficiency for whole projects

| Proposal | Why it helps | Possible cost |
|---|---|---|
| **4.1 `ail api <module>`**: an outline of signatures, types and docs only, without bodies | Understanding a dependency costs a few hundred tokens instead of thousands. | Small tool. |
| **4.2 Structured doc comments**: parameters, errors and examples, where the examples run as tests | Documentation an AI can trust, because it's tested. | Formatter and doc-tool work. |
| **4.3 Tests next to the code, with minimal syntax** | The AI sees behaviour and implementation together. | None significant. |
| **4.4 Stable symbol identities** (not line numbers) in the semantic API and diagnostics | Edits and queries keep targeting the right thing while the file changes. | Compiler bookkeeping. |
| **4.5 Inspectable compile-time code**: any generated code can be viewed through the API | No invisible code. The AI can see what metaprogramming produced. | Tooling work. |
| **4.6 A versioned "language card"** under 4k tokens, plus an automatic migration tool (`ail fix`) | Any AI can load the whole language at once, and code written against an older version gets migrated, not left stale. | Discipline to keep the card short. |

## 5. Safety for AI-written code

| Proposal | Why it helps | Possible cost |
|---|---|---|
| **5.1 Capability-based package permissions**: a dependency can't touch the filesystem, network, processes or FFI unless the project grants it | AI agents add dependencies quickly. This limits the damage a malicious or careless package can do (supply-chain safety). | Design work in the module and package system. Some friction. |
| **5.2 Unsafe code is a visible, queryable boundary** (already in the brief) | An AI and a reviewer can list every unsafe region instantly. | — |

## 6. Further additions (second round of feedback, 2026-10-04)

| Proposal | Why it helps | Possible cost |
|---|---|---|
| **6.1 Lightweight contracts**: optional `requires`/`ensures` conditions on functions, proved at compile time where possible and checked at run time otherwise | The AI can state its *intent*, and the compiler checks the code against it. Contracts are also compact, trustworthy documentation. They also give the optimizer facts (e.g. ranges that remove bounds checks). | Syntax and checker work. Must stay optional and cheap. |
| **6.2 Built-in property-based tests**: `for all x: T, property(x)`, with automatic input generation and shrinking | One line of intent covers thousands of cases. AI-written code gets tested far more thoroughly per token. | A generator for every type (derivable). |
| **6.3 Deterministic output everywhere**: formatter, diagnostics, reports and API answers in a stable order, byte-identical across runs | Diffs between two runs show only real changes. The AI never gets misled by reordering noise. | Discipline across every tool. |
| **6.4 `ail explain <rule-id or diagnostic>`**: prints the spec rule, a minimal example and a typical fix | Repairing an error needs no web search and no full spec. | Small tool, built on ADR 0002. |
| **6.5 Literal syntax for common data**: lists, maps and sets | Removes construction boilerplate in tests and configuration. | Grammar space. |
| **6.6 No hidden global state**: anything ambient (clock, randomness, environment, filesystem) is reached through explicit capabilities | Functions are testable and predictable, and their dependencies are visible. This pairs with 5.1. | Some plumbing. Needs ergonomic defaults for `main`. |

## Topics these feed into

- Core syntax / grammar: 1.2, 1.5, 3.1–3.5
- Type system: 1.1, 2.1, 2.2, 2.3
- Error handling: 1.4
- Modules / packages: 1.3, 5.1
- AI semantic API / diagnostics: 2.4, 2.5, 4.1, 4.4, 4.5
- Testing / docs: 4.2, 4.3, 4.6, 6.2, 6.4
- Type system / verification: 6.1
- Tooling: 6.3
- Standard library / runtime: 6.6

## 7. Agent corpus round three (2026-10-20)

From the ten programs in [program-measurements-agent-r3.md](program-measurements-agent-r3.md).
Only the four that cost more tokens than their Python twins are listed.

| Proposal | Why it helps | Possible cost |
|---|---|---|
| **7.1 A date module** (`std.time`: `date.from_unix(secs)`, `date.format(secs, "%Y-%m-%d")`, weekday, day arithmetic) | `report_dates` spends 233 tokens (+128%) rebuilding the civil date that Python's `datetime` gives in one call. Dates are common in agent tools, logs and reports. Covered by lane 129. | A calendar in the standard library, tested for leap years and time zones (UTC first). |
| **7.2 Bit operators** (`&`, `\|`, `^`, `<<`, `>>`, `~`) | `perms` spends 109 tokens (+41%) on `(mode / bit) % 2 == 1`. `mode & bit != 0` is the form an author and a reviewer both read at a glance. Covered by lane 140. | Operator precedence must match what authors expect; a new rule ID. |
| **7.3 Tuples as values** (`(a, b)`, `.0`, `s, b := f()`) | `route` and `perms` need a struct for every pair of results. Tuples give a short two-value return and an iterable list of pairs (`perms`' file table is an error today: `gram.syntax.1`). | Grammar and a type form; struct-like naming stays the choice for named fields. |
| **7.4 Bytes as a value** (`bytes(path) ! fs.IoError`, `b.len()`, `b[i]`, `b.hex()`) | `hexdump` shells out to `od` (+26%) because there is no bytes type. A byte-level inspector should not need a process. Covered by lane 123. | A new value type next to `str`; the UTF-8 rule stays on `str`. |

Rejected for this round: named arguments for `proc.run` (measured by round 9 at
+1.4 tokens, [ADR 0342](../decisions/0342-named-arguments-for-std-optionals-rejected.md));
an `enumerate` form of `for` (an index loop is already short, and the
diagnostic `expr.loop.1` teaches it).
