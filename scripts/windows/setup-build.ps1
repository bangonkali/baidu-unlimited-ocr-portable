[CmdletBinding()]
param(
    [string] $Workspace = "C:\uocr",
    [string] $LlamaRepo = "git@github.com:bangonkali/llama.cpp-baidu-unlimited-ocr.git",
    [string] $LlamaBranch = "uocr-deepseek-ocr-parity",
    [string] $PortableRepo = "git@github.com:bangonkali/baidu-unlimited-ocr-portable.git",
    [string] $PortableBranch = "main",
    [string] $ModelRepo = "sahilchachra/Unlimited-OCR-GGUF",
    [string[]] $Models = @("Unlimited-OCR-Q4_K_M.gguf"),
    [switch] $IncludeDiagnostics,
    [switch] $SkipClone,
    [switch] $SkipModelDownload,
    [switch] $SkipBuild,
    [string] $Generator = "",
    [string] $CudaArchitectures = "",
    [string] $Config = "Release"
)

Set-StrictMode -Version 3.0
$ErrorActionPreference = "Stop"

function Write-Step {
    param([string] $Message)
    Write-Host ""
    Write-Host "==> $Message"
}

function Require-Command {
    param([string] $Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command not found on PATH: $Name"
    }
}

function Invoke-Checked {
    param(
        [string] $File,
        [string[]] $Arguments,
        [string] $WorkingDirectory = (Get-Location).Path
    )
    Push-Location $WorkingDirectory
    try {
        Write-Host "+ $File $($Arguments -join ' ')"
        & $File @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "Command failed with exit code $LASTEXITCODE: $File $($Arguments -join ' ')"
        }
    }
    finally {
        Pop-Location
    }
}

function Sync-Repo {
    param(
        [string] $RepoUrl,
        [string] $Branch,
        [string] $Path
    )
    if (-not (Test-Path $Path)) {
        Invoke-Checked git @("clone", "-b", $Branch, $RepoUrl, $Path) $Workspace
        return
    }

    Invoke-Checked git @("-C", $Path, "fetch", "--all", "--prune") $Workspace
    Invoke-Checked git @("-C", $Path, "checkout", $Branch) $Workspace
    Invoke-Checked git @("-C", $Path, "pull", "--ff-only") $Workspace
}

function Download-HfFile {
    param(
        [string] $Repo,
        [string] $FileName,
        [string] $TargetDir
    )
    $target = Join-Path $TargetDir $FileName
    if (Test-Path $target) {
        Write-Host "Already present: $target"
        return
    }
    Invoke-Checked hf @("download", $Repo, $FileName, "--local-dir", $TargetDir) $Workspace
}

function Find-BuiltExe {
    param(
        [string] $BuildDir,
        [string] $Name
    )
    $match = Get-ChildItem -Path $BuildDir -Recurse -Filter "$Name.exe" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (-not $match) {
        throw "Built executable not found under $BuildDir: $Name.exe"
    }
    return $match.FullName
}

$Workspace = [System.IO.Path]::GetFullPath($Workspace)
$ThirdpartyDir = Join-Path $Workspace "thirdparty"
$LlamaDir = Join-Path $ThirdpartyDir "llama.cpp"
$PortableDir = Join-Path $Workspace "unlimited-ocr-portable"
$ModelDir = Join-Path $ThirdpartyDir "uocr-gguf"
$BuildDir = Join-Path $LlamaDir "build"

Write-Step "Checking tools"
Require-Command git
Require-Command cmake
Require-Command uv
Require-Command hf
Require-Command nvcc
Require-Command nvidia-smi

if (-not $env:VSCMD_VER -and -not $env:VSINSTALLDIR) {
    Write-Warning "This does not look like a Visual Studio Developer PowerShell. Start Visual Studio 2026 Developer PowerShell v18.8.0-insiders before building."
}

