from __future__ import annotations

import hashlib
import shutil
import urllib.request
import zipfile
from pathlib import Path
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[1]
USER_AGENT = "trapo-ocr-ffi-builder"
ALLOWED_HOSTS = {"api.nuget.org", "github.com"}
DIRECTML_VERSION = "1.15.4"
WINDOWS_OPENCV_ARCHIVE = "opencv-mobile-4.13.0-windows-vs2022.zip"
LINUX_OPENCV_ARCHIVE = "opencv-mobile-4.13.0-ubuntu-2404.zip"

WINDOWS_DEPS = (
    {
        "id": "directml",
        "url": (
            "https://api.nuget.org/v3-flatcontainer/microsoft.ai.directml/"
            f"{DIRECTML_VERSION}/microsoft.ai.directml.{DIRECTML_VERSION}.nupkg"
        ),
        "sha256": "4e7cb7ddce8cf837a7a75dc029209b520ca0101470fcdf275c1f49736a3615b9",
        "archive": f"microsoft.ai.directml.{DIRECTML_VERSION}.nupkg",
    },
    {
        "id": "opencv",
        "url": (
            f"https://github.com/nihui/opencv-mobile/releases/download/v35/{WINDOWS_OPENCV_ARCHIVE}"
        ),
        "sha256": "a08e31484c2598c88ffad3cc2408fc8b1020a7c354b180fc991ccd9ca5f7ab8d",
        "archive": WINDOWS_OPENCV_ARCHIVE,
    },
)
LINUX_DEPS = (
    {
        "id": "opencv",
        "url": (
            f"https://github.com/nihui/opencv-mobile/releases/download/v35/{LINUX_OPENCV_ARCHIVE}"
        ),
        "sha256": "130f0fde37e2cf4bac33e9f2c6b89c64b874181b7cfa5c37e93a246b03b9c62c",
        "archive": LINUX_OPENCV_ARCHIVE,
    },
)


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download(url: str, destination: Path, expected_sha256: str) -> None:
    if destination.is_file() and sha256_file(destination) == expected_sha256:
        return
    parsed = urlparse(url)
    if parsed.scheme != "https" or (parsed.hostname or "").lower() not in ALLOWED_HOSTS:
        die(f"refusing native dependency download URL: {url}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with (
        urllib.request.urlopen(
            request, timeout=300
        ) as response,  # skylos: ignore[SKY-D216] fixed host allowlist.
        destination.open("wb") as handle,  # skylos: ignore[SKY-D324] fixed cache path.
    ):
        shutil.copyfileobj(response, handle, length=1024 * 1024)
    actual = sha256_file(destination)
    if actual != expected_sha256:
        destination.unlink(missing_ok=True)
        die(f"SHA256 mismatch for {destination.name}: {actual}")


def safe_extract_zip(archive: Path, destination: Path) -> None:
    root = destination.resolve()
    if destination.exists():
        shutil.rmtree(destination)  # skylos: ignore[SKY-D215] bounded dependency extraction path.
    destination.mkdir(parents=True)
    with zipfile.ZipFile(archive) as zipf:
        for member in zipf.namelist():
            target = (destination / member).resolve()
            if root not in (target, *target.parents):
                die(f"refusing to extract outside destination: {member}")
        zipf.extractall(destination)


def extract_dependencies(dependencies: tuple[dict[str, str], ...], root: Path) -> None:
    downloads = REPO_ROOT / ".deps" / "downloads"
    for dependency in dependencies:
        archive = downloads / dependency["archive"]
        download(dependency["url"], archive, dependency["sha256"])
        safe_extract_zip(archive, root / dependency["id"])


def prepare_windows_deps(
    platform: str,
    ort: dict[str, object],
) -> dict[str, Path | list[Path]]:
    arch = "arm64" if "arm64" in platform else "x64"
    dml_bin = "arm64-win" if arch == "arm64" else "x64-win"
    deps_root = REPO_ROOT / ".deps" / "windows" / arch
    extract_dependencies(WINDOWS_DEPS, deps_root)
    opencv_root = deps_root / "opencv" / WINDOWS_OPENCV_ARCHIVE.removesuffix(".zip") / arch
    return {
        "ort_include": Path(str(ort["include_dir"])),
        "ort_lib": Path(str(ort["library"])),
        "ort_runtime_libraries": [Path(str(path)) for path in ort["runtime_libraries"]],
        "ort_notice_files": [Path(str(path)) for path in ort["notice_files"]],
        "directml_include": deps_root / "directml" / "include",
        "directml_bin": deps_root / "directml" / "bin" / dml_bin,
        "opencv": opencv_root,
    }


def prepare_linux_deps(
    platform: str,
    ort: dict[str, object],
) -> dict[str, Path | list[Path]]:
    if not platform.startswith("linux-x86_64-"):
        die(f"Linux native OCR dependencies are unavailable for {platform}")
    deps_root = REPO_ROOT / ".deps" / "linux" / "x64"
    extract_dependencies(LINUX_DEPS, deps_root)
    opencv_root = deps_root / "opencv" / LINUX_OPENCV_ARCHIVE.removesuffix(".zip") / "x64"
    return {
        "ort_include": Path(str(ort["include_dir"])),
        "ort_lib": Path(str(ort["library"])),
        "opencv": opencv_root,
    }
