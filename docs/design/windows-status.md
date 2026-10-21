# Windows status checklist

Run these top to bottom in PowerShell 7 or Windows PowerShell 5.1, from the
repository root. Each line is one command, and each step says what to expect.
A step that fails stays on the list: note the output and the ADR it cites.
Windows is a first-class target (AGENTS.md, settled constraints). The Linux
selftest is the merge gate, so this list is for the designer's check on Windows.

1. `powershell -ExecutionPolicy Bypass -File .\ail.ps1 selftest` — expect the first run to install Rust privately into `.toolchain\` (ADR 0087), then `test result: ok` lines and no `FAILED`.
2. `.\ail.ps1 run examples\hello.ail` — expect the text in `examples\hello.out`.
3. `.\ail.ps1 run bootstrap\tests\cases\std_path.ail` — expect `bootstrap\tests\cases\std_path.out`. Covers `std.path` separators (ADR 0504).
4. `.\ail.ps1 run bootstrap\tests\cases\std_proc_which.ail` — expect `true` and a `not_found` ProcError. Covers process lookup (ADR 0505).
5. `.\ail.ps1 run bootstrap\tests\cases\std_proc.ail` — not expected to pass on Windows: the case calls `echo` and `sh`, which Windows lacks. Expected failure, POSIX-only (ADR 0505).
6. `.\ail.ps1 check compiler` — expect no errors (the self-hosted checker, ADR 0380).
7. `.\ail.ps1 run examples\fetch.ail` — expect the summary lines. With no network it prints the reason on standard error and summarises the copy in `examples\data`, so the output matches (ADR 0283, ADR 0369).
8. Math, read with step 1: the spec's `type.num.16` examples run in `selftest`. Expect no `FAILED` line for them. If one differs in its last digit, record the line and the output: the spec allows one unit in the last place from the platform's libm (ADR 0592, `type.num.16`), so that is the known bound, not a bug. Any larger difference is a failure.
9. Night modules that are not on the batch head yet get a step when they land: `std.csv`, `std.args`, `std.log` (#238, ADRs 0626–0628), HTTP/1.1 keep-alive and cookies (#194), native stack traces (#235: the frame walk on Windows x64 uses its own unwind convention), native tensors with AVX2 (#233). Until then the Windows arm is unreviewed, and the step is not written, because it has no expected output yet.

## Platform code reviewed (batch 2 status)

- Paths and drives: `std.path` and glob handle `C:\` (ADR 0504).
- Process lookup: `std.proc.run` tries `PATHEXT` for bare names (ADR 0505).
- TLS root certificates: the Windows ROOT store (`bootstrap/src/net/x509/roots.rs`).
- Random bytes: a Windows branch (`bootstrap/src/net/crypto/rand.rs`).
- Console text: Rust's standard library writes console output as UTF-16 (`WriteConsoleW`), so no change is needed.
- Line endings: the lexer and formatter accept CRLF (`bootstrap/src/lexer.rs`, `bootstrap/src/fmt.rs`).
- UI: Windows key and metric branches (`bootstrap/src/ui.rs`).
- Not yet on the batch branch, so not reviewed here: bytes, FFI loading, dates and zones, compression, regex, and server sockets (lanes 123–129). Each adds a step here when it lands.
- Math (`std.math` float methods, ADR 0592): the interpreter calls the platform libm through Rust's `f64` methods, so on Windows the last bit can differ from glibc, within the bound of `type.num.16`. The native code is AIL (fdlibm-derived), so it does not depend on the platform libm. Step 8 checks this.
- Native PE writer (ADRs 0597–0598): not runnable on Windows yet. `ail build --target x86_64-windows` is not wired, so there is no step for the designer here.
- Build cache (ADR 0655): `ail build` has no Windows host target yet, so the cache is not reached; its directory falls back to `%LOCALAPPDATA%\ail\build` when `XDG_CACHE_HOME` is unset.
