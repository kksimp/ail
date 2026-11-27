# AIL Language Specification

> **Status:** drafted for v0.1.0 and updated for v0.2.0 (ADRs 1119 to 1172). Every chapter below holds normative rules
> with IDs, and the conformance harness checks its examples; *skeleton* means
> only a start. Open design questions are in [open-questions.md](../open-questions.md)
> and what is built is in [status.md](../status.md).

## Conventions

### Normative language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**,
**SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are to be
interpreted as described in RFC 2119 / RFC 8174 when they appear in bold.

Text inside a block that begins with **Note:** or **Rationale:** is
non-normative.

### Rule IDs

Every normative rule has a stable ID, written as a heading or a bold prefix:

```
<chapter>.<section>.<n>
```

- `chapter` is a short lowercase key that is fixed per chapter (e.g. `plat`,
  `lex`, `type`, `mem`).
- `section` is a short lowercase key within the chapter (e.g. `ptr`, `isa`).
- `n` is a positive integer, assigned in order within the section.

Rules for rule IDs:

1. Once published, a rule ID is **never reused or renumbered**.
2. A removed rule stays in place with the marker `[removed in <ADR>]`.
   A rule that only describes the language, with no behaviour for code to
   implement, carries the marker `[definitional]` and no citation is expected
   (`tools/rules-audit.sh` check 2 skips it).
3. A changed rule keeps its ID. The change is recorded in the ADR that
   justifies it.
4. Every rule cites the ADR(s) that justify it.
5. Compiler diagnostics and the AI semantic API will cite rule IDs.
6. Implementation limits (a construct the current implementation doesn't
   support yet) use the code `bootstrap.unsupported` (the code's name outlived the Rust bootstrap,
   ADR 0904). That is not a rule ID. The
   diagnostic names the spec rule of the unsupported construct.

### Examples and conformance tests

Every fenced block tagged `ail` in a chapter is a **conformance test**
(ADR 0002). the `spec` gate suite (`tools/gate/spec.ail`) runs `ail check --json` on each one
during the `spec` gate suite (`./ail selftest spec`) and reports failures by chapter and line. The
same rules apply to `docs/quick-reference.md` and `docs/language-card.md`, where a block
without `fn main` is a fragment without a tag (to save tokens). The fence
line says what to expect:

