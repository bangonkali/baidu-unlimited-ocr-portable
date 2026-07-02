#!/usr/bin/env bash
set -euo pipefail

repo_root_arg=""
workspace_arg=""
model_repo="sahilchachra/Unlimited-OCR-GGUF"
models=("Unlimited-OCR-Q4_K_M.gguf")
models_overridden=0
include_diagnostics=0
doctor=0
skip_submodule_update=0
skip_model_download=0
force_model_download=0
skip_runtime_download=0
force_runtime_download=0
skip_python_sync=0
skip_build=0
runtime_source="download"
runtime_version="latest"
runtime_repo="bangonkali/baidu-unlimited-ocr-portable"
generator=""
config="Release"

doctor_names=()
doctor_statuses=()
doctor_details=()
probe_status=0
probe_text=""
model_files=()
missing_model_files=()

usage() {
    cat <<'EOF'
Usage: scripts/mac/setup-build.sh [options]

Options:
  --repo-root PATH             Portable repo root. Defaults to this script's repo.
  --workspace PATH             Legacy workspace path. Uses PATH or PATH/unlimited-ocr-portable.
  --model-repo REPO            Hugging Face repo. Default: sahilchachra/Unlimited-OCR-GGUF.
  --model FILE                 Model GGUF to download/use. Repeatable. Default: Unlimited-OCR-Q4_K_M.gguf.
  --include-diagnostics        Also download Q5_K_M, Q6_K, and BF16 GGUFs.
  --doctor                     Run preflight checks without downloading, building, or writing files.
  --skip-submodule-update      Do not run git submodule update.
  --skip-model-download        Do not check Hugging Face auth or download model assets.
  --force-model-download       Redownload model files even when local files are non-empty.
  --runtime-source MODE        download, build, or auto. Default: download.
  --runtime-version VERSION    GitHub Release tag/version for prebuilt runtime. Default: latest.
  --runtime-repo OWNER/REPO    GitHub repo for runtime release assets. Default: bangonkali/baidu-unlimited-ocr-portable.
  --skip-runtime-download      Do not download a prebuilt runtime; validate the existing runtime path.
  --force-runtime-download     Redownload and reinstall prebuilt runtime files.
  --skip-python-sync           Do not run uv sync --frozen.
  --skip-build                 Do not configure/build llama.cpp when --runtime-source build or auto fallback builds.
  --generator NAME             Pass a CMake generator, for example "Ninja" or "Xcode".
  --config NAME                CMake build config. Default: Release.
  -h, --help                   Show this help.
EOF
}

die() {
    echo "error: $*" >&2
    exit 1
}

