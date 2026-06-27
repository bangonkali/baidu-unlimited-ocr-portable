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
cuda_architectures="75-virtual;80-virtual;86-real;89-real;90-virtual;120a-real;121a-real"
config="Release"

usage() {
    cat <<'EOF'
Usage: scripts/linux/setup-build.sh [options]

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
  --generator NAME             Pass a CMake generator, for example "Ninja".
  --cuda-architectures VALUE   Pass CMAKE_CUDA_ARCHITECTURES for source builds. Default includes RTX 5090 / sm_120.
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

test_usable_file() {
    [[ -f "$1" && -s "$1" ]]
}

model_files=()
missing_model_files=()

add_unique_model_file() {
    local file="$1"
    local existing
    for existing in "${model_files[@]}"; do
        [[ "$existing" == "$file" ]] && return
    done
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
    [[ -n "$match" ]] || return 1
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
        command_exists nvcc || missing+=("nvcc: CUDA compiler for GGML_CUDA")
        command_exists nvidia-smi || missing+=("nvidia-smi: verify NVIDIA driver/GPU visibility")
    fi

    if ((${#missing[@]} > 0)); then
        echo "Missing required tools:"
        for item in "${missing[@]}"; do
            printf '  - %s\n' "$item"
        done
        die "Install the missing tools, then rerun this script."
    fi
}

show_tool_versions() {
    printf 'system:     %s\n' "$(uname -sm)"
    command_exists git && printf 'git:        %s\n' "$(git --version)"
    command_exists cmake && printf 'cmake:      %s\n' "$(cmake --version | sed -n '1p')"
    command_exists uv && printf 'uv:         %s\n' "$(uv --version)"
    command_exists hf && printf 'hf:         %s\n' "$(hf --version 2>/dev/null | sed -n '1p')"
    if command_exists nvcc; then
        printf 'nvcc:       %s\n' "$(nvcc --version | sed -n '1p')"
    fi
    if command_exists nvidia-smi; then
        nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader 2>/dev/null || true
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

run_runtime_installer() {
    local repo_root="$1"
    local args
    args=(run --project "$repo_root" python "$repo_root/scripts/install_runtime.py" install
        --repo-root "$repo_root"
        --runtime-repo "$runtime_repo"
        --runtime-version "$runtime_version"
        --print-env sh)
    if ((force_runtime_download)); then
        args+=(--force)
    fi
    uv "${args[@]}"
}

run_doctor() {
    local repo_root="$1"
    local model_dir="$2"
    local build_dir="$3"
    local need_model_download="$4"
    local need_build="$5"
    local need_runtime_download="$6"
    local status=0

    write_step "Running portable Linux build doctor"
    [[ -f "$repo_root/pyproject.toml" ]] && echo "[OK] portable repo root: $repo_root" || { echo "[FAIL] portable repo root missing pyproject.toml"; status=1; }
    [[ -f "$repo_root/uv.lock" ]] && echo "[OK] uv.lock" || { echo "[FAIL] uv.lock missing"; status=1; }

    if command_exists uv; then
        if runtime_probe="$(cd "$repo_root" && uv run --project "$repo_root" python "$repo_root/scripts/install_runtime.py" detect --repo-root "$repo_root" 2>&1)"; then
            echo "[OK] runtime platform: $(first_line "$runtime_probe")"
        elif ((need_runtime_download)); then
            echo "[FAIL] runtime platform: $(first_line "$runtime_probe")"
            status=1
        else
            echo "[WARN] runtime platform: $(first_line "$runtime_probe")"
        fi
    else
        echo "[FAIL] uv: missing required command"
        status=1
    fi

    assert_tooling "$need_model_download" "$need_build" || status=1

    collect_model_files
    collect_missing_model_files "$model_dir"
    for file in "${model_files[@]}"; do
        if test_usable_file "$model_dir/$file"; then
            echo "[OK] model asset $file: $model_dir/$file"
        else
            echo "[WARN] model asset $file: missing; setup-build.sh downloads it from $model_repo"
        fi
    done

    if ((need_runtime_download)); then
        for exe in llama-uocr-parity llama-mtmd-cli llama-server; do
            if path="$(find_downloaded_exe "$repo_root" "$exe" 2>/dev/null)"; then
                echo "[OK] downloaded runtime $exe: $path"
            else
                echo "[WARN] downloaded runtime $exe: not installed yet"
            fi
        done
        if path="$(find_downloaded_file "$repo_root" "libuocr-ffi.so" 2>/dev/null)"; then
            echo "[OK] downloaded runtime libuocr-ffi.so: $path"
        else
            echo "[WARN] downloaded runtime libuocr-ffi.so: not installed yet"
        fi
    fi

    if ((need_build)); then
        for exe in llama-uocr-parity llama-mtmd-cli llama-server; do
            if path="$(find_built_exe "$build_dir" "$exe" 2>/dev/null)"; then
                echo "[OK] build output $exe: $path"
            else
                echo "[WARN] build output $exe: not built yet"
            fi
        done
        if path="$(find_built_exe "$build_dir" "libuocr-ffi.so" 2>/dev/null)"; then
            echo "[OK] build output libuocr-ffi.so: $path"
        else
            echo "[WARN] build output libuocr-ffi.so: not built yet"
        fi
    fi

    ((status == 0)) || die "Doctor found blocking issue(s)."
    echo "Doctor found 0 blocking issue(s)."
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
        --cuda-architectures)
            [[ $# -ge 2 ]] || die "--cuda-architectures requires a value"
            cuda_architectures="$2"
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

case "$runtime_source" in
    download|build|auto) ;;
    *) die "--runtime-source must be download, build, or auto." ;;
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
    run_doctor "$repo_root" "$model_dir" "$build_dir" "$((1 - skip_model_download))" "$need_build_now" "$need_runtime_download"
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
    else
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
            UOCR_FFI_LIB="${UOCR_FFI_LIB:-$(find_downloaded_file "$repo_root" "libuocr-ffi.so" || true)}"
        else
            runtime_source_actual="build"
        fi
    else
        write_step "Installing prebuilt native runtime"
        set +e
        runtime_exports="$(run_runtime_installer "$repo_root")"
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
    [[ -f "$llama_dir/CMakeLists.txt" ]] || die "llama.cpp submodule is missing at $llama_dir. Clone with --recursive or rerun without --skip-submodule-update."
fi

if [[ "$runtime_source_actual" == "build" && "$skip_build" == "0" ]]; then
    write_step "Configuring llama.cpp CUDA build"
    configure_args=(-B "$build_dir" -S "$llama_dir" -DGGML_CUDA=ON "-DCMAKE_BUILD_TYPE=$config")
    if [[ -n "$generator" ]]; then
        configure_args=(-G "$generator" "${configure_args[@]}")
    fi
    if [[ -n "$cuda_architectures" ]]; then
        configure_args+=("-DCMAKE_CUDA_ARCHITECTURES=$cuda_architectures")
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
    ffi_lib="$(find_built_exe "$build_dir" "libuocr-ffi.so")"
else
    uocr_exe="${UOCR_LLAMA_BIN:-}"
    mtmd_exe="${UOCR_LLAMA_MTMD_BIN:-$(find_downloaded_exe "$repo_root" "llama-mtmd-cli" || true)}"
    server_exe="${UOCR_LLAMA_SERVER_BIN:-$(find_downloaded_exe "$repo_root" "llama-server" || true)}"
    ffi_lib="${UOCR_FFI_LIB:-$(find_downloaded_file "$repo_root" "libuocr-ffi.so" || true)}"
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
    echo "# Generated by scripts/linux/setup-build.sh"
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
printf '%q --smoke --image %q\n' "$repo_root/scripts/linux/run-demo.sh" "<path-to-test-image>"
printf '%q\n' "$repo_root/scripts/linux/run-demo.sh"
