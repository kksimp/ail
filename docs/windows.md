# AIL on Windows

Windows x86-64 is a first-class target: the compiler builds native `.exe`
programs with no C compiler, no linker and no runtime to install. This page is
what to know that the README's two commands don't say. Windows ARM64 has no
seed yet ([ADR 0968](decisions/0968-aarch64-windows-target.md)); the compiler
can emit ARM64 executables, but none has been run on hardware.

It targets Windows PowerShell 5.1 (the one that ships with Windows) and
PowerShell 7.

## What you need

- Windows PowerShell 5.1 or PowerShell 7, and `git`. Nothing needs
  administrator rights.
- Internet access on the first run, to download the seed.
- `ail test`, `ail diff`, `ail verify` and `ail infer` need only `git` (ADR 1059):
  they use no Unix program. Only the gate under `tools/gate/`, which is not a
  shipped command, needs `sh`, `mktemp` and `cp`, which come with
  [Git for Windows](https://git-scm.com/download/win) in
  `C:\Program Files\Git\usr\bin`.

## Install and first run

From a PowerShell prompt:

```powershell
git clone --depth 1 https://github.com/kksimp/ail
cd ail
.\ail.ps1 run examples\hello.ail
```

The first run does three things (`tools\release\fetch-seed.ps1`):

1. Downloads the seed, `ail-seed-x86_64-windows.exe`, from the GitHub release
   named in `tools\release\SEED_TAG`. No account is needed.
2. Checks its SHA-256 against `tools\release\seeds.sha256` and refuses it on a
   mismatch ([ADR 0976](decisions/0976-public-release-and-seed-trust.md)).
3. Builds the current compiler from source with the seed into `.native\ail.exe`.

Later runs rebuild only when a file under `compiler\` or `std\` is newer than
`.native\ail.exe`. Use the clone's own `ail.ps1`; it is not on `PATH`.
To run it from another directory, call it by path (`C:\src\ail\ail.ps1 run
hello.ail`) or put the clone's directory on `PATH` and use `ail.cmd`; `ail doctor --path` prints the PowerShell line
that does it for the session (add it to your `$PROFILE` to keep it).

## Execution policy

A fresh Windows PowerShell 5.1 blocks unsigned scripts, so
`.\ail.ps1` may stop with "running scripts is disabled on this system". Pick
one:

```powershell
powershell -ExecutionPolicy Bypass -File .\ail.ps1 run examples\hello.ail
.\ail.cmd run examples\hello.ail
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
```

The first two change nothing on the machine: `ail.cmd` is a one-line wrapper
that starts `ail.ps1` with `-ExecutionPolicy Bypass`. The third is permanent
for your user. Programs that start `ail` themselves (`ail test`, tools) go
through `ail.cmd`. If you downloaded a zip instead of cloning, run
`Get-ChildItem -Recurse | Unblock-File` first; a clone doesn't need it.

## Where things go

| What | Where |
|---|---|
| The seed, the native compiler, their logs | `.native\` inside the clone (`seed.exe`, `ail.exe`, `seed.tag`) |
| Built programs (cache) | `%LOCALAPPDATA%\ail\build`, or `AIL_CACHE_DIR` if set |
| Git dependencies ([packages](packages.md)) | `%USERPROFILE%\.ail\deps`, or `%AIL_HOME%\deps` |

Nothing else outside the clone is written. To remove AIL, delete the clone and
those two directories.

## Paths, quoting and line endings

- Paths take `\` or `/`. `ail.ps1` hands the compiler forward slashes, and
  `use` paths in source are always `/`-style modules, never file paths.
- Windows PowerShell 5.1 mangles quotes inside native-command arguments;
  `ail.ps1` escapes them for you, so `.\ail.ps1 run prog.ail 'say "hi"'`
  reaches `env.args()` intact.
- Source files in the repository are LF on every platform (`.gitattributes`),
  whatever `core.autocrlf` says. `ail fmt --check` compares bytes, so a file
  you save with CRLF fails it; configure your editor for LF.

## What differs from macOS and Linux

- `ail test --watch` is not available here yet; `ail.ps1` says so and exits with
  status 2. `--budget` and `--from` work as on macOS and Linux.
- `ail run --json` on a trap uses `awk` when it finds one (Git for Windows
  ships it) and otherwise prints the text report.
- GUI programs (`std.app`, Garlic) need an interactive desktop session; they
  can't start from a service or an SSH session.

## Testing the toolchain

The native gate is `.\ail.ps1 selftest` (the same as `tools\gate\win-gate.ps1`).
It needs the Git for Windows tools above on `PATH`, prints only failures, and
ends with `GATE GREEN` or `GATE RED`. It runs several suites at once, one per
core up to 4 (`GATE_JOBS=N` sets the number; 1 runs them one after another),
and writes each suite's seconds to `%TEMP%\ail-gate\times.log`. A red suite prints all its failing lines
(`GATE_SHOW=N`, default 60); `tools\gate\win-gate.ps1 cases@2/4 spec@1/4` reruns named suites or shards. Most suites run
`./ail run` through its fast path (`AIL_FAST_RUN=1`, no PowerShell start per program; `GATE_FAST_RUN=0` turns it off).
It runs the suites of
`tools/gate/suites.txt`, the list the Mac and Linux gate runs, and it must be
green on Windows x86-64 before every release, with the fixed point of
`tools\self-host\run.ps1`
([ADR 0925](decisions/0925-windows-release-gate.md)).

## Rejected alternatives

- **An installer or `winget` package:** a second thing to sign, publish and
  keep in step with the seed; the clone is the toolchain.
- **Requiring PowerShell 7:** most Windows machines have only 5.1, and
  `ail.ps1` works on both.
- **Requiring WSL:** would hide Windows-only faults instead of fixing them.
