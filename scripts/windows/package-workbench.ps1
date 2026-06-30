[CmdletBinding()]
param(
    [string]$Configuration = "Release",
    [string]$Preset = "windows-workbench",
    [string]$Version = "",
    [string]$RuntimeVersion = "latest",
    [string]$RuntimeRepo = "bangonkali/baidu-unlimited-ocr-portable",
    [string]$RuntimePlatform = "windows-x86_64-cuda13",
    [string[]]$AdditionalRuntimePlatforms = @("windows-x86_64-cpu"),
    [string]$OutputDir = "",
    [switch]$NoBuild,
    [switch]$NoRuntimeDownload,
    [switch]$NoCpuRuntimeBuild
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

function Assert-VcpkgOpenSslRuntime {
    param([System.IO.FileInfo]$Exe)
    $dumpbin = Get-Command dumpbin -ErrorAction SilentlyContinue
    if (-not $dumpbin) {
        Write-Warning "dumpbin was not found; skipping OpenSSL DLL dependency inspection."
        return
    }
    $cryptoDll = Get-ChildItem -LiteralPath $Exe.Directory.FullName -Filter "libcrypto*.dll" |
        Select-Object -First 1
    $sslDll = Get-ChildItem -LiteralPath $Exe.Directory.FullName -Filter "libssl*.dll" |
        Select-Object -First 1
    if (-not $cryptoDll -or -not $sslDll) {
        throw "Expected vcpkg OpenSSL runtime DLLs beside uocr-server.exe."
    }
    $rootBinaries = @($Exe) + @(Get-ChildItem -LiteralPath $Exe.Directory.FullName -Filter "*.dll")
    $importsOpenSsl = $false
    foreach ($binary in $rootBinaries) {
        $deps = (& $dumpbin.Source /DEPENDENTS $binary.FullName 2>$null) -join "`n"
        if ($deps -match "libcrypto|libssl") {
            $importsOpenSsl = $true
        }
        if ($binary.Name -eq "trantor.dll" -and ($deps -notmatch "libssl" -or $deps -notmatch "libcrypto")) {
            throw "trantor.dll does not import OpenSSL; Drogon TLS support is not enabled."
        }
        if ($binary.Name -eq $Exe.Name -and $deps -notmatch "libcrypto") {
            throw "uocr-server.exe does not import libcrypto; SHA verification is not sharing vcpkg OpenSSL."
        }
    }
    if (-not $importsOpenSsl) {
        throw "No backend root binary imports OpenSSL; expected shared vcpkg OpenSSL dependency."
    }
}

function Copy-ServerRootDlls {
    param([string]$SourceDir, [string]$DestinationDir)
    Get-ChildItem -LiteralPath $SourceDir -Filter "*.dll" |
        Copy-Item -Destination $DestinationDir -Force
}

function Copy-VcpkgCopyright {
    param(
        [string]$Package,
        [string]$Destination
    )
    $match = Get-ChildItem -LiteralPath (Join-Path $RepoRoot "build") -Recurse -Filter "copyright" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match "\\vcpkg_installed\\.*\\share\\$([Regex]::Escape($Package))\\copyright$" } |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $match) {
        throw "vcpkg copyright file was not found for $Package."
    }
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $Destination) | Out-Null
    Copy-Item -LiteralPath $match.FullName -Destination $Destination -Force
}

function Get-VcpkgToolchainPath {
    if (-not $env:VCPKG_ROOT) {
        throw "VCPKG_ROOT is not set; the portable package requires vcpkg-managed dependencies."
    }
    $toolchain = Join-Path $env:VCPKG_ROOT "scripts\buildsystems\vcpkg.cmake"
    if (-not (Test-Path $toolchain)) {
        throw "vcpkg toolchain was not found: $toolchain"
    }
    return $toolchain
}

function Get-VcpkgTripletRoot {
    $triplet = "x64-windows-release"
    $preferred = Join-Path $RepoRoot "build\$Preset\vcpkg_installed\$triplet"
    if (Test-Path $preferred) {
        return $preferred
    }
    $fallback = Get-ChildItem -LiteralPath (Join-Path $RepoRoot "build") -Recurse -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match "\\vcpkg_installed\\$triplet$" } |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if ($fallback) {
        return $fallback.FullName
    }
    throw "vcpkg triplet root was not found for $triplet. Build the workbench before packaging."
}

