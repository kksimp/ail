# The AIL command-line tool (bootstrap edition) for Windows. From the repository root:
#
#   .\ail.ps1 run examples\hello.ail     check and run a program
#   .\ail.ps1 check FILE                 check without running (add --json for tools)
#   .\ail.ps1 selftest                   build and run the bootstrap's own tests
#
# If scripts are blocked: powershell -ExecutionPolicy Bypass -File .\ail.ps1 run examples\hello.ail
#
# Rust (ADR 0087): if the system Rust doesn't exactly match the version pinned in
# bootstrap\rust-toolchain.toml, that version is installed privately into
# .toolchain\ on first run. Nothing outside this repository is changed.
# Like ./ail (ADR 0855): run, build, check FILE, test FILE, fmt, fix, api, effects and pkg are done by the native CLI
# (compiler\main.ail built into .native\ail.exe). With no Rust binary built, the published seed is downloaded and
# verified (tools\release\fetch-seed.ps1) and builds it, so no cargo is needed. AIL_RUST=1 keeps the Rust `ail`.
# Status: tested under PowerShell 7 on Linux; not yet tested on Windows itself.
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
    & $init -y --no-modify-path --profile minimal --default-toolchain $pin --default-host $triple *> $log
    if ($LASTEXITCODE -ne 0) {
        Get-Content $log | ForEach-Object { [Console]::Error.WriteLine($_) }
        Fail 'Rust installation failed'
    }
    Remove-Item -Recurse -Force $tmp
    [Console]::Error.WriteLine("ail: Rust $pin installed")
}

# Package options for the native loader (mod.pkg.7-8): `run` and `test` write ail.lock, `--deterministic` refuses a stale one,
# `--deny net` fetches nothing.
if ($args.Count -gt 0 -and $args[0] -in 'run', 'test') { $env:AIL_PKG_WRITE_LOCK = '1' }
for ($i = 0; $i -lt $args.Count; $i++) {
    if ($args[$i] -eq '--deterministic') { $env:AIL_PKG_FROZEN = '1' }
    if ($i -gt 0 -and $args[$i - 1] -eq '--deny' -and "$($args[$i])" -like '*net*') { $env:AIL_PKG_OFFLINE = '1' }
}

# Which commands the native CLI does (the same conditions as ./ail).
$cmd = if ($args.Count -gt 0) { [string]$args[0] } else { '' }
$rest = @($args | Select-Object -Skip 1 | ForEach-Object { if ("$_" -and (Test-Path -LiteralPath "$_" -ErrorAction SilentlyContinue)) { Fwd "$_" } else { $_ } })
$nativeOn = ($env:AIL_RUST -ne '1') -and ($env:AIL_NATIVE_CLI -ne '0') -and (-not $env:AIL_BINARY -or $env:AIL_NATIVE_CLI -eq '1')
$isFile = { param($a) $a -and -not "$a".StartsWith('-') }
$runFile = $null; $runSkip = 0
if ($cmd -eq 'run' -and $nativeOn) {
    $k = 0
    while ($k -lt $rest.Count) {
        $a = [string]$rest[$k]
        if ($a -in '--timeout', '--max-memory') {
            if ($k + 1 -ge $rest.Count -or "$($rest[$k + 1])" -notmatch '^\d+$') { break }
            $k += 2
        } elseif ($a.StartsWith('-')) { break }
        else { $runFile = $a; $k++; break }
    }
    $runSkip = $k
}
$route = $nativeOn -and (
    $runFile -or $cmd -in 'build', 'pkg', 'fmt', 'fix' -or
    ($cmd -eq 'check' -and ((& $isFile $rest[0]) -or ($rest[0] -eq '--json' -and (& $isFile $rest[1])))) -or
    ($cmd -eq 'test' -and (& $isFile $rest[0])) -or
    ($cmd -in 'api', 'effects' -and (@($rest | Where-Object { & $isFile $_ }).Count -eq 1)))
if ($cmd -eq 'build' -and ($rest -contains '--derived-out')) { $route = $false }

# Pick cargo: the system's if it matches the pin, otherwise the private copy.
function Get-Cargo {
    $systemRust = $null
    if ((Get-Command cargo -ErrorAction SilentlyContinue) -and (Get-Command rustc -ErrorAction SilentlyContinue)) {
        $systemRust = ((& rustc --version) -split ' ')[1]
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

if ($route) {
    $nativeDir = Join-Path $root '.native'
    $native = Join-Path $nativeDir "ail$exe"
    $stale = -not (Test-Path $native)
    if (-not $stale) {
        $t = (Get-Item $native).LastWriteTimeUtc
        $newer = Get-ChildItem -Recurse -File -Filter *.ail -Path (Join-Path $root 'compiler'), (Join-Path $bootstrap 'src\std') -ErrorAction SilentlyContinue |
            Where-Object { $_.LastWriteTimeUtc -gt $t } | Select-Object -First 1
        $stale = [bool]$newer
    }
    if ($stale) {
        New-Item -ItemType Directory -Force -Path $nativeDir | Out-Null
        $log = "$native.log"
        $mainAil = Fwd (Join-Path $root 'compiler\main.ail')
        $rc = 1
        # A built Rust ail is used as it is; otherwise the verified seed, and cargo only when no seed is published for this host.
        if (-not (Test-Path $binary)) {
            & (Join-Path $PSScriptRoot 'tools\release\fetch-seed.ps1')
        }
        $seed = Join-Path $nativeDir "seed$exe"
        if ((Test-Path $binary) -or -not (Test-Path $seed)) {
            if (-not (Test-Path $binary)) {
                & (Get-Cargo) build --quiet --release --manifest-path $manifest
                if ($LASTEXITCODE -ne 0) { Fail 'no seed for this host and building the bootstrap failed; see ADR 0855' }
            }
            $env:AIL_RUST = '1'; & $binary build $mainAil -o "$native.new" 2> $log; $rc = $LASTEXITCODE; Remove-Item Env:AIL_RUST
        } else {
            & $seed build $mainAil -o "$native.new" 2> $log; $rc = $LASTEXITCODE
        }
        if ($rc -ne 0) { Get-Content $log | ForEach-Object { [Console]::Error.WriteLine($_) }; Fail "couldn't build the native CLI" }
        Remove-Item -Force $log -ErrorAction SilentlyContinue
        Move-Item -Force "$native.new" $native
    }
    $env:AIL_COMPILER = Fwd (Join-Path $root 'compiler')
    # `ail test` and tools that start `ail` themselves go through ail.cmd.
    if (-not $env:AIL) { $env:AIL = Fwd (Join-Path $root 'ail.cmd') }
    switch ($cmd) {
        { $_ -in 'pkg', 'fmt', 'fix', 'api', 'effects' } { & $native $cmd @rest; exit $LASTEXITCODE }
        'check' { & $native check-program @rest; exit $LASTEXITCODE }
        'test' { & $native run (Fwd (Join-Path $root 'tools\ailtest\ailtest.ail')) @rest; exit $LASTEXITCODE }
        'build' {
            # The native CLI's build doesn't check: check the file (the first operand) first.
            $f = $rest | Where-Object { (& $isFile $_) -and (Test-Path $_ -PathType Leaf) } | Select-Object -First 1
            if ($f) { & $native check-program $f; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE } }
            & $native build @rest; exit $LASTEXITCODE
        }
        'run' {
            & $native check-program $runFile
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            $runOpts = @($rest | Select-Object -First ($runSkip - 1))
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

$cargo = Get-Cargo

if ($args.Count -gt 0 -and $args[0] -eq 'selftest') {
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
