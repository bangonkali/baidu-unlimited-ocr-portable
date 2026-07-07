from __future__ import annotations

import json
import os
import platform
import subprocess
import tarfile
import zipfile
from pathlib import Path
from typing import Any

from runtime_engine_guard_manifest import (
    ENGINE_EXECUTABLE_BASES,
    executable_name,
    load_platforms,
)


def required_asset_files(platform_id: str, asset_dir: str) -> list[str]:
    if asset_dir == "ppocrv6":
        binary = (
            "ppocrv6/bin/trapo_ppocrv6_engine.exe"
            if platform_id.startswith("windows-")
            else "ppocrv6/bin/trapo_ppocrv6_engine"
        )
        return [
            "ppocrv6/trapo_ppocrv6_engine.py",
            "ppocrv6/models/manifest.json",
            binary,
        ]
    if asset_dir == "tesseract":
        binary = (
            "tesseract/bin/tesseract.exe"
            if platform_id.startswith("windows-")
            else "tesseract/bin/tesseract"
        )
        return [binary, "tesseract/tessdata/eng.traineddata"]
    return []


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def host_platform_parts() -> tuple[str, str]:
    system = platform.system().lower()
    machine = platform.machine().lower()
    os_name = {"darwin": "macos"}.get(system, system)
    arch = "arm64" if machine in {"arm64", "aarch64", "arm64ec"} else "x86_64"
    return os_name, arch


def host_matches_platform(platform_id: str) -> bool:
    os_name, arch = host_platform_parts()
    return f"{os_name}-{arch}" in platform_id


def runtime_is_installed(repo_root: Path, platform_id: str, target: dict[str, Any]) -> bool:
    bin_dir = repo_root / "thirdparty" / "uocr-runtime" / platform_id / "bin"
    runtime_dir = bin_dir.parent
    return all(
        (bin_dir / library).is_file() for library in target.get("required_libraries", [])
    ) and all(
        (runtime_dir / required_file).is_file()
        for asset_dir in target.get("engine_asset_dirs", [])
        for required_file in required_asset_files(platform_id, asset_dir)
    )


def local_platforms(repo_root: Path) -> list[str]:
    platforms = load_platforms(repo_root)
    os_name, arch = host_platform_parts()
    candidates = [
        platform_id
        for platform_id, target in platforms["targets"].items()
        if target.get("support_status") == "supported"
        and target.get("os") == os_name
        and target.get("arch") == arch
        and runtime_is_installed(repo_root, platform_id, target)
    ]
    return sorted(candidates)


def local_smoke(args: Any) -> None:
    repo_root = args.repo_root.resolve()
    platform_ids = local_platforms(repo_root)
    if not platform_ids:
        message = "no installed runtime for this host platform was found"
        if (
            args.optional
            or os.environ.get("TRAPO_RUNTIME_SMOKE_OPTIONAL") == "1"
            or os.environ.get("GITHUB_ACTIONS") == "true"
        ):
            print(f"runtime engine smoke skipped: {message}")
            return
        die(message)
    release_dir = repo_root / "target" / "release"
    for platform_id in platform_ids:
        runtime_bin = repo_root / "thirdparty" / "uocr-runtime" / platform_id / "bin"
        smoke_platform(platform_id, [runtime_bin, release_dir], require_all=True)


def smoke_runners(args: Any) -> None:
    smoke_platform(args.platform, [args.build_dir.resolve()], require_all=True)


def smoke_platform(platform_id: str, search_roots: list[Path], *, require_all: bool) -> None:
    found = []
    missing = []
    for base in ENGINE_EXECUTABLE_BASES:
        name = executable_name(base, platform_id)
        path = find_file(search_roots, name)
        if path is None:
            missing.append(name)
        else:
            found.append(path)
    if missing and require_all:
        die(f"{platform_id} is missing engine commands: {', '.join(missing)}")
    if not host_matches_platform(platform_id):
        print(f"{platform_id}: executable smoke skipped on non-matching host")
        return
    for path in found:
        smoke_help(path)
        if path.name.startswith("trapo-pp-ocrv6-runner"):
            smoke_self_check(path)
        if path.name.startswith("trapo-tesseract-rs-runner"):
            smoke_self_check(path)
    print(f"{platform_id}: runtime engine command smoke passed")


def find_file(roots: list[Path], name: str) -> Path | None:
    for root in roots:
        direct = root / name
        if direct.is_file():
            return direct
        for match in root.glob(f"**/{name}"):
            if match.is_file():
                return match
    return None


def smoke_help(path: Path) -> None:
    result = subprocess.run(
        [str(path), "--help"],
        text=True,
        capture_output=True,
        timeout=30,
        check=False,
    )
    output = f"{result.stdout}\n{result.stderr}".lower()
    if result.returncode not in {0, 1, 2} or not any(
        token in output for token in ("usage", "options", "help")
    ):
        die(f"{path} did not respond like a CLI help command")


def smoke_self_check(path: Path) -> None:
    result = subprocess.run(
        [str(path), "--self-check"],
        text=True,
        capture_output=True,
        timeout=120,
        check=False,
    )
    if result.returncode != 0:
        detail = (result.stderr or result.stdout).strip()
        die(f"{path} self-check failed: {detail}")


def packaged_runtime(args: Any) -> None:
    repo_root = args.repo_root.resolve()
    platforms = load_platforms(repo_root)
    target = platforms["targets"][args.platform]
    archive_name = (
        f"{platforms['asset_prefix']}-{args.platform}-{args.version}.{target['archive_ext']}"
    )
    archive = args.dist_dir.resolve() / archive_name
    sidecar = args.dist_dir.resolve() / f"{archive_name}.runtime.json"
    if not archive.is_file():
        die(f"runtime archive was not produced: {archive}")
    if not sidecar.is_file():
        die(f"runtime sidecar was not produced: {sidecar}")
    manifest = json.loads(sidecar.read_text(encoding="utf-8"))
    listed = set(manifest.get("layout", {}).get("executables", {}))
    expected = set(target["executables"])
    if listed != expected:
        die(f"{args.platform} sidecar executable mismatch: {sorted(expected - listed)}")
    archived = archive_members(archive)
    missing = [
        name
        for name in target["executables"]
        if not any(member.endswith(f"/bin/{name}") for member in archived)
    ]
    if missing:
        die(f"{args.platform} archive is missing executables: {', '.join(missing)}")
    layout_files = set(manifest.get("layout", {}).get("files", []))
    for asset_dir in target.get("engine_asset_dirs", []):
        for required_file in required_asset_files(args.platform, asset_dir):
            if required_file not in layout_files:
                die(f"{args.platform} sidecar is missing engine asset {required_file}")
            if not any(member.endswith(f"/{required_file}") for member in archived):
                die(f"{args.platform} archive is missing engine asset {required_file}")
    print(f"{args.platform}: packaged runtime archive guard passed")


def archive_members(archive: Path) -> set[str]:
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as zipf:
            return {f"/{name}" for name in zipf.namelist()}
    with tarfile.open(archive, "r:gz") as tar:
        return {f"/{member.name}" for member in tar.getmembers()}
