# 0161. `std.json` lives in the prelude as `Json` and `parse_json`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Standard library; AI-first plan section 4
- **Spec rules:** [json.type.1, json.err.1, json.parse.1, json.print.1, json.query.1](../spec/std-json.md)

## Context

Agent tools read structured output, so a JSON value, parser and printer are
standard. `examples/json.ail` already holds a complete parser. The module
mechanism (`use std.NAME`, lane 4) may not exist yet.

## Options considered

### Option A — prelude names `Json`, `JsonMember`, `JsonError`, `parse_json` (chosen)
- **Pros:** works today; one name each; tests don't depend on lane 4.
- **Cons:** the names are global until the module mechanism lands.

### Option B — wait for `use std.json`
- **Pros:** final spelling (`json.parse`).
- **Cons:** blocks the lane on another one; nothing to test.

### Option C — numbers as `Int(int)` and `Float(float)`, as in `examples/json.ail`
- **Pros:** exact for integers above 2^53.
- **Cons:** two ways to match a number; every consumer needs two arms.

## Decision

Ship the module in `prelude.ail` with `Num(n float)` for every number, the
accessors `get`, `at`, `as_str`, `as_num`, `as_bool`, `as_arr`, `as_obj`, and
`Json.to_text` (compact, members in document order). `\u` escapes decode
surrogate pairs and reject unpaired ones. When `use std.NAME` exists the
items move to `use std.json` as `json.parse` and so on (TODO for lane 4).

## Rationale

**AI understanding** decided: one number variant means one match arm and no
int-versus-float guessing, as in JavaScript. Tokens were a tie. Callers who need exact large integers can read
the text with `Str` today; an `Int` variant can be added later as a
backward-compatible variant.

## Consequences

- A user program that declares `Json` keeps it (see [ADR 0162](0162-optional-prelude-modules.md)); `examples/json.ail` is unchanged.
- Integers print as `1.0`; large integers above 2^53 lose precision.
- `JsonError` has no `ToStr`: the checker rejects implementing a prelude trait for a prelude type (`trait.impl.4`), so callers print `err.at` and `err.message`.

## Rejected alternatives

- **`Int`/`Float` split:** deferred (Option C), see above.
- **Native Rust parser:** the language should parse its own data format; the prelude parser is the reference for the later `std` build.
