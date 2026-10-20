# 0150. `ail api` prints a program's declarations without bodies, as one-line JSON

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.out.1-5, api.text.1 ([semantic-api.md](../design/semantic-api.md#ail-api))
- **Extends:** [ADR 0024](0024-semantic-api-from-stage-2.md) (the semantic API grows each stage)
- **Evidence:** token counts below, `tools/tokbench/run.sh` (mean of five tokenizers)

## Context

An agent that needs one function from a file reads the whole file. The plan
(§2.3) asks for an outline: every declaration with its signature, doc
comment, position, stable id and effects, without bodies. The brief proposed
JSON pretty-printed with a two-space indent, with `--text` for a human.

Measured on four examples (gradebook, shapes, json, sheet; 8,956 source
tokens in all):

| Form | Tokens | vs source |
|---|---|---|
| Source | 8,956 | 100% |
| JSON, two-space indent | 23,494 | 262% |
| **JSON, one line (chosen default)** | **14,582** | 163% |
| **`--text` outline** | **3,634** | 41% |

## Options considered

### Option A — one-line JSON by default, `--pretty` and `--text` on request (chosen)
- **Pros:** 38% fewer tokens than indented JSON, for the same bytes a parser
  sees; matches the one-line JSON of diagnostics (ADR 0024).
- **Cons:** unreadable without `--pretty` or a JSON viewer.

### Option B — indented JSON by default (the brief)
- **Pros:** readable and diffable as is.
- **Cons:** 61% more tokens than one line when an agent reads it.

### Option C — the text outline by default
- **Pros:** the fewest tokens, 41% of the source.
- **Cons:** tools need a stable machine shape; the plan makes JSON the default.

## Decision

`ail api FILE` prints one JSON object on one line: `api` (version 1),
`file`, `items` (declarations sorted by position, members nested under
their type, trait or `impl`), then `references` with `--symbols`
(ADR 0151). `--pretty` indents it; `--text` prints a compact outline. Keys
are in a fixed order, so output is byte-stable. `effects` is present and
empty until effects land (plan §3). A file with errors has no outline; a
file without `main` does.

## Rationale

Tokens decide (ADR 0076, criterion 1): the default form is the one an agent
reads, and indentation costs 61% more for no information. `--text` is what
an agent should read to understand a file (41% of the source); JSON is what
a tool parses. Nesting members under their owner keeps each type's surface in
one place, which is how an agent asks ("what can I do with a `Drawing`?").

## Consequences

- Agents get a file's surface for well under half its tokens with `--text`.
- The effects lane fills `effects` without a shape change.
- Multi-file programs (modules) will need `ail api` on a module; the shape
  extends with a `module` key.

## Rejected alternatives

- Indented JSON by default: 61% more tokens than one line for the same data.
- Text by default: tools need the JSON shape, and the plan fixes JSON as default.
- Omitting `null` and empty keys: saves a little but makes the shape vary by
  item, so consumers need more cases (AI understanding).
