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
    $got = ((& powershell -NoProfile -ExecutionPolicy Bypass -File $ail run $prog @c 2>&1) -join "`n").TrimEnd()
    if ($got -ne $want) { $fails++; "RED args: $($c -join ' | ')`n  want: $($want -replace "`n",' ')`n  got:  $($got -replace "`n",' ')" }
}
if ($fails -eq 0) { 'PS1-ARGS GREEN' } else { "PS1-ARGS RED ($fails)" }
exit ([int]($fails -gt 0))
