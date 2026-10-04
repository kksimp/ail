# 0086. Finish the core design before Stage 2; Hello World is the first Stage 2 milestone

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer (entry criteria proposed by the AI under ADR 0076)
- **Topic:** Roadmap

## Context

The designer wants to reach a point where a Hello World command-line program
runs on their Mac. Asked whether to start Stage 2 immediately with a minimal
end-to-end slice, or to finish the core design first, the designer chose to
**finish the core design first**.

## Decision

Stage 2 starts once these core topics have accepted decisions (enough that
early implementation won't need rework):

| Topic | Why it must come first |
|---|---|
| 11 Concurrency model (tasks, `spawn`/`await`, channels) | Shapes the runtime and the interpreter's execution model |
| 12/13 Modules, visibility, packages (core) | Every file, including Hello World, lives in this structure |
| Strings, collections, loops, literals | Needed by any real program, including Hello World |
| Program entry point and printing | The Hello World program itself |
| 14/15 Unsafe and FFI (core model) | Shapes the runtime boundary and the standard library's OS layer |

Reflection, metaprogramming, compile-time execution, and the detailed
standard library may follow during Stage 2.

**First Stage 2 milestone:** from the repository root on macOS,

```
./ail run examples/hello.ail
```

prints `Hello, world!`. It runs on the Rust bootstrap interpreter
(ADR 0023, ADR 0032) and is tested on Linux and macOS (ADR 0085).

## Consequences

- The roadmap shows these entry criteria.
- Hello World runs through the **interpreter** first. Native binaries
  arrive with Stage 4 (the optimising backend written in AIL, ADR 0018 and
  ADR 0022).

## Rejected alternatives

- **Start Stage 2 now with a thin slice:** declined, because the designer
  prefers less rework.
