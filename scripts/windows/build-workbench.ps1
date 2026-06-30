param(
    [string]$Configuration = "Release",
    [string]$Preset = "windows-workbench",
    [string]$Version = "",
    [switch]$NoClientBuild,
    [switch]$NoMupdfBuild
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$BuildPreset = "$Preset-$($Configuration.ToLowerInvariant())"
$LogDir = Join-Path $RepoRoot ".logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

function Invoke-NativeChecked {
    param(
        [string]$Label,
        [scriptblock]$Command
    )

    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

function Resolve-UocrServerExe {
    param(
        [string]$Root,
        [string]$Config
    )

    $buildRoot = Join-Path $Root "build"
    $matches = Get-ChildItem -LiteralPath $buildRoot -Recurse -Filter "uocr-server.exe" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -notmatch "\\CMakeFiles\\" } |
        Sort-Object LastWriteTime -Descending
    if (-not $matches) {
        throw "Expected executable was not produced under $buildRoot"
    }
    return $matches[0].FullName
}

function Remove-ObsoleteMutoolBundle {
    param([string]$ExePath)

    $mutoolDir = Join-Path (Split-Path -Parent $ExePath) "thirdparty\mupdf"
    Remove-Item -LiteralPath $mutoolDir -Recurse -Force -ErrorAction SilentlyContinue
}

if (-not $Version) {
    $Version = (git -C $RepoRoot describe --tags --dirty --always 2>$null)
    if (-not $Version) {
        $Version = "0.0.0-dev"
    }
}

if (-not $env:VCPKG_ROOT) {
    $defaultVcpkg = "C:\Program Files\Microsoft Visual Studio\18\Community\VC\vcpkg"
    if (Test-Path $defaultVcpkg) {
        $env:VCPKG_ROOT = $defaultVcpkg
    }
}

if (-not $env:VCPKG_ROOT -or -not (Test-Path (Join-Path $env:VCPKG_ROOT "scripts\buildsystems\vcpkg.cmake"))) {
    throw "VCPKG_ROOT is not set to a usable vcpkg checkout."
}

if (-not $NoClientBuild) {
    Push-Location (Join-Path $RepoRoot "src\uocr-client")
    try {
        Invoke-NativeChecked "bun install" { bun install }
        Invoke-NativeChecked "bun run build" { bun run build }
    } finally {
        Pop-Location
    }
}

if ($NoMupdfBuild) {
    Write-Warning "-NoMupdfBuild is no longer needed; MuPDF resolves through the vcpkg libmupdf package."
}

Push-Location $RepoRoot
try {
    Invoke-NativeChecked "cmake configure" {
        cmake --preset $Preset `
            "-DUOCR_VERSION=$Version"
    }
    Invoke-NativeChecked "cmake build" { cmake --build --preset $BuildPreset }
} finally {
    Pop-Location
}

$Exe = Resolve-UocrServerExe -Root $RepoRoot -Config $Configuration
Remove-ObsoleteMutoolBundle -ExePath $Exe

Write-Host "Built $Exe"
Write-Host "Linked embedded MuPDF renderer through vcpkg libmupdf"
Write-Host "Version $Version"
Write-Host "Double-click uocr-server.exe to launch the backend and open the hosted React app."
