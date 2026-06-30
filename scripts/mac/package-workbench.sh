#!/usr/bin/env bash
set -euo pipefail

preset="macos-arm64-workbench-ci"
version=""
runtime_version="v0.0.7"
runtime_repo="bangonkali/baidu-unlimited-ocr-portable"
runtime_platform="macos-arm64-metal"
output_dir=""
no_build=0
no_runtime_download=0

usage() {
  cat <<'EOF'
Usage: scripts/mac/package-workbench.sh [options]

Options:
  --version VERSION              Release version, for example v0.0.28.
  --runtime-version VERSION      Runtime release tag. Default: v0.0.7.
  --runtime-repo OWNER/REPO      Runtime release repo.
  --output-dir PATH              Output directory. Default: dist.
  --preset NAME                  CMake preset. Default: macos-arm64-workbench-ci.
  --no-build                     Skip CMake build and use existing uocr-server.
  --no-runtime-download          Require an existing runtime under thirdparty/uocr-runtime.
  -h, --help                     Show this help.
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
    --output-dir) output_dir="$2"; shift 2 ;;
    --preset) preset="$2"; shift 2 ;;
    --no-build) no_build=1; shift ;;
    --no-runtime-download) no_runtime_download=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown option: $1" ;;
  esac
done

if [[ -z "$version" ]]; then
  version="$(git -C "$repo_root" describe --tags --dirty --always 2>/dev/null || true)"
  [[ -n "$version" ]] || version="0.0.0-dev"
fi
safe_version="${version//\//-}"
safe_version="${safe_version//\\/-}"

find_server() {
  find "$repo_root/build" -type f -name uocr-server -perm -111 2>/dev/null | sort | tail -n 1
}

find_vcpkg_copyright() {
  local package="$1"
  find "$repo_root/build" -path "*/vcpkg_installed/*/share/$package/copyright" -type f 2>/dev/null | sort | tail -n 1
}

if ((no_build == 0)); then
  (cd "$repo_root/src/uocr-client" && bun install --frozen-lockfile)
  cmake --preset "$preset" -DUOCR_VERSION="$version"
  cmake --build --preset "${preset}-release"
fi

runtime_dir="$repo_root/thirdparty/uocr-runtime/$runtime_platform"
runtime_ffi="$runtime_dir/bin/libuocr-ffi.dylib"
if [[ ! -f "$runtime_ffi" && "$no_runtime_download" == "0" ]]; then
  python3 "$repo_root/scripts/install_runtime.py" install \
    --repo-root "$repo_root" \
    --install-dir "$repo_root/thirdparty/uocr-runtime" \
    --runtime-repo "$runtime_repo" \
    --runtime-version "$runtime_version" \
    --platform "$runtime_platform"
fi
[[ -f "$runtime_ffi" ]] || die "runtime FFI library is missing: $runtime_ffi"

exe="$(find_server)"
[[ -n "$exe" ]] || die "uocr-server was not found under build/"
exe_dir="$(cd "$(dirname "$exe")" && pwd)"

stage_root="$output_dir/uocr-workbench-macos-arm64-$safe_version"
zip_path="$output_dir/uocr-workbench-macos-arm64-$safe_version.zip"
sha_path="$zip_path.sha256"
rm -rf "$stage_root" "$zip_path" "$sha_path"
mkdir -p "$stage_root"

cp "$exe_dir/uocr-server" "$stage_root/uocr-server"
chmod 755 "$stage_root/uocr-server"
find "$exe_dir" -maxdepth 1 -name '*.dylib' -type f -exec cp {} "$stage_root" \;
[[ -d "$exe_dir/web" ]] && cp -R "$exe_dir/web" "$stage_root/web"
[[ -d "$exe_dir/openapi" ]] && cp -R "$exe_dir/openapi" "$stage_root/openapi"

mkdir -p "$stage_root/thirdparty/uocr-runtime" "$stage_root/thirdparty/libmupdf"
cp -R "$runtime_dir" "$stage_root/thirdparty/uocr-runtime/$runtime_platform"
mupdf_copyright="$(find_vcpkg_copyright libmupdf)"
[[ -n "$mupdf_copyright" ]] || die "vcpkg copyright file was not found for libmupdf"
cp "$mupdf_copyright" "$stage_root/thirdparty/libmupdf/copyright"

for dir in models data cache logs config uploads; do
  mkdir -p "$stage_root/$dir"
done

cat > "$stage_root/uocr-server.command" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
exec ./uocr-server "$@"
EOF
chmod 755 "$stage_root/uocr-server.command"

cat > "$stage_root/README.txt" <<EOF
Unlimited-OCR Workbench $version

Run uocr-server.command or ./uocr-server to start the local backend and hosted React app.
Default URL: http://127.0.0.1:8765/
Logs: logs/uocr-server.log
Optional authenticated model downloads: export HF_TOKEN before launching.
PDF support: native MuPDF is linked into uocr-server through vcpkg libmupdf.
Runtime: $runtime_platform from $runtime_repo $runtime_version.
Uninstall: delete this folder.
EOF

cat > "$stage_root/install-manifest.json" <<EOF
{
  "schema_version": 1,
  "name": "uocr-workbench",
  "version": "$version",
  "platform": "macos-arm64",
  "runtime_platform": "$runtime_platform",
  "runtime_version": "$runtime_version",
  "pdf_renderer": "vcpkg-libmupdf",
  "created_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

mkdir -p "$output_dir"
(cd "$output_dir" && zip -qry "$(basename "$zip_path")" "$(basename "$stage_root")")
hash="$(shasum -a 256 "$zip_path" | awk '{print $1}')"
printf '%s  %s\n' "$hash" "$(basename "$zip_path")" > "$sha_path"
echo "Packaged $zip_path"
echo "Checksum $sha_path"
