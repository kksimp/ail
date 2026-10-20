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

## Platform code reviewed (batch 2 status)

- Paths and drives: `std.path` and glob handle `C:\` (ADR 0504).
- Process lookup: `std.proc.run` tries `PATHEXT` for bare names (ADR 0505).
- TLS root certificates: the Windows ROOT store (`bootstrap/src/net/x509/roots.rs`).
- Random bytes: a Windows branch (`bootstrap/src/net/crypto/rand.rs`).
- Console text: Rust's standard library writes console output as UTF-16 (`WriteConsoleW`), so no change is needed.
- Line endings: the lexer and formatter accept CRLF (`bootstrap/src/lexer.rs`, `bootstrap/src/fmt.rs`).
- UI: Windows key and metric branches (`bootstrap/src/ui.rs`).
- Not yet on the batch branch, so not reviewed here: bytes, FFI loading, dates and zones, compression, regex, and server sockets (lanes 123–129). Each adds a step here when it lands.
- Native PE writer and Windows runtime (ADRs 0597, 0598, 0606–0608): `ail build --target x86_64-windows examples/hello.ail` builds a PE32+ file whose imports are kernel32 functions (`ExitProcess`, `GetStdHandle`, `WriteFile`, `VirtualAlloc`). Under Wine 9.0 it prints `examples/hello.out` and exits 0 (`bootstrap/tests/native_pe.rs`). Not yet verified on a real Windows host, and the file, time, env and thread arms are still missing: under Wine, 8 of 67 native run programs differ from the interpreter (atomics, par, task, and the os_env_time and os_fs programs). No step here yet: a native Windows build is not in the designer's list.
