#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import urllib.request
import zipfile
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = REPO_ROOT / "runtime" / "nvidia-redist.json"
ALLOWED_DOWNLOAD_HOSTS = {"files.pythonhosted.org"}
USER_AGENT = "trapo-nvidia-redist-stager"


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_manifest(path: Path = MANIFEST_PATH) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        manifest = json.load(handle)
    if manifest.get("schema_version") != 1:
        die(f"unsupported NVIDIA redistributable manifest: {path}")
    return manifest


def target_config(platform: str, manifest: dict[str, Any]) -> dict[str, Any] | None:
    target = manifest.get("targets", {}).get(platform)
    if target is None and "cuda13" in platform:
        die(f"NVIDIA redistributables are not defined for {platform}")
    return target


def download(url: str, destination: Path, expected_sha256: str) -> None:
    if destination.is_file() and sha256_file(destination) == expected_sha256:
        return
    parsed = urlparse(url)
    if parsed.scheme != "https" or (parsed.hostname or "").lower() not in ALLOWED_DOWNLOAD_HOSTS:
        die(f"refusing NVIDIA dependency download URL: {url}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_suffix(destination.suffix + ".part")
    temporary.unlink(missing_ok=True)
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with (
        urllib.request.urlopen(
            request, timeout=600
        ) as response,  # skylos: ignore[SKY-D216] fixed host allowlist.
        temporary.open("wb") as handle,  # skylos: ignore[SKY-D324] fixed dependency cache.
    ):
        shutil.copyfileobj(response, handle, length=1024 * 1024)
    actual = sha256_file(temporary)
    if actual != expected_sha256:
        temporary.unlink(missing_ok=True)
        die(f"SHA256 mismatch for {destination.name}: {actual}")
    temporary.replace(destination)


def safe_extract_wheel(archive: Path, destination: Path, expected_sha256: str) -> Path:
    marker = destination / ".complete.json"
    if marker.is_file():
        try:
            payload = json.loads(marker.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            payload = {}
        if payload.get("sha256") == expected_sha256:
            return destination
    if destination.exists():
        shutil.rmtree(destination)  # skylos: ignore[SKY-D215] bounded dependency extraction path.
    destination.mkdir(parents=True)
    root = destination.resolve()
    with zipfile.ZipFile(archive) as wheel:
        for member in wheel.namelist():
            target = (destination / member).resolve()
            if root not in (target, *target.parents):
                die(f"refusing wheel member outside destination: {member}")
        wheel.extractall(destination)
    marker.write_text(
        json.dumps({"sha256": expected_sha256}, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return destination


def prepare_cudnn(platform: str, target: dict[str, Any]) -> Path:
    cudnn = target["cudnn"]
    archive = REPO_ROOT / ".deps" / "downloads" / cudnn["archive"]
    download(cudnn["url"], archive, cudnn["sha256"])
    destination = REPO_ROOT / ".deps" / "nvidia" / platform / f"cudnn-{cudnn['version']}"
    return safe_extract_wheel(archive, destination, cudnn["sha256"])


def cuda_roots() -> list[Path]:
    names = ["CUDA_PATH", "CUDA_HOME"]
    names.extend(
        sorted(
            (name for name in os.environ if name.startswith("CUDA_PATH_V")),
            reverse=True,
        )
    )
    roots: list[Path] = []
    seen: set[Path] = set()
    for name in names:
        raw = os.environ.get(name)
        if not raw:
            continue
        root = Path(raw).resolve(strict=False)
        if root.is_dir() and root not in seen:
            roots.append(root)
            seen.add(root)
    return roots


def find_cuda_source_dirs(target: dict[str, Any]) -> tuple[Path, list[Path]]:
    for root in cuda_roots():
        directories = [
            root / relative for relative in target["cuda_search_dirs"] if (root / relative).is_dir()
        ]
        if directories:
            return root, directories
    die("CUDA 13 toolkit was not found; set CUDA_PATH before packaging a cuda13 runtime")


def select_pattern_files(directories: list[Path], patterns: list[str]) -> list[Path]:
    selected: list[Path] = []
    names: set[str] = set()
    missing: list[str] = []
    for pattern in patterns:
        matches = [
            match
            for directory in directories
            for match in sorted(directory.glob(pattern))
            if match.is_file() or match.is_symlink()
        ]
        if not matches:
            missing.append(pattern)
            continue
        for match in matches:
            if match.name not in names:
                selected.append(match)
                names.add(match.name)
    if missing:
        die("CUDA toolkit is missing required runtime libraries: " + ", ".join(missing))
    return selected


def copy_runtime_file(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    if os.path.lexists(destination):
        destination.unlink()
    if source.is_symlink():
        destination.symlink_to(os.readlink(source))
    else:
        shutil.copy2(source, destination)


def find_notice(root: Path, candidates: list[str]) -> Path:
    for candidate in candidates:
        path = root / candidate
        if path.is_file():
            return path
    die(f"NVIDIA CUDA license notice was not found under {root}")


def validate_staged_nvidia_redist(
    directory: Path,
    platform: str,
    manifest_path: Path = MANIFEST_PATH,
) -> None:
    manifest = load_manifest(manifest_path)
    target = target_config(platform, manifest)
    if target is None:
        return
    required = [
        *target["required_libraries"],
        *manifest["notice_names"].values(),
    ]
    missing = [name for name in required if not (directory / name).exists()]
    if missing:
        die(f"staged NVIDIA runtime is missing required files: {', '.join(missing)}")


def stage_nvidia_redist(
    output_dir: Path,
    platform: str,
    manifest_path: Path = MANIFEST_PATH,
) -> list[Path]:
    manifest = load_manifest(manifest_path)
    target = target_config(platform, manifest)
    if target is None:
        return []

    output_dir.mkdir(parents=True, exist_ok=True)
    cuda_root, source_dirs = find_cuda_source_dirs(target)
    selected = select_pattern_files(source_dirs, target["cuda_patterns"])
    cudnn_root = prepare_cudnn(platform, target)
    cudnn_files = sorted(cudnn_root.glob(target["cudnn"]["library_pattern"]))
    if not cudnn_files:
        die(f"cuDNN wheel contains no runtime libraries for {platform}")
    selected.extend(cudnn_files)

    staged: list[Path] = []
    for source in selected:
        destination = output_dir / source.name
        copy_runtime_file(source, destination)
        staged.append(destination)

    notice_names = manifest["notice_names"]
    cuda_notice = find_notice(cuda_root, target["cuda_notice_candidates"])
    cudnn_notices = sorted(cudnn_root.glob(target["cudnn"]["license_pattern"]))
    if not cudnn_notices:
        die("cuDNN wheel does not contain its license notice")
    for source, name in (
        (cuda_notice, notice_names["cuda"]),
        (cudnn_notices[0], notice_names["cudnn"]),
    ):
        destination = output_dir / name
        shutil.copy2(source, destination)
        staged.append(destination)

    validate_staged_nvidia_redist(output_dir, platform, manifest_path)
    print(f"Staged {len(staged)} NVIDIA CUDA/cuDNN files for {platform}", flush=True)
    return staged


def main() -> None:
    parser = argparse.ArgumentParser(description="Stage NVIDIA CUDA/cuDNN redistributables.")
    parser.add_argument("--platform", required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    stage_nvidia_redist(args.output_dir.resolve(), args.platform)


if __name__ == "__main__":
    main()
