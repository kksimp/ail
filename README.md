# AIL

**AI-First Language**, by Voltare. Pronounced "ale".

AIL is a native, memory-safe, garbage-collected systems programming language
designed from the ground up for humans **and** AI agents. It aims for C-class
performance, maximum semantic density per token, canonical formatting, and a
compiler that answers structured questions about your program.

```
main.ail      ail build · ail run · ail test · ail fmt · ail check
```

> **Status:** Stage 1, language specification and design. There is no compiler
> yet. See [docs/status.md](docs/status.md).

## Targets

Linux, Windows, and macOS are all first-class.

| | x86-64 (v3) | ARM64 (v8.2-A) |
|---|---|---|
| Linux | ✓ | ✓ |
| Windows | ✓ | ✓ |
| macOS | — | ✓ |

All targets are 64-bit and little-endian. x86-64 and ARM64 are developed in
strict lockstep ([ADR 0008](docs/decisions/0008-isa-priority-policy.md)), and all
five targets gate CI.

## Documentation

- [Documentation index](docs/README.md)
- [Vision and philosophy](docs/vision.md)
- [Design decisions](docs/decisions/README.md)
- [Language specification](docs/spec/README.md)
- [Open questions](docs/open-questions.md)
- [Roadmap](docs/roadmap.md)
- [Feasibility: token efficiency and performance estimates](docs/design/feasibility.md)
- [Research: beating C on systems and allocation-heavy code](docs/design/performance-beyond-c.md)

AI agents: read [AGENTS.md](AGENTS.md) first.

## License

Apache License 2.0. See [LICENSE](LICENSE).
