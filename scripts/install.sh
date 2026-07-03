#!/usr/bin/env sh
set -eu

REPO="${TRAPO_REPO:-${UOCR_REPO:-bangonkali/baidu-unlimited-ocr-portable}}"
VERSION="${TRAPO_VERSION:-${UOCR_VERSION:-latest}}"
INSTALL_DIR="${TRAPO_INSTALL_DIR:-${UOCR_INSTALL_DIR:-$HOME/.trapo}}"

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
import re
import shutil
import sys
import tarfile
import tempfile
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path

repo, version, install_dir = sys.argv[1:4]
GITHUB_REPO_RE = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$")
GITHUB_DOWNLOAD_HOSTS = {
    "api.github.com",
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
}


def die(message):
    raise SystemExit(message)


def validate_github_repo(value):
    candidate = value.strip("/")
    if not GITHUB_REPO_RE.fullmatch(candidate):
        die(f"TRAPO_REPO must be OWNER/REPO with GitHub-safe characters, got: {value}")
    return candidate


def validate_github_url(url):
    parsed = urllib.parse.urlparse(url)
    host = (parsed.hostname or "").lower()
    if parsed.scheme != "https" or host not in GITHUB_DOWNLOAD_HOSTS:
        die(f"refusing non-GitHub download URL: {url}")
    return url


def safe_asset_name(value):
    name = Path(value).name
    if name != value or not name:
        die(f"release asset name must be a plain file name, got: {value}")
    return name


def safe_install_dir(value):
    target = Path(value).expanduser().resolve()
    forbidden = {Path(target.anchor).resolve(), Path.home().resolve()}
    if target in forbidden:
        die(f"refusing to install into destructive target: {target}")
    return target


def safe_extract_zip(archive, destination):
    root = destination.resolve()
    with zipfile.ZipFile(archive) as zf:
        for member in zf.namelist():
            target = (destination / member).resolve()
            if root not in (target, *target.parents):
                die(f"refusing to extract archive member outside destination: {member}")
        zf.extractall(destination)  # skylos: ignore[SKY-D326] zip members are bounded to the temporary extract directory above.


def safe_extract_tar(archive, destination):
    root = destination.resolve()
    with tarfile.open(archive) as tf:
        for member in tf.getmembers():
            target = (destination / member.name).resolve()
            if root not in (target, *target.parents):
                die(f"refusing to extract archive member outside destination: {member.name}")
        tf.extractall(destination, filter="data")  # skylos: ignore[SKY-D326] tar members are bounded and Python's data filter rejects special files.


def download(url, destination):
    request = urllib.request.Request(validate_github_url(url), headers={"User-Agent": "trapo-installer"})
    with (
        urllib.request.urlopen(request, timeout=300) as response,  # skylos: ignore[SKY-D216] validate_github_url restricts release downloads to GitHub hosts.
        destination.open("wb") as fh,  # skylos: ignore[SKY-D324] destination is a validated asset name under TemporaryDirectory.
    ):
        shutil.copyfileobj(response, fh)


repo = validate_github_repo(repo)
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
    api = f"https://api.github.com/repos/{repo}/releases/tags/{urllib.parse.quote(version, safe='')}"
request = urllib.request.Request(api, headers={"User-Agent": "trapo-installer"})
with urllib.request.urlopen(request, timeout=60) as response:  # skylos: ignore[SKY-D216] api is built from a validated GitHub repo and HTTPS tag path.
    release = json.loads(response.read().decode("utf-8"))

asset = None
for candidate in release.get("assets", []):
    name = candidate.get("name", "")
    if f"trapo-workbench-{platform_key}-" in name and any(name.endswith(ext) for ext in patterns):
        asset = candidate
        break
if asset is None:
    raise SystemExit(
        f"No {platform_key} workbench asset exists on {release.get('tag_name')}. "
        "Choose a release that includes this platform."
    )

target = safe_install_dir(install_dir)
with tempfile.TemporaryDirectory(prefix="trapo-install-") as tmp:
    archive = Path(tmp) / safe_asset_name(asset["name"])
    download(asset["browser_download_url"], archive)
    extract = Path(tmp) / "extract"
    extract.mkdir()
    if archive.name.endswith(".zip"):
        safe_extract_zip(archive, extract)
    else:
        safe_extract_tar(archive, extract)
    candidates = list(extract.rglob("trapo-server"))
    if not candidates:
        raise SystemExit("Downloaded archive did not contain trapo-server.")
    source = candidates[0].parent
    if target.exists():
        shutil.rmtree(target)  # skylos: ignore[SKY-D215] target is the validated operator-selected install directory, never root or HOME.
    shutil.copytree(source, target)
    launcher = target / "trapo-server.sh"
    if launcher.exists():
        launcher.chmod(launcher.stat().st_mode | 0o755)
    else:
        launcher = target / "trapo-server"
    print(f"Installed Trapo Workbench to {target}")
    print(f"Run: {launcher}")
    print("Uninstall: delete that folder")
PY
