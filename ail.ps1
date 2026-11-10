# The AIL command-line tool for Windows. From the repository root:
#
#   .\ail.ps1 run examples\hello.ail     check and run a program
#   .\ail.ps1 check FILE                 check without running (add --json for tools)
#   .\ail.ps1 selftest                   the native gate (tools\gate\gate.ail's suites)
#
# If scripts are blocked: powershell -ExecutionPolicy Bypass -File .\ail.ps1 run examples\hello.ail
#
# Like ./ail (ADR 0855): the native CLI (compiler\main.ail built into .native\ail.exe) does run, build, check,
# test FILE, fmt, fix, diff, explain, edit, ask, api, effects and pkg. The published seed (stage 0), downloaded and
# verified by tools\release\fetch-seed.ps1, builds it. Not yet native here: `run --budget`.
$ErrorActionPreference = 'Stop'
# A native program's exit status is data (ail run passes it on), never a PowerShell error (PowerShell 7.4+).
$PSNativeCommandUseErrorActionPreference = $false

$root = $PSScriptRoot
# The native CLI splits paths on `/`; Windows takes `/` too, so hand it paths that way.
function Fwd([string]$p) { $p -replace '\\', '/' }
# std.app.10: a program finds the app host built under the toolchain.
$env:AIL_ROOT = (Fwd $root)
# Windows PowerShell 5.1 doesn't define $IsLinux/$IsMacOS, so they read as false there.
$onWindows = -not ($IsLinux -or $IsMacOS)
$exe = if ($onWindows) { '.exe' } else { '' }
function Fail([string]$message) {
    [Console]::Error.WriteLine("ail: $message")
    exit 2
}

# Windows PowerShell 5.1 (and PowerShell before 7.3) joins native arguments into a command line without escaping
# embedded quotes: `d"e` loses its quote and merges with the arguments after it. Escape them for the Windows
# command-line rules ourselves; 7.3+ already does that, so there the arguments go through untouched.
$legacyArgs = $PSVersionTable.PSVersion -lt [version]'7.3'
function Esc-Native($list) {
    foreach ($a in @($list)) {
        $t = [string]$a
        if (-not $legacyArgs) { $t; continue }
        if ($t -eq '') { '""'; continue }
        if ($t -match '[ \t]') {
            # Quote it ourselves. PowerShell 5.1 wraps an argument when it sees whitespace outside quotes, counting every `"`
            # (even a backslash-escaped one) as a toggle, so `"c\"d e"` would be wrapped again and split. Inside a quoted
            # argument the Windows rules read `""` as one literal quote and stay inside, which the count also leaves balanced.
            $t = '"' + ($t -replace '(\\*)"', '$1$1""' -replace '(\\+)$', '$1$1') + '"'
        }
        elseif ($t.Contains('"')) { $t = $t -replace '(\\*)"', '$1$1\"' }
        $t
    }
}

$cmd = if ($args.Count -gt 0) { [string]$args[0] } else { '' }
# A path that exists is handed on with forward slashes (see Fwd).
$rest = @($args | Select-Object -Skip 1 | ForEach-Object { if ("$_" -and (Test-Path -LiteralPath "$_" -ErrorAction SilentlyContinue)) { Fwd "$_" } else { $_ } })
function Add-Instrument([string]$what) {
    $env:AIL_INSTRUMENT = if ($env:AIL_INSTRUMENT) { "$($env:AIL_INSTRUMENT),$what" } else { $what }
}

# conc.sched.1: `--schedule=fixed|random[:SEED]` of `run` and `test` becomes AIL_SCHEDULE, as ./ail does.
if ($cmd -in 'run', 'test') {
    $kept = @()
    foreach ($a in $rest) {
        if ("$a" -like '--schedule=*') {
            $w = "$a".Substring(11)
            if ($w -eq 'fixed') { $env:AIL_SCHEDULE = 'fixed' }
            elseif ($w -eq 'random') {
                $w = 'random:' + ([int64]([DateTimeOffset]::UtcNow.ToUnixTimeSeconds() % 1000000))
                [Console]::Error.WriteLine("schedule seed $($w.Substring(7)): replay with --schedule=$w")
                $env:AIL_SCHEDULE = $w
            }
            elseif ($w -match '^random:\d+$') { $env:AIL_SCHEDULE = $w }
            else { Fail "``$a`` isn't a schedule; use ``--schedule=random`` or ``--schedule=random:SEED`` with a non-negative SEED (conc.sched.1)" }
        } else { $kept += $a }
    }
    $rest = $kept
}

