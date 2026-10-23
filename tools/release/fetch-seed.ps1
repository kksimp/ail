# Download the native compiler seed for this host from the private repository's release and install it
# as .native\seed(.exe) (ADR 0855), the PowerShell twin of fetch-seed.sh. The SHA-256 in
# tools\release\seeds.sha256 must match; nothing unverified is installed. Needs `gh` signed in or GITHUB_TOKEN.
#   tools\release\fetch-seed.ps1 [TAG]     TAG defaults to the first line of tools\release\SEED_TAG
param([string]$Tag = '')
$ErrorActionPreference = 'Stop'
$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$onWindows = -not ($IsLinux -or $IsMacOS)
$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
function Fail([string]$m) { [Console]::Error.WriteLine("fetch-seed: $m"); exit 2 }
if ($onWindows -and $arch -eq 'X64') { $asset = 'ail-seed-x86_64-windows.exe' }
elseif ($IsLinux -and $arch -eq 'X64') { $asset = 'ail-seed-x86_64-linux' }
elseif ($IsLinux -and $arch -eq 'Arm64') { $asset = 'ail-seed-aarch64-linux' }
elseif ($IsMacOS -and $arch -eq 'Arm64') { $asset = 'ail-seed-aarch64-macos' }
else { Fail "unsupported host $arch" }
if (-not $Tag) { $Tag = (Get-Content (Join-Path $PSScriptRoot 'SEED_TAG') -TotalCount 1 -ErrorAction SilentlyContinue) }
$line = Get-Content (Join-Path $PSScriptRoot 'seeds.sha256') -ErrorAction SilentlyContinue | Where-Object { ($_ -split '\s+')[1] -eq $asset }
if (-not $Tag -or -not $line) { Fail "no seed is published yet for $asset (tools\release\seeds.sha256)" }
$want = ($line -split '\s+')[0].ToLower()
$repo = 'kksimp/ail'
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("ail-seed-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
$file = Join-Path $tmp $asset
try {
    $gh = Get-Command gh -ErrorAction SilentlyContinue
    if ($gh) { & gh auth status *> $null }
    if ($gh -and $LASTEXITCODE -eq 0) {
        $id = & gh api "repos/$repo/releases/tags/$Tag" --jq ".assets[] | select(.name == \`"$asset\`") | .id"
        if (-not $id) { Fail "release $Tag has no asset $asset" }
        & gh api -H 'Accept: application/octet-stream' "repos/$repo/releases/assets/$id" > $file
    } elseif ($env:GITHUB_TOKEN) {
        $h = @{ Authorization = "Bearer $env:GITHUB_TOKEN" }
        $rel = Invoke-RestMethod -Headers $h -Uri "https://api.github.com/repos/$repo/releases/tags/$Tag"
        $id = ($rel.assets | Where-Object { $_.name -eq $asset } | Select-Object -First 1).id
        if (-not $id) { Fail "release $Tag has no asset $asset" }
        $h['Accept'] = 'application/octet-stream'
        Invoke-WebRequest -UseBasicParsing -Headers $h -Uri "https://api.github.com/repos/$repo/releases/assets/$id" -OutFile $file
    } else {
        Fail 'sign in with gh (gh auth login) or set GITHUB_TOKEN'
    }
    $got = (Get-FileHash -Algorithm SHA256 $file).Hash.ToLower()
    if ($got -ne $want) { Fail "SHA-256 mismatch for $asset (got $got, want $want); not installing" }
    $dest = Join-Path $root '.native'
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    $seed = Join-Path $dest $(if ($onWindows) { 'seed.exe' } else { 'seed' })
    Copy-Item -Force $file $seed
    if (-not $onWindows) { & chmod +x $seed }
    [Console]::Error.WriteLine("fetch-seed: installed $asset ($Tag) as $seed")
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
