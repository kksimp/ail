# Builds the std.audio host for Windows (ADR 0949) and, with -Beep, plays a
# half-second 440 Hz tone through it. From the repository root:
#   tools\audio-host\windows\run.ps1          # build only
#   tools\audio-host\windows\run.ps1 -Beep    # build, then play the tone
# Needs only Windows: the C# compiler that ships with the .NET Framework
# (csc.exe), no SDK and no third-party packages.
param([switch]$Beep)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = (Resolve-Path (Join-Path $here '..\..\..')).Path
$exe = Join-Path $here 'build\ail-audio-host.exe'
$src = Join-Path $here 'Host.cs'

$csc = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
if (-not (Test-Path $csc)) { [Console]::Error.WriteLine('audio-host: csc.exe (.NET Framework 4) is missing'); exit 2 }

if (-not (Test-Path $exe) -or ((Get-Item $src).LastWriteTime -gt (Get-Item $exe).LastWriteTime)) {
    [Console]::Error.WriteLine("audio-host: building $exe")
    New-Item -ItemType Directory -Force (Split-Path $exe) | Out-Null
    & $csc /nologo /optimize /target:exe /platform:x64 "/out:$exe" $src
    if ($LASTEXITCODE -ne 0) { [Console]::Error.WriteLine('audio-host: the build failed'); exit 1 }
}

if ($Beep) {
    $env:AIL_AUDIO_HOST = $exe
    & (Join-Path $root 'ail.ps1') run (Join-Path $root 'tools\audio-host\beep.ail')
    exit $LASTEXITCODE
}
