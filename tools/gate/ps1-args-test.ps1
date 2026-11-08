# ail.ps1 hands program arguments over unchanged, quotes and all (Windows PowerShell 5.1 does not escape embedded quotes).
#   tools\gate\ps1-args-test.ps1       from the repository root; prints `PS1-ARGS GREEN` or what differed
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$ail = Join-Path $root 'ail.ps1'
$prog = 'tests/winargs/echo_args.ail'
$cases = @(
    @('d"e', 'f'),
    @('a b', 'c"d e', 'g'),
    @('x\"y', 'z'),
    @('trail\', 'next'),
    @('two words\', 'next'),
    @('"quoted"', 'last'),
    @('plain', '--flag=1', '-v')
)
$fails = 0
foreach ($c in $cases) {
    $want = ($c | ForEach-Object { "[$_]" }) -join "`n"
    # The outer call is native too: quote its arguments by the Windows rules so ail.ps1 sees them whole.
    $q = { param($t) '"' + ("$t" -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"' }
    $line = (@('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $ail, 'run', $prog) + $c | ForEach-Object { & $q $_ }) -join ' '
    $o = [IO.Path]::GetTempFileName(); $e = [IO.Path]::GetTempFileName()
    $p = Start-Process powershell -ArgumentList $line -NoNewWindow -PassThru -RedirectStandardOutput $o -RedirectStandardError $e
    $null = $p.Handle; $p.WaitForExit()
    $got = ((@(Get-Content $o) + @(Get-Content $e)) -join "`n").TrimEnd()
    Remove-Item -Force $o, $e
    if ($got -ne $want) { $fails++; "RED args: $($c -join ' | ')`n  want: $($want -replace "`n",' ')`n  got:  $($got -replace "`n",' ')" }
}
if ($fails -eq 0) { 'PS1-ARGS GREEN' } else { "PS1-ARGS RED ($fails)" }
exit ([int]($fails -gt 0))
