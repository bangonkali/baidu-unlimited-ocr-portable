#!/usr/bin/env bash
set -euo pipefail

preset="linux-x64-workbench-ci"
version=""
runtime_version="latest"
runtime_repo="bangonkali/baidu-unlimited-ocr-portable"
runtime_platform="linux-x86_64-cuda13"
additional_runtime_platforms="linux-x86_64-cpu"
package_arch="x64"
output_dir=""
no_build=0
no_runtime_download=0
runtime_build_parallel="${UOCR_RUNTIME_BUILD_PARALLEL:-3}"

usage() {
  cat <<'EOF'
Usage: scripts/linux/package-workbench.sh [options]

Options:
  --version VERSION                    Release version, for example v0.0.31.
  --runtime-version VERSION            Runtime release tag. Default: latest.
  --runtime-repo OWNER/REPO            Runtime release repo.
  --runtime-platform PLATFORM          Primary runtime label.
  --additional-runtime-platforms LIST  Comma-separated fallback runtime labels.
  --package-arch ARCH                  Output arch label, x64 or arm64.
  --output-dir PATH                    Output directory. Default: dist.
  --preset NAME                        CMake preset.
  --no-build                           Skip CMake build and use existing uocr-server.
  --no-runtime-download                Require existing runtime under thirdparty/uocr-runtime.
  -h, --help                           Show this help.
EOF
}

die() {
  echo "error: $*" >&2
  exit 1
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
output_dir="${output_dir:-$repo_root/dist}"

while (($# > 0)); do
  case "$1" in
    --version) version="$2"; shift 2 ;;
    --runtime-version) runtime_version="$2"; shift 2 ;;
    --runtime-repo) runtime_repo="$2"; shift 2 ;;
    --runtime-platform) runtime_platform="$2"; shift 2 ;;
    --additional-runtime-platforms) additional_runtime_platforms="$2"; shift 2 ;;
    --package-arch) package_arch="$2"; shift 2 ;;
    --output-dir) output_dir="$2"; shift 2 ;;
    --preset) preset="$2"; shift 2 ;;
    --no-build) no_build=1; shift ;;
    --no-runtime-download) no_runtime_download=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown option: $1" ;;
  esac
done

if [[ "$package_arch" != "x64" && "$package_arch" != "arm64" ]]; then
  die "--package-arch must be x64 or arm64"
fi
if [[ -z "$version" ]]; then
  version="$(git -C "$repo_root" describe --tags --dirty --always 2>/dev/null || true)"
  [[ -n "$version" ]] || version="0.0.0-dev"
fi
safe_version="${version//\//-}"
safe_version="${safe_version//\\/-}"

find_server() {
  find "$repo_root/build" -type f -name uocr-server -perm -111 2>/dev/null | sort | tail -n 1
}

find_vcpkg_root() {
  local preferred="$repo_root/build/$preset/vcpkg_installed"
  if [[ -d "$preferred" ]]; then
    echo "$preferred"
    return
  fi
  find "$repo_root/build" -path '*/vcpkg_installed' -type d 2>/dev/null | sort | tail -n 1
}

copy_vcpkg_shared_libraries() {
  local destination="$1"
  local vcpkg_root
  vcpkg_root="$(find_vcpkg_root)"
  [[ -n "$vcpkg_root" ]] || return 0
  while IFS= read -r library; do
    cp -P "$library" "$destination/"
  done < <(find "$vcpkg_root" -path '*/lib/*.so*' -type f -o -path '*/lib/*.so*' -type l 2>/dev/null)
}

