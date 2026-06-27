[CmdletBinding()]
param(
    [string] $RepoRoot = "",
    [string] $Workspace = "",
    [string] $ModelRepo = "sahilchachra/Unlimited-OCR-GGUF",
    [string[]] $Models = @("Unlimited-OCR-Q4_K_M.gguf"),
    [switch] $IncludeDiagnostics,
    [switch] $SkipSubmoduleUpdate,
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
        throw "Portable repo root not found: $full. Run this script from a cloned baidu-unlimited-ocr-portable repo or pass -RepoRoot."
    }
    return $full
}

function Invoke-Checked {
    param(
        [string] $File,
        [string[]] $Arguments,
        [string] $WorkingDirectory
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

function Test-Tool {
    param([string] $Name)
    return [bool] (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Assert-Tooling {
    param(
        [bool] $NeedHf,
        [bool] $NeedBuild
    )

    $requirements = @(
        @{ Name = "git"; Reason = "clone/update git submodules" },
        @{ Name = "uv"; Reason = "create and run the portable Python environment" }
    )
    if ($NeedHf) {
        $requirements += @{ Name = "hf"; Reason = "download GGUF model assets from Hugging Face" }
    }
    if ($NeedBuild) {
        $requirements += @(
            @{ Name = "cmake"; Reason = "configure and build llama.cpp" },
            @{ Name = "cl.exe"; Reason = "MSVC C/C++ compiler from Visual Studio Developer PowerShell" },
            @{ Name = "nvcc"; Reason = "CUDA compiler for GGML_CUDA" },
            @{ Name = "nvidia-smi"; Reason = "verify NVIDIA driver/GPU visibility" }
        )
    }

    $missing = @()
    foreach ($requirement in $requirements) {
        if (-not (Test-Tool $requirement.Name)) {
            $missing += $requirement
        }
    }

    if ($missing.Count -gt 0) {
        Write-Host "Missing required tools:"
        foreach ($item in $missing) {
            Write-Host "  - $($item.Name): $($item.Reason)"
        }
        throw "Install the missing tools, open Visual Studio 2026 Developer PowerShell v18.8.0-insiders, then rerun this script."
    }
}

function Show-ToolVersions {
    if (Test-Tool "git") {
        Write-Host "git:        $((git --version) -join ' ')"
    }
    if (Test-Tool "cmake") {
        Write-Host "cmake:      $((cmake --version | Select-Object -First 1) -join ' ')"
    }
    if (Test-Tool "uv") {
        Write-Host "uv:         $((uv --version) -join ' ')"
    }
    if (Test-Tool "hf") {
        Write-Host "hf:         $((hf --version) -join ' ')"
    }
    if (Test-Tool "cl.exe") {
        Write-Host "cl.exe:     $((cl.exe 2>&1 | Select-Object -First 1) -join ' ')"
    }
    if (Test-Tool "nvcc") {
        Write-Host "nvcc:"
        nvcc --version
    }
    if (Test-Tool "nvidia-smi") {
        Write-Host "nvidia-smi:"
        nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader
    }
}

function Download-HfFile {
    param(
        [string] $Repo,
        [string] $FileName,
        [string] $TargetDir,
        [string] $WorkingDirectory
    )
    $target = Join-Path $TargetDir $FileName
    if (Test-Path $target) {
        Write-Host "Already present: $target"
        return
    }
    Invoke-Checked -File "hf" -Arguments @("download", $Repo, $FileName, "--local-dir", $TargetDir) -WorkingDirectory $WorkingDirectory
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

$RepoRoot = Resolve-PortableRoot -ExplicitRepoRoot $RepoRoot -LegacyWorkspace $Workspace
$ThirdpartyDir = Join-Path $RepoRoot "thirdparty"
$LlamaDir = Join-Path $ThirdpartyDir "llama.cpp"
$ModelDir = Join-Path $ThirdpartyDir "uocr-gguf"
$BuildDir = Join-Path $LlamaDir "build"

Write-Step "Checking required tools"
Assert-Tooling -NeedHf:(-not $SkipModelDownload) -NeedBuild:(-not $SkipBuild)
Show-ToolVersions

if (-not $env:VSCMD_VER -and -not $env:VSINSTALLDIR) {
    Write-Warning "This does not look like a Visual Studio Developer PowerShell. Start Visual Studio 2026 Developer PowerShell v18.8.0-insiders before building."
}

if (Test-Tool "nvcc") {
    $nvccText = (& nvcc --version) -join "`n"
    if ($nvccText -notmatch "13\.3" -and $nvccText -notmatch "cuda_13\.3") {
        Write-Warning "Expected CUDA 13.3 for the Windows validation target. Current nvcc output:`n$nvccText"
    }
}

Write-Step "Preparing portable thirdparty directory"
New-Item -ItemType Directory -Force -Path $ThirdpartyDir, $ModelDir | Out-Null

if (-not $SkipSubmoduleUpdate) {
    Write-Step "Initializing git submodules"
    Invoke-Checked -File "git" -Arguments @("-C", $RepoRoot, "submodule", "sync", "--recursive") -WorkingDirectory $RepoRoot
    Invoke-Checked -File "git" -Arguments @("-C", $RepoRoot, "submodule", "update", "--init", "--recursive") -WorkingDirectory $RepoRoot
}

if (-not (Test-Path (Join-Path $LlamaDir "CMakeLists.txt"))) {
    throw "llama.cpp submodule is missing at $LlamaDir. Clone with --recursive or rerun without -SkipSubmoduleUpdate."
}

Write-Step "Submodule status"
git -C $RepoRoot submodule status --recursive

if (-not $SkipModelDownload) {
    Write-Step "Checking Hugging Face authentication"
    Invoke-Checked -File "hf" -Arguments @("auth", "whoami") -WorkingDirectory $RepoRoot

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
        Download-HfFile -Repo $ModelRepo -FileName $_ -TargetDir $ModelDir -WorkingDirectory $RepoRoot
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
    Invoke-Checked -File "cmake" -Arguments $configureArgs -WorkingDirectory $RepoRoot

    Write-Step "Building native executables"
    Invoke-Checked -File "cmake" -Arguments @(
        "--build", $BuildDir,
        "--config", $Config,
        "--target", "llama-mtmd-cli", "llama-uocr-parity", "llama-server",
        "--parallel"
    ) -WorkingDirectory $RepoRoot
}

Write-Step "Validating outputs"
$uocrExe = Find-BuiltExe -BuildDir $BuildDir -Name "llama-uocr-parity"
$mtmdExe = Find-BuiltExe -BuildDir $BuildDir -Name "llama-mtmd-cli"
$serverExe = Find-BuiltExe -BuildDir $BuildDir -Name "llama-server"
$modelPath = Join-Path $ModelDir "Unlimited-OCR-Q4_K_M.gguf"
$mmprojPath = Join-Path $ModelDir "mmproj-Unlimited-OCR-F16.gguf"

foreach ($path in @($uocrExe, $mtmdExe, $serverExe, $modelPath, $mmprojPath)) {
    if (-not (Test-Path $path)) {
        throw "Expected file is missing: $path"
    }
    Write-Host "OK $path"
}

Write-Step "Writing runtime environment"
$envFile = Join-Path $RepoRoot "uocr-runtime-env.ps1"
$envLines = @(
    "# Generated by scripts/windows/setup-build.ps1",
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
Write-Host "uv run --project '$RepoRoot' baidu-uocr-client --help"
Write-Host "& '$RepoRoot\scripts\windows\run-demo.ps1' -Smoke -Image '<path-to-test-image>'"
Write-Host "& '$RepoRoot\scripts\windows\run-demo.ps1'"
