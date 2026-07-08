from __future__ import annotations

import os
import platform
import subprocess
from pathlib import Path
from typing import Any

from runtime_engine_guard_manifest import (
    ENGINE_EXECUTABLE_BASES,
    executable_name,
    load_platforms,
)


def required_asset_files(platform_id: str, asset_dir: str) -> list[str]:
    if asset_dir == "ppocrv6":
        return [
            "ppocrv6/models/manifest.json",
        ]
    if asset_dir == "paddleocr_vl_1_6":
        return [
            "paddleocr_vl_1_6/manifest.json",
            "paddleocr_vl_1_6/layout_detection/inference.onnx",
            "paddleocr_vl_1_6/layout_detection/inference.yml",
        ]
    if asset_dir == "tesseract":
        binary = (
            "tesseract/bin/tesseract.exe"
            if platform_id.startswith("windows-")
            else "tesseract/bin/tesseract"
        )
        return [binary, "tesseract/tessdata/eng.traineddata"]
    return []


def forbidden_asset_files(asset_dir: str) -> list[str]:
    if asset_dir == "ppocrv6":
        return [
            "ppocrv6/trapo_ppocrv6_engine.py",
            "ppocrv6/.venv",
        ]
    if asset_dir == "paddleocr_vl_1_6":
        return [
            "paddleocr_vl_1_6/.venv",
            "paddleocr_vl_1_6/__pycache__",
        ]
    return []


def is_forbidden_asset_path(asset_dir: str, path: str) -> bool:
    normalized = asset_relative_path(asset_dir, path)
    if asset_dir not in {"ppocrv6", "paddleocr_vl_1_6"} or not normalized.startswith(
        f"{asset_dir}/"
    ):
        return False
    parts = normalized.split("/")
    filename = parts[-1]
    if ".venv" in parts or ".paddlex" in parts or "__pycache__" in parts:
        return True
    if filename.endswith((".py", ".pyc", ".pyo", ".pyd", ".spec")):
        return True
    return asset_dir == "ppocrv6" and (
        "trapo_ppocrv6_engine" in parts or filename.startswith("trapo_ppocrv6_engine")
    )


def asset_relative_path(asset_dir: str, path: str) -> str:
    normalized = path.replace("\\", "/").strip("/")
    if normalized == asset_dir or normalized.startswith(f"{asset_dir}/"):
        return normalized
    marker = f"/{asset_dir}/"
    index = normalized.find(marker)
    if index >= 0:
        return normalized[index + 1 :]
    return normalized


def local_forbidden_asset_paths(runtime_dir: Path, asset_dir: str) -> list[str]:
    asset_root = runtime_dir / asset_dir
    if not asset_root.exists():
        return []
    return sorted(
        str(path.relative_to(runtime_dir)).replace("\\", "/")
        for path in asset_root.rglob("*")
        if is_forbidden_asset_path(asset_dir, str(path.relative_to(runtime_dir)))
    )


def archived_forbidden_asset_paths(archived: set[str], asset_dir: str) -> list[str]:
    return sorted(
        member.strip("/") for member in archived if is_forbidden_asset_path(asset_dir, member)
    )


def ppocrv6_ffi_names(platform_id: str) -> list[str]:
    if platform_id.startswith("windows-"):
        return ["trapo-ocr-ffi.dll", "agus_ocr_core.dll"]
    if platform_id.startswith("macos-"):
        return ["libtrapo-ocr-ffi.dylib", "libagus_ocr_core.dylib"]
    return ["libtrapo-ocr-ffi.so", "libagus_ocr_core.so"]


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
    return (
        all((bin_dir / library).is_file() for library in target.get("required_libraries", []))
        and all((bin_dir / notice).is_file() for notice in target.get("bundled_notice_files", []))
        and all(
            (runtime_dir / required_file).is_file()
            for asset_dir in target.get("engine_asset_dirs", [])
            for required_file in required_asset_files(platform_id, asset_dir)
        )
        and not any(
            (runtime_dir / forbidden_file).exists()
            for asset_dir in target.get("engine_asset_dirs", [])
            for forbidden_file in forbidden_asset_files(asset_dir)
        )
        and not any(
            local_forbidden_asset_paths(runtime_dir, asset_dir)
            for asset_dir in target.get("engine_asset_dirs", [])
        )
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
