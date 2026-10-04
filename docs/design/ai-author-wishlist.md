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

## Topics these feed into

- Core syntax / grammar: 1.2, 1.5, 3.1–3.5
- Type system: 1.1, 2.1, 2.2, 2.3
- Error handling: 1.4
- Modules / packages: 1.3, 5.1
- AI semantic API / diagnostics: 2.4, 2.5, 4.1, 4.4, 4.5
- Testing / docs: 4.2, 4.3, 4.6
