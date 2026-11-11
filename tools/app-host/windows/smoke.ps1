# Checks the Windows host's protocol with no AIL toolchain (ADR 0888, ADR 1044): plays the
# shared conformance script tests/ui/host_conformance.txt (the `all` and `win` lines), then the
# Windows-only checks: a shared surface made as a named file mapping is drawn and answered
# with `vsync` (std.gfx.8), and the host exits when its input closes.
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools\app-host\windows\smoke.ps1
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = (Resolve-Path (Join-Path $here '..\..\..')).Path
$exe = Join-Path $here 'build\AILAppHost.exe'
if (-not (Test-Path $exe)) { Write-Output 'smoke: build the host first (run.ps1)'; exit 2 }
Add-Type -AssemblyName System.Core

function Start-Host {
    $psi = New-Object System.Diagnostics.ProcessStartInfo $exe
    $psi.UseShellExecute = $false
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    return [System.Diagnostics.Process]::Start($psi)
}

# Reads output lines into $lines until $done returns true or $ms passes; keeps one pending read.
function Read-Lines($p, $lines, $ms, [scriptblock]$done) {
    $deadline = [DateTime]::Now.AddMilliseconds($ms)
    while ([DateTime]::Now -lt $deadline) {
        if (& $done) { return }
        if ($null -eq $script:pending) { $script:pending = $p.StandardOutput.ReadLineAsync() }
        if (-not $script:pending.Wait(250)) { continue }
        if ($null -eq $script:pending.Result) { $script:pending = $null; return }
        Write-Output "line: $($script:pending.Result)"
        [void]$lines.Add($script:pending.Result)
        $script:pending = $null
    }
}

function Stop-Host($p) {
    $p.StandardInput.Close()
    if (-not $p.WaitForExit(10000)) { Write-Output 'smoke: RED host did not exit'; $p.Kill(); exit 1 }
}

# ---- the shared script ----
$script:pending = $null
$p = Start-Host
$expects = @(); $absents = @(); $firsts = @(); $orders = @()
foreach ($raw in Get-Content -Encoding UTF8 (Join-Path $root 'tests\ui\host_conformance.txt')) {
    if ($raw -eq '' -or $raw.StartsWith('#')) { continue }
    $sp = $raw.IndexOf(' '); $plats = $raw.Substring(0, $sp).Split(',')
    if (($plats -notcontains 'all') -and ($plats -notcontains 'win')) { continue }
    $rest = $raw.Substring($sp + 1); $sp = $rest.IndexOf(' ')
    $kind = $rest.Substring(0, $sp); $text = $rest.Substring($sp + 1)
    switch ($kind) {
        'send' { $p.StandardInput.WriteLine($text); $p.StandardInput.Flush() }
        'expect' { $expects += $text }
        'absent' { $absents += $text }
        'first' { $firsts += $text }
        'order' { $orders += ,$text }
    }
}
$lines = New-Object System.Collections.ArrayList
Read-Lines $p $lines 20000 { foreach ($e in $expects) { if (-not ($lines | Where-Object { $_.Contains($e) })) { return $false } }; return $true }
Stop-Host $p
foreach ($e in $expects) { if (-not ($lines | Where-Object { $_.Contains($e) })) { Write-Output "smoke: RED no output line contains: $e"; exit 1 } }
foreach ($a in $absents) { if ($lines | Where-Object { $_.Contains($a) }) { Write-Output "smoke: RED an output line contains: $a"; exit 1 } }
foreach ($f in $firsts) { if ($lines.Count -eq 0 -or -not $lines[0].Contains($f)) { Write-Output "smoke: RED the first line lacks: $f"; exit 1 } }
foreach ($o in $orders) {
    $parts = $o.Split('|'); $ia = -1; $ib = -1
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($ia -lt 0 -and $lines[$i].Contains($parts[0])) { $ia = $i }
        if ($ib -lt 0 -and $lines[$i].Contains($parts[1])) { $ib = $i }
    }
    if ($ib -ge 0 -and ($ia -lt 0 -or $ia -gt $ib)) { Write-Output "smoke: RED $($parts[0]) does not come before $($parts[1])"; exit 1 }
}

# ---- a shared surface: a named file mapping, shown and answered with vsync (std.gfx.8, ADR 1044) ----
$script:pending = $null
$p = Start-Host
$w = 64; $h = 48; $stride = $w * 4; $guard = 16384; $total = $guard + $stride * $h * 2 + $guard
$name = "Local\ail-smoke-$PID-$([Environment]::TickCount)"
$mmf = [System.IO.MemoryMappedFiles.MemoryMappedFile]::CreateNew($name, $total)
$view = $mmf.CreateViewAccessor()
for ($i = 0; $i -lt $w * $h; $i++) { $view.Write($guard + $i * 4, [uint32]0xff0000ff) } # R=255 G=0 B=0 A=255 in buffer 0
$p.StandardInput.WriteLine('{"v":1,"t":"view","title":"surface","focus":"","hover":true,"tree":{"kind":"Canvas","id":"gfx","w":64,"h":48,"ops":[]}}')
$p.StandardInput.WriteLine("{`"v`":1,`"t`":`"surface`",`"id`":`"gfx`",`"w`":$w,`"h`":$h,`"stride`":$stride,`"offset`":$guard,`"count`":2,`"name`":`"$name`"}")
$p.StandardInput.WriteLine('{"v":1,"t":"text","id":"gfx","items":[{"x":2,"y":2,"s":"ok","size":12,"color":16777215}]}')
$p.StandardInput.WriteLine('{"v":1,"t":"present","id":"gfx","buf":0}')
$p.StandardInput.Flush()
$lines = New-Object System.Collections.ArrayList
Read-Lines $p $lines 20000 { [bool]($lines | Where-Object { $_.Contains('"kind":"vsync"') }) }
$v1 = $lines | Where-Object { $_.Contains('"kind":"vsync"') } | Select-Object -First 1
if (-not $v1 -or $v1 -notmatch '"id":"gfx"' -or $v1 -notmatch '"dx":[0-9.]+' -or $v1 -notmatch '"text":"0"') { Write-Output "smoke: RED no vsync for the surface: $v1"; exit 1 }
$p.StandardInput.WriteLine('{"v":1,"t":"present","id":"gfx","buf":1}')
$p.StandardInput.Flush()
Read-Lines $p $lines 20000 { @($lines | Where-Object { $_.Contains('"text":"1"') -and $_.Contains('"kind":"vsync"') }).Count -ge 1 }
if (-not ($lines | Where-Object { $_.Contains('"kind":"vsync"') -and $_.Contains('"text":"1"') })) { Write-Output 'smoke: RED no vsync after presenting buffer 1'; exit 1 }
Stop-Host $p
$view.Dispose(); $mmf.Dispose()
Write-Output 'smoke: GREEN'
exit 0
