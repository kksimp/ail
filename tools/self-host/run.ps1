# The self-host fixed point on Windows (the PowerShell twin of run.sh's stages 1 to 3; ADR 0022, ADR 0925).
#   tools\self-host\run.ps1 [OUT_DIR]      from the repository root, on the tree under test
# Stage 1: `.\ail.ps1 build compiler\main.ail` (the seed-hosted native CLI builds the compiler).
# Stage 2: stage 1 builds compiler\main.ail. Stage 3: stage 2 builds it again; stage 2 and stage 3 must be the same bytes.
# The native test programs (run.sh's stage T) are the gate's nativex suite here, which runs them with this tree's compiler.
# Prints one line per stage; the last line is `OK: ...` or `BLOCKED at STAGE: WHY`. Exit 0 only for OK; 2 on a setup error.
# Writes only under OUT_DIR (default: a new directory under $env:TEMP); the stage executables stay there.
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location $root
$ail = Join-Path $root 'ail.ps1'
if (-not (Test-Path $ail)) { [Console]::Error.WriteLine('run.ps1: no ail.ps1 launcher'); exit 2 }
$out = if ($args.Count -gt 0) { [string]$args[0] } else { Join-Path $env:TEMP "self-host-$PID" }
New-Item -ItemType Directory -Force -Path $out | Out-Null
$out = (Resolve-Path $out).Path
$fwd = { param($p) "$p" -replace '\\', '/' }
$target = 'x86_64-windows'
function Blocked([string]$stage, [string]$why) { ''; "BLOCKED at ${stage}: $why"; exit 1 }
function Size([string]$p) { (Get-Item $p).Length }

"# Self-host fixed point on Windows ($target)"
'## Stage 1: .\ail.ps1 build compiler\main.ail'
$stage1 = Join-Path $out 'stage1.exe'
& powershell -NoProfile -ExecutionPolicy Bypass -File $ail build compiler/main.ail -o (& $fwd $stage1) --target $target *> (Join-Path $out 'stage1.log')
if ($LASTEXITCODE -ne 0 -or -not (Test-Path $stage1)) {
    Get-Content (Join-Path $out 'stage1.log') | Select-Object -First 8
    Blocked 'stage 1' "the seed-hosted build of compiler\main.ail failed (log: $out\stage1.log)"
}
"built $stage1 ($(Size $stage1) bytes)"

# A stage compiler finds the runtime and std through these, as ail.ps1 sets them. Both later stages get the same
# command line but for the output name, and --no-cache, so each one really compiles.
$env:AIL_ROOT = & $fwd $root
$env:AIL_COMPILER = & $fwd (Join-Path $root 'compiler')
function Build-With([string]$compiler, [string]$name) {
    $dst = Join-Path $out "$name.exe"
    & $compiler build (& $fwd (Join-Path $root 'compiler\main.ail')) -o (& $fwd $dst) --target $target --no-cache *> (Join-Path $out "$name.log")
    return ($LASTEXITCODE -eq 0 -and (Test-Path $dst))
}
function Same([string]$a, [string]$b) { (Get-FileHash -Algorithm SHA256 $a).Hash -eq (Get-FileHash -Algorithm SHA256 $b).Hash }

'## Stage 2: stage 1 builds compiler\main.ail'
if (-not (Build-With $stage1 'stage2')) {
    Get-Content (Join-Path $out 'stage2.log') | Select-Object -First 8
    Blocked 'stage 2' "the stage-1 compiler can't build compiler\main.ail (log: $out\stage2.log)"
}
$stage2 = Join-Path $out 'stage2.exe'
"built $stage2 ($(Size $stage2) bytes)"
# The paths on the command line end up in the executable, and ail.ps1's differ from ours: stage 2 == stage 3 decides.
if (Same $stage1 $stage2) { 'stage 1 == stage 2 (byte-identical)' } else { 'stage 1 != stage 2 (a note, not an error: stage 2 == stage 3 decides)' }

'## Stage 3: stage 2 builds compiler\main.ail (the fixed point)'
if (-not (Build-With $stage2 'stage3')) {
    Get-Content (Join-Path $out 'stage3.log') | Select-Object -First 8
    Blocked 'stage 3' "stage 2 can't build compiler\main.ail (log: $out\stage3.log)"
}
$stage3 = Join-Path $out 'stage3.exe'
if (-not (Same $stage2 $stage3)) {
    "stage 2 != stage 3 ($(Size $stage2) and $(Size $stage3) bytes)"
    Blocked 'stage 3' "no fixed point (compare $stage2 and $stage3)"
}
"stage 2 == stage 3 (byte-identical, $(Size $stage2) bytes): fixed point reached"
''
'OK: the native compiler builds itself to a fixed point on Windows.'
exit 0
