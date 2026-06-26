[CmdletBinding()]
param(
    [string] $Workspace = "C:\uocr",
    [string] $HostName = "127.0.0.1",
    [int] $Port = 7861,
    [switch] $Smoke,
    [string] $Image = "",
    [ValidateSet("best-zero-empty-q4", "experimental-exact-prefill-q4")]
    [string] $Profile = "best-zero-empty-q4",
    [int] $MaxTokens = 64
)

Set-StrictMode -Version 3.0
$ErrorActionPreference = "Stop"

function Resolve-FirstExisting {
    param([string[]] $Paths)
    foreach ($path in $Paths) {
        if (Test-Path $path) {
            return [System.IO.Path]::GetFullPath($path)
        }
    }
    return ""
}

function Require-File {
    param(
        [string] $Label,
        [string] $Path
    )
    if (-not $Path -or -not (Test-Path $Path)) {
        throw "$Label not found: $Path"
    }
}

$Workspace = [System.IO.Path]::GetFullPath($Workspace)
$PortableDir = Join-Path $Workspace "unlimited-ocr-portable"
$ThirdpartyDir = Join-Path $Workspace "thirdparty"
$EnvFile = Join-Path $Workspace "uocr-runtime-env.ps1"

if (Test-Path $EnvFile) {
    . $EnvFile
}

if (-not $env:UOCR_LLAMA_BIN) {
    $env:UOCR_LLAMA_BIN = Resolve-FirstExisting @(
        (Join-Path $ThirdpartyDir "llama.cpp\build\bin\Release\llama-uocr-parity.exe"),
        (Join-Path $ThirdpartyDir "llama.cpp\build\bin\llama-uocr-parity.exe")
    )
}
if (-not $env:UOCR_MODEL) {
    $env:UOCR_MODEL = Join-Path $ThirdpartyDir "uocr-gguf\Unlimited-OCR-Q4_K_M.gguf"
}
if (-not $env:UOCR_MMPROJ) {
    $env:UOCR_MMPROJ = Join-Path $ThirdpartyDir "uocr-gguf\mmproj-Unlimited-OCR-F16.gguf"
}

Require-File "portable repo" $PortableDir
Require-File "native runner" $env:UOCR_LLAMA_BIN
Require-File "model" $env:UOCR_MODEL
Require-File "mmproj" $env:UOCR_MMPROJ

if (-not (Get-Command uv -ErrorAction SilentlyContinue)) {
    throw "uv is not on PATH."
}

if ($Smoke) {
    if (-not $Image) {
        $Image = Join-Path $Workspace "dataset\sc-02.png"
    }
    Require-File "smoke image" $Image
    $uvArgs = @(
        "run", "--project", $PortableDir,
        "baidu-uocr-client",
        "--smoke",
        "--image", $Image,
        "--profile", $Profile,
        "--max-tokens", [string] $MaxTokens
    )
}
else {
    $uvArgs = @(
        "run", "--project", $PortableDir,
        "baidu-uocr-client",
        "--host", $HostName,
        "--port", [string] $Port
    )
}

Write-Host "+ uv $($uvArgs -join ' ')"
& uv @uvArgs
exit $LASTEXITCODE
