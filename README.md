# AIL

**AI-First Language**, by Voltare. Pronounced "ale".

AIL is a native, memory-safe, garbage-collected systems programming language
designed from the ground up for humans **and** AI agents. It aims for C-class
performance, maximum semantic density per token, canonical formatting, and a
compiler that answers structured questions about your program.

```
main.ail      ail build · ail run · ail test · ail fmt · ail check
```

> **Status:** Stage 2, native self-hosting. The Rust bootstrap is deleted
> ([ADR 0904](docs/decisions/0904-rust-bootstrap-deleted.md)); `ail` builds itself.
> See [docs/status.md](docs/status.md).

## Try it

From the repository root on macOS or Linux:

```sh
./ail run examples/hello.ail
```

```
fn main()
    print("Hello, world!")
```

The first run downloads the published seed compiler, checks it, and builds the
native `ail` with it into `.native/`. Nothing outside the repository is changed. On Windows use `.\ail.ps1 run examples\hello.ail`
(not yet tested on Windows).

## Targets

Linux, Windows, and macOS are all first-class.

| | x86-64 (v3) | ARM64 (v8.2-A) |
|---|---|---|
| Linux | ✓ | ✓ |
| Windows | ✓ | ✓ |
| macOS | — | ✓ |

All targets are 64-bit and little-endian. x86-64 and ARM64 are developed in
strict lockstep ([ADR 0008](docs/decisions/0008-isa-priority-policy.md)). For
now, testing is local on Linux and macOS
([ADR 0842](docs/decisions/0842-mac-first-gate.md)).

## Documentation

- [Documentation index](docs/README.md)
- [Vision and philosophy](docs/vision.md)
- [Design decisions](docs/decisions/README.md)
- [Language card](docs/language-card.md): the whole language in one short read
- [Language specification](docs/spec/README.md)
- [Open questions](docs/open-questions.md)
- [Roadmap](docs/roadmap.md)
- [Feasibility: token efficiency and performance estimates](docs/design/feasibility.md)
- [Research: beating C on systems and allocation-heavy code](docs/design/performance-beyond-c.md)

AI agents: read [AGENTS.md](AGENTS.md) first.

## Community token

An AIL community token exists on pump.fun: [HFwDPcrXvYLNTyNbv4CQuqx3rH54f4AHNRn1n56Npump](https://pump.fun/coin/HFwDPcrXvYLNTyNbv4CQuqx3rH54f4AHNRn1n56Npump).

It is not an investment and gives no rights in the project. The address above is the only official one; treat any other as an impersonation.

## License

Apache License 2.0 with a Runtime Library Exception. See [LICENSE](LICENSE):
the toolchain, its sources and docs are Apache-2.0; code the toolchain
embeds into the programs you compile carries no attribution requirement
(ADR 0510).
