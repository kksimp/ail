# The native gate on Windows in one command, printing only what failed (the PowerShell twin of mac-gate.sh, ADR 0842).
#   tools\gate\win-gate.ps1            from the repository root, on the branch under test (or: .\ail.ps1 selftest)
# Runs gate.ail's suites, which build and run everything with the native compiler, and prints one line per failing
# suite plus the first lines of its message. Silence means green: the last line is `GATE GREEN` or `GATE RED (N failures)`.
# Full logs stay in $env:TEMP\ail-gate\*.log. The suites are tools\gate\suites.txt, the list mac-gate.sh runs: this gate
# gates a release like the other two (ADR 0925). The suites need sh, perl and friends from Git for Windows.
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

# Every native binary must start.
$hello = Join-Path $out 'hello.exe'
& powershell -NoProfile -ExecutionPolicy Bypass -File $ail build tests/native/run/arith.ail -o $hello *> $null
if ($LASTEXITCODE -ne 0) { $fails++; 'RED native build of arith.ail' }
else {
    & $hello *> $null
    if ($LASTEXITCODE -ne 0) { $fails++; 'RED native binaries do not start' }
}
# The build above made .native\ail.exe if it was missing or older than the compiler sources. Nothing in the gate edits
# those sources, so the thousands of `ail` calls the suites make skip that scan of compiler\ and std\ (ail.ps1).
$env:AIL_NATIVE_FRESH = '1'
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
# The program suites (tools\gate\suites.txt), GATE_JOBS at a time (default: one per core, at most 4; GATE_JOBS=1 is one after another). Each suite
# is its own gate.ail run with its own TMPDIR and log (gate_NAME.log), so no two share a scratch tree. The first suite
# runs alone, so gate.ail is built once and the rest start from the build cache. nativex starts one process per core
# itself, so it runs alone after the others. times.log has the seconds each suite took, slowest first.
$suites = @(Get-Content (Join-Path $PSScriptRoot 'suites.txt') | ForEach-Object { $_.Trim() } | Where-Object { $_ -and -not $_.StartsWith('#') })
if ($args.Count -gt 0) { $suites = $args }
$jobs = if ("$env:GATE_JOBS" -match '^[1-9]\d*$') { [int]$env:GATE_JOBS } else { [Math]::Min($(if ("$env:GATE_JOBS_MAX" -match '^[1-9]\d*$') { [int]$env:GATE_JOBS_MAX } else { 4 }), [Math]::Max(1, [int]$env:NUMBER_OF_PROCESSORS)) }
$tmpBase = $env:TMPDIR.TrimEnd('/')
$times = @()

