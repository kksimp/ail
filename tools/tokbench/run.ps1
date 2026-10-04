# One-command token measurement for Windows (PowerShell).
#
#   powershell -ExecutionPolicy Bypass -File tools\tokbench\run.ps1             # every corpus file
#   powershell -ExecutionPolicy Bypass -File tools\tokbench\run.ps1 FILE ...    # specific files
#   powershell -ExecutionPolicy Bypass -File tools\tokbench\run.ps1 --claude    # add current-Claude counts
#   powershell -ExecutionPolicy Bypass -File tools\tokbench\run.ps1 --json      # machine-readable output
#
# The first run creates a private Python environment in tools\tokbench\.venv
# and downloads tokenizer files into tools\tokbench\.cache. Nothing is
# installed system-wide. Set $env:PYTHON to choose the interpreter.
$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot   # file arguments stay relative to the caller's directory

function Invoke-Checked {
    param([string]$Exe, [string[]]$Arguments)
    & $Exe @Arguments
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$python = $env:PYTHON
if (-not $python) {
    foreach ($candidate in 'py', 'python', 'python3') {
        if (Get-Command $candidate -ErrorAction SilentlyContinue) { $python = $candidate; break }
    }
}
if (-not $python) {
    Write-Error 'Python 3.10 or newer is required (https://www.python.org/downloads/)'
    exit 1
}
& $python -c 'import sys; sys.exit(sys.version_info < (3, 10))'
if ($LASTEXITCODE -ne 0) {
    Write-Error "$(& $python --version 2>&1) is too old; Python 3.10 or newer is required"
    exit 1
}

# Windows PowerShell 5.1 doesn't define $IsLinux/$IsMacOS, so they read as false there.
if ($IsLinux -or $IsMacOS) { $venvPython = Join-Path $here '.venv/bin/python' }
else { $venvPython = Join-Path $here '.venv\Scripts\python.exe' }
if (-not (Test-Path $venvPython)) {
    Write-Host 'setting up Python environment in tools\tokbench\.venv ...'
    Invoke-Checked $python @('-m', 'venv', (Join-Path $here '.venv'))
}
$stamp = Join-Path (Join-Path $here '.venv') '.installed'
$requirements = Join-Path $here 'requirements.txt'
if (-not (Test-Path $stamp) -or (Get-Item -Force $requirements).LastWriteTime -gt (Get-Item -Force $stamp).LastWriteTime) {
    Invoke-Checked $venvPython @('-m', 'pip', 'install', '--quiet', '--upgrade', 'pip')
    Invoke-Checked $venvPython @('-m', 'pip', 'install', '--quiet', '-r', $requirements)
    New-Item -ItemType File -Force $stamp | Out-Null
}
if ($args -contains '--claude') {
    & $venvPython -c 'import anthropic' 2>$null
    if ($LASTEXITCODE -ne 0) { Invoke-Checked $venvPython @('-m', 'pip', 'install', '--quiet', 'anthropic') }
}
if (-not (Test-Path (Join-Path (Join-Path $here '.cache') '.complete'))) {
    Invoke-Checked $venvPython @((Join-Path $here 'fetch_tokenizers.py'))
}

& $venvPython (Join-Path $here 'tokbench.py') @args
exit $LASTEXITCODE
