[CmdletBinding()]
param(
    [string] $RepoRoot = "",
    [string] $Workspace = "",
    [string] $ModelRepo = "sahilchachra/Unlimited-OCR-GGUF",
    [string[]] $Models = @("Unlimited-OCR-Q4_K_M.gguf"),
    [switch] $IncludeDiagnostics,
    [Alias("-doctor")]
    [switch] $Doctor,
    [switch] $SkipSubmoduleUpdate,
    [switch] $SkipModelDownload,
    [switch] $ForceModelDownload,
    [ValidateSet("download", "build", "auto")]
    [string] $RuntimeSource = "download",
    [string] $RuntimeVersion = "latest",
    [string] $RuntimeRepo = "bangonkali/baidu-unlimited-ocr-portable",
    [switch] $SkipRuntimeDownload,
    [switch] $ForceRuntimeDownload,
    [switch] $SkipPythonSync,
    [switch] $SkipBuild,
    [string] $Generator = "",
    [string] $CudaArchitectures = "75-virtual;80-virtual;86-real;89-real;90-virtual;120a-real;121a-real",
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
            throw "Command failed with exit code ${LASTEXITCODE}: $File $($Arguments -join ' ')"
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

function Invoke-CommandProbe {
    param(
        [string] $File,
        [string[]] $Arguments,
        [string] $WorkingDirectory
    )

    Push-Location $WorkingDirectory
    try {
        $previousErrorActionPreference = $ErrorActionPreference
        try {
            $ErrorActionPreference = "Continue"
            $output = & $File @Arguments 2>&1
            $exitCode = $LASTEXITCODE
        }
        finally {
            $ErrorActionPreference = $previousErrorActionPreference
        }
        return [pscustomobject]@{
            Ok = ($exitCode -eq 0)
            ExitCode = $exitCode
            Text = (($output | Out-String).Trim())
        }
    }
    catch {
        return [pscustomobject]@{
            Ok = $false
            ExitCode = -1
            Text = $_.Exception.Message
        }
    }
    finally {
        Pop-Location
    }
}

function Add-DoctorResult {
    param(
        [System.Collections.Generic.List[object]] $Results,
        [string] $Name,
        [string] $Status,
        [string] $Detail
    )

    $Results.Add([pscustomobject]@{
        Name = $Name
        Status = $Status
        Detail = $Detail
    }) | Out-Null
}

function Show-DoctorResults {
    param([System.Collections.Generic.List[object]] $Results)

    $failures = 0
    $warnings = 0
    foreach ($item in $Results) {
        if ($item.Status -eq "FAIL") {
            $failures += 1
        }
        elseif ($item.Status -eq "WARN") {
            $warnings += 1
        }
        Write-Host ("[{0}] {1}" -f $item.Status, $item.Name)
        if ($item.Detail) {
            Write-Host ("      {0}" -f $item.Detail)
        }
    }

    Write-Host ""
    if ($failures -gt 0) {
        throw "Doctor found $failures blocking issue(s) and $warnings warning(s)."
    }
    Write-Host "Doctor found 0 blocking issue(s) and $warnings warning(s)."
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
        $previousErrorActionPreference = $ErrorActionPreference
        try {
            $ErrorActionPreference = "Continue"
            Write-Host "cl.exe:     $((cl.exe 2>&1 | Select-Object -First 1) -join ' ')"
        }
        finally {
            $ErrorActionPreference = $previousErrorActionPreference
        }
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

function Test-UsableFile {
    param([string] $Path)
    if (-not (Test-Path $Path)) {
        return $false
    }
    $item = Get-Item $Path
    return $item.Length -gt 0
}

function Get-ModelFileNames {
    param(
        [string[]] $Models,
        [bool] $IncludeDiagnostics
    )

    $files = @("mmproj-Unlimited-OCR-F16.gguf") + $Models
    if ($IncludeDiagnostics) {
        $files += @(
            "Unlimited-OCR-Q5_K_M.gguf",
            "Unlimited-OCR-Q6_K.gguf",
            "Unlimited-OCR-BF16.gguf"
        )
    }
    return $files | Select-Object -Unique
}

function Get-MissingModelFiles {
    param(
        [string] $ModelDir,
        [string[]] $Files
    )

    return @($Files | Where-Object {
        -not (Test-UsableFile (Join-Path $ModelDir $_))
    })
}

function Invoke-Doctor {
    param(
        [string] $RepoRoot,
        [string] $ThirdpartyDir,
        [string] $LlamaDir,
        [string] $ModelDir,
        [string] $ModelRepo,
        [string[]] $Models,
        [bool] $IncludeDiagnostics,
        [string] $BuildDir,
        [bool] $ForceModelDownload,
        [bool] $NeedModelDownload,
        [bool] $NeedBuild,
        [bool] $NeedRuntimeDownload
    )

    Write-Step "Running portable build doctor"
    $results = [System.Collections.Generic.List[object]]::new()

    if (Test-Path (Join-Path $RepoRoot "pyproject.toml")) {
        Add-DoctorResult $results "portable repo root" "OK" $RepoRoot
    }
    else {
        Add-DoctorResult $results "portable repo root" "FAIL" "pyproject.toml is missing at $RepoRoot."
    }

    if (Test-Path (Join-Path $RepoRoot "uv.lock")) {
        Add-DoctorResult $results "uv.lock" "OK" "Pinned dependency lockfile found."
    }
    else {
        Add-DoctorResult $results "uv.lock" "FAIL" "uv.lock is missing; setup uses uv sync --frozen."
    }

    if (Test-Path (Join-Path $RepoRoot ".gitmodules")) {
        Add-DoctorResult $results "git submodule manifest" "OK" ".gitmodules found."
    }
    else {
        Add-DoctorResult $results "git submodule manifest" "FAIL" ".gitmodules is missing."
    }

    if (Test-Path (Join-Path $LlamaDir "CMakeLists.txt")) {
        Add-DoctorResult $results "llama.cpp submodule" "OK" $LlamaDir
    }
    else {
        Add-DoctorResult $results "llama.cpp submodule" "FAIL" "Missing at $LlamaDir; run git submodule update --init --recursive."
    }

    if (Test-Path $ModelDir) {
        Add-DoctorResult $results "model directory" "OK" $ModelDir
    }
    else {
        Add-DoctorResult $results "model directory" "WARN" "$ModelDir is missing; setup-build.ps1 creates it."
    }

    $runtimeProbe = Invoke-CommandProbe `
        -File "uv" `
        -Arguments @("run", "--project", $RepoRoot, "python", (Join-Path $RepoRoot "scripts\install_runtime.py"), "detect", "--repo-root", $RepoRoot) `
        -WorkingDirectory $RepoRoot
    $runtimeDetail = if ($runtimeProbe.Text) { ($runtimeProbe.Text -split "`r?`n" | Select-Object -First 1) -join "" } else { "runtime detection returned no output" }
    if ($runtimeProbe.Ok) {
        Add-DoctorResult $results "runtime platform" "OK" $runtimeDetail
    }
    elseif ($NeedRuntimeDownload) {
        Add-DoctorResult $results "runtime platform" "FAIL" $runtimeDetail
    }
    else {
        Add-DoctorResult $results "runtime platform" "WARN" $runtimeDetail
    }

    $toolChecks = @(
        @{ Name = "git"; Args = @("--version"); Reason = "clone/update git submodules" },
        @{ Name = "uv"; Args = @("--version"); Reason = "run Python release tooling" }
    )
    if ($NeedModelDownload) {
        $toolChecks += @{ Name = "hf"; Args = @("--version"); Reason = "download GGUF assets from Hugging Face" }
    }
    if ($NeedBuild) {
        $toolChecks += @(
            @{ Name = "cmake"; Args = @("--version"); Reason = "configure and build llama.cpp" },
            @{ Name = "cl.exe"; Args = @(); Reason = "MSVC C/C++ compiler from Visual Studio Developer PowerShell"; PresenceOnly = $true },
            @{ Name = "nvcc"; Args = @("--version"); Reason = "CUDA compiler for GGML_CUDA" },
            @{ Name = "nvidia-smi"; Args = @("--query-gpu=name,driver_version,memory.total", "--format=csv,noheader"); Reason = "verify NVIDIA driver/GPU visibility" }
        )
    }

    foreach ($tool in $toolChecks) {
        if (-not (Test-Tool $tool.Name)) {
            Add-DoctorResult $results $tool.Name "FAIL" "Missing: $($tool.Reason)."
            continue
        }

        if ($tool.ContainsKey("PresenceOnly") -and $tool.PresenceOnly) {
            $command = Get-Command $tool.Name -ErrorAction SilentlyContinue
            Add-DoctorResult $results $tool.Name "OK" $command.Source
            continue
        }

        $probe = Invoke-CommandProbe -File $tool.Name -Arguments $tool.Args -WorkingDirectory $RepoRoot
        $detail = if ($probe.Text) { ($probe.Text -split "`r?`n" | Select-Object -First 1) -join "" } else { "Command exited $($probe.ExitCode)." }
        if ($probe.Ok) {
            Add-DoctorResult $results $tool.Name "OK" $detail
        }
        else {
            Add-DoctorResult $results $tool.Name "FAIL" "Command failed with exit code $($probe.ExitCode): $detail"
        }
    }

    if ($NeedBuild) {
        if ($env:VSCMD_VER -or $env:VSINSTALLDIR) {
            Add-DoctorResult $results "Visual Studio Developer PowerShell" "OK" "VSCMD_VER=$env:VSCMD_VER VSINSTALLDIR=$env:VSINSTALLDIR"
        }
        else {
            Add-DoctorResult $results "Visual Studio Developer PowerShell" "WARN" "Environment variables are not set. Use a Developer PowerShell before building."
        }
    }

    if ($NeedBuild -and (Test-Tool "nvcc")) {
        $nvccProbe = Invoke-CommandProbe -File "nvcc" -Arguments @("--version") -WorkingDirectory $RepoRoot
        if ($nvccProbe.Ok -and ($nvccProbe.Text -match "13\.3" -or $nvccProbe.Text -match "cuda_13\.3")) {
            Add-DoctorResult $results "CUDA target version" "OK" "CUDA 13.3 detected."
        }
        elseif ($nvccProbe.Ok) {
            Add-DoctorResult $results "CUDA target version" "WARN" "Expected CUDA 13.3 for the Windows validation target; current nvcc differs."
        }
    }

    if (Test-Tool "git") {
        $gitProbe = Invoke-CommandProbe -File "git" -Arguments @("-C", $RepoRoot, "submodule", "status", "--recursive") -WorkingDirectory $RepoRoot
        if ($gitProbe.Ok) {
            Add-DoctorResult $results "git submodule status" "OK" (($gitProbe.Text -split "`r?`n" | Select-Object -First 1) -join "")
        }
        else {
            Add-DoctorResult $results "git submodule status" "FAIL" "git submodule status failed: $($gitProbe.Text)"
        }
    }

    if (Test-Tool "uv") {
        $uvProbe = Invoke-CommandProbe -File "uv" -Arguments @("sync", "--frozen", "--dry-run") -WorkingDirectory $RepoRoot
        if ($uvProbe.Ok) {
            Add-DoctorResult $results "uv frozen dry-run" "OK" "Dependency lock can be resolved without writing."
        }
        else {
            Add-DoctorResult $results "uv frozen dry-run" "FAIL" "uv sync --frozen --dry-run failed: $($uvProbe.Text)"
        }
    }

    $requiredFiles = @(Get-ModelFileNames -Models $Models -IncludeDiagnostics $IncludeDiagnostics)
    $missingModelFiles = @(Get-MissingModelFiles -ModelDir $ModelDir -Files $requiredFiles)
    $needsHfAuth = $NeedModelDownload -and ($ForceModelDownload -or $missingModelFiles.Count -gt 0)

    if ($needsHfAuth) {
        if (Test-Tool "hf") {
            $hfProbe = Invoke-CommandProbe -File "hf" -Arguments @("auth", "whoami") -WorkingDirectory $RepoRoot
            if ($hfProbe.Ok) {
                Add-DoctorResult $results "Hugging Face auth" "OK" (($hfProbe.Text -split "`r?`n" | Select-Object -First 1) -join "")
            }
            else {
                Add-DoctorResult $results "Hugging Face auth" "FAIL" "hf auth whoami failed with exit code $($hfProbe.ExitCode): $($hfProbe.Text)"
            }
        }
    }
    elseif ($NeedModelDownload) {
        Add-DoctorResult $results "Hugging Face auth" "OK" "All requested model assets are already present; no download authentication needed."
    }

    foreach ($file in $requiredFiles) {
        $path = Join-Path $ModelDir $file
        if (Test-UsableFile $path) {
            Add-DoctorResult $results "model asset $file" "OK" $path
        }
        else {
            Add-DoctorResult $results "model asset $file" "WARN" "Missing or empty at $path; setup-build.ps1 downloads it from $ModelRepo."
        }
    }

    if ($NeedRuntimeDownload) {
        foreach ($exe in @("llama-uocr-parity", "llama-mtmd-cli", "llama-server")) {
            $path = Find-DownloadedExe -RepoRoot $RepoRoot -Name $exe
            if ($path) {
                Add-DoctorResult $results "downloaded runtime $exe" "OK" $path
            }
            else {
                Add-DoctorResult $results "downloaded runtime $exe" "WARN" "$exe.exe is not installed yet; setup-build.ps1 downloads it from GitHub."
            }
        }
        $ffiPath = Find-DownloadedFile -RepoRoot $RepoRoot -Name "uocr-ffi.dll"
        if ($ffiPath) {
            Add-DoctorResult $results "downloaded runtime uocr-ffi.dll" "OK" $ffiPath
        }
        else {
            Add-DoctorResult $results "downloaded runtime uocr-ffi.dll" "WARN" "uocr-ffi.dll is not installed yet; setup-build.ps1 downloads it from GitHub."
        }
    }

    if ($NeedBuild) {
        foreach ($exe in @("llama-uocr-parity", "llama-mtmd-cli", "llama-server")) {
            try {
                $path = Find-BuiltExe -BuildDir $BuildDir -Name $exe
                Add-DoctorResult $results "build output $exe" "OK" $path
            }
            catch {
                Add-DoctorResult $results "build output $exe" "WARN" "$exe.exe is not built yet; setup-build.ps1 builds it."
            }
        }
        try {
            $ffiPath = Find-BuiltFile -BuildDir $BuildDir -Name "uocr-ffi.dll"
            Add-DoctorResult $results "build output uocr-ffi.dll" "OK" $ffiPath
        }
        catch {
            Add-DoctorResult $results "build output uocr-ffi.dll" "WARN" "uocr-ffi.dll is not built yet; setup-build.ps1 builds it."
        }
    }

    Show-DoctorResults $results
}

function Download-HfFile {
    param(
        [string] $Repo,
        [string] $FileName,
        [string] $TargetDir,
        [bool] $Force,
        [string] $WorkingDirectory
    )
    $target = Join-Path $TargetDir $FileName
    if (-not $Force -and (Test-UsableFile $target)) {
        Write-Host "Already cached locally: $target"
        return
    }

    $args = @(
        "download",
        $Repo,
        $FileName,
        "--local-dir", $TargetDir
    )
    if ($Force) {
        $args += "--force-download"
    }
    Invoke-Checked -File "hf" -Arguments $args -WorkingDirectory $WorkingDirectory

    if (-not (Test-UsableFile $target)) {
        throw "Downloaded model asset is missing or empty: $target"
    }
}

function Find-BuiltExe {
    param(
        [string] $BuildDir,
        [string] $Name
    )
    $match = Get-ChildItem -Path $BuildDir -Recurse -Filter "$Name.exe" -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (-not $match) {
        throw "Built executable not found under ${BuildDir}: $Name.exe"
    }
    return $match.FullName
}

function Find-BuiltFile {
    param(
        [string] $BuildDir,
        [string] $Name
    )
    $match = Get-ChildItem -Path $BuildDir -Recurse -Filter $Name -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (-not $match) {
        throw "Built file not found under ${BuildDir}: $Name"
    }
    return $match.FullName
}

function Find-DownloadedExe {
    param(
        [string] $RepoRoot,
        [string] $Name
    )
    $runtimeRoot = Join-Path $RepoRoot "thirdparty\uocr-runtime"
    if (-not (Test-Path $runtimeRoot)) {
        return ""
    }
    $match = Get-ChildItem -Path $runtimeRoot -Recurse -Filter "$Name.exe" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match "\\bin\\" } |
        Select-Object -First 1
    if (-not $match) {
        return ""
    }
    return $match.FullName
}

function Find-DownloadedFile {
    param(
        [string] $RepoRoot,
        [string] $Name
    )
    $runtimeRoot = Join-Path $RepoRoot "thirdparty\uocr-runtime"
    if (-not (Test-Path $runtimeRoot)) {
        return ""
    }
    $match = Get-ChildItem -Path $runtimeRoot -Recurse -Filter $Name -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match "\\bin\\" } |
        Select-Object -First 1
    if (-not $match) {
        return ""
    }
    return $match.FullName
}

function Invoke-RuntimeInstaller {
    param(
        [string] $RepoRoot,
        [string] $RuntimeRepo,
        [string] $RuntimeVersion,
        [bool] $ForceRuntimeDownload
    )

    $args = @(
        "run", "--project", $RepoRoot,
        "python", (Join-Path $RepoRoot "scripts\install_runtime.py"),
        "install",
        "--repo-root", $RepoRoot,
        "--runtime-repo", $RuntimeRepo,
        "--runtime-version", $RuntimeVersion,
        "--print-env", "powershell"
    )
    if ($ForceRuntimeDownload) {
        $args += "--force"
    }

    Push-Location $RepoRoot
    try {
        $output = & uv @args
        if ($LASTEXITCODE -ne 0) {
            throw "Runtime installer failed with exit code $LASTEXITCODE."
        }
        return (($output | Out-String).Trim())
    }
    finally {
        Pop-Location
    }
}

$RepoRoot = Resolve-PortableRoot -ExplicitRepoRoot $RepoRoot -LegacyWorkspace $Workspace
$ThirdpartyDir = Join-Path $RepoRoot "thirdparty"
$LlamaDir = Join-Path $ThirdpartyDir "llama.cpp"
$ModelDir = Join-Path $RepoRoot "models"
$BuildDir = Join-Path $LlamaDir "build"
$NeedRuntimeDownload = (($RuntimeSource -ne "build") -and (-not $SkipRuntimeDownload))
$NeedBuildNow = (($RuntimeSource -eq "build") -and (-not $SkipBuild))

if ($Doctor) {
    Invoke-Doctor `
        -RepoRoot $RepoRoot `
        -ThirdpartyDir $ThirdpartyDir `
        -LlamaDir $LlamaDir `
        -ModelDir $ModelDir `
        -ModelRepo $ModelRepo `
        -Models $Models `
        -IncludeDiagnostics:$IncludeDiagnostics `
        -BuildDir $BuildDir `
        -ForceModelDownload:$ForceModelDownload `
        -NeedModelDownload:(-not $SkipModelDownload) `
        -NeedBuild:$NeedBuildNow `
        -NeedRuntimeDownload:$NeedRuntimeDownload
    return
}

Write-Step "Checking required tools"
Assert-Tooling -NeedHf:(-not $SkipModelDownload) -NeedBuild:$NeedBuildNow
Show-ToolVersions

if ($NeedBuildNow -and -not $env:VSCMD_VER -and -not $env:VSINSTALLDIR) {
    Write-Warning "This does not look like a Visual Studio Developer PowerShell. Start Visual Studio 2026 Developer PowerShell v18.8.0-insiders before building."
}

if ($NeedBuildNow -and (Test-Tool "nvcc")) {
    $nvccText = (& nvcc --version) -join "`n"
    if ($nvccText -notmatch "13\.3" -and $nvccText -notmatch "cuda_13\.3") {
        Write-Warning "Expected CUDA 13.3 for the Windows validation target. Current nvcc output:`n$nvccText"
    }
}

Write-Step "Preparing portable directories"
New-Item -ItemType Directory -Force -Path $ThirdpartyDir, $ModelDir | Out-Null

if ($RuntimeSource -eq "build" -or $RuntimeSource -eq "auto") {
    if (-not $SkipSubmoduleUpdate) {
        Write-Step "Initializing git submodules"
        Invoke-Checked -File "git" -Arguments @("-C", $RepoRoot, "submodule", "sync", "--recursive") -WorkingDirectory $RepoRoot
        Invoke-Checked -File "git" -Arguments @("-C", $RepoRoot, "submodule", "update", "--init", "--recursive") -WorkingDirectory $RepoRoot
    }

    if (-not (Test-Path (Join-Path $LlamaDir "CMakeLists.txt"))) {
        if ($RuntimeSource -eq "build") {
            throw "llama.cpp submodule is missing at $LlamaDir. Clone with --recursive or rerun without -SkipSubmoduleUpdate."
        }
        Write-Host "llama.cpp submodule is not initialized; auto mode can still use a prebuilt runtime."
    }
    else {
        Write-Step "Submodule status"
        git -C $RepoRoot submodule status --recursive
    }
}

if (-not $SkipPythonSync) {
    Write-Step "Syncing Python dependencies"
    Invoke-Checked -File "uv" -Arguments @("sync", "--frozen") -WorkingDirectory $RepoRoot
}

if (-not $SkipModelDownload) {
    $files = @(Get-ModelFileNames -Models $Models -IncludeDiagnostics $IncludeDiagnostics)
    $missingModelFiles = @(Get-MissingModelFiles -ModelDir $ModelDir -Files $files)
    if ($ForceModelDownload -or $missingModelFiles.Count -gt 0) {
        Write-Step "Checking Hugging Face authentication"
        Invoke-Checked -File "hf" -Arguments @("auth", "whoami") -WorkingDirectory $RepoRoot
    }
    else {
        Write-Step "Checking Hugging Face authentication"
        Write-Host "All requested GGUF assets are already present; skipping Hugging Face authentication."
    }

    Write-Step "Downloading GGUF assets"
    $files | ForEach-Object {
        Download-HfFile `
            -Repo $ModelRepo `
            -FileName $_ `
            -TargetDir $ModelDir `
            -Force:$ForceModelDownload `
            -WorkingDirectory $RepoRoot
    }
}

$RuntimeSourceActual = $RuntimeSource

if ($RuntimeSource -eq "download" -or $RuntimeSource -eq "auto") {
    if ($SkipRuntimeDownload) {
        if ($RuntimeSource -eq "download") {
            Write-Step "Using existing prebuilt runtime"
            if (-not $env:UOCR_LLAMA_BIN) {
                $env:UOCR_LLAMA_BIN = Find-DownloadedExe -RepoRoot $RepoRoot -Name "llama-uocr-parity"
            }
            if (-not $env:UOCR_LLAMA_MTMD_BIN) {
                $env:UOCR_LLAMA_MTMD_BIN = Find-DownloadedExe -RepoRoot $RepoRoot -Name "llama-mtmd-cli"
            }
            if (-not $env:UOCR_LLAMA_SERVER_BIN) {
                $env:UOCR_LLAMA_SERVER_BIN = Find-DownloadedExe -RepoRoot $RepoRoot -Name "llama-server"
            }
            if (-not $env:UOCR_FFI_LIB) {
                $env:UOCR_FFI_LIB = Find-DownloadedFile -RepoRoot $RepoRoot -Name "uocr-ffi.dll"
            }
        }
        else {
            $RuntimeSourceActual = "build"
        }
    }
    else {
        Write-Step "Installing prebuilt native runtime"
        try {
            $runtimeExports = Invoke-RuntimeInstaller `
                -RepoRoot $RepoRoot `
                -RuntimeRepo $RuntimeRepo `
                -RuntimeVersion $RuntimeVersion `
                -ForceRuntimeDownload:$ForceRuntimeDownload
            Invoke-Expression $runtimeExports
            $RuntimeSourceActual = "download"
        }
        catch {
            if ($RuntimeSource -eq "download") {
                throw "Prebuilt runtime download failed. Rerun with -RuntimeSource build to compile locally. $($_.Exception.Message)"
            }
            Write-Warning "Prebuilt runtime download failed; falling back to local build. $($_.Exception.Message)"
            $RuntimeSourceActual = "build"
        }
    }
}

if ($RuntimeSourceActual -eq "build") {
    Assert-Tooling -NeedHf:$false -NeedBuild:(-not $SkipBuild)
    if (-not (Test-Path (Join-Path $LlamaDir "CMakeLists.txt"))) {
        throw "llama.cpp submodule is missing at $LlamaDir. Clone with --recursive or rerun without -SkipSubmoduleUpdate."
    }
}

if ($RuntimeSourceActual -eq "build" -and -not $SkipBuild) {
    Write-Step "Configuring llama.cpp CUDA build"
    $configureArgs = @(
        "-B", $BuildDir,
        "-S", $LlamaDir,
        "-DGGML_CUDA=ON",
        "-DLLAMA_OPENSSL=OFF",
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
        "--target", "llama-mtmd-cli", "llama-uocr-parity", "llama-server", "uocr-ffi",
        "--parallel"
    ) -WorkingDirectory $RepoRoot
}

Write-Step "Validating outputs"
if ($RuntimeSourceActual -eq "build") {
    $uocrExe = Find-BuiltExe -BuildDir $BuildDir -Name "llama-uocr-parity"
    $mtmdExe = Find-BuiltExe -BuildDir $BuildDir -Name "llama-mtmd-cli"
    $serverExe = Find-BuiltExe -BuildDir $BuildDir -Name "llama-server"
    $ffiLib = Find-BuiltFile -BuildDir $BuildDir -Name "uocr-ffi.dll"
}
else {
    $uocrExe = $env:UOCR_LLAMA_BIN
    $mtmdExe = if ($env:UOCR_LLAMA_MTMD_BIN) { $env:UOCR_LLAMA_MTMD_BIN } else { Find-DownloadedExe -RepoRoot $RepoRoot -Name "llama-mtmd-cli" }
    $serverExe = if ($env:UOCR_LLAMA_SERVER_BIN) { $env:UOCR_LLAMA_SERVER_BIN } else { Find-DownloadedExe -RepoRoot $RepoRoot -Name "llama-server" }
    $ffiLib = if ($env:UOCR_FFI_LIB) { $env:UOCR_FFI_LIB } else { Find-DownloadedFile -RepoRoot $RepoRoot -Name "uocr-ffi.dll" }
}
$modelPath = Join-Path $ModelDir ($Models[0])
$mmprojPath = Join-Path $ModelDir "mmproj-Unlimited-OCR-F16.gguf"

foreach ($path in @($uocrExe, $mtmdExe, $serverExe, $ffiLib, $modelPath, $mmprojPath)) {
    if (-not (Test-Path $path)) {
        throw "Expected file is missing: $path"
    }
    Write-Host "OK $path"
}

Write-Step "Writing runtime environment"
$envFile = Join-Path $RepoRoot "uocr-runtime-env.ps1"
$envLines = [System.Collections.Generic.List[string]]::new()
$envLines.Add("# Generated by scripts/windows/setup-build.ps1") | Out-Null
function Add-EnvLine {
    param(
        [System.Collections.Generic.List[string]] $Lines,
        [string] $Name,
        [string] $Value
    )
    if ($null -eq $Value -or $Value -eq "") {
        return
    }
    $escaped = $Value.Replace("'", "''")
    $Lines.Add('$env:' + $Name + " = '$escaped'") | Out-Null
}
Add-EnvLine -Lines $envLines -Name "UOCR_RUNTIME_SOURCE" -Value $RuntimeSourceActual
Add-EnvLine -Lines $envLines -Name "UOCR_RUNTIME_LABEL" -Value $env:UOCR_RUNTIME_LABEL
Add-EnvLine -Lines $envLines -Name "UOCR_RUNTIME_VERSION" -Value $env:UOCR_RUNTIME_VERSION
Add-EnvLine -Lines $envLines -Name "UOCR_RUNTIME_ROOT" -Value $env:UOCR_RUNTIME_ROOT
Add-EnvLine -Lines $envLines -Name "UOCR_LLAMA_BIN" -Value $uocrExe
Add-EnvLine -Lines $envLines -Name "UOCR_LLAMA_MTMD_BIN" -Value $mtmdExe
Add-EnvLine -Lines $envLines -Name "UOCR_LLAMA_SERVER_BIN" -Value $serverExe
Add-EnvLine -Lines $envLines -Name "UOCR_FFI_LIB" -Value $ffiLib
Add-EnvLine -Lines $envLines -Name "UOCR_MODEL" -Value $modelPath
Add-EnvLine -Lines $envLines -Name "UOCR_MMPROJ" -Value $mmprojPath
Add-EnvLine -Lines $envLines -Name "UOCR_CLIENT_HOST" -Value "127.0.0.1"
Add-EnvLine -Lines $envLines -Name "UOCR_CLIENT_PORT" -Value "7861"
Set-Content -Path $envFile -Value $envLines -Encoding UTF8
Write-Host "Wrote $envFile"

Write-Step "Next commands"
Write-Host ". '$envFile'"
Write-Host "uv run --project '$RepoRoot' baidu-uocr-client --help"
Write-Host "Trapo releases are packaged through scripts/package_trapo_workbench.py."
