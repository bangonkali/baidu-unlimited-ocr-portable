#!/usr/bin/env sh
set -eu

REPO="${UOCR_REPO:-bangonkali/baidu-unlimited-ocr-portable}"
VERSION="${UOCR_VERSION:-latest}"
INSTALL_DIR="${UOCR_INSTALL_DIR:-$HOME/.uocr}"

if command -v python3 >/dev/null 2>&1; then
  PYTHON=python3
elif command -v python >/dev/null 2>&1; then
  PYTHON=python
else
  echo "python3 is required for the portable installer." >&2
  exit 1
fi

"$PYTHON" - "$REPO" "$VERSION" "$INSTALL_DIR" <<'PY'
import json
import os
import platform
import shutil
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from pathlib import Path

repo, version, install_dir = sys.argv[1:4]
system = platform.system().lower()
machine = platform.machine().lower()
if system == "darwin" and machine in {"arm64", "aarch64"}:
    platform_key = "macos-arm64"
    patterns = [".tar.gz", ".zip"]
elif system == "linux" and machine in {"x86_64", "amd64"}:
    platform_key = "linux-x64"
    patterns = [".tar.gz", ".zip"]
elif system == "linux" and machine in {"arm64", "aarch64"}:
    platform_key = "linux-arm64"
    patterns = [".tar.gz", ".zip"]
else:
    raise SystemExit(f"Unsupported platform for prebuilt workbench install: {system}/{machine}")

api = f"https://api.github.com/repos/{repo}/releases/latest"
if version != "latest":
    api = f"https://api.github.com/repos/{repo}/releases/tags/{version}"
request = urllib.request.Request(api, headers={"User-Agent": "uocr-installer"})
with urllib.request.urlopen(request, timeout=60) as response:
    release = json.loads(response.read().decode("utf-8"))

asset = None
for candidate in release.get("assets", []):
    name = candidate.get("name", "")
    if f"uocr-workbench-{platform_key}-" in name and any(name.endswith(ext) for ext in patterns):
        asset = candidate
        break
if asset is None:
    raise SystemExit(
        f"No {platform_key} workbench asset exists on {release.get('tag_name')}. "
        "Choose a release that includes this platform."
    )

target = Path(install_dir).expanduser()
with tempfile.TemporaryDirectory(prefix="uocr-install-") as tmp:
    archive = Path(tmp) / asset["name"]
    urllib.request.urlretrieve(asset["browser_download_url"], archive)
    extract = Path(tmp) / "extract"
    extract.mkdir()
    if archive.name.endswith(".zip"):
        with zipfile.ZipFile(archive) as zf:
            zf.extractall(extract)
    else:
        with tarfile.open(archive) as tf:
            tf.extractall(extract)
    candidates = list(extract.rglob("uocr-server"))
    if not candidates:
        raise SystemExit("Downloaded archive did not contain uocr-server.")
    source = candidates[0].parent
    if target.exists():
        shutil.rmtree(target)
    shutil.copytree(source, target)
    launcher = target / ("uocr-server.command" if system == "darwin" else "uocr-server.sh")
    if launcher.exists():
        launcher.chmod(launcher.stat().st_mode | 0o755)
    else:
        launcher = target / "uocr-server"
    print(f"Installed Unlimited-OCR Workbench to {target}")
    print(f"Run: {launcher}")
    print("Uninstall: delete that folder")
PY
