# The shipped commands with no Unix program on PATH (ADR 1059): `ail test`, `ail ask --affected`, `ail verify --diff`
# and `ail infer --contracts` must work with a PATH that holds only git, the system directory (tar.exe, cmd.exe) and
# PowerShell. Prints one line per failure; silence is green. Called by win-gate.ps1.
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$ail = Join-Path $root 'ail.ps1'
$gitCmd = Get-Command git.exe -ErrorAction SilentlyContinue
if (-not $gitCmd) { 'RED shipped tools: git.exe is not on PATH'; exit 1 }
$sys = Join-Path $env:SystemRoot 'System32'
$onlyGit = "$(Split-Path $gitCmd.Source);$sys;$sys\WindowsPowerShell\v1.0"
$work = Join-Path $env:TEMP "ail-shipped-$PID"
New-Item -ItemType Directory -Force -Path $work | Out-Null
$src = "fn pay(n int) int`n    n + 1`n@test fn pays()`n    for i in 1..30`n        assert_eq(pay(i), i + 1)`nfn main()`n    print(pay(1))`n"
[System.IO.File]::WriteAllText((Join-Path $work 'prog.ail'), $src)
$saved = $env:PATH
$fails = 0
try {
    Push-Location $work
    & git init -q . ; & git config user.email gate@example.com ; & git config user.name gate
    & git add prog.ail ; & git commit -q -m base
    $env:PATH = $onlyGit
    foreach ($tool in 'sh', 'mktemp', 'rm', 'cp', 'env') {
        if (Get-Command $tool -ErrorAction SilentlyContinue) { "RED shipped tools: $tool is on PATH, so this run proves nothing"; $fails++ }
    }
    $cases = @(
        @{ name = 'test'; args = @('test', 'prog.ail') },
        @{ name = 'ask --affected'; args = @('ask', 'prog.ail', '--affected', '--rev', 'HEAD') },
        @{ name = 'verify --diff'; args = @('verify', '--diff', '--rev', 'HEAD', 'prog.ail') },
        @{ name = 'infer --contracts'; args = @('infer', '--contracts', 'pay', 'prog.ail') }
    )
    foreach ($c in $cases) {
        $out = & powershell -NoProfile -ExecutionPolicy Bypass -File $ail @($c.args) 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) { "RED shipped tools: ail $($c.name) exit $LASTEXITCODE"; ($out -split "`n" | Select-Object -First 6); $fails++ }
    }
    Pop-Location
} finally {
    $env:PATH = $saved
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $work
}
if ($fails -gt 0) { exit 1 }