write_step() {
    printf '\n==> %s\n' "$1"
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

first_line() {
    printf '%s\n' "$1" | sed -n '1p'
}

resolve_portable_root() {
    local explicit_repo_root="$1"
    local legacy_workspace="$2"
    local script_root default_root candidate

    script_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    default_root="$(cd "$script_root/../.." && pwd)"

    if [[ -n "$explicit_repo_root" ]]; then
        candidate="$explicit_repo_root"
    elif [[ -n "$legacy_workspace" ]]; then
        if [[ -f "$legacy_workspace/pyproject.toml" ]]; then
            candidate="$legacy_workspace"
        else
            candidate="$legacy_workspace/unlimited-ocr-portable"
        fi
    else
        candidate="$default_root"
    fi

    if ! candidate="$(cd "$candidate" 2>/dev/null && pwd)"; then
        die "Portable repo root not found. Run from a cloned baidu-unlimited-ocr-portable repo or pass --repo-root."
    fi
    if [[ ! -f "$candidate/pyproject.toml" ]]; then
        die "Portable repo root not found: $candidate. Missing pyproject.toml."
    fi
    printf '%s\n' "$candidate"
}

invoke_checked() {
    local working_directory="$1"
    shift

    printf '+'
    printf ' %q' "$@"
    printf '\n'
    if ! (cd "$working_directory" && "$@"); then
        die "Command failed: $*"
    fi
}

invoke_probe() {
    local working_directory="$1"
    shift

    set +e
    probe_text="$(cd "$working_directory" && "$@" 2>&1)"
    probe_status=$?
    set -e
}

add_doctor_result() {
    doctor_names+=("$1")
    doctor_statuses+=("$2")
    doctor_details+=("$3")
}

show_doctor_results() {
    local failures=0
    local warnings=0
    local i

    for ((i = 0; i < ${#doctor_names[@]}; i++)); do
        if [[ "${doctor_statuses[$i]}" == "FAIL" ]]; then
            failures=$((failures + 1))
        elif [[ "${doctor_statuses[$i]}" == "WARN" ]]; then
            warnings=$((warnings + 1))
        fi
        printf '[%s] %s\n' "${doctor_statuses[$i]}" "${doctor_names[$i]}"
        if [[ -n "${doctor_details[$i]}" ]]; then
            printf '      %s\n' "${doctor_details[$i]}"
        fi
    done

    printf '\n'
    if ((failures > 0)); then
        die "Doctor found $failures blocking issue(s) and $warnings warning(s)."
    fi
    printf 'Doctor found 0 blocking issue(s) and %d warning(s).\n' "$warnings"
}

test_usable_file() {
    [[ -f "$1" && -s "$1" ]]
}

add_unique_model_file() {
    local file="$1"
    local existing
    if ((${#model_files[@]} > 0)); then
        for existing in "${model_files[@]}"; do
            if [[ "$existing" == "$file" ]]; then
                return
            fi
        done
    fi
    model_files+=("$file")
}

collect_model_files() {
    local model
    model_files=()
    add_unique_model_file "mmproj-Unlimited-OCR-F16.gguf"
    for model in "${models[@]}"; do
        add_unique_model_file "$model"
    done
    if ((include_diagnostics)); then
        add_unique_model_file "Unlimited-OCR-Q5_K_M.gguf"
        add_unique_model_file "Unlimited-OCR-Q6_K.gguf"
        add_unique_model_file "Unlimited-OCR-BF16.gguf"
    fi
}

collect_missing_model_files() {
    local model_dir="$1"
    local file
    missing_model_files=()
    for file in "${model_files[@]}"; do
        if ! test_usable_file "$model_dir/$file"; then
            missing_model_files+=("$file")
        fi
    done
}

find_built_exe() {
    local build_dir="$1"
    local name="$2"
    local match

    match="$(find "$build_dir" -type f -name "$name" -perm -111 2>/dev/null | head -n 1 || true)"
    if [[ -z "$match" ]]; then
        match="$(find "$build_dir" -type f -name "$name" 2>/dev/null | head -n 1 || true)"
    fi
    if [[ -z "$match" ]]; then
        die "Built executable not found under $build_dir: $name"
    fi
    printf '%s\n' "$match"
}

find_downloaded_exe() {
    local repo_root="$1"
    local name="$2"
    local match

    match="$(find "$repo_root/thirdparty/uocr-runtime" -path "*/bin/$name" -type f -perm -111 2>/dev/null | head -n 1 || true)"
    if [[ -z "$match" ]]; then
        match="$(find "$repo_root/thirdparty/uocr-runtime" -path "*/bin/$name" -type f 2>/dev/null | head -n 1 || true)"
    fi
    if [[ -z "$match" ]]; then
        return 1
    fi
    printf '%s\n' "$match"
}

find_downloaded_file() {
    local repo_root="$1"
    local name="$2"
    local match

    match="$(find "$repo_root/thirdparty/uocr-runtime" -path "*/bin/$name" -type f 2>/dev/null | head -n 1 || true)"
    [[ -n "$match" ]] || return 1
    printf '%s\n' "$match"
}

run_runtime_installer() {
    local repo_root="$1"
    local env_style="$2"
    local args

    args=(run --project "$repo_root" python "$repo_root/scripts/install_runtime.py" install
        --repo-root "$repo_root"
        --runtime-repo "$runtime_repo"
        --runtime-version "$runtime_version"
        --print-env "$env_style")
    if ((force_runtime_download)); then
        args+=(--force)
    fi
    uv "${args[@]}"
}

assert_tooling() {
    local need_hf="$1"
    local need_build="$2"
    local missing=()
    local item

    command_exists git || missing+=("git: clone/update git submodules")
    command_exists uv || missing+=("uv: create and run the portable Python environment")
    if ((need_hf)); then
        command_exists hf || missing+=("hf: download GGUF model assets from Hugging Face")
    fi
    if ((need_build)); then
        command_exists cmake || missing+=("cmake: configure and build llama.cpp")
        command_exists xcode-select || missing+=("xcode-select: verify Xcode command line tools")
        command_exists xcrun || missing+=("xcrun: locate Apple clang and macOS SDK")
    fi

    if ((${#missing[@]} > 0)); then
        echo "Missing required tools:"
        for item in "${missing[@]}"; do
            printf '  - %s\n' "$item"
        done
        die "Install the missing tools, then rerun this script."
    fi

    if ((need_build)); then
        xcode-select -p >/dev/null 2>&1 || die "Xcode command line tools are not selected. Run: xcode-select --install"
        xcrun --find clang >/dev/null 2>&1 || die "Apple clang was not found through xcrun. Run: xcode-select --install"
    fi
}

show_tool_versions() {
    printf 'system:     %s\n' "$(uname -sm)"
    if command_exists git; then
        printf 'git:        %s\n' "$(git --version)"
    fi
    if command_exists cmake; then
        printf 'cmake:      %s\n' "$(cmake --version | sed -n '1p')"
    fi
    if command_exists uv; then
        printf 'uv:         %s\n' "$(uv --version)"
    fi
    if command_exists hf; then
        printf 'hf:         %s\n' "$(hf --version 2>/dev/null | sed -n '1p')"
    fi
    if command_exists xcode-select && xcode-select -p >/dev/null 2>&1; then
        printf 'xcode:      %s\n' "$(xcode-select -p)"
    fi
    if command_exists xcrun && xcrun --find clang >/dev/null 2>&1; then
        printf 'clang:      %s\n' "$(xcrun clang --version | sed -n '1p')"
        printf 'sdk:        %s\n' "$(xcrun --sdk macosx --show-sdk-path 2>/dev/null || true)"
    fi
}

download_hf_file() {
    local repo="$1"
    local file_name="$2"
    local target_dir="$3"
    local force="$4"
    local working_directory="$5"
    local target="$target_dir/$file_name"
    local args

    if [[ "$force" == "0" ]] && test_usable_file "$target"; then
        printf 'Already cached locally: %s\n' "$target"
        return
    fi

    args=(download "$repo" "$file_name" --local-dir "$target_dir")
    if [[ "$force" == "1" ]]; then
        args+=(--force-download)
    fi
    invoke_checked "$working_directory" hf "${args[@]}"

    test_usable_file "$target" || die "Downloaded model asset is missing or empty: $target"
}

run_doctor() {
    local repo_root="$1"
    local llama_dir="$2"
    local model_dir="$3"
    local build_dir="$4"
    local need_model_download="$5"
    local need_build="$6"
    local need_runtime_download="$7"
    local needs_hf_auth=0
    local tool detail file exe path runtime_probe_status runtime_probe_text

    write_step "Running portable macOS build doctor"
    doctor_names=()
    doctor_statuses=()
    doctor_details=()

    if [[ -f "$repo_root/pyproject.toml" ]]; then
        add_doctor_result "portable repo root" "OK" "$repo_root"
    else
        add_doctor_result "portable repo root" "FAIL" "pyproject.toml is missing at $repo_root."
    fi

    if [[ -f "$repo_root/uv.lock" ]]; then
        add_doctor_result "uv.lock" "OK" "Pinned dependency lockfile found."
    else
        add_doctor_result "uv.lock" "FAIL" "uv.lock is missing; setup uses uv sync --frozen."
    fi

    if [[ -f "$repo_root/.gitmodules" ]]; then
        add_doctor_result "git submodule manifest" "OK" ".gitmodules found."
    else
        add_doctor_result "git submodule manifest" "FAIL" ".gitmodules is missing."
    fi

    if [[ -f "$llama_dir/CMakeLists.txt" ]]; then
        add_doctor_result "llama.cpp submodule" "OK" "$llama_dir"
    else
        add_doctor_result "llama.cpp submodule" "FAIL" "Missing at $llama_dir; run git submodule update --init --recursive."
    fi

    if [[ -d "$model_dir" ]]; then
        add_doctor_result "model directory" "OK" "$model_dir"
    else
        add_doctor_result "model directory" "WARN" "$model_dir is missing; setup-build.sh creates it."
    fi

    set +e
    runtime_probe_text="$(cd "$repo_root" && uv run --project "$repo_root" python "$repo_root/scripts/install_runtime.py" detect --repo-root "$repo_root" 2>&1)"
    runtime_probe_status=$?
    set -e
    if [[ "$runtime_probe_status" == "0" ]]; then
        add_doctor_result "runtime platform" "OK" "$(first_line "$runtime_probe_text")"
    elif ((need_runtime_download)); then
        add_doctor_result "runtime platform" "FAIL" "$(first_line "$runtime_probe_text")"
    else
        add_doctor_result "runtime platform" "WARN" "$(first_line "$runtime_probe_text")"
    fi

    for tool in git uv; do
        if ! command_exists "$tool"; then
            add_doctor_result "$tool" "FAIL" "Missing required command."
            continue
        fi
        invoke_probe "$repo_root" "$tool" --version
        detail="$(first_line "$probe_text")"
        if [[ "$probe_status" == "0" ]]; then
            add_doctor_result "$tool" "OK" "$detail"
        else
            add_doctor_result "$tool" "FAIL" "Command failed with exit code $probe_status: $detail"
        fi
    done

    if ((need_model_download)); then
        if command_exists hf; then
            invoke_probe "$repo_root" hf --version
            detail="$(first_line "$probe_text")"
            if [[ "$probe_status" == "0" ]]; then
                add_doctor_result "hf" "OK" "$detail"
            else
                add_doctor_result "hf" "FAIL" "Command failed with exit code $probe_status: $detail"
            fi
        else
            add_doctor_result "hf" "FAIL" "Missing: download GGUF assets from Hugging Face."
        fi
    fi

    if ((need_build)); then
        if command_exists cmake; then
            invoke_probe "$repo_root" cmake --version
            detail="$(first_line "$probe_text")"
            if [[ "$probe_status" == "0" ]]; then
                add_doctor_result "cmake" "OK" "$detail"
            else
                add_doctor_result "cmake" "FAIL" "Command failed with exit code $probe_status: $detail"
            fi
        else
            add_doctor_result "cmake" "FAIL" "Missing: configure and build llama.cpp."
        fi

        if command_exists xcode-select && xcode-select -p >/dev/null 2>&1; then
            add_doctor_result "Xcode command line tools" "OK" "$(xcode-select -p)"
        else
            add_doctor_result "Xcode command line tools" "FAIL" "Run xcode-select --install."
        fi

        if command_exists xcrun && xcrun --find clang >/dev/null 2>&1; then
            add_doctor_result "Apple clang" "OK" "$(xcrun --find clang)"
            invoke_probe "$repo_root" xcrun clang --version
            detail="$(first_line "$probe_text")"
            [[ -n "$detail" ]] && add_doctor_result "clang version" "OK" "$detail"
        else
            add_doctor_result "Apple clang" "FAIL" "clang was not found through xcrun."
        fi

        if command_exists xcrun && xcrun --sdk macosx --show-sdk-path >/dev/null 2>&1; then
            add_doctor_result "macOS SDK" "OK" "$(xcrun --sdk macosx --show-sdk-path)"
        else
            add_doctor_result "macOS SDK" "FAIL" "macOS SDK was not found through xcrun."
        fi
    fi

    if command_exists git; then
        invoke_probe "$repo_root" git -C "$repo_root" submodule status --recursive
        detail="$(first_line "$probe_text")"
        if [[ "$probe_status" == "0" ]]; then
            add_doctor_result "git submodule status" "OK" "$detail"
        else
            add_doctor_result "git submodule status" "FAIL" "git submodule status failed: $detail"
        fi
    fi

    if command_exists uv; then
        invoke_probe "$repo_root" uv sync --frozen --dry-run
        detail="$(first_line "$probe_text")"
        if [[ "$probe_status" == "0" ]]; then
            add_doctor_result "uv frozen dry-run" "OK" "Dependency lock can be resolved without writing."
        else
            add_doctor_result "uv frozen dry-run" "FAIL" "uv sync --frozen --dry-run failed: $detail"
        fi
    fi

    collect_model_files
    collect_missing_model_files "$model_dir"
    if ((need_model_download)) && { ((force_model_download)) || ((${#missing_model_files[@]} > 0)); }; then
        needs_hf_auth=1
    fi

    if ((needs_hf_auth)); then
        if command_exists hf; then
            invoke_probe "$repo_root" hf auth whoami
            detail="$(first_line "$probe_text")"
            if [[ "$probe_status" == "0" ]]; then
                add_doctor_result "Hugging Face auth" "OK" "$detail"
            else
                add_doctor_result "Hugging Face auth" "FAIL" "hf auth whoami failed with exit code $probe_status: $detail"
            fi
        fi
    elif ((need_model_download)); then
        add_doctor_result "Hugging Face auth" "OK" "All requested model assets are already present; no download authentication needed."
    fi

    for file in "${model_files[@]}"; do
        path="$model_dir/$file"
        if test_usable_file "$path"; then
            add_doctor_result "model asset $file" "OK" "$path"
        else
            add_doctor_result "model asset $file" "WARN" "Missing or empty at $path; setup-build.sh downloads it from $model_repo."
        fi
    done

    if ((need_runtime_download)); then
        for exe in llama-uocr-parity llama-mtmd-cli llama-server; do
            if path="$(find_downloaded_exe "$repo_root" "$exe" 2>/dev/null)"; then
                add_doctor_result "downloaded runtime $exe" "OK" "$path"
            else
                add_doctor_result "downloaded runtime $exe" "WARN" "$exe is not installed yet; setup-build.sh downloads it from GitHub."
            fi
        done
        if path="$(find_downloaded_file "$repo_root" "libuocr-ffi.dylib" 2>/dev/null)"; then
            add_doctor_result "downloaded runtime libuocr-ffi.dylib" "OK" "$path"
        else
            add_doctor_result "downloaded runtime libuocr-ffi.dylib" "WARN" "libuocr-ffi.dylib is not installed yet; setup-build.sh downloads it from GitHub."
        fi
    fi

    if ((need_build)); then
        for exe in llama-uocr-parity llama-mtmd-cli llama-server; do
            if path="$(find_built_exe "$build_dir" "$exe" 2>/dev/null)"; then
                add_doctor_result "build output $exe" "OK" "$path"
            else
                add_doctor_result "build output $exe" "WARN" "$exe is not built yet; setup-build.sh builds it."
            fi
        done
        if path="$(find_built_exe "$build_dir" "libuocr-ffi.dylib" 2>/dev/null)"; then
            add_doctor_result "build output libuocr-ffi.dylib" "OK" "$path"
        else
            add_doctor_result "build output libuocr-ffi.dylib" "WARN" "libuocr-ffi.dylib is not built yet; setup-build.sh builds it."
        fi
    fi

    show_doctor_results
}

while (($# > 0)); do
    case "$1" in
        --repo-root)
            [[ $# -ge 2 ]] || die "--repo-root requires a value"
            repo_root_arg="$2"
            shift 2
            ;;
        --workspace)
            [[ $# -ge 2 ]] || die "--workspace requires a value"
            workspace_arg="$2"
            shift 2
            ;;
        --model-repo)
            [[ $# -ge 2 ]] || die "--model-repo requires a value"
            model_repo="$2"
            shift 2
            ;;
        --model)
            [[ $# -ge 2 ]] || die "--model requires a value"
            if ((models_overridden == 0)); then
                models=()
                models_overridden=1
            fi
            models+=("$2")
            shift 2
            ;;
        --include-diagnostics)
            include_diagnostics=1
            shift
            ;;
        --doctor|-Doctor)
            doctor=1
            shift
            ;;
        --skip-submodule-update)
            skip_submodule_update=1
            shift
            ;;
        --skip-model-download)
            skip_model_download=1
            shift
            ;;
        --force-model-download)
            force_model_download=1
            shift
            ;;
        --runtime-source)
            [[ $# -ge 2 ]] || die "--runtime-source requires a value"
            runtime_source="$2"
            shift 2
            ;;
        --runtime-version)
            [[ $# -ge 2 ]] || die "--runtime-version requires a value"
            runtime_version="$2"
            shift 2
            ;;
        --runtime-repo)
            [[ $# -ge 2 ]] || die "--runtime-repo requires a value"
            runtime_repo="$2"
            shift 2
            ;;
        --skip-runtime-download)
            skip_runtime_download=1
            shift
            ;;
        --force-runtime-download)
            force_runtime_download=1
            shift
            ;;
        --skip-python-sync)
            skip_python_sync=1
            shift
            ;;
        --skip-build)
            skip_build=1
            shift
            ;;
        --generator)
            [[ $# -ge 2 ]] || die "--generator requires a value"
            generator="$2"
            shift 2
            ;;
        --config)
            [[ $# -ge 2 ]] || die "--config requires a value"
            config="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "Unknown option: $1"
            ;;
    esac
done

if ((${#models[@]} == 0)); then
    die "At least one --model value is required."
fi

case "$runtime_source" in
    download|build|auto)
        ;;
    *)
        die "--runtime-source must be download, build, or auto."
        ;;
esac

repo_root="$(resolve_portable_root "$repo_root_arg" "$workspace_arg")"
thirdparty_dir="$repo_root/thirdparty"
llama_dir="$thirdparty_dir/llama.cpp"
model_dir="$repo_root/models"
build_dir="$llama_dir/build"
need_runtime_download=0
need_build_now=0

if [[ "$runtime_source" != "build" && "$skip_runtime_download" == "0" ]]; then
    need_runtime_download=1
fi
if [[ "$runtime_source" == "build" && "$skip_build" == "0" ]]; then
    need_build_now=1
fi

if ((doctor)); then
    run_doctor "$repo_root" "$llama_dir" "$model_dir" "$build_dir" "$((1 - skip_model_download))" "$need_build_now" "$need_runtime_download"
    exit 0
fi

write_step "Checking required tools"
assert_tooling "$((1 - skip_model_download))" "$need_build_now"
show_tool_versions

write_step "Preparing portable directories"
mkdir -p "$thirdparty_dir" "$model_dir"

if [[ "$runtime_source" == "build" || "$runtime_source" == "auto" ]]; then
if ((skip_submodule_update == 0)); then
    write_step "Initializing git submodules"
    invoke_checked "$repo_root" git -C "$repo_root" submodule sync --recursive
    invoke_checked "$repo_root" git -C "$repo_root" submodule update --init --recursive
fi

    if [[ ! -f "$llama_dir/CMakeLists.txt" ]]; then
        if [[ "$runtime_source" == "build" ]]; then
            die "llama.cpp submodule is missing at $llama_dir. Clone with --recursive or rerun without --skip-submodule-update."
        fi
        echo "llama.cpp submodule is not initialized; auto mode can still use a prebuilt runtime."
    fi

    if [[ -f "$llama_dir/CMakeLists.txt" ]]; then
        write_step "Submodule status"
        git -C "$repo_root" submodule status --recursive
    fi
fi

if ((skip_python_sync == 0)); then
    write_step "Syncing Python dependencies"
    invoke_checked "$repo_root" uv sync --frozen
fi

collect_model_files
if ((skip_model_download == 0)); then
    collect_missing_model_files "$model_dir"
    write_step "Checking Hugging Face authentication"
    if ((force_model_download)) || ((${#missing_model_files[@]} > 0)); then
        invoke_checked "$repo_root" hf auth whoami
    else
        echo "All requested GGUF assets are already present; skipping Hugging Face authentication."
    fi

    write_step "Downloading GGUF assets"
    for file in "${model_files[@]}"; do
        download_hf_file "$model_repo" "$file" "$model_dir" "$force_model_download" "$repo_root"
    done
fi

runtime_source_actual="$runtime_source"

if [[ "$runtime_source" == "download" || "$runtime_source" == "auto" ]]; then
    if ((skip_runtime_download)); then
        if [[ "$runtime_source" == "download" ]]; then
            write_step "Using existing prebuilt runtime"
            UOCR_LLAMA_BIN="${UOCR_LLAMA_BIN:-$(find_downloaded_exe "$repo_root" "llama-uocr-parity" || true)}"
            UOCR_LLAMA_MTMD_BIN="${UOCR_LLAMA_MTMD_BIN:-$(find_downloaded_exe "$repo_root" "llama-mtmd-cli" || true)}"
            UOCR_LLAMA_SERVER_BIN="${UOCR_LLAMA_SERVER_BIN:-$(find_downloaded_exe "$repo_root" "llama-server" || true)}"
            UOCR_FFI_LIB="${UOCR_FFI_LIB:-$(find_downloaded_file "$repo_root" "libuocr-ffi.dylib" || true)}"
        else
            runtime_source_actual="build"
        fi
    else
        write_step "Installing prebuilt native runtime"
        set +e
        runtime_exports="$(run_runtime_installer "$repo_root" sh)"
        runtime_status=$?
        set -e
        if [[ "$runtime_status" == "0" ]]; then
            eval "$runtime_exports"
            runtime_source_actual="download"
        elif [[ "$runtime_source" == "download" ]]; then
            die "Prebuilt runtime download failed. Rerun with --runtime-source build to compile locally."
        else
            echo "Prebuilt runtime download failed; falling back to local build."
            runtime_source_actual="build"
        fi
    fi
fi

if [[ "$runtime_source_actual" == "build" ]]; then
    assert_tooling 0 "$((1 - skip_build))"
    if [[ ! -f "$llama_dir/CMakeLists.txt" ]]; then
        die "llama.cpp submodule is missing at $llama_dir. Clone with --recursive or rerun without --skip-submodule-update."
    fi
fi

if [[ "$runtime_source_actual" == "build" && "$skip_build" == "0" ]]; then
    write_step "Configuring llama.cpp Metal build"
    configure_args=(-B "$build_dir" -S "$llama_dir" -DGGML_METAL=ON "-DCMAKE_BUILD_TYPE=$config")
    if [[ -n "$generator" ]]; then
        configure_args=(-G "$generator" "${configure_args[@]}")
    fi
    invoke_checked "$repo_root" cmake "${configure_args[@]}"

    write_step "Building native executables"
    invoke_checked "$repo_root" cmake --build "$build_dir" --config "$config" --target llama-mtmd-cli llama-uocr-parity llama-server uocr-ffi --parallel
fi

write_step "Validating outputs"
if [[ "$runtime_source_actual" == "build" ]]; then
    uocr_exe="$(find_built_exe "$build_dir" "llama-uocr-parity")"
    mtmd_exe="$(find_built_exe "$build_dir" "llama-mtmd-cli")"
    server_exe="$(find_built_exe "$build_dir" "llama-server")"
    ffi_lib="$(find_built_exe "$build_dir" "libuocr-ffi.dylib")"
else
    uocr_exe="${UOCR_LLAMA_BIN:-}"
    mtmd_exe="${UOCR_LLAMA_MTMD_BIN:-$(find_downloaded_exe "$repo_root" "llama-mtmd-cli" || true)}"
    server_exe="${UOCR_LLAMA_SERVER_BIN:-$(find_downloaded_exe "$repo_root" "llama-server" || true)}"
    ffi_lib="${UOCR_FFI_LIB:-$(find_downloaded_file "$repo_root" "libuocr-ffi.dylib" || true)}"
fi
model_path="$model_dir/${models[0]}"
mmproj_path="$model_dir/mmproj-Unlimited-OCR-F16.gguf"

for path in "$uocr_exe" "$mtmd_exe" "$server_exe" "$ffi_lib" "$model_path" "$mmproj_path"; do
    [[ -e "$path" ]] || die "Expected file is missing: $path"
    printf 'OK %s\n' "$path"
done

write_step "Writing runtime environment"
env_file="$repo_root/uocr-runtime-env.sh"
{
    echo "# Generated by scripts/mac/setup-build.sh"
    printf 'export UOCR_RUNTIME_SOURCE=%q\n' "$runtime_source_actual"
    if [[ -n "${UOCR_RUNTIME_LABEL:-}" ]]; then
        printf 'export UOCR_RUNTIME_LABEL=%q\n' "$UOCR_RUNTIME_LABEL"
    fi
    if [[ -n "${UOCR_RUNTIME_VERSION:-}" ]]; then
        printf 'export UOCR_RUNTIME_VERSION=%q\n' "$UOCR_RUNTIME_VERSION"
    fi
    if [[ -n "${UOCR_RUNTIME_ROOT:-}" ]]; then
        printf 'export UOCR_RUNTIME_ROOT=%q\n' "$UOCR_RUNTIME_ROOT"
    fi
    printf 'export UOCR_LLAMA_BIN=%q\n' "$uocr_exe"
    printf 'export UOCR_LLAMA_MTMD_BIN=%q\n' "$mtmd_exe"
    printf 'export UOCR_LLAMA_SERVER_BIN=%q\n' "$server_exe"
    printf 'export UOCR_FFI_LIB=%q\n' "$ffi_lib"
    printf 'export UOCR_MODEL=%q\n' "$model_path"
    printf 'export UOCR_MMPROJ=%q\n' "$mmproj_path"
    printf 'export UOCR_CLIENT_HOST=%q\n' "127.0.0.1"
    printf 'export UOCR_CLIENT_PORT=%q\n' "7861"
} > "$env_file"
printf 'Wrote %s\n' "$env_file"

write_step "Next commands"
printf 'source %q\n' "$env_file"
printf 'uv run --project %q baidu-uocr-client --help\n' "$repo_root"
printf '%s\n' "Trapo releases are packaged through scripts/package_trapo_workbench.py."