# run.limit.4: the limit options belong to `run` and `test`; `check` with one is a usage error.
if ($cmd -eq 'check') {
    foreach ($a in $rest) {
        if ("$a" -in '--timeout', '--max-memory', '--max-steps') { Fail "error[run.limit.4]: ``$a`` works with ``ail run`` and ``ail test`` (run.limit.4)" }
    }
}

# Tracing and recording belong to `run` and `test` (trace.run.1, replay.rec.1).
if ($cmd -notin 'run', 'test') {
    foreach ($a in $rest) {
        if ("$a" -in '--trace', '--trace-limit', '--watch', '--verbose', '--at', '--show' -or "$a" -like '--trace=*' -or "$a" -like '--watch=*') {
            [Console]::Error.WriteLine("error[trace.run.1]: the tracing options (``$a``) work with ``ail run`` and ``ail test`` (trace.run.1)"); exit 2
        }
        if ("$a" -in '--record', '--replay') {
            [Console]::Error.WriteLine("error[replay.rec.1]: ``$a`` works with ``ail run`` and ``ail test`` and a file name (replay.rec.1)"); exit 2
        }
    }
}

# Package options for the native loader (mod.pkg.7-8): `run` and `test` write ail.lock, `--deterministic` refuses a stale one,
# `--deny net` fetches nothing.
if ($cmd -in 'run', 'test') { $env:AIL_PKG_WRITE_LOCK = '1' }
for ($i = 0; $i -lt $args.Count; $i++) {
    if ($args[$i] -eq '--deterministic') { $env:AIL_PKG_FROZEN = '1' }
    if ($i -gt 0 -and $args[$i - 1] -eq '--deny' -and "$($args[$i])" -like '*net*') { $env:AIL_PKG_OFFLINE = '1' }
}

