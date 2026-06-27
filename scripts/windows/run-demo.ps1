[CmdletBinding()]
param(
    [string] $RepoRoot = "",
    [string] $Workspace = "",
    [string] $HostName = "127.0.0.1",
    [int] $Port = 7861,
    [switch] $Smoke,
    [string] $Image = "",
    [ValidateSet("best-zero-empty-q4", "experimental-exact-prefill-q4")]
    [string] $Profile = "best-zero-empty-q4",
    [int] $MaxTokens = 64,
    [ValidateSet("ffi", "executable")]
    [string] $RuntimeBackend = "ffi"
)

Set-StrictMode -Version 3.0
$ErrorActionPreference = "Stop"

function Resolve-PortableRoot {
    param(
        [string] $ExplicitRepoRoot,
        [string] $LegacyWorkspace
    )

    $scriptRoot = Split-Path -Parent $PSCommandPath
    $defaultRoot = Join-Path $scriptRoot "..\.."

    if ($ExplicitRepoRoot) {
        $candidate = $ExplicitRepoRoot
    }
    elseif ($LegacyWorkspace) {
        if (Test-Path (Join-Path $LegacyWorkspace "pyproject.toml")) {
            $candidate = $LegacyWorkspace
        }
        else {
            $candidate = Join-Path $LegacyWorkspace "unlimited-ocr-portable"
        }
    }
    else {
        $candidate = $defaultRoot
    }

    $full = [System.IO.Path]::GetFullPath($candidate)
    if (-not (Test-Path (Join-Path $full "pyproject.toml"))) {
        throw "Portable repo root not found: $full. Run this script from the cloned baidu-unlimited-ocr-portable repo or pass -RepoRoot."
    }
    return $full
}

function Resolve-FirstExisting {
    param([string[]] $Paths)
    foreach ($path in $Paths) {
        if (Test-Path $path) {
            return [System.IO.Path]::GetFullPath($path)
        }
    }
    return ""
}

function Require-Path {
    param(
        [string] $Label,
        [string] $Path
    )
    if (-not $Path -or -not (Test-Path $Path)) {
        throw "$Label not found: $Path"
    }
}

$RepoRoot = Resolve-PortableRoot -ExplicitRepoRoot $RepoRoot -LegacyWorkspace $Workspace
$ThirdpartyDir = Join-Path $RepoRoot "thirdparty"
$ModelsDir = Join-Path $RepoRoot "models"
$EnvFile = Join-Path $RepoRoot "uocr-runtime-env.ps1"

if (Test-Path $EnvFile) {
    . $EnvFile
}

if (-not $env:UOCR_LLAMA_BIN) {
    $env:UOCR_LLAMA_BIN = Resolve-FirstExisting @(
        (Join-Path $ThirdpartyDir "uocr-runtime\windows-x86_64-cuda13\bin\llama-uocr-parity.exe"),
        (Join-Path $ThirdpartyDir "llama.cpp\build\bin\Release\llama-uocr-parity.exe"),
        (Join-Path $ThirdpartyDir "llama.cpp\build\bin\llama-uocr-parity.exe")
    )
}
if (-not $env:UOCR_LLAMA_SERVER_BIN) {
    $env:UOCR_LLAMA_SERVER_BIN = Resolve-FirstExisting @(
        (Join-Path $ThirdpartyDir "uocr-runtime\windows-x86_64-cuda13\bin\llama-server.exe"),
        (Join-Path $ThirdpartyDir "llama.cpp\build\bin\Release\llama-server.exe"),
        (Join-Path $ThirdpartyDir "llama.cpp\build\bin\llama-server.exe")
    )
}
if (-not $env:UOCR_MODEL) {
    $env:UOCR_MODEL = Resolve-FirstExisting @(
        (Join-Path $ModelsDir "Unlimited-OCR-Q4_K_M.gguf"),
        (Join-Path $ThirdpartyDir "uocr-gguf\Unlimited-OCR-Q4_K_M.gguf")
    )
}
if (-not $env:UOCR_MMPROJ) {
    $env:UOCR_MMPROJ = Resolve-FirstExisting @(
        (Join-Path $ModelsDir "mmproj-Unlimited-OCR-F16.gguf"),
        (Join-Path $ThirdpartyDir "uocr-gguf\mmproj-Unlimited-OCR-F16.gguf")
    )
}
$env:UOCR_RUNTIME_BACKEND = $RuntimeBackend

Require-Path "portable pyproject" (Join-Path $RepoRoot "pyproject.toml")
Require-Path "native runner" $env:UOCR_LLAMA_BIN
Require-Path "native server" $env:UOCR_LLAMA_SERVER_BIN
Require-Path "model" $env:UOCR_MODEL
Require-Path "mmproj" $env:UOCR_MMPROJ

if (-not (Get-Command uv -ErrorAction SilentlyContinue)) {
    throw "uv is not on PATH."
}

if ($Smoke) {
    if (-not $Image) {
        $Image = Resolve-FirstExisting @(
            (Join-Path $RepoRoot "dataset\sc-02.png"),
            (Join-Path $RepoRoot "..\dataset\sc-02.png")
        )
    }
    Require-Path "smoke image" $Image
    $uvArgs = @(
        "run", "--project", $RepoRoot,
        "baidu-uocr-client",
        "--smoke",
        "--image", $Image,
        "--profile", $Profile,
        "--max-tokens", [string] $MaxTokens,
        "--runtime-backend", $RuntimeBackend
    )
}
else {
    $uvArgs = @(
        "run", "--project", $RepoRoot,
        "baidu-uocr-client",
        "--host", $HostName,
        "--port", [string] $Port
    )
}

Write-Host "+ uv $($uvArgs -join ' ')"
& uv @uvArgs
exit $LASTEXITCODE
