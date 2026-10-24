# Checks the Windows host's protocol with no AIL toolchain (ADR 0888): starts
# the host, sends one view, expects the first line back to be {"kind":"start","v":1},
# presses nothing, then closes stdin and expects the host to exit.
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools\app-host\windows\smoke.ps1
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$exe = Join-Path $here 'build\AILAppHost.exe'
if (-not (Test-Path $exe)) { Write-Output 'smoke: build the host first (run.ps1)'; exit 2 }
$psi = New-Object System.Diagnostics.ProcessStartInfo $exe
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$p = [System.Diagnostics.Process]::Start($psi)
$view = '{"v":1,"t":"view","title":"smoke","focus":"","tree":{"kind":"Column","kids":[{"kind":"Label","text":"hi"},{"kind":"Field","id":"f","value":"x","hint":"h"},{"kind":"Button","id":"b","caption":"Go"},{"kind":"Canvas","id":"c","w":0,"h":0,"ops":[{"kind":"Fill","x":0,"y":0,"w":20,"h":20,"color":16711680},{"kind":"Str","x":4,"y":4,"text":"A","font":"mono","size":13,"fg":0}]}]}}'
$p.StandardInput.WriteLine($view)
$p.StandardInput.Flush()
$t = $p.StandardOutput.ReadLineAsync()
if (-not $t.Wait(10000)) { Write-Output 'smoke: RED no line in 10 s'; $p.Kill(); exit 1 }
Write-Output "first line: $($t.Result)"
# Every event for 3 s: the canvas's resize events show its final size.
$deadline = [DateTime]::Now.AddSeconds(3)
while ([DateTime]::Now -lt $deadline) {
    $u = $p.StandardOutput.ReadLineAsync()
    if (-not $u.Wait(500)) { continue }
    if ($null -eq $u.Result) { break }
    Write-Output "next line: $($u.Result)"
}
$p.StandardInput.Close()
if (-not $p.WaitForExit(10000)) { Write-Output 'smoke: RED host did not exit'; $p.Kill(); exit 1 }
if ($t.Result -like '*"kind":"start"*' -and $t.Result -like '*"v":1*') { Write-Output 'smoke: GREEN'; exit 0 }
Write-Output 'smoke: RED unexpected first line'; exit 1