# `run`'s options before the file (the same set as ./ail).
$isFile = { param($a) $a -and -not "$a".StartsWith('-') }
$runFile = $null; $runSkip = 0; $runOpts = @(); $checkOpts = @(); $traceRun = $false; $jsonOpt = $false; $snapJson = $false; $recRun = $false; $repRun = $false; $replayArgs = $null; $probeAt = $null; $probeShow = $null
if ($cmd -eq 'run') {
    $k = 0
    :scan while ($k -lt $rest.Count) {
        $a = [string]$rest[$k]
        $v = if ($k + 1 -lt $rest.Count) { [string]$rest[$k + 1] } else { '' }
        switch -Regex ($a) {
            '^(--timeout|--max-memory)$' {
                if ($v -notmatch '^[1-9]\d*$') { Fail "error[run.limit.4]: ``$a`` needs a positive whole number (run.limit.4)" }
                $runOpts += @($a, $v); $k += 2; continue
            }
            '^--json$' { $jsonOpt = $true; $k++; continue }
            '^--(record|replay)$' {
                if (-not $v) { Fail "``$a`` needs a recording file, e.g. ``$a r.json`` (replay.rec.1)" }
                if ($a -eq '--record') { $env:AIL_RECORD = Fwd $v; $recRun = $true } else { $env:AIL_REPLAY = Fwd $v; $repRun = $true }
                Add-Instrument 'record'; $k += 2; continue
            }
            '^--(budget|from)$' { Fail "``$a`` is not native on Windows yet (run it on Linux or macOS)" }
            '^--max-steps$' {
                if ($v -notmatch '^[1-9]\d*$') { Fail '`--max-steps` needs a positive number of steps (run.limit.4)' }
                $env:AIL_MAX_STEPS = $v; Add-Instrument 'steps'; $k += 2; continue
            }
            '^--watch$' {
                if ($v -notmatch '^[A-Za-z0-9_]+$') { Fail '`--watch` needs a variable name, e.g. `--watch total` (trace.watch.1)' }
                Add-Instrument "watch=$v"; $traceRun = $true; $k += 2; continue
            }
            '^--at$' {
                if ($v -notmatch '^.+:[1-9]\d*$') { Fail 'error[probe.run.4]: `--at` needs FILE:LINE, e.g. `--at main.ail:12` (probe.run.4)' }
                $probeAt = $v; $k += 2; continue
            }
            '^--show$' {
                if ($v -notmatch '^[A-Za-z0-9_,]+$') { Fail 'error[probe.run.4]: `--show` needs variable names, e.g. `--show i,total` (probe.run.4)' }
                $probeShow = $v; $k += 2; continue
            }
            '^--no-contracts$' { Add-Instrument 'nocontracts'; $k++; continue }
            '^--trace(=all)?$' { Add-Instrument 'trace,traceeffects'; $traceRun = $true; $k++; continue }
            '^--trace=calls$' { Add-Instrument 'trace'; $traceRun = $true; $k++; continue }
            '^--trace=effects$' { Add-Instrument 'traceeffects'; $traceRun = $true; $k++; continue }
            '^--trace=' { Fail "``$a`` isn't a trace kind: use ``--trace``, ``--trace=calls``, ``--trace=effects`` or ``--trace=all`` (trace.run.1)" }
            '^--trace-limit$' {
                if ($v -notmatch '^[1-9]\d*$') { Fail '`--trace-limit` needs a positive number of events (trace.run.4)' }
                $env:AIL_TRACE_LIMIT = $v; $k += 2; continue
            }
            '^--attach-input$' { $env:AIL_ATTACH_INPUT = '1'; $k++; continue }
            '^--verbose$' { Add-Instrument 'values,verbose'; $k++; continue }
            '^--values$' { Add-Instrument 'values'; $k++; continue }
            '^--deterministic$' { $env:AIL_DETERMINISTIC = '1'; $checkOpts += '--deterministic'; $k++; continue }
            '^--deny$' {
                if (-not $v) { Fail '`--deny` needs a list of capabilities, e.g. `--deny fs,proc`' }
                $checkOpts += @('--deny', $v); $k += 2; continue
            }
            '^--parallel$' { $env:AIL_PARALLEL = '1'; $k++; continue }
            '^--unsafe$' {
                if ($v -ne 'driver') { break scan }
                $checkOpts += @('--unsafe', 'driver'); $k += 2; continue
            }
            '^--seed$' {
                if ($v -notmatch '^\d+$') { Fail '`--seed` needs an integer, e.g. `--seed 42` (std.random.3)' }
                $env:AIL_SEED = $v; $k += 2; continue
            }
            '^(-.*)?$' { break scan }
            default { $runFile = $a; $k++; break scan }
        }
    }
    $runSkip = $k
    # Record and replay (ADR 0873, replay.rec.1-2, replay.run.1): the recording's first line says what ran.
    if ($repRun -and -not $recRun) {
        $recPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($env:AIL_REPLAY)
        if (-not (Test-Path -LiteralPath $recPath -PathType Leaf)) { Fail "can't read the recording $($env:AIL_REPLAY) (replay.run.1)" }
        $head = $null
        try { $head = ([IO.File]::ReadAllLines($recPath) | Select-Object -First 1) | ConvertFrom-Json } catch { }
        if (-not $head -or -not $head.program) { Fail "$($env:AIL_REPLAY) is not a recording (replay.run.1)" }
        if ($null -ne $head.seed) { $env:AIL_SEED = [string]$head.seed }
        if (-not $runFile) { $runFile = [string]$head.program }
        $replayArgs = @($head.args | ForEach-Object { [string]$_ })
    }
    elseif ($recRun -and $runFile) {
        if (-not $env:AIL_SEED) { $env:AIL_SEED = [string]([int64]([DateTimeOffset]::UtcNow.ToUnixTimeSeconds()) * 1000003 + $PID) }
        $quote = { param($t) '"' + ("$t" -replace '\\', '\\' -replace '"', '\"') + '"' }
        $recArgs = @($rest | Select-Object -Skip $runSkip | ForEach-Object { & $quote $_ }) -join ','
        $recPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($env:AIL_RECORD)
        $recHead = '{"ail_record":1,"program":' + (& $quote $runFile) + ',"args":[' + $recArgs + '],"seed":' + $env:AIL_SEED + ',"limit":67108864}' + "`n"
        try { [IO.File]::WriteAllText($recPath, $recHead, (New-Object Text.UTF8Encoding($false))) } catch { Fail "can't write the recording $($env:AIL_RECORD)" }
    }
    # probe.run.1: `--at FILE:LINE --show a,b` builds the probe in; each needs the other.
    if ($probeAt -or $probeShow) {
        if (-not ($probeAt -and $probeShow)) { Fail 'error[probe.run.4]: `--at` and `--show` go together, e.g. `--at main.ail:12 --show i,total` (probe.run.4)' }
        Add-Instrument ('probe=' + $probeAt + ':' + ($probeShow -replace ',', '|'))
        $traceRun = $true
    }
    # trace.run.1: with a trace, `--json` makes the events JSON objects; trace.snap.4: alone, it makes a trap one JSON object.
    if ($jsonOpt) {
        if ($traceRun) { $env:AIL_TRACE_JSON = '1'; Add-Instrument 'tracejson' }
        elseif ($runFile) { $snapJson = $true; Add-Instrument 'values,verbose' }
    }
}

