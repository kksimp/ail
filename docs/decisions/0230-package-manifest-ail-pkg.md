# 0230. A package's manifest is `ail.pkg`, one line per dependency

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 13 — Package management
- **Spec rules:** [mod.pkg.1–2](../spec/12-modules.md#packages)
- **Design:** [packages.md](../design/packages.md)

## Context

Packages need a file naming their dependencies and what each may reach
(OQ-045). The plan asks for the name and format to be chosen by tokens and
AI understanding, with an AIL literal or flat key-value file preferred over
TOML or JSON only if measured. Corpus `tools/tokbench/corpus/r9/` holds one
4-dependency manifest in seven formats.

## Options considered

Mean tokens over five tokenizers (`tools/tokbench/run.sh tools/tokbench/corpus/r9/*`):

### Option A — one line per entry: `dep geo path ../geo caps fs` (chosen)
- **Pros:** 72.0 tokens, the fewest. One dependency is one line: greppable,
  one-line diffs, and a diagnostic points at the line to change.
- **Cons:** a new mini-format; a value with spaces needs `"..."`.

### Option B — indented blocks, one key per line
- **Pros:** looks like AIL. **Cons:** 89.0 (+24%); a dependency spans lines.

### Option C — dotted `key = value` lines
- **Cons:** 107.4 (+49%); the name repeats on every line.

### Option D — TOML (inline tables 113.6, +58%; tables 118.0, +64%)
- **Pros:** familiar from Cargo. **Cons:** more tokens, two ways to write the
  same table, quoting rules.

### Option E — JSON
- **Cons:** 128.6 (+79%); no comments.

### Option F — an AIL struct literal
- **Pros:** one syntax for everything. **Cons:** 131.4 (+83%), the most; the
  loader would need the parser before it knows where packages are.

## Decision

A package root may hold `ail.pkg`: words separated by spaces, `#`
comments, `"..."` quoting. The first line is `package NAME [caps LIST]`;
each other line is `dep NAME (path DIR | git URL (rev R | tag T)) [caps
LIST]`. Unknown lines and keys, a repeated key, and a missing value are
errors (`mod.pkg.2`).

## Rationale

Tokens decided: 20% fewer than the next format and 37–45% fewer than TOML and
JSON. AI understanding agrees: the fixed keys per line kind leave one way to
write each entry.

## Consequences

- `ail.lock` (ADR 0233) uses the same line format.
- A registry can add a `version` key to `dep` lines without a new format.

## Rejected alternatives

- **B:** 24% more tokens and multi-line entries.
- **C:** repeats each dependency's name on every key.
- **D, E:** 58–79% more tokens; JSON has no comments.
- **F:** the most tokens, and a parser dependency in the loader.
