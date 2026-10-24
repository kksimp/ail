# The AIL command-line tool for Windows. From the repository root:
#
#   .\ail.ps1 run examples\hello.ail     check and run a program
#   .\ail.ps1 check FILE                 check without running (add --json for tools)
#   .\ail.ps1 selftest                   the native gate (tools\gate\gate.ail's suites)
#
# If scripts are blocked: powershell -ExecutionPolicy Bypass -File .\ail.ps1 run examples\hello.ail
#
# Like ./ail (ADR 0855, D4): the native CLI (compiler\main.ail built into .native\ail.exe) does run, build, check,
# test FILE, fmt, fix, diff, explain, edit, ask, api, effects and pkg. The published seed (stage 0), downloaded and
# verified by tools\release\fetch-seed.ps1, builds it; there is no cargo fallback. AIL_RUST=1 (or a pinned AIL_BINARY)
# is the one way to the Rust `ail` and its interpreter, until the Rust is deleted: it needs the pinned Rust (ADR 0087),
# installed privately into .toolchain\ on first use. Not yet native here: `run --record/--replay/--budget`.
$ErrorActionPreference = 'Stop'

$root = $PSScriptRoot
# The native CLI splits paths on `/`; Windows takes `/` too, so hand it paths that way.
function Fwd([string]$p) { $p -replace '\\', '/' }
# std.app.10: a program finds the app host built under the toolchain.
$env:AIL_ROOT = (Fwd $root)
$bootstrap = Join-Path $root 'bootstrap'
$manifest = Join-Path $bootstrap 'Cargo.toml'
$pin = (Select-String -Path (Join-Path $bootstrap 'rust-toolchain.toml') -Pattern '^channel\s*=\s*"(.*)"').Matches[0].Groups[1].Value
$toolDir = if ($env:AIL_TOOLCHAIN_DIR) { $env:AIL_TOOLCHAIN_DIR } else { Join-Path $root '.toolchain' }
# Windows PowerShell 5.1 doesn't define $IsLinux/$IsMacOS, so they read as false there.
$onWindows = -not ($IsLinux -or $IsMacOS)
$exe = if ($onWindows) { '.exe' } else { '' }
$binary = Join-Path (Join-Path (Join-Path $bootstrap 'target') 'release') "ail$exe"

# A test run pins the binary it is testing (AIL_BINARY): a program that starts `ail` itself isn't rebuilt under it.
if ($args.Count -gt 0 -and $args[0] -ne 'selftest' -and $env:AIL_BINARY -and (Test-Path $env:AIL_BINARY)) {
    & $env:AIL_BINARY @args
    exit $LASTEXITCODE
}

function Fail([string]$message) {
    [Console]::Error.WriteLine("ail: $message")
    exit 2
}

function Get-HostTriple {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    if ($onWindows) {
        # The GNU toolchain links with its bundled tools, so Visual Studio isn't needed on x64.
        if ($arch -eq 'X64') { return 'x86_64-pc-windows-gnu' }
        if ($arch -eq 'Arm64') { return 'aarch64-pc-windows-msvc' }
    } elseif ($IsLinux) {
        if ($arch -eq 'X64') { return 'x86_64-unknown-linux-gnu' }
        if ($arch -eq 'Arm64') { return 'aarch64-unknown-linux-gnu' }
    } elseif ($IsMacOS -and $arch -eq 'Arm64') {
        return 'aarch64-apple-darwin'
    }
    Fail "unsupported host architecture: $arch"
}