copy_vcpkg_copyright() {
  local package="$1"
  local destination="$2"
  local match
  match="$(find "$repo_root/build" -path "*/vcpkg_installed/*/share/$package/copyright" -type f 2>/dev/null | sort | tail -n 1)"
  [[ -n "$match" ]] || die "vcpkg copyright file was not found for $package"
  mkdir -p "$(dirname "$destination")"
  cp "$match" "$destination"
}

find_built_runtime_file() {
  local build_dir="$1"
  local name="$2"
  local match
  match="$(
    find "$build_dir" -type f -name "$name" ! -path '*/CMakeFiles/*' 2>/dev/null |
      sort |
      tail -n 1
  )"
  [[ -n "$match" ]] || die "built runtime file was not found under $build_dir: $name"
  echo "$match"
}

install_cpu_runtime_from_source() {
  local platform="$1"
  local runtime_dir="$2"
  local llama_dir="$repo_root/thirdparty/llama.cpp"
  local build_dir="$llama_dir/build-$platform"
  [[ -f "$llama_dir/CMakeLists.txt" ]] || die "llama.cpp submodule is missing; cannot build $platform runtime"

  echo "Building Linux CPU runtime for $platform" >&2
  cmake -B "$build_dir" \
    -S "$llama_dir" \
    -DGGML_NATIVE=OFF \
    -DCMAKE_BUILD_TYPE=Release >&2
  cmake --build "$build_dir" --config Release --target \
    llama-mtmd-cli \
    llama-uocr-parity \
    llama-server \
    uocr-ffi \
    --parallel "$runtime_build_parallel" >&2

  rm -rf "$runtime_dir"
  mkdir -p "$runtime_dir/bin"
  for name in llama-uocr-parity llama-mtmd-cli llama-server libuocr-ffi.so; do
    cp "$(find_built_runtime_file "$build_dir" "$name")" "$runtime_dir/bin/"
  done
  while IFS= read -r shared_library; do
    [[ "$(basename "$shared_library")" == "libuocr-ffi.so" ]] && continue
    cp "$shared_library" "$runtime_dir/bin/"
  done < <(find "$build_dir" -type f \( -name '*.so' -o -name '*.so.*' \) ! -path '*/CMakeFiles/*')
  chmod 755 "$runtime_dir/bin/llama-uocr-parity" "$runtime_dir/bin/llama-mtmd-cli" "$runtime_dir/bin/llama-server"
}

ensure_runtime_platform() {
  local platform="$1"
  local runtime_dir="$repo_root/thirdparty/uocr-runtime/$platform"
  local runtime_ffi="$runtime_dir/bin/libuocr-ffi.so"
  if [[ ! -f "$runtime_ffi" && "$no_runtime_download" == "0" ]]; then
    if ! python3 "$repo_root/scripts/install_runtime.py" install \
      --repo-root "$repo_root" \
      --install-dir "$repo_root/thirdparty/uocr-runtime" \
      --runtime-repo "$runtime_repo" \
      --runtime-version "$runtime_version" \
      --platform "$platform" \
      --skip-accelerator-probe >&2; then
      if [[ "$platform" == linux-*-cpu ]]; then
        echo "No downloadable runtime asset found for $platform; building CPU runtime from source." >&2
        install_cpu_runtime_from_source "$platform" "$runtime_dir"
      else
        return 1
      fi
    fi
  fi
  [[ -f "$runtime_ffi" ]] || die "runtime FFI library is missing: $runtime_ffi"
  echo "$runtime_dir"
}

if ((no_build == 0)); then
  (cd "$repo_root/src/uocr-client" && bun install --frozen-lockfile)
  cmake --preset "$preset" -DUOCR_VERSION="$version"
  cmake --build --preset "${preset}-release"
fi

declare -a runtime_platforms=("$runtime_platform")
if [[ -n "$additional_runtime_platforms" ]]; then
  IFS=',' read -ra extra_platforms <<< "$additional_runtime_platforms"
  for platform in "${extra_platforms[@]}"; do
    [[ -n "$platform" ]] && runtime_platforms+=("$platform")
  done
fi

declare -A runtime_dirs=()
for platform in "${runtime_platforms[@]}"; do
  runtime_dirs["$platform"]="$(ensure_runtime_platform "$platform")"
done

exe="$(find_server)"
[[ -n "$exe" ]] || die "uocr-server was not found under build/"
exe_dir="$(cd "$(dirname "$exe")" && pwd)"

stage_root="$output_dir/uocr-workbench-linux-$package_arch-$safe_version"
archive_path="$output_dir/uocr-workbench-linux-$package_arch-$safe_version.tar.gz"
sha_path="$archive_path.sha256"
rm -rf "$stage_root" "$archive_path" "$sha_path"
mkdir -p "$stage_root"

cp "$exe_dir/uocr-server" "$stage_root/uocr-server"
chmod 755 "$stage_root/uocr-server"
find "$exe_dir" -maxdepth 1 \( -name '*.so' -o -name '*.so.*' \) -type f -exec cp -P {} "$stage_root" \;
copy_vcpkg_shared_libraries "$stage_root"
[[ -d "$exe_dir/web" ]] && cp -R "$exe_dir/web" "$stage_root/web"
[[ -d "$exe_dir/openapi" ]] && cp -R "$exe_dir/openapi" "$stage_root/openapi"

mkdir -p "$stage_root/thirdparty/uocr-runtime" "$stage_root/thirdparty/libmupdf"
for platform in "${runtime_platforms[@]}"; do
  cp -R "${runtime_dirs[$platform]}" "$stage_root/thirdparty/uocr-runtime/$platform"
done
copy_vcpkg_copyright libmupdf "$stage_root/thirdparty/libmupdf/copyright"

for dir in models data cache logs config uploads; do
  mkdir -p "$stage_root/$dir"
done

cat > "$stage_root/uocr-server.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
export LD_LIBRARY_PATH="$PWD:${LD_LIBRARY_PATH:-}"
exec ./uocr-server "$@"
EOF
chmod 755 "$stage_root/uocr-server.sh"

cat > "$stage_root/README.txt" <<EOF
Unlimited-OCR Workbench $version

Run ./uocr-server.sh to start the local backend and hosted React app.
Default URL: http://127.0.0.1:8765/
Logs: logs/uocr-server.log
Optional authenticated model downloads: export HF_TOKEN before launching.
PDF support: native MuPDF is linked into uocr-server through vcpkg libmupdf.
Primary runtime: $runtime_platform from $runtime_repo $runtime_version.
Bundled runtimes: ${runtime_platforms[*]}.
Uninstall: delete this folder.
EOF

cat > "$stage_root/install-manifest.json" <<EOF
{
  "schema_version": 1,
  "name": "uocr-workbench",
  "version": "$version",
  "platform": "linux-$package_arch",
  "runtime_platform": "$runtime_platform",
  "runtime_platforms": $(printf '%s\n' "${runtime_platforms[@]}" | python3 -c 'import json,sys; print(json.dumps([line.strip() for line in sys.stdin if line.strip()]))'),
  "runtime_version": "$runtime_version",
  "pdf_renderer": "vcpkg-libmupdf",
  "created_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

mkdir -p "$output_dir"
(cd "$output_dir" && tar -czf "$(basename "$archive_path")" "$(basename "$stage_root")")
hash="$(sha256sum "$archive_path" | awk '{print $1}')"
printf '%s  %s\n' "$hash" "$(basename "$archive_path")" > "$sha_path"
echo "Packaged $archive_path"
echo "Checksum $sha_path"
