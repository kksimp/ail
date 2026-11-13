# 1081. The formatter writes `self.` before a bare member

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** designer (Kaleb)
- **Topic:** 10 — Traits; tooling (`ail fmt`)
- **Spec rules:** [trait.method.6](../spec/10-traits.md); fmt.self in [formatting.md](../design/formatting.md)
- **Supersedes:** the formatter clause of [ADR 0141](0141-implicit-self.md) (rule 2, "the formatter writes the bare form"); the rest of ADR 0141 stands
- **Evidence:** the AI-first review of 2026-11-12, item 7

## Context

ADR 0141 put a method's receiver members in scope by bare name to save 4% of
tokens against Python, and said the formatter writes the bare form. The
decision order is now AI understanding, then human readability, speed, tokens
(ADR 1077). At a bare name an agent can't tell a field from a local, or a
method from a function, without reading the type's declaration, often in
another file; `self.x` says it at the use site. Every mainstream language an
agent has seen marks the receiver either always (Python, Rust) or at least in
idiomatic code that crosses files.

## Options considered

### Option A — keep the bare form canonical
- **Pros:** fewest tokens (ADR 0141's 4%); no tree-wide change.
- **Cons:** the use site doesn't say what a name is; a reader has to find the
  type's fields and the trait's methods first.

### Option B — the formatter writes `self.`, bare names stay legal (chosen)
- **Pros:** canonical code says "member of the receiver" where it is used;
  code written bare by habit still compiles and `ail fmt` makes it explicit;
  reversible, and no program breaks.
- **Cons:** about 4% more tokens in method-heavy code; two spellings stay
  legal until a later version decides.

### Option C — `self.` required, bare names an error
- **Pros:** one spelling.
- **Cons:** breaks every program and the compiler's own source at once;
  bare-name habits from Swift, Kotlin, Java and C# become errors. A question for
  a later language version (the version line), not 0.1.

## Decision

`ail fmt` writes `self.x` and `self.f(...)` wherever a method or invariant
refers to a member of its receiver by bare name. The places are exactly those
the implicit-`self` pass rewrites (trait.method.6), so formatting never changes
a program's meaning. Bare names stay legal (ADR 0141 rules 1, 3 and 4 are
unchanged).

## Rationale

AI understanding comes first (ADR 1077), and a use site that names its receiver
is local reasoning. The token cost is the measured 4% of ADR 0141, which ranks
last. Keeping bare names legal costs nothing at run time and keeps habits from
other languages working.

## Consequences

- The formatter sees one file. A member declared in another file of the package
  (a field of a struct declared elsewhere, a method of a trait from another
  module) is left as written. A member the file can see is never mistaken: a
  method's local can't take a field's name (trait.method.7), and a bare call
  that could also mean a function is an error the pass leaves alone.
- The formatter refuses (fmt.safe) when the text with `self.` added doesn't
  parse, or the pass still finds a bare member in it.
- Landed in two steps: the formatter change shipped behind `ail fmt --self`
  (the constant `WRITE_SELF` in `compiler/fmt/fmt.ail`), so open lanes' format
  checks didn't change; then, as the last commit before the release candidate,
  one change made `self.` the only formatter output, removed `--self` and
  `WRITE_SELF`, ran `ail fmt` over the tree and refroze the fmt goldens. 0.1.0
  ships one canonical layout and no formatter option.
- Whether bare names become an error is a later version's question.

## Rejected alternatives

- **Keep the bare form canonical:** saves tokens, the lowest-ranked criterion,
  at the cost of the highest.
- **Require `self.` now:** breaks every existing program for no gain over the
  formatter writing it; can still be chosen later.
- **Turn the formatter change on at once:** every lane's `ail fmt --check` would
  start asking for `self.` in the files it touches, spreading the reformat
  across many open branches and their conflicts.
