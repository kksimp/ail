# Editor baseline on the Rust bootstrap

The designer asked (2026-10-20) that once the shipped toolchain needs no
Rust (end of Stage 4, [ADR 0022](../decisions/0022-staged-self-hosting.md)),
the editor in `tools/edit` is rebuilt with the self-hosted toolchain and
compared with how it runs today. This note is the "today" half of that
comparison. Re-measure the same rows with the self-hosted toolchain and add
a column; the roadmap item points here.

## Measured 2026-10-20 (main at 3666941, after the bytecode tier, ADR 0359)

Linux x86-64, release bootstrap binary called directly, best of 5 runs,
`max RSS` is the process peak resident set.

| Measure | Value |
|---|---|
| `tools/edit/edit.ail` size | 5,249 lines; 53,259 tokens (tokbench mean of five tokenizers; cl100k 49,241) |
| `ail check tools/edit/edit.ail` | 41 ms, 36 MB |
| `ail api --ast tools/edit/edit.ail` | 44 ms, 36 MB |
| `ail test tools/edit/edit.ail` (55 headless tests) | 121 ms, 36 MB |
| `ail run examples/hello.ail` (process floor) | 3 ms, 36 MB |

macOS ARM64 (Kaleb's Mac), same binary built there:

| Measure | Value |
|---|---|
| Start-up to first frame (`ail run tools/edit`) | to be measured on the next Mac run |
| `ail check tools/edit/edit.ail` | to be measured on the next Mac run |

Keystroke and redraw latency are not measured yet: the editor is a headless
`Editor` value plus a pure render, so a timing test over `Command`s and
`render` is the right instrument once the self-hosted build exists; add it
as an `@test` that prints timings rather than a separate harness.

## How to re-measure

```
./ail build
python3 - <<'PY'
import subprocess, time, resource
B = "bootstrap/target/release/ail"   # or the self-hosted binary
for name, args in [("check", [B, "check", "tools/edit/edit.ail"]),
                   ("api --ast", [B, "api", "--ast", "tools/edit/edit.ail"]),
                   ("test", [B, "test", "tools/edit/edit.ail"])]:
    best = 9e9
    for _ in range(5):
        t = time.time(); subprocess.run(args, capture_output=True); best = min(best, time.time() - t)
    print(name, f"{best*1000:.0f} ms", f"{resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss/1024:.0f} MB")
PY
tools/tokbench/run.sh --json tools/edit/edit.ail
```