| Fence | Expectation |
|---|---|
| ```` ```ail ```` | A whole program: must check without errors (warnings are fine). |
| ```` ```ail fragment ```` | A snippet. Column-0 declarations (`struct`, `enum`, `error`, `unique`, `fn`, `trait`, `impl`, `const`, `use`, `type`, `@attribute`, `pub`, and `#` comments before them) stay top level; every other line goes into a generated `fn main()`. Must then check without errors. |
| ```` ```ail error=RULE ```` | Must report `RULE` among its errors. Handled as `fragment` when it has no `fn main`. |
| ```` ```ail skip=bootstrap ```` | Uses a feature that isn't built yet (the name dates from the Rust bootstrap, ADR 0904). Counted, not run. Remove the tag when the feature lands. |
| ```` ```ail skip=context ```` | An illustration that uses declarations the chapter doesn't show. Counted, not run. Prefer making the example self-contained. |
| ```` ```ail excerpt ```` | Outside `docs/spec/`: an incomplete snippet whose point is one line or one step. Not checked by `tools/check-doc-snippets.py`. Every other `ail` or `ail fragment` block in the docs must check. |
| ```` ```ail skip=bootstrap rules=ID ```` | Counted for the rules it cites, not checked or run (ADR 0498): the feature is specified but not yet built. |
| ```` ```ail norun ```` | Checked but not run: it traps on purpose, or needs a window, network or input the harness doesn't have. |
| ```` ```ail rules=ID,ID ```` | Demonstrates the listed rules (ADR 0498). Every rule of the types, expressions, std, effects and tools chapters must be demonstrated; `AIL_LIST_UNCOVERED=1 ./ail selftest spec` lists the rest. |
| ```` ```ail flags=A,B ```` | Checked with those `ail check` flags, such as `--deterministic` or `--deny fs`. Not run. |
| ```` ```ail tool=A,B,FILE expect=TEXT ```` | Runs `ail A B FILE` on the block and requires `TEXT` in stdout (ADR 0500). `DIR` is the block's directory. |
| ```` ```ail pkg=DIR status=N ```` | With `tool=`: `DIR` is a copy of the case package `tests/cases/packages/DIR`, and the tool must exit N. |
| ```` ```ail fixture=DIR ```` | Checks the case fixture `tests/cases/DIR`, which must report each rule in `rules=` (ADR 0500). |

A program that checks clean and has a `main` is also built and run natively
and must print its `.out` golden (ADR 0499, amended by ADR 0904). Blocks marked `norun`,
`flags=`, `tool=` or `fixture=` are not run that way.

An unknown tag fails the suite. Blocks in other languages (`text`, `ebnf`)
aren't run.

## Chapters

| # | Chapter | Key | State |
|---|---|---|---|
| 01 | [Platforms and targets](01-platforms.md) | `plat` | Drafted (partial) |
| 02 | [Lexical structure](02-lexical.md) | `lex` | Drafted (partial) |
| 03 | [Grammar](03-grammar.md) | `gram` | Drafted (partial) |
| 04 | [Types](04-types.md) | `type` | Drafted (partial) |
| 05 | [Expressions and statements](05-expressions.md) | `expr` | Drafted (partial) |
| 06 | [Memory model and allocation](06-memory.md) | `mem` | Drafted (partial) |
| 07 | [Ownership and value semantics](07-ownership.md) | `own` | Drafted (partial) |
| 08 | [Errors](08-errors.md) | `err` | Drafted (partial) |
| 09 | [Generics](09-generics.md) | `gen` | Drafted (partial) |
| 10 | [Methods and traits](10-traits.md) | `trait` | Drafted (partial) |
| 11 | [Concurrency](11-concurrency.md) | `conc` | Drafted (partial) |
| 12 | [Modules and programs](12-modules.md) | `mod` | Drafted (partial) |
| 13 | [Unsafe](13-unsafe.md) | `unsafe` | Drafted (partial) |
| 14 | [FFI and C interoperability](14-ffi.md) | `ffi` | Drafted (partial) |
| 15 | [Compile-time execution and metaprogramming](15-compile-time.md) | `ct` | Drafted (partial) |
| 16 | [Reflection](16-reflection.md) | `refl` | Drafted (skeleton) |
| 17 | [Standard modules](17-std.md) | `std` | Drafted (partial) |
| 18 | [Effects and capabilities](18-effects.md) | `eff` | Drafted (partial) |
| 19 | [Testing](19-testing.md) | `test` | Drafted (partial) |
| 20 | [Contracts](20-contracts.md) | `contract` | Drafted (partial) |
| 21 | [Runtime traces and trap snapshots](21-trace.md) | `trace`, `probe` | Drafted (partial) |
| 22 | [Tools: api, explain, fix, budgets, diff, edit, replay, bundles, snapshots](22-tools.md) | `api`, `explain`, `fix`, `budget`, `snap`, `diff`, `edit`, `replay`, `bundle`, `verify`, `shrink` | Drafted (partial) |
| 23 | [Native compilation](23-native.md) | `native` | Drafted (partial) |
| std | [Standard library: json](std-json.md) | `json` | Drafted |
| std | [Standard library: ini and .env](std-ini.md) | `std` | Drafted |
| std | [Standard library: cbor](std-cbor.md) | `std` | Drafted |
| std | [Standard library: msgpack](std-msgpack.md) | `std` | Drafted |
| std | [Standard library: regex](std-regex.md) | `std` | Drafted |
| std | [Standard library: xml](std-xml.md) | `std` | Drafted |
| std | [Standard library: textwrap](std-textwrap.md) | `std` | Drafted |
| std | [Standard library: decimal](std-decimal.md) | `std` | Drafted |
| std | [Standard library: semver](std-semver.md) | `std` | Drafted |
| std | [Standard library: lru caches](std-lru.md) | `std` | Drafted |
| std | [Standard library: screen](std-screen.md) | `std` | Drafted |
| std | [Standard library: text diffs](std-textdiff.md) | `std` | Drafted |
| std | [Standard library: stats](std-stats.md) | `std` | Drafted |
| std | [Standard library: throttle](std-throttle.md) | `std` | Drafted |
| std | [Standard library: uuid](std-uuid.md) | `std` | Drafted |
| std | [Standard library: base32](std-base32.md) | `std` | Drafted |
| std | [Standard library: base58](std-base58.md) | `std` | Drafted |
| std | [Standard library: bitset](std-bitset.md) | `std` | Drafted |
| std | [Standard library: bloom](std-bloom.md) | `std` | Drafted |
| std | [Standard library: cron](std-cron.md) | `std` | Drafted |
| attach | [Attach and `@observe`](std-attach.md) | `attach` | Drafted |
| std | [Standard library: HTTP server and router](std-net-http.md) | `http` | Drafted |
| std | [Standard library: native windows](std-app.md) | `std` | Drafted |
| std | [Standard library: real-time frame loop](std-gfx.md) | `std` | Drafted |
| std | [Standard library: WebM demuxing](std-webm.md) | `std` | Drafted |
| std | [Standard library: Opus decoding](std-opus.md) | `std` | Drafted |
| std | [Standard library: VP9 decoding](std-vp9.md) | `std` | Drafted |
| std | [Standard library: `std.audio` sound](std-audio.md) | `std` | Drafted |
| std | [Standard library: terminal control](std-term.md) | `std` | Drafted |
| std | [Standard library: URLs](std-url.md) | `std` | Drafted |
| std | [Standard library: text templates](std-template.md) | `std` | Drafted |

Chapters and keys are provisional until the designer approves the spec outline
(see [open-questions.md](../open-questions.md)).
