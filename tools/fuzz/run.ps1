# Fuzz and differential run (ADR 0269-0271), Windows. See run.sh.
#   tools\fuzz\run.ps1 [N [SEED]]
$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$env:AIL_FUZZ_N = if ($args.Count -gt 0) { $args[0] } else { '10000' }
$env:AIL_FUZZ_SEED = if ($args.Count -gt 1) { $args[1] } else { '1' }
if (-not $env:AIL_FUZZ_OUT) { $env:AIL_FUZZ_OUT = Join-Path $root 'tools\fuzz\failures' }
if (Test-Path $env:AIL_FUZZ_OUT) { Remove-Item -Recurse -Force $env:AIL_FUZZ_OUT }
Write-Host "fuzz: $($env:AIL_FUZZ_N) programs from seed $($env:AIL_FUZZ_SEED)"
& (Join-Path $root 'ail.ps1') selftest --test fuzz fuzz_ -- --nocapture
exit $LASTEXITCODE
