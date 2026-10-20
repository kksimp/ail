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
# Status: tested under PowerShell 7 on Linux; not yet tested on Windows itself.
$ErrorActionPreference = 'Stop'

$root = $PSScriptRoot
$bootstrap = Join-Path $root 'bootstrap'
$manifest = Join-Path $bootstrap 'Cargo.toml'
$pin = (Select-String -Path (Join-Path $bootstrap 'rust-toolchain.toml') -Pattern '^channel\s*=\s*"(.*)"').Matches[0].Groups[1].Value
$toolDir = if ($env:AIL_TOOLCHAIN_DIR) { $env:AIL_TOOLCHAIN_DIR } else { Join-Path $root '.toolchain' }
# Windows PowerShell 5.1 doesn't define $IsLinux/$IsMacOS, so they read as false there.
$onWindows = -not ($IsLinux -or $IsMacOS)
$exe = if ($onWindows) { '.exe' } else { '' }
$binary = Join-Path (Join-Path (Join-Path $bootstrap 'target') 'release') "ail$exe"

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

# Pick cargo: the system's if it matches the pin, otherwise the private copy.
$systemRust = $null
if ((Get-Command cargo -ErrorAction SilentlyContinue) -and (Get-Command rustc -ErrorAction SilentlyContinue)) {
    $systemRust = ((& rustc --version) -split ' ')[1]
}
if ($systemRust -eq $pin) {
    $cargo = 'cargo'
} else {
    $env:RUSTUP_HOME = Join-Path $toolDir 'rustup'
    $env:CARGO_HOME = Join-Path $toolDir 'cargo'
    $env:RUSTUP_TOOLCHAIN = $pin
    $cargoBin = Join-Path $env:CARGO_HOME 'bin'
    $cargo = Join-Path $cargoBin "cargo$exe"
    if (-not (Test-Path $cargo)) {
        Install-PrivateRust
    } else {
        $installed = & (Join-Path $cargoBin "rustup$exe") toolchain list 2>$null
        if (-not ($installed | Where-Object { $_ -like "$pin*" })) {
            & (Join-Path $cargoBin "rustup$exe") toolchain install --profile minimal $pin
            if ($LASTEXITCODE -ne 0) { Fail "couldn't install Rust $pin" }
        }
    }
}

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