# `check --errors` and `--budget`/`--from` anywhere in `check` are native too, as in ./ail.
$checkErrors = ($cmd -eq 'check') -and ($rest -contains '--errors')
$checkBudget = ($cmd -eq 'check') -and (@($rest | Where-Object { "$_" -in '--budget', '--from' }).Count -gt 0)

$replayShrink = ($cmd -eq 'replay') -and ($rest.Count -gt 0) -and ($rest[0] -eq '--shrink')
$native1 = $cmd -in 'build', 'pkg', 'fmt', 'fix', 'diff', 'explain', 'edit', 'ask', 'attach', 'api', 'new', 'verify', 'infer', 'task', 'mcp', 'leak', 'header'
$route = (
    $runFile -or $native1 -or $replayShrink -or $checkErrors -or $checkBudget -or
    ($cmd -eq 'check' -and ($rest.Count -gt 0) -and (
        ($rest[0] -in '--deny', '--deterministic', '--unsafe', '--budget', '--from') -or
        ($rest -contains '--fix') -or
        (& $isFile $rest[0]) -or ($rest[0] -eq '--json' -and (& $isFile $rest[1])))) -or
    ($cmd -eq 'test' -and $rest.Count -gt 0) -or
    ($cmd -eq 'effects' -and (@($rest | Where-Object { & $isFile $_ }).Count -eq 1)))
if ($cmd -eq 'build' -and ($rest -contains '--derived-out')) { $route = $false }