function Invoke-LoggedNative {
    param(
        [string]$FilePath,
        [string[]]$Arguments,
        [string]$FailureMessage
    )

    & $FilePath @Arguments 2>&1 | ForEach-Object { Write-Host $_ }
    if ($LASTEXITCODE -ne 0) {
        throw "$FailureMessage with exit code $LASTEXITCODE"
    }
}

function Copy-VcpkgOpenSslRuntimeDlls {
    param(
        [string]$TripletRoot,
        [string]$Destination
    )
    $bin = Join-Path $TripletRoot "bin"
    foreach ($pattern in @("libcrypto*.dll", "libssl*.dll")) {
        $matches = Get-ChildItem -LiteralPath $bin -Filter $pattern -ErrorAction SilentlyContinue
        if (-not $matches) {
            throw "Expected vcpkg OpenSSL runtime DLL matching $pattern under $bin."
        }
        $matches | Copy-Item -Destination $Destination -Force
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

function Find-BuiltRuntimeFile {
    param([string]$BuildDir, [string]$Name)
    $match = Get-ChildItem -LiteralPath $BuildDir -Recurse -Filter $Name -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -notmatch "\\CMakeFiles\\" } |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $match) {
        throw "Built runtime file was not found under $BuildDir`: $Name"
    }
    return $match
}

function Install-CpuRuntimeFromSource {
    param([string]$RuntimeDir)

    if ($NoCpuRuntimeBuild) {
        throw "CPU runtime is missing and -NoCpuRuntimeBuild was specified."
    }
    $llamaDir = Join-Path $RepoRoot "thirdparty\llama.cpp"
    if (-not (Test-Path (Join-Path $llamaDir "CMakeLists.txt"))) {
        throw "llama.cpp submodule is missing; cannot build CPU runtime."
    }
    $buildDir = Join-Path $llamaDir "build-windows-x86_64-cpu"
    $vcpkgToolchain = Get-VcpkgToolchainPath
    $vcpkgTripletRoot = Get-VcpkgTripletRoot
    Write-Host "Building Windows CPU runtime for portable fallback"
    Invoke-LoggedNative `
        -FilePath "cmake" `
        -Arguments @(
            "-B", $buildDir,
            "-S", $llamaDir,
            "-G", "Visual Studio 18 2026",
            "-A", "x64",
            "-DGGML_NATIVE=OFF",
            "-DCMAKE_TOOLCHAIN_FILE=$vcpkgToolchain",
            "-DVCPKG_TARGET_TRIPLET=x64-windows-release",
            "-DCMAKE_PREFIX_PATH=$vcpkgTripletRoot",
            "-DOPENSSL_ROOT_DIR=$vcpkgTripletRoot",
            "-DOPENSSL_USE_STATIC_LIBS=OFF"
        ) `
        -FailureMessage "CPU runtime configure failed"
    Invoke-LoggedNative `
        -FilePath "cmake" `
        -Arguments @("--build", $buildDir, "--config", "Release", "--target", "llama-mtmd-cli", "llama-uocr-parity", "llama-server", "uocr-ffi", "--parallel", "3") `
        -FailureMessage "CPU runtime build failed"

    $binDir = Join-Path $RuntimeDir "bin"
    Remove-Item -LiteralPath $RuntimeDir -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
    foreach ($name in @("uocr-ffi.dll", "llama-uocr-parity.exe", "llama-mtmd-cli.exe", "llama-server.exe")) {
        $file = Find-BuiltRuntimeFile -BuildDir $buildDir -Name $name
        Copy-Item -LiteralPath $file.FullName -Destination $binDir -Force
    }
    $runtimeDllDirs = Get-ChildItem -LiteralPath $buildDir -Recurse -Filter "*.dll" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -notmatch "\\CMakeFiles\\" } |
        Select-Object -ExpandProperty DirectoryName -Unique
    foreach ($dir in $runtimeDllDirs) {
        Get-ChildItem -LiteralPath $dir -Filter "*.dll" |
            Copy-Item -Destination $binDir -Force
    }
    Copy-VcpkgOpenSslRuntimeDlls -TripletRoot $vcpkgTripletRoot -Destination $binDir
}

