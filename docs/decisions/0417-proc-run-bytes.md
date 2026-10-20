# 0417. `proc.run_bytes`: bytes in, bytes out, `Output` unchanged

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Binary data
- **Spec rules:** [std.proc.8](../spec/17-std.md#processes-stdproc)
- **Builds on:** [ADR 0412](0412-bytes-type.md)

## Context

`proc.run` takes `input str?` and returns `Output` with `out` and `err` as
`str`, decoded lossily. Programs that pipe images, archives or compressed
data through a tool need the bytes. `proc.Output` is built by programs too
(the editor and several tests construct it).

## Options considered

### Option A — `run_bytes` returning a `RawOutput` (chosen)
- Same arguments as `run` with `input bytes?`; the result differs only in
  field types. No existing program changes.

### Option B — overload `run` on the type of `input`
- **Cons:** the result type would depend on the argument type, which the
  checker and a reader must both track; `run` has optional trailing
  arguments, so the overload is easy to misread.

### Option C — make `Output.out` a `bytes`
- **Cons:** breaks `r.out.trim()` and every use of the output as text.

## Decision

Option A. `proc.run_bytes(cmd, args, input?, timeout_ms?)` gives
`RawOutput { status, out bytes, err bytes }`; effects, failure kinds and the
time limit are those of `run`.

## Consequences

- `run` and `run_bytes` share one implementation; `run` decodes lossily as
  before.
