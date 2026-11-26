# Download the native compiler seed for this host, from seeds.voltare.us first and then the repository's release, and install it
# as .native\seed(.exe) (ADR 0855), the PowerShell twin of fetch-seed.sh. The SHA-256 in
# tools\release\seeds.sha256 must match; nothing unverified is installed. Tries the plain release-download URL
# first (no login); if that fails, as it does while the repository is private, falls back to `gh` or GITHUB_TOKEN.
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
    $plain = $false
    $mirror = if ($env:AIL_SEED_HOST -ne $null) { $env:AIL_SEED_HOST } else { 'https://seeds.voltare.us' }
    if ($mirror) {
        try {
            $ProgressPreference = 'SilentlyContinue'
            [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
            Invoke-WebRequest -UseBasicParsing -TimeoutSec 120 -Uri "$mirror/ail/$Tag/$asset" -OutFile $file
            # A mirror copy that fails the pinned hash is discarded and the release is tried.
            if ((Get-FileHash -Algorithm SHA256 $file).Hash.ToLower() -eq $want) { $plain = $true } else { Remove-Item -Force $file }
        } catch { Remove-Item -Force $file -ErrorAction SilentlyContinue }
    }
    if (-not $plain) { try {
        # TLS 1.2 for Windows PowerShell 5.1, whose default may be older; no progress bar, which slows 5.1 down.
        $ProgressPreference = 'SilentlyContinue'
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/$repo/releases/download/$Tag/$asset" -OutFile $file
        $plain = $true
    } catch { Remove-Item -Force $file -ErrorAction SilentlyContinue } }
    $gh = $null
    if (-not $plain) { $gh = Get-Command gh -ErrorAction SilentlyContinue }
    if ($gh) { & gh auth status *> $null }
    if ($plain) {
        # Downloaded without a login; the SHA-256 check below still applies.
    } elseif ($gh -and $LASTEXITCODE -eq 0) {
        $id = & gh api "repos/$repo/releases/tags/$Tag" --jq ".assets[] | select(.name == \`"$asset\`") | .id"
        if (-not $id) { Fail "release $Tag has no asset $asset" }
        # `gh release download`, not `gh api ... > $file`: Windows PowerShell 5.1 would re-encode a redirected native
        # stream as text, and the seed would not match its SHA-256.
        & gh release download $Tag --repo $repo --pattern $asset --output $file --clobber
        if ($LASTEXITCODE -ne 0) { Fail "can't download $asset from release $Tag" }
    } elseif ($env:GITHUB_TOKEN) {
        $h = @{ Authorization = "Bearer $env:GITHUB_TOKEN" }
        $rel = Invoke-RestMethod -Headers $h -Uri "https://api.github.com/repos/$repo/releases/tags/$Tag"
        $id = ($rel.assets | Where-Object { $_.name -eq $asset } | Select-Object -First 1).id
        if (-not $id) { Fail "release $Tag has no asset $asset" }
        $h['Accept'] = 'application/octet-stream'
        Invoke-WebRequest -UseBasicParsing -Headers $h -Uri "https://api.github.com/repos/$repo/releases/assets/$id" -OutFile $file
    } else {
        Fail "can't download $asset from release $Tag (for a private copy, sign in with gh or set GITHUB_TOKEN)"
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
# Success is exit 0 for every caller: the plain download runs no native command, so $LASTEXITCODE would be unset.
exit 0
