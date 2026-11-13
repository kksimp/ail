# The FFI ABI check on Windows (ADR 1074; tools/ffi-abi/run.sh): builds abi.c
# as ailabi.dll with clang or gcc when one is installed, builds abi.ail with
# this checkout's compiler, runs it from the DLL's folder and compares with abi.out.
$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
Set-Location $root
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("ffi-abi-" + [Guid]::NewGuid())
New-Item -ItemType Directory $tmp | Out-Null
try {
    $cc = (Get-Command clang, gcc -ErrorAction SilentlyContinue | Select-Object -First 1)
    if (-not $cc) { Write-Output 'ffi-abi: SKIPPED (no clang or gcc)'; exit 0 }
    & $cc.Source -O2 -shared tools/ffi-abi/abi.c -o (Join-Path $tmp 'ailabi.dll')
    if ($LASTEXITCODE -ne 0) { throw 'the C library did not build' }
    & powershell -NoProfile -ExecutionPolicy Bypass -File .\ail.ps1 build tools/ffi-abi/abi.ail -o (Join-Path $tmp 'abi.exe')
    if ($LASTEXITCODE -ne 0) { throw 'abi.ail did not build' }
    Push-Location $tmp
    $out = & .\abi.exe 2>&1 | Out-String
    Pop-Location
    $want = Get-Content -Raw tools/ffi-abi/abi.out
    if ($out.Replace("`r`n", "`n") -eq $want.Replace("`r`n", "`n")) { Write-Output 'ffi-abi: GREEN' }
    else { Write-Output $out; Write-Output 'ffi-abi: RED'; exit 1 }
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