function Start-Suite([string]$name) {
    # `cases@1/4` is shard 1 of 4 of a suite (tools\gate\suites.txt): its files are named cases_1_4.
    $tag = $name -replace '[@/]', '_'
    $tmp = "$tmpBase/gate-$tag"
    if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue }
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    $log = Join-Path $out "gate_$tag.log"
    # The child takes the environment as it is now: its own TMPDIR, then ours back.
    $env:TMPDIR = $tmp
    # AIL_FAST_RUN=1 (`./ail`, under Git's sh): a plain `ail run FILE` skips the PowerShell start (about a second each, and the
    # cases and spec suites make thousands). `examples`, `cli`, `build_cache` and `docs` keep the full ail.ps1 path, so it
    # stays gated here; GATE_FAST_RUN=0 turns the fast path off for every suite.
    if ($env:GATE_FAST_RUN -ne '0' -and $name -notin 'examples', 'cli', 'build_cache', 'docs') { $env:AIL_FAST_RUN = '1' } else { Remove-Item Env:AIL_FAST_RUN -ErrorAction SilentlyContinue }
    $p = Start-Process -FilePath powershell -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$ail`" run tools/gate/gate.ail $name" -WorkingDirectory $root -NoNewWindow -PassThru -RedirectStandardOutput $log -RedirectStandardError "$log.err"
    $null = $p.Handle   # without this, ExitCode can read empty after the process ends
    $env:TMPDIR = $tmpBase
    Remove-Item Env:AIL_FAST_RUN -ErrorAction SilentlyContinue
    @{ Name = $name; Proc = $p; Log = $log; Tmp = $tmp; Start = Get-Date }
}

function Finish-Suite($r) {
    $r.Proc.WaitForExit()
    $rc = $r.Proc.ExitCode
    $script:times += [pscustomobject]@{ Suite = $r.Name; Seconds = [int]((Get-Date) - $r.Start).TotalSeconds }
    if (Test-Path "$($r.Log).err") {
        [IO.File]::AppendAllText($r.Log, [IO.File]::ReadAllText("$($r.Log).err"))
        Remove-Item -Force "$($r.Log).err" -ErrorAction SilentlyContinue
    }
    Remove-Item -Recurse -Force $r.Tmp -ErrorAction SilentlyContinue
    # The gate prints `RED name` lines for what failed: a log with one is red whatever the exit status says.
    $redLines = @(Select-String -Path $r.Log -Pattern '^RED ' -ErrorAction SilentlyContinue)
    if ($rc -eq 0 -and $redLines.Count -eq 0) { return }
    $script:fails++
    "RED $($r.Name)"
    # Every failing case of the suite in one pass (GATE_SHOW lines, default 60), so one round fixes them all.
    $show = if ("$env:GATE_SHOW" -match '^[1-9]\d*$') { [int]$env:GATE_SHOW } else { 60 }
    Select-String -Path $r.Log -Pattern $pattern | Select-Object -First $show | ForEach-Object { $_.Line.Substring(0, [Math]::Min(200, $_.Line.Length)) }
}

function Run-Pool([string[]]$names, [int]$width) {
    $queue = New-Object System.Collections.Queue
    foreach ($n in $names) { $queue.Enqueue($n) }
    $limit = 1
    $running = @()
    while ($queue.Count -gt 0 -or $running.Count -gt 0) {
        while ($queue.Count -gt 0 -and $running.Count -lt $limit) { $running += @(Start-Suite $queue.Dequeue()) }
        Start-Sleep -Milliseconds 500
        $still = @()
        foreach ($r in $running) {
            if ($r.Proc.HasExited) { Finish-Suite $r; $limit = $width } else { $still += @($r) }
        }
        $running = $still
    }
}

# nativex is sharded (`nativex@K/N`, one process each, suites.txt), so its shards share the pool with the other suites
# instead of one process per core running alone at the end. A bare `nativex` argument is still the whole suite, alone.
$alone = @($suites | Where-Object { $_ -eq 'nativex' })
Run-Pool @($suites | Where-Object { $_ -ne 'nativex' }) $jobs
if ($alone.Count -gt 0) { Run-Pool $alone 1 }
$times | Sort-Object Seconds -Descending | ForEach-Object { "$($_.Seconds)`t$($_.Suite)" } | Set-Content (Join-Path $out 'times.log')
# The shards of a suite must have run every item exactly once (all N shards, one total, ran counts add up, none empty).
if ($args.Count -eq 0 -and $fails -eq 0) {
    $sh = @(Get-ChildItem $out -Filter 'gate_*.log' | ForEach-Object { Select-String -Path $_.FullName -Pattern '^SHARD ' } | ForEach-Object { $f = $_.Line -split ' '; [pscustomobject]@{ Name = $f[1]; K = [int](($f[2] -split '/')[0]); N = [int](($f[2] -split '/')[1]); Total = [int]$f[4]; Ran = [int]$f[6] } })
    foreach ($g in ($sh | Group-Object Name)) {
        $n = $g.Group[0].N; $ks = @($g.Group | ForEach-Object { $_.K } | Sort-Object -Unique)
        $sum = ($g.Group | Measure-Object Ran -Sum).Sum
        $totals = @($g.Group | ForEach-Object { $_.Total } | Sort-Object -Unique)
        if ($g.Count -ne $n -or $ks.Count -ne $n -or $totals.Count -ne 1 -or $sum -ne $totals[0] -or @($g.Group | Where-Object { $_.Ran -eq 0 }).Count -gt 0) {
            $fails++; "RED shards of $($g.Name) do not cover it exactly once ($($g.Count) of $n shards, ran $sum of $($totals -join '/'))"
        }
    }
}
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'ps1-args-test.ps1') *> (Join-Path $out 'ps1_args.log')
if ($LASTEXITCODE -ne 0) { $fails++; 'RED ps1_args'; Get-Content (Join-Path $out 'ps1_args.log') | Select-Object -First 8 }
# The shipped commands with only git on PATH (ADR 1059).
$shipped = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'win-shipped-tools.ps1') 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) { $fails++; 'RED shipped tools'; $shipped }
if ($fails -eq 0) { 'GATE GREEN' } else { "GATE RED ($fails failures); logs in $out" }
exit ([int]($fails -gt 0))
