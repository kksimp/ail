# 1232. JSON Pointer and JSON Merge Patch on `Json`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 Standard library
- **Spec rules:** [json.pointer.1, json.merge.1](../spec/std-json.md)

## Decision

`j.pointer(path) Json?` (RFC 6901) and `t.merge(patch) Json` (RFC 7386), written in AIL as prelude methods
(`tools/native/prelude.ail`, embedded in `compiler/mir/prelude_embed.ail`). No checker, lowering or runtime change.
A malformed or missing path is `none`, never a trap. A target's own `null` members stay; a patch `null` removes the key.

## Rejected alternatives

- Built-in methods in `check/json_builtin.ail`: `Json` is a prelude enum, so AIL methods need no compiler work.
- `pointer(path) Json ! JsonError`: absence is normal for a read, and `get` and `at` return `none` already.
- A free function `json.merge(a, b)`: methods match `get`, `set` and `at`.
- Pointer writes and RFC 6902 patches: not asked for; a later ADR.
