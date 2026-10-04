# 0099. Imports bring in modules, and their contents are used through qualified names

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 12 — Modules
- **Spec rules:** [mod.use.1–3](../spec/12-modules.md)
- **Background:** [design/concurrency-modules-options.md](../design/concurrency-modules-options.md) (qualified 35.6, selective 38.0)

## Deciding criterion

**Token usage:** importing the module and qualifying uses was 6% cheaper
than importing individual names. **AI understanding** agrees: `http.get` says
where `get` comes from (ADR 0084).

## Decision

1. `use a.b.c` imports module `a.b.c` and binds it to its last segment, `c`.
2. Imported items are accessed as `c.name`. There are no selective imports
   of individual names, and no wildcard imports.
3. `use a.b.c as d` binds a different name. That's needed when two imports
   share a last segment, since names can't be shadowed (ADR 0084).

```
use net.http

fn fetch(url str) Data ! FetchError
    resp := http.get(url)?
    json.decode(resp.body)?
```
