#!/usr/bin/env bash
set -euo pipefail

repo_root_arg=""
workspace_arg=""
host_name="127.0.0.1"
port="7861"
smoke=0
image=""
profile="best-zero-empty-q4"
max_tokens="64"

usage() {
    cat <<'EOF'
Usage: scripts/mac/run-demo.sh [options]

Options:
  --repo-root PATH       Portable repo root. Defaults to this script's repo.
  --workspace PATH       Legacy workspace path. Uses PATH or PATH/unlimited-ocr-portable.
  --host HOST            Gradio host. Default: 127.0.0.1.
  --port PORT            Gradio port. Default: 7861.
  --smoke                Run one OCR smoke instead of launching Gradio.
  --image PATH           Smoke image path.
  --profile KEY          best-zero-empty-q4 or experimental-exact-prefill-q4.
  --max-tokens N         Smoke max tokens. Default: 64.
  -h, --help             Show this help.
EOF
}

die() {
    echo "error: $*" >&2
    exit 1
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

absolute_existing_path() {
    local path="$1"
    local dir base
    dir="$(cd "$(dirname "$path")" && pwd)"
    base="$(basename "$path")"
    printf '%s/%s\n' "$dir" "$base"
}

resolve_first_existing() {
    local path
    for path in "$@"; do
        if [[ -e "$path" ]]; then
            absolute_existing_path "$path"
            return 0
        fi
    done
    return 1
}

require_path() {
    local label="$1"
    local path="$2"
    if [[ -z "$path" || ! -e "$path" ]]; then
        die "$label not found: $path"
    fi
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
        --host|--host-name)
            [[ $# -ge 2 ]] || die "$1 requires a value"
            host_name="$2"
            shift 2
            ;;
        --port)
            [[ $# -ge 2 ]] || die "--port requires a value"
            port="$2"
            shift 2
            ;;
        --smoke)
            smoke=1
            shift
            ;;
        --image)
            [[ $# -ge 2 ]] || die "--image requires a value"
            image="$2"
            shift 2
            ;;
        --profile)
            [[ $# -ge 2 ]] || die "--profile requires a value"
            profile="$2"
            shift 2
            ;;
        --max-tokens)
            [[ $# -ge 2 ]] || die "--max-tokens requires a value"
            max_tokens="$2"
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

case "$profile" in
    best-zero-empty-q4|experimental-exact-prefill-q4)
        ;;
    *)
        die "Unknown profile: $profile"
        ;;
esac

repo_root="$(resolve_portable_root "$repo_root_arg" "$workspace_arg")"
thirdparty_dir="$repo_root/thirdparty"
models_dir="$repo_root/models"
env_file="$repo_root/uocr-runtime-env.sh"

if [[ -f "$env_file" ]]; then
    # shellcheck disable=SC1090
    source "$env_file"
fi

if [[ -z "${UOCR_LLAMA_BIN:-}" ]]; then
    UOCR_LLAMA_BIN="$(resolve_first_existing \
        "$thirdparty_dir/llama.cpp/build/bin/llama-uocr-parity" \
        "$thirdparty_dir/llama.cpp/build/tools/mtmd/llama-uocr-parity" \
        "$thirdparty_dir/llama.cpp/build/bin/Release/llama-uocr-parity" \
        "$thirdparty_dir/llama.cpp/build/Release/llama-uocr-parity" || true)"
fi
if [[ -z "${UOCR_MODEL:-}" ]]; then
    UOCR_MODEL="$(resolve_first_existing \
        "$models_dir/Unlimited-OCR-Q4_K_M.gguf" \
        "$thirdparty_dir/uocr-gguf/Unlimited-OCR-Q4_K_M.gguf" || true)"
fi
if [[ -z "${UOCR_MMPROJ:-}" ]]; then
    UOCR_MMPROJ="$(resolve_first_existing \
        "$models_dir/mmproj-Unlimited-OCR-F16.gguf" \
        "$thirdparty_dir/uocr-gguf/mmproj-Unlimited-OCR-F16.gguf" || true)"
fi

export UOCR_LLAMA_BIN
export UOCR_MODEL
export UOCR_MMPROJ

require_path "portable pyproject" "$repo_root/pyproject.toml"
require_path "native runner" "$UOCR_LLAMA_BIN"
require_path "model" "$UOCR_MODEL"
require_path "mmproj" "$UOCR_MMPROJ"

if ! command -v uv >/dev/null 2>&1; then
    die "uv is not on PATH."
fi

if ((smoke)); then
    if [[ -z "$image" ]]; then
        image="$(resolve_first_existing "$repo_root/dataset/sc-02.png" "$repo_root/../dataset/sc-02.png" || true)"
    fi
    require_path "smoke image" "$image"
    uv_args=(run --project "$repo_root" baidu-uocr-client --smoke --image "$image" --profile "$profile" --max-tokens "$max_tokens")
else
    uv_args=(run --project "$repo_root" baidu-uocr-client --host "$host_name" --port "$port")
fi

printf '+ uv'
printf ' %q' "${uv_args[@]}"
printf '\n'
exec uv "${uv_args[@]}"
