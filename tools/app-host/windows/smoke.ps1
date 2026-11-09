# Checks the Windows host's protocol with no AIL toolchain (ADR 0888): starts
# the host, sends one view, a junk line and a measure request, and expects the first line to carry "v":1, a start event before any resize, and a measured reply,
# then closes stdin and expects the host to exit.
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
# Lines that aren't views are skipped (std.app.8), and a measure request is answered (std.app.13).
$p.StandardInput.WriteLine('not json')
$p.StandardInput.WriteLine('{"v":1,"t":"measure","runs":[{"text":"hello","font":"mono","size":13,"bold":false,"italic":false},{"text":"hi","font":"sans","size":13,"bold":true,"italic":false}]}')
$p.StandardInput.Flush()
# Events for up to 15 s (a loaded machine is slow to show the first line), ending once `start` and `measured` are in; keep the pending read, a second one would fail.
$lines = @()
$u = $null
$deadline = [DateTime]::Now.AddSeconds(15)
while ([DateTime]::Now -lt $deadline) {
    if ($null -eq $u) { $u = $p.StandardOutput.ReadLineAsync() }
    if (-not $u.Wait(500)) { continue }
    if ($null -eq $u.Result) { break }
    Write-Output "line: $($u.Result)"
    $lines += $u.Result
    $u = $null
    if (($lines -like '*"kind":"start"*') -and ($lines -like '*"kind":"measured"*')) { break }
}
$p.StandardInput.Close()
if (-not $p.WaitForExit(10000)) { Write-Output 'smoke: RED host did not exit'; $p.Kill(); exit 1 }
if ($lines.Count -eq 0 -or $lines[0] -notlike '*"v":1*') { Write-Output 'smoke: RED the first line lacks "v":1'; exit 1 }
$start = ($lines | Where-Object { $_ -like '*"kind":"start"*' } | Select-Object -First 1)
$firstResize = ($lines | Where-Object { $_ -like '*"kind":"resize"*' } | Select-Object -First 1)
if (-not $start) { Write-Output 'smoke: RED no start event'; exit 1 }
if ($firstResize -and [array]::IndexOf($lines, $start) -gt [array]::IndexOf($lines, $firstResize)) { Write-Output 'smoke: RED resize came before start'; exit 1 }
$measured = ($lines | Where-Object { $_ -like '*"kind":"measured"*' } | Select-Object -First 1)
if ($measured -notmatch '"text":"[0-9.]+,[0-9.]+"') { Write-Output "smoke: RED measured reply missing or wrong: $measured"; exit 1 }
# Edge mode (std.app.25, std.app.33): `start` carries the window controls' insets, the controls at the right.
$p2 = [System.Diagnostics.Process]::Start($psi)
$edge = '{"v":1,"t":"view","title":"edge","focus":"","edge":true,"tree":{"kind":"Canvas","id":"c","w":0,"h":0,"ops":[{"kind":"Fill","x":0,"y":0,"w":20,"h":20,"color":2105376}]}}'
$p2.StandardInput.WriteLine($edge)
$p2.StandardInput.Flush()
$e1 = $p2.StandardOutput.ReadLineAsync()
$got = if ($e1.Wait(15000)) { $e1.Result } else { '' }
Write-Output "edge line: $got"
$p2.StandardInput.Close()
if (-not $p2.WaitForExit(10000)) { $p2.Kill() }
if ($got -notmatch '"kind":"start"' -or $got -notmatch '"inset_right":138' -or $got -notmatch '"inset_top":32') { Write-Output 'smoke: RED edge mode start lacks inset_right 138 and inset_top 32'; exit 1 }
Write-Output 'smoke: GREEN'
exit 0
