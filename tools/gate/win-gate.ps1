# The native gate on Windows in one command, printing only what failed (the PowerShell twin of mac-gate.sh, ADR 0842).
#   tools\gate\win-gate.ps1            from the repository root, on the branch under test (or: .\ail.ps1 selftest)
# Runs gate.ail's suites, which build and run everything with the native compiler, and prints one line per failing
# suite plus the first lines of its message. Silence means green: the last line is `GATE GREEN` or `GATE RED (N failures)`.
# Full logs stay in $env:TEMP\ail-gate\*.log. Windows is tracked, not gating: some suites need Unix tools (sh, tar,
# git behaviour) and fail here for that reason, which the failure list says by cause.
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location $root
# A std.app program would open the host's window and wait for ever: a host path that doesn't exist makes `app.run` fail at once.
if (-not $env:AIL_APP_HOST) { $env:AIL_APP_HOST = 'C:/nonexistent/ail-app-host' }
# The suites run `sh`, `perl`, `rm` and friends: Git for Windows has them in usr\bin. Put that on PATH, or stop here.
$gitUsr = Join-Path $env:ProgramFiles 'Git\usr\bin'
if ((Test-Path $gitUsr) -and (($env:PATH -split ';') -notcontains $gitUsr)) { $env:PATH = "$env:PATH;$gitUsr" }
$missing = @('sh', 'perl', 'rm', 'mktemp') | Where-Object { -not (Get-Command $_ -ErrorAction SilentlyContinue) }
if ($missing) { "GATE RED (missing tools on PATH: $($missing -join ' '); install Git for Windows or add its usr\bin)"; exit 1 }
# gate.ail builds into $TMPDIR (default /tmp) and Git's perl and sh start the programs from there: with TMPDIR unset, /tmp means
# a different directory to the native build and to Git's tools, and every example "ran" with no output. A Windows path with `/` is the same to both.
if (-not $env:TMPDIR) { $env:TMPDIR = $env:TEMP -replace '\\', '/' }
$out = Join-Path $env:TEMP 'ail-gate'
New-Item -ItemType Directory -Force -Path $out | Out-Null
$ail = Join-Path $root 'ail.ps1'
$fails = 0
$pattern = '^RED |^test .* FAILED|panicked at|differs from|^error|disagreement|FAIL '

function Run([string]$name, [string[]]$cmdArgs) {
    $log = Join-Path $out "$name.log"
    & powershell -NoProfile -ExecutionPolicy Bypass -File $ail @cmdArgs *> $log
    # The gate prints `RED name` lines for what failed: a log with one is red whatever the exit status says.
    $rc = $LASTEXITCODE
    $redLines = @(Select-String -Path $log -Pattern '^RED ' -ErrorAction SilentlyContinue)
    if ($rc -eq 0 -and $redLines.Count -eq 0) { return }
    $script:fails++
    "RED $name"
    Select-String -Path $log -Pattern $pattern | Select-Object -First 8 | ForEach-Object { $_.Line.Substring(0, [Math]::Min(200, $_.Line.Length)) }
}

# Self-host stage 1: the native compiler must build from compiler/main.ail. .native\ail.exe is that very build (the seed
# made it), so nativex reuses it (NATIVE_COMPILER) and there is no extra 25-minute build here.
$native = Join-Path $root '.native\ail.exe'
if (Test-Path $native) {
    # Forward slashes: gate.ail hands this path to `sh -c`, which eats backslashes.
    $env:NATIVE_COMPILER = $native -replace '\\', '/'
} else {
    Run 'selfhost1' @('build', 'compiler/main.ail', '-o', (Join-Path $out 'native-compiler.exe'))
    $env:NATIVE_COMPILER = (Join-Path $out 'native-compiler.exe') -replace '\\', '/'
}
# The program suites.
$suites = 'examples cases explain fix cli mistakes contracts misc sched docs fmt limits diff build_cache api_queries api_card edit git_deps net spec native_misc nativex globals pe asm snapshot budget replay cert try'.Split(' ')
if ($args.Count -gt 0) { $suites = $args }
Run 'gate_ail' (@('run', 'tools/gate/gate.ail') + $suites)
# Every native binary must start.
$hello = Join-Path $out 'hello.exe'
& powershell -NoProfile -ExecutionPolicy Bypass -File $ail build tests/native/run/arith.ail -o $hello *> $null
if ($LASTEXITCODE -ne 0) { $fails++; 'RED native build of arith.ail' }
else {
    & $hello *> $null
    if ($LASTEXITCODE -ne 0) { $fails++; 'RED native binaries do not start' }
}
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'ps1-args-test.ps1') *> (Join-Path $out 'ps1_args.log')
if ($LASTEXITCODE -ne 0) { $fails++; 'RED ps1_args'; Get-Content (Join-Path $out 'ps1_args.log') | Select-Object -First 8 }
if ($fails -eq 0) { 'GATE GREEN' } else { "GATE RED ($fails failures); logs in $out" }
exit ([int]($fails -gt 0))
