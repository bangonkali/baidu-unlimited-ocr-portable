[CmdletBinding()]
param(
    [string]$Configuration = "Release",
    [string]$Preset = "windows-workbench",
    [string]$Version = "",
    [string]$RuntimeVersion = "latest",
    [string]$RuntimeRepo = "bangonkali/baidu-unlimited-ocr-portable",
    [string]$RuntimePlatform = "windows-x86_64-cuda13",
    [string]$OutputDir = "",
    [switch]$NoBuild,
    [switch]$NoRuntimeDownload
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
if (-not $OutputDir) {
    $OutputDir = Join-Path $RepoRoot "dist"
}
if (-not $Version) {
    $Version = (git -C $RepoRoot describe --tags --dirty --always 2>$null)
    if (-not $Version) {
        $Version = "0.0.0-dev"
    }
}
$SafeVersion = $Version.Replace("/", "-").Replace("\", "-")
$UserAgent = "uocr-workbench-packager"

function New-GitHubHeaders {
    $headers = @{
        "Accept" = "application/vnd.github+json"
        "User-Agent" = $UserAgent
        "X-GitHub-Api-Version" = "2022-11-28"
    }
    $token = $env:GH_TOKEN
    if (-not $token) {
        $token = $env:GITHUB_TOKEN
    }
    if ($token) {
        $headers["Authorization"] = "Bearer $token"
    }
    return $headers
}

function Get-GitHubRelease {
    param([string]$Repo, [string]$Tag)
    if ($Tag -eq "latest") {
        $url = "https://api.github.com/repos/$Repo/releases/latest"
    }
    else {
        $url = "https://api.github.com/repos/$Repo/releases/tags/$Tag"
    }
    Invoke-RestMethod -Uri $url -Headers (New-GitHubHeaders)
}

function Get-GitHubRuntimeRelease {
    param([string]$Repo, [string]$Tag, [string]$Platform)
    if ($Tag -ne "latest") {
        return Get-GitHubRelease -Repo $Repo -Tag $Tag
    }

    $page = 1
    while ($true) {
        $url = "https://api.github.com/repos/$Repo/releases?per_page=30&page=$page"
        $releases = @(Invoke-RestMethod -Uri $url -Headers (New-GitHubHeaders))
        if (-not $releases -or $releases.Count -eq 0) {
            break
        }
        foreach ($release in $releases) {
            $asset = $release.assets |
                Where-Object { $_.name -like "uocr-runtime-$Platform-*.zip" } |
                Select-Object -First 1
            if ($asset) {
                return $release
            }
        }
        $page += 1
    }

    throw "No release with a Windows runtime zip asset was found on $Repo."
}

function Find-ServerExe {
    $matches = Get-ChildItem -LiteralPath (Join-Path $RepoRoot "build") -Recurse -Filter "uocr-server.exe" |
        Where-Object { $_.FullName -notmatch "\\CMakeFiles\\" } |
        Sort-Object LastWriteTime -Descending
    if (-not $matches) {
        throw "uocr-server.exe was not found under build\."
    }
    return $matches[0]
}

function Copy-DirectoryIfExists {
    param([string]$Source, [string]$Destination)
    if (Test-Path $Source) {
        Copy-Item -LiteralPath $Source -Destination $Destination -Recurse -Force
    }
}

function Save-ReleaseAsset {
    param($Asset, [string]$Destination)
    Invoke-WebRequest `
        -Uri $Asset.browser_download_url `
        -OutFile $Destination `
        -Headers (New-GitHubHeaders)
}

function Install-RuntimeArchive {
    param(
        [string]$ArchivePath,
        [string]$RuntimeDir
    )

    $extractRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("uocr-runtime-" + [System.Guid]::NewGuid())
    try {
        Expand-Archive -LiteralPath $ArchivePath -DestinationPath $extractRoot -Force
        $ffi = Get-ChildItem -LiteralPath $extractRoot -Recurse -Filter "uocr-ffi.dll" |
            Select-Object -First 1
        if (-not $ffi) {
            throw "Runtime archive did not contain uocr-ffi.dll."
        }
        $sourceRoot = $ffi.Directory.Parent.FullName
        Remove-Item -LiteralPath $RuntimeDir -Recurse -Force -ErrorAction SilentlyContinue
        New-Item -ItemType Directory -Force -Path $RuntimeDir | Out-Null
        Copy-Item -Path (Join-Path $sourceRoot "*") -Destination $RuntimeDir -Recurse -Force
    }
    finally {
        Remove-Item -LiteralPath $extractRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Install-RuntimeFromRelease {
    param(
        [string]$Repo,
        [string]$Tag,
        [string]$Platform,
        [string]$RuntimeDir
    )

    $release = Get-GitHubRuntimeRelease -Repo $Repo -Tag $Tag -Platform $Platform
    $asset = $release.assets |
        Where-Object { $_.name -like "uocr-runtime-$Platform-*.zip" } |
        Sort-Object name -Descending |
        Select-Object -First 1
    if (-not $asset) {
        throw "No Windows runtime zip asset found on $Repo release $($release.tag_name)."
    }

    $downloadRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("uocr-runtime-download-" + [System.Guid]::NewGuid())
    New-Item -ItemType Directory -Force -Path $downloadRoot | Out-Null
    try {
        $archive = Join-Path $downloadRoot $asset.name
        Save-ReleaseAsset -Asset $asset -Destination $archive
        $shaAsset = $release.assets | Where-Object { $_.name -eq "$($asset.name).sha256" } | Select-Object -First 1
        if ($shaAsset) {
            $shaPath = Join-Path $downloadRoot $shaAsset.name
            Save-ReleaseAsset -Asset $shaAsset -Destination $shaPath
            $expected = ((Get-Content -LiteralPath $shaPath -Raw).Trim() -split "\s+")[0].ToLowerInvariant()
            $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($expected -and $expected -ne $actual) {
                throw "Runtime checksum mismatch. Expected $expected, got $actual."
            }
        }
        Install-RuntimeArchive -ArchivePath $archive -RuntimeDir $RuntimeDir
    }
    finally {
        Remove-Item -LiteralPath $downloadRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}

if (-not $NoBuild) {
    & (Join-Path $PSScriptRoot "build-workbench.ps1") `
        -Configuration $Configuration `
        -Preset $Preset `
        -Version $Version
}

$RuntimeRoot = Join-Path $RepoRoot "thirdparty\uocr-runtime"
$RuntimeDir = Join-Path $RuntimeRoot $RuntimePlatform
$RuntimeFfi = Join-Path $RuntimeDir "bin\uocr-ffi.dll"
if (-not (Test-Path $RuntimeFfi) -and -not $NoRuntimeDownload) {
    Install-RuntimeFromRelease `
        -Repo $RuntimeRepo `
        -Tag $RuntimeVersion `
        -Platform $RuntimePlatform `
        -RuntimeDir $RuntimeDir
}
if (-not (Test-Path $RuntimeFfi)) {
    throw "Runtime FFI library is missing: $RuntimeFfi"
}

$Exe = Find-ServerExe
$ExeDir = $Exe.Directory.FullName
$StageRoot = Join-Path $OutputDir "uocr-workbench-windows-x64-$SafeVersion"
$ZipPath = Join-Path $OutputDir "uocr-workbench-windows-x64-$SafeVersion.zip"
$ShaPath = "$ZipPath.sha256"

Remove-Item -LiteralPath $StageRoot -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $ZipPath -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $ShaPath -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $StageRoot | Out-Null

Copy-Item -LiteralPath (Join-Path $ExeDir "uocr-server.exe") -Destination $StageRoot -Force
Get-ChildItem -LiteralPath $ExeDir -Filter "*.dll" | Copy-Item -Destination $StageRoot -Force
Copy-DirectoryIfExists -Source (Join-Path $ExeDir "web") -Destination (Join-Path $StageRoot "web")
Copy-DirectoryIfExists -Source (Join-Path $ExeDir "openapi") -Destination (Join-Path $StageRoot "openapi")

$RuntimeStage = Join-Path $StageRoot "thirdparty\uocr-runtime"
New-Item -ItemType Directory -Force -Path $RuntimeStage | Out-Null
Copy-Item -LiteralPath $RuntimeDir -Destination $RuntimeStage -Recurse -Force

New-Item -ItemType Directory -Force -Path (Join-Path $StageRoot "thirdparty\mupdf") | Out-Null
Copy-Item -LiteralPath (Join-Path $RepoRoot "thirdparty\mupdf\COPYING") `
    -Destination (Join-Path $StageRoot "thirdparty\mupdf\COPYING") `
    -Force

foreach ($dir in @("models", "data", "cache", "logs", "config", "uploads")) {
    New-Item -ItemType Directory -Force -Path (Join-Path $StageRoot $dir) | Out-Null
}

@"
@echo off
setlocal
set UOCR_HOME=%~dp0
"%~dp0uocr-server.exe" %*
"@ | Set-Content -LiteralPath (Join-Path $StageRoot "uocr-server.cmd") -Encoding ascii

@"
Unlimited-OCR Workbench $Version

Run uocr-server.exe to start the local backend and hosted React app.
Default URL: http://127.0.0.1:8765/
Logs: logs\uocr-server.log
Optional authenticated model downloads: set HF_TOKEN before launching uocr-server.exe.
Open Models and click Download missing to download GGUF files with per-file progress, MiB/s, ETA, retry, and cancel.
PDF support: native MuPDF is embedded in uocr-server.exe and renders pages at 200 DPI.
Uninstall: delete this folder.
"@ | Set-Content -LiteralPath (Join-Path $StageRoot "README.txt") -Encoding utf8

$manifest = [ordered]@{
    schema_version = 1
    name = "uocr-workbench"
    version = $Version
    platform = "windows-x64"
    runtime_platform = $RuntimePlatform
    runtime_version = $RuntimeVersion
    pdf_renderer = "embedded-mupdf"
    created_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
}
$manifest | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $StageRoot "install-manifest.json") -Encoding utf8

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
Compress-Archive -LiteralPath $StageRoot -DestinationPath $ZipPath -CompressionLevel Optimal
$hash = (Get-FileHash -LiteralPath $ZipPath -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $(Split-Path -Leaf $ZipPath)" | Set-Content -LiteralPath $ShaPath -Encoding ascii

Write-Host "Packaged $ZipPath"
Write-Host "Checksum $ShaPath"
