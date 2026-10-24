# Self-host bootstrap harness

`tools/self-host/run.sh [OUT_DIR]` (ADR 0610) answers "can the native compiler
build itself, and is it right?" and, until it can, "what is the first
blocker?". It needs only a Linux or macOS shell, `perl` (hex to bytes) and
the repository's `./ail`.

| Stage | What it does | Passes when |
|---|---|---|
| 0 | Lists the `fs`, `io` and `env` calls in `compiler/` outside `rt/` | informational: each is marked supported or not natively |
| 1 | `./ail build compiler/main.ail` (Rust-hosted) | it builds; else the problem count and the top constructs |
| 2 | stage 1 runs `build compiler/main.ail compiler/rt/native` and the hex it prints becomes the stage-2 executable | it builds, and stage 1 == stage 2 byte for byte |
| 3 | stage 2 builds the compiler again | stage 2 == stage 3 byte for byte (the fixed point) |
| T | every `tests/native/run/*.ail` is built by the stage-2 compiler, run, and compared with `ail run` | same stdout, exit code and first trap line |

A blocked stage prints `BLOCKED at <stage>: <why>` and exits 1; the report is
`OUT_DIR/report.md`. Environment: `STAGE1=PATH` supplies a stage-1 compiler
(a wrapper running `ail run compiler "$@"` stands in for it), `TESTS_ONLY=1`
runs only stage T with it, `ONLY=a,b` picks tests, `TARGET=T` picks the
target (only the host's can run). Delete `OUT_DIR` afterwards: each stage
executable is several MB.

## State on 2026-10-20

Blocked at stage 1: `./ail build compiler/main.ail` reports about 1,100
problems (constructs per `tools/self-compile/run.sh`, ranked in
[self-compile-status.md](self-compile-status.md)). Stage 0 shows what the
stage-1 compiler could not do even once it builds: read its own source
(`fs.read_text`, `fs.list_dir`) and report errors (`io.eprint`, `io.exit`);
only `env.args` is native. Those four need runtime functions and lowering
(lane 177's OS layer is the owner). With the stand-in compiler, stage T
passes, which checks the harness itself.
