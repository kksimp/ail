# Builds the std.app host for Windows (ADR 0838) and, given arguments, runs
# an AIL program with it. From the repository root:
#   tools\app-host\windows\run.ps1                       # build only
#   tools\app-host\windows\run.ps1 examples\app_todo.ail
# Needs only Windows: the C# compiler that ships with the .NET Framework
# (csc.exe), no SDK and no third-party packages.
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = (Resolve-Path (Join-Path $here '..\..\..')).Path
$exe = Join-Path $here 'build\AILAppHost.exe'
$src = Join-Path $here 'Host.cs'

$csc = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
if (-not (Test-Path $csc)) { [Console]::Error.WriteLine('app-host: csc.exe (.NET Framework 4) is missing'); exit 2 }

if (-not (Test-Path $exe) -or ((Get-Item $src).LastWriteTime -gt (Get-Item $exe).LastWriteTime)) {
    [Console]::Error.WriteLine("app-host: building $exe")
    New-Item -ItemType Directory -Force (Split-Path $exe) | Out-Null
    & $csc /nologo /optimize /target:winexe /platform:x64 "/out:$exe" `
        /r:System.dll /r:System.Drawing.dll /r:System.Windows.Forms.dll $src
    if ($LASTEXITCODE -ne 0) { [Console]::Error.WriteLine('app-host: the build failed'); exit 1 }
}

if ($args.Count -gt 0) {
    $env:AIL_APP_HOST = $exe
    # Garlic's window and taskbar icon (../garlic/icon); the host reads AIL_APP_ICON.
    foreach ($a in $args) {
        if ("$a" -match '(^|[\\/])garlic\.ail$') {
            foreach ($n in 'garlic-256.png', 'garlic-512.png') {
                $icon = Join-Path $root "..\garlic\icon\$n"
                if (Test-Path $icon) { $env:AIL_APP_ICON = (Resolve-Path $icon).Path; break }
            }
        }
    }
    # The run's output also goes to build\last-run.log, for runs without a console.
    $log = Join-Path $here 'build\last-run.log'
    "run: $($args -join ' ')" | Set-Content $log
    & (Join-Path $root 'ail.ps1') run @args 2>&1 | Tee-Object -FilePath $log -Append
    "exit: $LASTEXITCODE" | Add-Content $log
    exit $LASTEXITCODE
}
