[CmdletBinding()]
param(
    [string]$Version = "latest",
    [string]$InstallDir = (Join-Path $HOME ".uocr"),
    [string]$Repo = "bangonkali/baidu-unlimited-ocr-portable"
)

$ErrorActionPreference = "Stop"
$UserAgent = "uocr-installer"

function Get-Release {
    if ($Version -eq "latest") {
        $url = "https://api.github.com/repos/$Repo/releases/latest"
    }
    else {
        $url = "https://api.github.com/repos/$Repo/releases/tags/$Version"
    }
    Invoke-RestMethod -Uri $url -Headers @{
        "Accept" = "application/vnd.github+json"
        "User-Agent" = $UserAgent
        "X-GitHub-Api-Version" = "2022-11-28"
    }
}

function Get-Asset {
    param($Release)
    $asset = $Release.assets |
        Where-Object { $_.name -like "uocr-workbench-windows-x64-*.zip" } |
        Sort-Object name -Descending |
        Select-Object -First 1
    if (-not $asset) {
        throw "No Windows workbench zip asset was found on release $($Release.tag_name)."
    }
    $asset
}

$release = Get-Release
$asset = Get-Asset -Release $release
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("uocr-install-" + [System.Guid]::NewGuid())
$zipPath = Join-Path $tempRoot $asset.name
$extractRoot = Join-Path $tempRoot "extract"

New-Item -ItemType Directory -Force -Path $tempRoot, $extractRoot | Out-Null
try {
    Write-Host "Downloading $($asset.name) from $Repo $($release.tag_name)..."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath -Headers @{ "User-Agent" = $UserAgent }
    Expand-Archive -LiteralPath $zipPath -DestinationPath $extractRoot -Force
    $exe = Get-ChildItem -LiteralPath $extractRoot -Recurse -Filter "uocr-server.exe" |
        Select-Object -First 1
    if (-not $exe) {
        throw "Downloaded archive did not contain uocr-server.exe."
    }
    $sourceRoot = $exe.Directory.FullName
    Remove-Item -LiteralPath $InstallDir -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item -Path (Join-Path $sourceRoot "*") -Destination $InstallDir -Recurse -Force

    $commandPath = Join-Path $InstallDir "uocr-server.exe"
    Write-Host ""
    Write-Host "Installed Unlimited-OCR Workbench to $InstallDir"
    Write-Host "Run: & `"$commandPath`""
    Write-Host "Default URL: http://127.0.0.1:8765/"
    Write-Host "Uninstall: delete $InstallDir"
}
finally {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
}
