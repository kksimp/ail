# Metaprogramming: derives that generate ordinary AIL

> **Non-normative design note,** 2026-10-20 (lane 37 of the
> [AI-first plan](ai-first-plan.md)). Topic 17. Decided by ADRs
> [0227](../decisions/0227-derive-generators.md) and
> [0228](../decisions/0228-generated-code-is-visible-hir.md); normative rules
> `ct.derive.*` and `ct.gen.*` in [spec chapter 15](../spec/15-compile-time.md).
> Uses compile-time execution ([compile-time-execution.md](compile-time-execution.md))
> and type information ([reflection.md](reflection.md)).

## 1. What AI authors need, and what they must not get

The common reason for metaprogramming in systems languages is **writing the
same implementation for every type**: serialisation (`ToJson`, `FromJson`),
test-input generation (property tests, ADR 0198), comparison, hashing,
debug text, builders, conversions. AIL already makes three of these free:
`Eq`, `ToStr` and `Hash` are automatic (ADR 0126, ADR 0133), and an empty
`impl Ord for T` derives ordering (`trait.derive.1`, ADR 0127). What's
missing is a way for a *library* trait to be derived the same way.

What an AI author must **not** get is hidden code (wishlist 1.4, 4.5):

- code that appears at a use site without a declaration saying so;
- syntax that only exists after expansion (an agent can't parse what it
  can't see);
- generation that depends on anything but the program (files, network,
  time), so two builds differ;
- code the API can't show, so an agent debugs a stack trace into nothing.

## 2. The mechanism

**One new attribute on a trait, nothing new at the use site.**

```ail skip=bootstrap
## Converts a value to JSON.
@derive(derive_to_json) trait ToJson
    fn to_json(self) Json

## The generator: given a type's description, the body of its `impl ToJson`.
fn derive_to_json(t TypeInfo) str ! DeriveError
    if t.kind != TypeKind.Struct
        return DeriveError("ToJson can only be derived for structs, not {t.name}")
    mut members := List[str]()
    for f in t.fields
        members.push("JsonMember(\"{f.name}\", self.{f.name}.to_json())")
    "fn to_json(self) Json\n    Json.Obj([{members.join(", ")}])\n"

struct Version
    major int
    minor int

impl ToJson for Version    # derived: no methods written (trait.derive.1, ct.derive.1)
```

1. `@derive(fn_name)` on a trait names its **generator**: an ordinary
   function `fn(TypeInfo) str ! DeriveError` (`ct.derive.1`).
2. An `impl Trait for T` with no method definitions **derives** it, exactly
   as `impl Ord for T` already does. If the trait has a generator, the
   compiler calls it at compile time with `T`'s `TypeInfo` (`ct.derive.2`).
3. The generator returns **AIL source text**: the methods of the impl body.
   The compiler parses it, formats it canonically (`ail fmt`), and checks it
   exactly as if the author had written it in that `impl` (`ct.gen.1`).
4. A failure (`DeriveError`, a trap, or code that doesn't check) is a
   compile error **at the `impl` line**, with a note pointing into the
   expanded text (`ct.derive.4`, `ct.gen.3`).

The generator is compile-time code, so everything in
[compile-time-execution.md](compile-time-execution.md) applies: it must be
compile-time-safe (no `io`, no capability), it runs under the step and
memory limits, it is deterministic, and its output is cached by the
`derive(impl id)` query, keyed by the type's `TypeInfo` and the generator's
HIR.

## 3. Why this shape

- **The use site says what happens.** `impl ToJson for Version` is a
  declaration an agent can grep, that `ail api` lists (`impl:Version.ToJson`),
  and whose id the expanded methods hang off. Nothing is generated anywhere
  else.
- **The output is ordinary AIL.** `ail api --expanded` prints it (ADR 0228).
  An agent reads it with the same knowledge it uses for any code, and a stack
  trace into a derived method points at a real line of that text.
- **The generator is an ordinary function.** It can be called from a test
  with a hand-built `TypeInfo` and its output compared with `assert_eq`
  (ADR 0168): the generator is tested like any other code. No new language
  for macros.
- **No hygiene machinery is needed** beyond scope: AIL has no shadowing
  (ADR 0044), so a generated local that collides with a name in scope is an
  error in the expanded text, never a silent capture.
- **Sandboxed by construction.** A dependency's derive can't read a file or
  phone home while you build, because compile-time code has no capabilities.

## 4. Scope of generated code

Generated text is checked **in the scope of the `impl` that requested it**:
the derive site's module, its imports, and its private fields (mod.vis.3),
since the impl is in that module. The trait's own module is also reachable
by its qualified name, so a generator in package `json` can emit `json.Json`
whether or not the user imported it (`ct.gen.2`). Names the generator gets
from `TypeInfo` (fields, variants, the type) are valid at that site by
construction.

The output may contain only the impl's **method definitions**: no new
types, functions, `use` lines, or nested derives (`ct.gen.1`). That keeps a
derive's effect local to one `impl` block and one id prefix.

## 5. The built-in derives

`trait.derive.1`'s derives of `Eq`, `Ord` and `ToStr` are, by specification,
what a generator in the prelude would produce; `ail api --expanded` shows
their expansion like any other. The bootstrap keeps its native
implementation; showing their text is a follow-up. Automatic implementations
(`trait.auto.*`) are not derives: no `impl` asks for them, so `--expanded`
lists them by name on the type, not as text.

## 6. Comparison with other languages

| Language feature | What it gives | Why AIL doesn't copy it |
|---|---|---|
| C preprocessor | Textual substitution anywhere | Hidden code at every use site, no types, no scoping. The brief rules out textual macros. |
| Rust `macro_rules!` | Pattern-based expansion at call sites | New syntax that only exists after expansion; an agent can't parse `foo!(…)` without knowing the macro. Call-site macros are rejected outright. |
| Rust procedural macros (`#[derive]`, attribute macros) | Arbitrary token-stream rewriting in a separate crate | The derive *use* is good and AIL's empty `impl` matches it. But proc macros run with every capability, rewrite tokens (not typed items), and their output is invisible without `cargo expand`. AIL keeps the use, sandboxes the generator, and makes the output a first-class API answer. |
| Zig `comptime` | Types as values, arbitrary code at compile time, generics by evaluation | Very powerful and needs no separate macro language, which AIL keeps (generators are ordinary functions). But a signature that takes `comptime T: type` says nothing until evaluated, so local reasoning and the query cache suffer. AIL uses generics for type parameters and `TypeInfo` values for reflection. |
| D string mixins | Paste a compile-time string as code anywhere | The closest to AIL's "generator returns text", but placement is unrestricted: a mixin can appear in any statement or declaration. AIL fixes the placement to one `impl` body, requested by a declaration, and shows the result. |
| D template mixins | Instantiate declarations into a scope | Generics and default methods (ADR 0081) already cover it. |
| Swift macros | Typed expansion in a sandboxed plug-in, shown by the IDE | Close in spirit (sandboxed, inspectable). Its freestanding expression macros are call-site macros, which AIL rejects. |
| Go `go generate` | External tools write source files | Output is real code, which is good, but the generator runs with every capability and can go stale relative to the input. AIL regenerates on every build, from the cache. |
| Lisp macros | Code as data, unlimited | Unlimited is the problem: an agent can't read code that redefines the language. |

## 7. Rejected alternatives

- **Generators that return a typed syntax tree, built with `quote`/`$()`
  splicing.** Measured with `tools/tokbench` on a `to_json` generator:
  69.6 mean tokens against 77.6 for the text form (10% fewer). Generators are
  rare library code, so the difference is a tie on representative programs
  (ADR 0076: rare constructs). AI understanding decides: quoting adds two
  syntaxes, a `Code` type and splicing rules, while text is AIL the agent
  already writes, and the compiler checks it immediately in context. A typed
  builder can be added later as a library that returns text.
- **`@derive(Ord, ToJson)` on the type** (as Rust writes it). Measured:
  three derives on one struct cost 25.4 tokens against 31.8 for three empty
  `impl` lines, and one derive 15.6 against 16.0. Derives appear about once
  per 200 lines in `examples/` (10 in 2,099), so the saving is under 0.2% of
  a program: a tie. AI understanding decides: the empty `impl` already
  exists, keeps one way to implement a trait, and gives each derive its own
  id and line for diagnostics.
- **A `derives` clause on the type** (`struct Version derives Ord`): 12.8
  against 16.0 for one derive, the same tie; adds a keyword and a second way.
- **Attribute macros that rewrite the item they decorate:** the declaration
  the author wrote would no longer be the declaration that exists.
- **Item-level generation** (a generator that adds new types or functions to
  a module): not needed by any current use; left open as [OQ-093](../open-questions.md).
- **Unrestricted compile-time code that edits the program** (Zig's
  `@Type`, D's `mixin` everywhere): breaks "the use site says what happens".