function Install-PrivateRust {
    $triple = Get-HostTriple
    [Console]::Error.WriteLine("ail: installing Rust $pin privately into .toolchain (first run only; takes a few minutes) ...")
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("ail-rust-" + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $tmp | Out-Null
    $init = Join-Path $tmp "rustup-init$exe"
    $url = "https://static.rust-lang.org/rustup/dist/$triple/rustup-init$exe"
    try {
        Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $init
        Invoke-WebRequest -UseBasicParsing -Uri "$url.sha256" -OutFile "$init.sha256"
    } catch {
        Fail "couldn't download $url"
    }
    $expected = ((Get-Content "$init.sha256" -Raw).Trim() -split '\s+')[0].ToLower()
    $actual = (Get-FileHash -Algorithm SHA256 $init).Hash.ToLower()
    if ($expected -ne $actual) { Fail 'rustup-init checksum mismatch; not installing' }
    if (-not $onWindows) { & chmod +x $init }
    $log = Join-Path $tmp 'install.log'
    # Windows PowerShell 5.1 turns a native command's stderr into an error record, which `Stop` would take for a failure.
    $ErrorActionPreference = 'Continue'
    & $init -y --no-modify-path --profile minimal --default-toolchain $pin --default-host $triple *> $log
    $ErrorActionPreference = 'Stop'
    if ($LASTEXITCODE -ne 0) {
        Get-Content $log | ForEach-Object { [Console]::Error.WriteLine($_) }
        Fail 'Rust installation failed'
    }
    Remove-Item -Recurse -Force $tmp
    [Console]::Error.WriteLine("ail: Rust $pin installed")
}

# Pick cargo: the system's if it matches the pin, otherwise the private copy. Only the Rust paths (AIL_RUST=1,
# AIL_SELFTEST=cargo and the commands the native CLI doesn't do yet) come here.
function Get-Cargo {
    $systemRust = $null
    if ((Get-Command cargo -ErrorAction SilentlyContinue) -and (Get-Command rustc -ErrorAction SilentlyContinue)) {
        $systemRust = ((& rustc --version) -split ' ')[1]
    }
    # bootstrap/src/std is a symlink to ../../std; a Windows checkout without symlink rights makes it a text file,
    # which the Rust build cannot read through. A junction does the same job.
    $stdLink = Join-Path $bootstrap 'src\std'
    if ($onWindows -and (Test-Path $stdLink -PathType Leaf)) {
        Remove-Item -Force $stdLink
        New-Item -ItemType Junction -Path $stdLink -Target (Join-Path $root 'std') | Out-Null
        & git -C $root update-index --assume-unchanged -- bootstrap/src/std 2>$null
    }
    if ($systemRust -eq $pin) { return 'cargo' }
    $env:RUSTUP_HOME = Join-Path $toolDir 'rustup'
    $env:CARGO_HOME = Join-Path $toolDir 'cargo'
    $env:RUSTUP_TOOLCHAIN = $pin
    $cargoBin = Join-Path $env:CARGO_HOME 'bin'
    $c = Join-Path $cargoBin "cargo$exe"
    if (-not (Test-Path $c)) {
        Install-PrivateRust
    } else {
        $installed = & (Join-Path $cargoBin "rustup$exe") toolchain list 2>$null
        if (-not ($installed | Where-Object { $_ -like "$pin*" })) {
            & (Join-Path $cargoBin "rustup$exe") toolchain install --profile minimal $pin
            if ($LASTEXITCODE -ne 0) { Fail "couldn't install Rust $pin" }
        }
    }
    return $c
}

$cmd = if ($args.Count -gt 0) { [string]$args[0] } else { '' }
# A path that exists is handed on with forward slashes (see Fwd).
$rest = @($args | Select-Object -Skip 1 | ForEach-Object { if ("$_" -and (Test-Path -LiteralPath "$_" -ErrorAction SilentlyContinue)) { Fwd "$_" } else { $_ } })
$nativeOn = ($env:AIL_RUST -ne '1') -and (-not $env:AIL_BINARY)
function Add-Instrument([string]$what) {
    $env:AIL_INSTRUMENT = if ($env:AIL_INSTRUMENT) { "$($env:AIL_INSTRUMENT),$what" } else { $what }
}

# Tracing and recording belong to `run` and `test` (trace.run.1, replay.rec.1).
if ($cmd -notin 'run', 'test') {
    foreach ($a in $rest) {
        if ("$a" -in '--trace', '--trace-limit', '--watch', '--verbose' -or "$a" -like '--trace=*' -or "$a" -like '--watch=*') {
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
$runFile = $null; $runSkip = 0; $runOpts = @(); $checkOpts = @()
if ($cmd -eq 'run' -and $nativeOn) {
    $k = 0
    :scan while ($k -lt $rest.Count) {
        $a = [string]$rest[$k]
        $v = if ($k + 1 -lt $rest.Count) { [string]$rest[$k + 1] } else { '' }
        switch -Regex ($a) {
            '^(--timeout|--max-memory)$' {
                if ($v -notmatch '^\d+$') { break scan }
                $runOpts += @($a, $v); $k += 2; continue
            }
            '^--(budget|from|json|record|replay)$' { Fail "``$a`` is not native on Windows yet (run it on Linux or macOS)" }
            '^--max-steps$' {
                if ($v -notmatch '^[1-9]\d*$') { Fail '`--max-steps` needs a positive number of steps (run.limit.4)' }
                $env:AIL_MAX_STEPS = $v; Add-Instrument 'steps'; $k += 2; continue
            }
            '^--watch$' {
                if ($v -notmatch '^[A-Za-z0-9_]+$') { Fail '`--watch` needs a variable name, e.g. `--watch total` (trace.watch.1)' }
                Add-Instrument "watch=$v"; $k += 2; continue
            }
            '^--no-contracts$' { Add-Instrument 'nocontracts'; $k++; continue }
            '^--trace(=calls|=all)?$' { Add-Instrument 'trace'; $k++; continue }
            '^--trace=effects$' { Fail '`--trace=effects` is not native yet: effect events come with the next trace step (trace.run.6)' }
            '^--trace=' { Fail "``$a`` isn't a trace kind: use ``--trace``, ``--trace=calls``, ``--trace=effects`` or ``--trace=all`` (trace.run.1)" }
            '^--trace-limit$' {
                if ($v -notmatch '^[1-9]\d*$') { Fail '`--trace-limit` needs a positive number of events (trace.run.4)' }
                $env:AIL_TRACE_LIMIT = $v; $k += 2; continue
            }
            '^--verbose$' { $k++; continue }
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
}

$native1 = $nativeOn -and $cmd -in 'build', 'pkg', 'fmt', 'fix', 'diff', 'explain', 'edit', 'ask', 'api'
$route = $nativeOn -and (
    $runFile -or $native1 -or
    ($cmd -eq 'check' -and ($rest.Count -gt 0) -and (
        ($rest[0] -in '--deny', '--deterministic', '--unsafe', '--budget', '--from') -or
        (& $isFile $rest[0]) -or ($rest[0] -eq '--json' -and (& $isFile $rest[1])))) -or
    ($cmd -eq 'test' -and (& $isFile $rest[0])) -or
    ($cmd -eq 'effects' -and (@($rest | Where-Object { & $isFile $_ }).Count -eq 1)))
if ($cmd -eq 'build' -and ($rest -contains '--derived-out')) { $route = $false }

if ($route) {
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
        # The published seed is stage 0: it builds the native CLI (ADR 0855, D4); there is no cargo fallback.
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
            Fail 'no seed for this host: sign in with gh or set GITHUB_TOKEN, then rerun (tools\release\fetch-seed.ps1; ADR 0855)'
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
        { $_ -in 'pkg', 'fmt', 'fix', 'api', 'effects', 'diff', 'explain', 'edit', 'ask' } { & $native $cmd @rest; exit $LASTEXITCODE }
        'check' {
            if ($rest[0] -in '--budget', '--from') {
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
        'test' { $env:AIL_NATIVE_CLI = ''; & $native run (Fwd (Join-Path $root 'tools\ailtest\ailtest.ail')) @rest; exit $LASTEXITCODE }
        'build' {
            # The native CLI's build doesn't check: check the file (the first operand) first.
            $f = $rest | Where-Object { (& $isFile $_) -and (Test-Path $_ -PathType Leaf) } | Select-Object -First 1
            $unsafe = @()
            for ($j = 0; $j -lt $rest.Count - 1; $j++) { if ($rest[$j] -eq '--unsafe' -and $rest[$j + 1] -eq 'driver') { $unsafe = @('--unsafe', 'driver') } }
            if ($f) { & $native check-program @unsafe $f; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE } }
            & $native build @rest; exit $LASTEXITCODE
        }
        'run' {
            & $native check-program @checkOpts $runFile
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            $errFile = New-TemporaryFile
            $prep = & $native prepare @runOpts $runFile 2> $errFile
            if ($LASTEXITCODE -eq 0) {
                Remove-Item -Force $errFile
                & ([string]($prep | Select-Object -Last 1)) @($rest | Select-Object -Skip $runSkip)
                exit $LASTEXITCODE
            }
            [Console]::Error.Write((Get-Content $errFile -Raw)); Remove-Item -Force $errFile
            exit 1
        }
    }
}

# `ail selftest` with no arguments is the native gate (ADR 0865): tools\gate\win-gate.ps1 runs gate.ail's suites with the
# native compiler. `selftest SUITE...` runs those gate suites, `selftest --fuzz N [SEED]` the AIL fuzzers;
# `AIL_SELFTEST=cargo` keeps the cargo tests.
if ($cmd -eq 'selftest' -and $nativeOn -and $env:AIL_SELFTEST -ne 'cargo') {
    if ($rest.Count -eq 0) { & (Join-Path $root 'tools\gate\win-gate.ps1'); exit $LASTEXITCODE }
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

# What is left goes to the Rust `ail`, which is only for AIL_RUST=1 (or AIL_BINARY) and the commands not native yet.
$cargo = Get-Cargo

if ($cmd -eq 'selftest') {
    & $cargo test --quiet --release --manifest-path $manifest @($args | Select-Object -Skip 1)
    exit $LASTEXITCODE
}

if (-not (Test-Path $binary)) { [Console]::Error.WriteLine('ail: building the bootstrap (first run only) ...') }
& $cargo build --quiet --release --manifest-path $manifest
if ($LASTEXITCODE -ne 0) {
    if ($onWindows -and (Get-HostTriple) -eq 'aarch64-pc-windows-msvc') {
        Fail 'building failed; on Windows ARM64 the Visual Studio Build Tools (C++ workload) are required'
    }
    Fail 'building the bootstrap failed'
}
& $binary @args
exit $LASTEXITCODE
