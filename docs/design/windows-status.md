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
- Native `proc.run` and `proc.which` (ADR 0770): Linux only, by `clone` and `execve`. There is no Windows or macOS native arm yet (a macOS program traps `native.subset.1`), so there is no step for the designer here.
- TLS root certificates: the Windows ROOT store (`bootstrap/src/net/x509/roots.rs`).
- Random bytes: a Windows branch (`bootstrap/src/net/crypto/rand.rs`).
- Console text: Rust's standard library writes console output as UTF-16 (`WriteConsoleW`), so no change is needed.
- Line endings: the lexer and formatter accept CRLF (`bootstrap/src/lexer.rs`, `bootstrap/src/fmt.rs`).
- UI: Windows key and metric branches (`bootstrap/src/ui.rs`).
- Not yet on the batch branch, so not reviewed here: bytes, FFI loading, dates and zones, compression, regex, and server sockets (lanes 123–129). Each adds a step here when it lands.
- Math (`std.math` float methods, ADR 0592): the interpreter calls the platform libm through Rust's `f64` methods, so on Windows the last bit can differ from glibc, within the bound of `type.num.16`. The native code is AIL (fdlibm-derived), so it does not depend on the platform libm. Step 8 checks this.
- Tool output paths (ADR 0799): diagnostics, JSON and `ail api` spell every path with `/` on Windows, so expected output is the same as on Linux.
- Wine run of the bootstrap suite (Windows target `x86_64-pc-windows-gnu`, `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine`, Wine 9.0): 383 passed, 7 failed. Left: the editor and garlic tests (paths baked in from the Linux build host), and examples and cases that start `sh`, `tar` or `ail` as processes. C calls (`ffi.load.N`) and `ail build` for Windows print a skip with the reason.
- Native PE writer and Windows runtime (ADRs 0597, 0598, 0606–0608): `ail build --target x86_64-windows examples/hello.ail` builds a PE32+ file whose imports are kernel32 functions (`ExitProcess`, `GetStdHandle`, `WriteFile`, `VirtualAlloc`). Under Wine 9.0 it prints `examples/hello.out` and exits 0 (`bootstrap/tests/native_pe.rs`). Not yet verified on a real Windows host, and the file, time and thread arms are still missing, and `env.get` reports every variable as unset: under Wine, 8 of 67 native run programs differ from the interpreter (atomics, par, task, and the os_env_time and os_fs programs). No step here yet: a native Windows build is not in the designer's list.
- Build cache (ADR 0655): `ail build` has no Windows host target yet, so the cache is not reached; its directory falls back to `%LOCALAPPDATA%\ail\build` when `XDG_CACHE_HOME` is unset.
- Sockets, HTTP/1.1 keep-alive (ADRs 0637, 0639): the client pool and server keep-alive use only `std::net::TcpStream` and a non-blocking `peek`, which Rust implements with `MSG_PEEK` on Winsock; untested on Windows. Step to add when run: `.\ail.ps1 run bootstrap\tests\cases\std_net_keepalive.ail` should print `reused: true` first, and `std_net_jar.ail`, `std_net_timeouts.ail` should match their `.out` files.
- Sockets, HTTP/1.1 keep-alive and streaming bodies (ADRs 0637, 0639, 0716): the client pool and server keep-alive use only `std::net::TcpStream` and a non-blocking `peek`, which Rust implements with `MSG_PEEK` on Winsock; untested on Windows. Step to add when run: `.\ail.ps1 run bootstrap\tests\cases\std_net_keepalive.ail` should print `reused: true` first, and `std_net_jar.ail`, `std_net_timeouts.ail` should match their `.out` files.
- Sockets, HTTP/1.1 keep-alive and streaming bodies (ADRs 0637, 0639, 0716, 0717): the client pool and server keep-alive use only `std::net::TcpStream` and a non-blocking `peek`, which Rust implements with `MSG_PEEK` on Winsock; untested on Windows. Step to add when run: `.\ail.ps1 run bootstrap\tests\cases\std_net_keepalive.ail` should print `reused: true` first, and `std_net_jar.ail`, `std_net_timeouts.ail` should match their `.out` files.
- `http.serve_dir` and `req.stop()` (ADRs 0754–0755): AIL over `fs` and `net`, no platform code; untested on Windows. Step to add when run: `.\ail.ps1 run bootstrap\tests\cases\http_serve_dir.ail` should match its `.out` file.
- `fs.copy` on a directory (ADR 0751): Rust `std::fs::copy` and `read_dir` per file, untested on Windows. Step to add when run: `.\ail.ps1 run bootstrap\tests\cases\fs_copy_tree.ail` should match `fs_copy_tree.out` (it uses `%TEMP%`).
- Allocator (ADR 0737): `ail run` and `ail test` reserve one 16 GiB `MAP_NORESERVE` region on Unix and serve blocks up to 1 KiB from per-thread free lists; on Windows the reservation is skipped and the system allocator is used (untested there).