function Ensure-RuntimePlatform {
    param([string]$Platform)

    $runtimeRoot = Join-Path $RepoRoot "thirdparty\uocr-runtime"
    $runtimeDir = Join-Path $runtimeRoot $Platform
    $runtimeFfi = Join-Path $runtimeDir "bin\uocr-ffi.dll"
    if (Test-Path $runtimeFfi) {
        return $runtimeDir
    }
    if ($Platform -eq "windows-x86_64-cpu") {
        Install-CpuRuntimeFromSource -RuntimeDir $runtimeDir
        return $runtimeDir
    }
    if (-not $NoRuntimeDownload) {
        Install-RuntimeFromRelease `
            -Repo $RuntimeRepo `
            -Tag $RuntimeVersion `
            -Platform $Platform `
            -RuntimeDir $runtimeDir
    }
    if (-not (Test-Path $runtimeFfi)) {
        throw "Runtime FFI library is missing: $runtimeFfi"
    }
    return $runtimeDir
}

if (-not $NoBuild) {
    & (Join-Path $PSScriptRoot "build-workbench.ps1") `
        -Configuration $Configuration `
        -Preset $Preset `
        -Version $Version
}

$RuntimeDirs = @{}
$AllRuntimePlatforms = @($RuntimePlatform) + @($AdditionalRuntimePlatforms) | Select-Object -Unique
foreach ($platform in $AllRuntimePlatforms) {
    $RuntimeDirs[$platform] = Ensure-RuntimePlatform -Platform $platform
}

$Exe = Find-ServerExe
$ExeDir = $Exe.Directory.FullName
Assert-VcpkgOpenSslRuntime -Exe $Exe
$StageRoot = Join-Path $OutputDir "uocr-workbench-windows-x64-$SafeVersion"
$ZipPath = Join-Path $OutputDir "uocr-workbench-windows-x64-$SafeVersion.zip"
$ShaPath = "$ZipPath.sha256"

Remove-Item -LiteralPath $StageRoot -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $ZipPath -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $ShaPath -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $StageRoot | Out-Null

Copy-Item -LiteralPath (Join-Path $ExeDir "uocr-server.exe") -Destination $StageRoot -Force
Copy-ServerRootDlls -SourceDir $ExeDir -DestinationDir $StageRoot
Copy-DirectoryIfExists -Source (Join-Path $ExeDir "web") -Destination (Join-Path $StageRoot "web")
Copy-DirectoryIfExists -Source (Join-Path $ExeDir "openapi") -Destination (Join-Path $StageRoot "openapi")

$RuntimeStage = Join-Path $StageRoot "thirdparty\uocr-runtime"
New-Item -ItemType Directory -Force -Path $RuntimeStage | Out-Null
foreach ($platform in $AllRuntimePlatforms) {
    Copy-Item -LiteralPath $RuntimeDirs[$platform] -Destination $RuntimeStage -Recurse -Force
}

Copy-VcpkgCopyright `
    -Package "libmupdf" `
    -Destination (Join-Path $StageRoot "thirdparty\libmupdf\copyright")

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
Open Settings to choose the runtime/accelerator and default model/profile. CUDA is preferred when supported; the zip also bundles a CPU runtime fallback.
Open Models, choose a GGUF variant, click Download, then Use; progress shows per-file bytes, MiB/s, ETA, retry, and cancel.
PDF support: native vcpkg libmupdf is statically linked into uocr-server.exe and renders pages at 200 DPI.
Uninstall: delete this folder.
"@ | Set-Content -LiteralPath (Join-Path $StageRoot "README.txt") -Encoding utf8

$manifest = [ordered]@{
    schema_version = 1
    name = "uocr-workbench"
    version = $Version
    platform = "windows-x64"
    runtime_platform = $RuntimePlatform
    runtime_platforms = @($AllRuntimePlatforms)
    runtime_version = $RuntimeVersion
    pdf_renderer = "vcpkg-libmupdf"
    created_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
}
$manifest | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $StageRoot "install-manifest.json") -Encoding utf8

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
Compress-Archive -LiteralPath $StageRoot -DestinationPath $ZipPath -CompressionLevel Optimal
$hash = (Get-FileHash -LiteralPath $ZipPath -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $(Split-Path -Leaf $ZipPath)" | Set-Content -LiteralPath $ShaPath -Encoding ascii

Write-Host "Packaged $ZipPath"
Write-Host "Checksum $ShaPath"