if ($route) {
    $rawRest = $rest
    $rest = @(Esc-Native $rest)
    $nativeDir = Join-Path $root '.native'
    $native = Join-Path $nativeDir "ail$exe"
    $stale = -not (Test-Path $native)
    if (-not $stale) {
        $t = (Get-Item $native).LastWriteTimeUtc
        $newer = Get-ChildItem -Recurse -File -Filter *.ail -Path (Join-Path $root 'compiler'), (Join-Path $root 'std') -ErrorAction SilentlyContinue |
            Where-Object { $_.LastWriteTimeUtc -gt $t } | Select-Object -First 1
        $stale = [bool]$newer
    }
    if ($stale) {
        # The published seed is stage 0: it builds the native CLI (ADR 0855).
        New-Item -ItemType Directory -Force -Path $nativeDir | Out-Null
        $seed = Join-Path $nativeDir "seed$exe"
        $tagFile = Join-Path $nativeDir 'seed.tag'
        $wantTag = (Get-Content (Join-Path $root 'tools\release\SEED_TAG') -TotalCount 1 -ErrorAction SilentlyContinue)
        $haveTag = if (Test-Path $tagFile) { (Get-Content $tagFile -TotalCount 1) } else { '' }
        if (-not (Test-Path $seed) -or $haveTag -ne $wantTag) {
            & (Join-Path $PSScriptRoot 'tools\release\fetch-seed.ps1')
            if ($LASTEXITCODE -eq 0 -and (Test-Path $seed)) {
                Set-Content -NoNewline -Encoding ascii $tagFile $wantTag
            } else {
                Remove-Item -Force $seed -ErrorAction SilentlyContinue
            }
        }
        if (-not (Test-Path $seed)) {
            Fail 'no seed for this host: check the network and rerun (tools\release\fetch-seed.ps1; ADR 0855)'
        }
        $log = "$native.log"
        $mainAil = Fwd (Join-Path $root 'compiler\main.ail')
        & $seed build $mainAil -o "$native.new" 2> $log
        if ($LASTEXITCODE -ne 0) { Get-Content $log | ForEach-Object { [Console]::Error.WriteLine($_) }; Fail "the seed couldn't build the native CLI" }
        Remove-Item -Force $log -ErrorAction SilentlyContinue
        Move-Item -Force "$native.new" $native
    }
    $env:AIL_COMPILER = Fwd (Join-Path $root 'compiler')
    # `ail test` and tools that start `ail` themselves go through ail.cmd.
    if (-not $env:AIL) { $env:AIL = Fwd (Join-Path $root 'ail.cmd') }
    switch ($cmd) {
        { $_ -in 'pkg', 'fmt', 'fix', 'api', 'effects', 'header', 'diff', 'explain', 'edit', 'ask', 'attach', 'new' } { & $native $cmd @rest; exit $LASTEXITCODE }
        'check' {
            if ($checkErrors) { & $native check-errors @rest; exit $LASTEXITCODE }
            if ($checkBudget) {
                # `ail check --budget [N] [--from K] [--json] FILE`: the native check under the budget (budget.out.1-7).
                $bopts = @(); $copts = @(); $prev = ''
                foreach ($a in $rest) {
                    if ($prev -eq '--budget' -and "$a" -match '^\d+$') { $bopts += $a; $prev = ''; continue }
                    if ($prev -eq '--from') { $bopts += $a; $prev = ''; continue }
                    $prev = ''
                    if ($a -in '--budget', '--from') { $bopts += $a; $prev = $a }
                    elseif ($a -eq '--json') { $bopts += $a; $copts += $a }
                    else { $copts += $a }
                }
                & $native budget check @bopts -- $native check-program @copts; exit $LASTEXITCODE
            }
            & $native check-program @rest; exit $LASTEXITCODE
        }
        'verify' { $env:AIL_NATIVE_CLI = ''; & $native run (Fwd (Join-Path $root 'tools\verify\verify.ail')) @rest; exit $LASTEXITCODE }
        'infer' { $env:AIL_NATIVE_CLI = ''; & $native run (Fwd (Join-Path $root 'tools\infer\infer.ail')) @rest; exit $LASTEXITCODE }
        'task' { $env:AIL_NATIVE_CLI = ''; & $native run (Fwd (Join-Path $root 'tools\ailtask\ailtask.ail')) @rest; exit $LASTEXITCODE }
        'mcp' { $env:AIL_NATIVE_CLI = ''; & $native run (Fwd (Join-Path $root 'tools\ailmcp\ailmcp.ail')) @rest; exit $LASTEXITCODE }
        'lsp' { $env:AIL_NATIVE_CLI = ''; & $native run (Fwd (Join-Path $root 'tools\ailsp\ailsp.ail')) @rest; exit $LASTEXITCODE }
        'sanitize' { $env:AIL_NATIVE_CLI = ''; & $native run (Fwd (Join-Path $root 'tools\ailsan\ailsan.ail')) @rest; exit $LASTEXITCODE }
        'replay' { $env:AIL_NATIVE_CLI = ''; & $native run (Fwd (Join-Path $root 'tools\shrink\shrink.ail')) @rest; exit $LASTEXITCODE }
        'test' {
            foreach ($a in $rest) {
                if ("$a" -like '--watch*') { [Console]::Error.WriteLine("ail test: $a is not native yet"); exit 2 }
                if ("$a" -eq '--native') { Fail '`ail test --native` is retired: `ail test` runs every test natively and there is no interpreter to compare with (native.test.1, ADR 0900)' }
            }
            $env:AIL_NATIVE_CLI = ''; & $native run (Fwd (Join-Path $root 'tools\ailtest\ailtest.ail')) @rest; exit $LASTEXITCODE }
        'leak' {
            # ADR 0969: build FILE, run it with AIL_LEAK set (san.leak.3); the exit status is the program's.
            $json = $false; $all = $false; $ops = @($rest)
            while ($ops.Count -gt 0 -and ("$($ops[0])" -in '--json', '--all')) {
                if ("$($ops[0])" -eq '--json') { $json = $true } else { $all = $true }
                $ops = @($ops | Select-Object -Skip 1)
            }
            if ($ops.Count -lt 1) { Fail 'usage: ail leak [--json] [--all] FILE [ARGS...]' }
            $mode = if ($all -and $json) { 'all,json' } elseif ($json) { 'json' } elseif ($all) { 'all' } else { '1' }
            $leakExe = Join-Path ([IO.Path]::GetTempPath()) ("ail-leak-" + [Guid]::NewGuid().ToString('N') + '.exe')
            & $native check-program $ops[0]
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            & $native build $ops[0] -o (Fwd $leakExe)
            if ($LASTEXITCODE -ne 0) { exit 2 }
            $env:AIL_LEAK = $mode
            # The report is on standard error: under 'Stop' PowerShell turns that into a NativeCommandError and exits 1.
            $prevPref = $ErrorActionPreference
            $ErrorActionPreference = 'Continue'
            & $leakExe @($ops | Select-Object -Skip 1)
            $leakRc = $LASTEXITCODE
            $ErrorActionPreference = $prevPref
            Remove-Item Env:AIL_LEAK -ErrorAction SilentlyContinue
            Remove-Item -Force $leakExe -ErrorAction SilentlyContinue
            exit $leakRc
        }
        'build' {
            # The native CLI's build doesn't check: check the file (the first operand) first.
            $f = $rest | Where-Object { (& $isFile $_) -and (Test-Path $_ -PathType Leaf) } | Select-Object -First 1
            $unsafe = @()
            for ($j = 0; $j -lt $rest.Count - 1; $j++) { if ($rest[$j] -eq '--unsafe' -and $rest[$j + 1] -eq 'driver') { $unsafe = @('--unsafe', 'driver') } }
            if ($f) { & $native check-program @unsafe $f; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE } }
            & $native build @rest; exit $LASTEXITCODE
        }
        'run' {
            # The program's run settings (run.limit, GC stats, leak report) are for the program, not for the
            # compiler: hide them from check-program and prepare, and put them back before the program starts.
            $runEnv = @{}
            foreach ($v in 'AIL_GC_STATS', 'AIL_GC_VERIFY', 'AIL_MAX_MEMORY_MB', 'AIL_TIMEOUT_MS', 'AIL_LEAK') {
                $val = [Environment]::GetEnvironmentVariable($v)
                if ($null -ne $val) { $runEnv[$v] = $val; Remove-Item "Env:$v" }
            }
            & $native check-program @checkOpts $runFile
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            $errFile = New-TemporaryFile
            # Start-Process with file redirection: PowerShell never wraps the child's stderr in error records,
            # so nothing on it can stop the script under 'Stop' (a `2>` redirection does, even with 'Continue').
            $outFile = New-TemporaryFile
            $q = { param($t) '"' + ("$t" -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"' }
            $prepArgs = (@('prepare') + @($runOpts) + @($runFile) | ForEach-Object { & $q $_ }) -join ' '
            $pp = Start-Process -FilePath $native -ArgumentList $prepArgs -NoNewWindow -PassThru -RedirectStandardOutput $outFile.FullName -RedirectStandardError $errFile.FullName
            $null = $pp.Handle
            $pp.WaitForExit()
            $prepRc = $pp.ExitCode
            $prep = @(Get-Content $outFile); Remove-Item -Force $outFile
            foreach ($k in $runEnv.Keys) { Set-Item "Env:$k" $runEnv[$k] }
            if ($prepRc -eq 0) {
                Remove-Item -Force $errFile
                # `env.program()` is the source path as given (std.env.4): ./ail sets argv[0], here an environment variable.
                $env:AIL_RUN_FILE = $runFile
                $progExe = [string]($prep | Select-Object -Last 1)
                $progArgsRaw = @(if ($null -ne $replayArgs) { $replayArgs } else { $rawRest | Select-Object -Skip $runSkip })
                $progArgs = @(Esc-Native $progArgsRaw)
                if ($snapJson) {
                    # tools/trapjson.awk turns the text trap report on stderr into one JSON object (trace.snap.4); awk comes with Git.
                    # Files and raw bytes, so no line ending is changed on the way.
                    $winQuote = { param($t) '"' + ("$t" -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"' }
                    $errTmp = [IO.Path]::GetTempFileName(); $outTmp = [IO.Path]::GetTempFileName()
                    try {
                        $startArgs = @{ FilePath = $progExe; NoNewWindow = $true; PassThru = $true; RedirectStandardError = $errTmp }
                        if ($progArgsRaw.Count -gt 0) { $startArgs.ArgumentList = (@($progArgsRaw | ForEach-Object { & $winQuote $_ }) -join ' ') }
                        $proc = Start-Process @startArgs
                        $null = $proc.Handle   # without this, ExitCode can read empty after WaitForExit
                        $proc.WaitForExit()
                        $rc = $proc.ExitCode
                        $report = $errTmp
                        $awk = Get-Command awk -ErrorAction SilentlyContinue
                        if ($awk) {
                            $awkArgs = '-v ' + (& $winQuote "main=$runFile") + ' -f ' + (& $winQuote (Join-Path $root 'tools\trapjson.awk'))
                            $a = Start-Process -FilePath $awk.Source -ArgumentList $awkArgs -NoNewWindow -PassThru -RedirectStandardInput $errTmp -RedirectStandardOutput $outTmp
                            $null = $a.Handle
                            $a.WaitForExit()
                            if ($a.ExitCode -eq 0) { $report = $outTmp }
                        }
                        $bytes = [IO.File]::ReadAllBytes($report)
                        $stderr = [Console]::OpenStandardError(); $stderr.Write($bytes, 0, $bytes.Length); $stderr.Flush()
                    } finally {
                        Remove-Item Env:AIL_RUN_FILE -ErrorAction SilentlyContinue
                        Remove-Item -Force $errTmp, $outTmp -ErrorAction SilentlyContinue
                    }
                    exit $rc
                }
                try { & $progExe @progArgs }
                finally { Remove-Item Env:AIL_RUN_FILE -ErrorAction SilentlyContinue }
                exit $LASTEXITCODE
            }
            [Console]::Error.Write((Get-Content $errFile -Raw)); Remove-Item -Force $errFile
            exit 1
        }
    }
}

# `ail selftest` with no arguments is the native gate (ADR 0865): tools\gate\win-gate.ps1 runs gate.ail's suites with the
# native compiler. `selftest SUITE...` runs those gate suites, `selftest --fuzz N [SEED]` the AIL fuzzers.
if ($cmd -eq 'selftest') {
    $rest = @(Esc-Native $rest)
    if ($rest.Count -eq 0) { & (Join-Path $root 'tools\gate\win-gate.ps1'); exit $LASTEXITCODE }
    if ($rest[0] -eq '--test') { Fail '`ail selftest --test NAME` ran one cargo test and is retired: name a gate suite, as `ail selftest cases`, or run `ail test FILE` (ADR 0900)' }
    if ($rest[0] -eq '--fuzz') {
        if ($rest.Count -lt 2) { Fail 'usage: ail selftest --fuzz N [SEED]' }
        $n = [string]$rest[1]; $seedNo = if ($rest.Count -gt 2 -and "$($rest[2])" -match '^\d+$') { [string]$rest[2] } else { '1' }
        [Console]::Error.WriteLine("fuzz: $n inputs per fuzzer from seed $seedNo")
        & (Join-Path $root 'ail.ps1') run (Fwd (Join-Path $root 'tools\fuzz\suite.ail')) $n $seedNo
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        & (Join-Path $root 'ail.ps1') run (Fwd (Join-Path $root 'tools\fuzz\modules.ail')) $n $seedNo
        exit $LASTEXITCODE
    }
    # Gate suites by name (tools\gate\gate.ail), as the gate runs them.
    & (Join-Path $root 'ail.ps1') run (Fwd (Join-Path $root 'tools\gate\gate.ail')) @rest
    exit $LASTEXITCODE
}

# `ail version` / `ail --version`: the line ./ail prints.
if ($cmd -eq 'version' -or $cmd -eq '--version') {
    $verFile = Join-Path $root 'tools\release\VERSION'
    $ver = '0.0.0'
    if (Test-Path -LiteralPath $verFile) { $first = Get-Content -LiteralPath $verFile -TotalCount 1; if ($first) { $ver = "$first".Trim() } }
    $desc = 'unknown'
    try {
        $prevEap = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
        $d = & git -C $root describe --tags --always --dirty 2>$null
        if ($LASTEXITCODE -eq 0 -and $d) { $desc = "$d".Trim() }
    } catch { } finally { $ErrorActionPreference = $prevEap }
    [Console]::Out.WriteLine("ail $ver ($desc, native)")
    exit 0
}

# Everything else has no native form.
Fail "``ail $cmd`` has no native form; run ``.\ail.ps1`` with a command from docs/quick-reference.md"