$nvccText = (& nvcc --version) -join "`n"
if ($nvccText -notmatch "13\.3" -and $nvccText -notmatch "cuda_13\.3") {
    Write-Warning "Expected CUDA 13.3 for the Windows validation target. Current nvcc output:`n$nvccText"
}

Write-Step "Creating workspace"
New-Item -ItemType Directory -Force -Path $Workspace, $ThirdpartyDir, $ModelDir | Out-Null

if (-not $SkipClone) {
    Write-Step "Cloning or updating repos"
    Sync-Repo $LlamaRepo $LlamaBranch $LlamaDir
    Sync-Repo $PortableRepo $PortableBranch $PortableDir
}

if (-not $SkipModelDownload) {
    Write-Step "Checking Hugging Face authentication"
    Invoke-Checked hf @("auth", "whoami") $Workspace

    Write-Step "Downloading GGUF assets"
    $files = @("mmproj-Unlimited-OCR-F16.gguf") + $Models
    if ($IncludeDiagnostics) {
        $files += @(
            "Unlimited-OCR-Q5_K_M.gguf",
            "Unlimited-OCR-Q6_K.gguf",
            "Unlimited-OCR-BF16.gguf"
        )
    }
    $files | Select-Object -Unique | ForEach-Object {
        Download-HfFile $ModelRepo $_ $ModelDir
    }
}

if (-not $SkipBuild) {
    Write-Step "Configuring llama.cpp CUDA build"
    $configureArgs = @(
        "-B", $BuildDir,
        "-S", $LlamaDir,
        "-DGGML_CUDA=ON",
        "-DCMAKE_BUILD_TYPE=$Config"
    )
    if ($Generator) {
        $configureArgs = @("-G", $Generator) + $configureArgs
    }
    if ($CudaArchitectures) {
        $configureArgs += "-DCMAKE_CUDA_ARCHITECTURES=$CudaArchitectures"
    }
    Invoke-Checked cmake $configureArgs $Workspace

    Write-Step "Building native executables"
    Invoke-Checked cmake @(
        "--build", $BuildDir,
        "--config", $Config,
        "--target", "llama-mtmd-cli", "llama-uocr-parity", "llama-server",
        "--parallel"
    ) $Workspace
}

Write-Step "Validating outputs"
$uocrExe = Find-BuiltExe $BuildDir "llama-uocr-parity"
$mtmdExe = Find-BuiltExe $BuildDir "llama-mtmd-cli"
$serverExe = Find-BuiltExe $BuildDir "llama-server"
$modelPath = Join-Path $ModelDir "Unlimited-OCR-Q4_K_M.gguf"
$mmprojPath = Join-Path $ModelDir "mmproj-Unlimited-OCR-F16.gguf"

foreach ($path in @($uocrExe, $mtmdExe, $serverExe, $modelPath, $mmprojPath)) {
    if (-not (Test-Path $path)) {
        throw "Expected file is missing: $path"
    }
    Write-Host "OK $path"
}

Write-Step "Writing runtime environment"
$envFile = Join-Path $Workspace "uocr-runtime-env.ps1"
$envLines = @(
    "# Generated by unlimited-ocr-portable/scripts/windows/setup-build.ps1",
    "`$env:UOCR_LLAMA_BIN = '$uocrExe'",
    "`$env:UOCR_MODEL = '$modelPath'",
    "`$env:UOCR_MMPROJ = '$mmprojPath'",
    "`$env:UOCR_CLIENT_HOST = '127.0.0.1'",
    "`$env:UOCR_CLIENT_PORT = '7861'"
)
Set-Content -Path $envFile -Value $envLines -Encoding UTF8
Write-Host "Wrote $envFile"

Write-Step "Next commands"
Write-Host ". '$envFile'"
Write-Host "uv run --project '$PortableDir' baidu-uocr-client --help"
Write-Host "& '$PortableDir\scripts\windows\run-demo.ps1' -Workspace '$Workspace' -Smoke -Image '<path-to-test-image>'"
Write-Host "& '$PortableDir\scripts\windows\run-demo.ps1' -Workspace '$Workspace'"
