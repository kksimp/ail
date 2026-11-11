# Headless check of the Windows audio host's protocol (std.audio.9): it must
# say `ready`, reject an unknown request, and finish a play of no samples,
# none of which needs a sound device. Builds the host first. From the
# repository root: tools\audio-host\windows\smoke.ps1
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
& (Join-Path $here 'run.ps1')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$exe = Join-Path $here 'build\ail-audio-host.exe'
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $exe
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.UseShellExecute = $false
$p = [System.Diagnostics.Process]::Start($psi)
function Expect($want) {
    $got = $p.StandardOutput.ReadLine()
    if ($got -ne $want) { [Console]::Error.WriteLine("audio-host smoke: wanted '$want', got '$got'"); $p.Kill(); exit 1 }
}
Expect 'ready'
$p.StandardInput.WriteLine('bogus'); Expect 'error unknown request'
$p.StandardInput.WriteLine('play 44100 1'); Expect 'done'
$p.StandardInput.Close()
if (-not $p.WaitForExit(5000)) { $p.Kill(); [Console]::Error.WriteLine('audio-host smoke: the host did not exit'); exit 1 }
# Streaming (std.audio.12) on the silent clock: no sound card needed.
$psi.Arguments = '--null'
$p = [System.Diagnostics.Process]::Start($psi)
Expect 'ready'
$p.StandardInput.WriteLine('open_stream 8000 1'); Expect 'ok'
$p.StandardInput.WriteLine('write AAAAAAAAAAA='); $got = $p.StandardOutput.ReadLine()
if ($got -notmatch '^played \d+ buffered \d+$') { [Console]::Error.WriteLine("audio-host smoke: bad write reply '$got'"); $p.Kill(); exit 1 }
$p.StandardInput.WriteLine('flush'); Expect 'played 0 buffered 0'
$p.StandardInput.WriteLine('close_stream'); Expect 'ok'
$p.StandardInput.Close()
if (-not $p.WaitForExit(5000)) { $p.Kill(); [Console]::Error.WriteLine('audio-host smoke: the null host did not exit'); exit 1 }
Write-Output 'audio-host smoke: GREEN'
