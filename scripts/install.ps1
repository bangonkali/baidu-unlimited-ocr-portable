[CmdletBinding()]
param(
    [string]$Version = "latest",
    [string]$InstallDir = (Join-Path $HOME ".uocr"),
    [string]$Repo = "bangonkali/baidu-unlimited-ocr-portable"
)

$ErrorActionPreference = "Stop"
$UserAgent = "uocr-installer"

function Resolve-ReleaseTag {
    if ($Version -ne "latest") {
        return $Version
    }

    $response = Invoke-WebRequest `
        -Uri "https://github.com/$Repo/releases/latest" `
        -Headers @{ "User-Agent" = $UserAgent } `
        -UseBasicParsing
    $finalUrl = $response.BaseResponse.RequestMessage.RequestUri.AbsoluteUri
    if ($finalUrl -notmatch "/releases/tag/([^/?#]+)") {
        throw "Could not resolve the latest release tag from $finalUrl."
    }
    return $Matches[1]
}

$releaseTag = Resolve-ReleaseTag
$assetName = "uocr-workbench-windows-x64-$releaseTag.zip"
$assetUrl = "https://github.com/$Repo/releases/download/$releaseTag/$assetName"
$shaUrl = "$assetUrl.sha256"
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("uocr-install-" + [System.Guid]::NewGuid())
$zipPath = Join-Path $tempRoot $assetName
$shaPath = "$zipPath.sha256"
$extractRoot = Join-Path $tempRoot "extract"

New-Item -ItemType Directory -Force -Path $tempRoot, $extractRoot | Out-Null
try {
    Write-Host "Downloading $assetName from $Repo $releaseTag..."
    Invoke-WebRequest -Uri $assetUrl -OutFile $zipPath -Headers @{ "User-Agent" = $UserAgent }
    Invoke-WebRequest -Uri $shaUrl -OutFile $shaPath -Headers @{ "User-Agent" = $UserAgent }
    $expected = ((Get-Content -LiteralPath $shaPath -Raw).Trim() -split "\s+")[0].ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expected -and $expected -ne $actual) {
        throw "Checksum mismatch. Expected $expected, got $actual."
    }
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
