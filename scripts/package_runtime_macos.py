from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any


def prepare_macos_runtime_files(files: list[Path], target: dict[str, Any]) -> None:
    macho_files = _macos_macho_files(files, target)
    for path in macho_files:
        if not _macos_has_rpath(path, "@loader_path"):
            _install_name_tool("-add_rpath", "@loader_path", str(path))
    _validate_macos_portable_dependencies(macho_files)


def _macos_macho_files(files: list[Path], target: dict[str, Any]) -> list[Path]:
    candidates = [
        path
        for path in files
        if path.name in target["executables"] or path.name.endswith(".dylib")
    ]
    unique: list[Path] = []
    seen: set[Path] = set()
    for candidate in candidates:
        resolved = candidate.resolve(strict=True)
        if resolved not in seen:
            unique.append(resolved)
            seen.add(resolved)
    return unique


def _otool_output(path: Path, *args: str) -> str:
    try:
        return subprocess.check_output(
            ["otool", *args, str(path)], text=True, stderr=subprocess.STDOUT
        )
    except FileNotFoundError as exc:
        raise RuntimeError(
            "otool is required to package macOS runtime artifacts"
        ) from exc
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(f"otool failed for {path}: {exc.output.strip()}") from exc


def _install_name_tool(*args: str) -> None:
    try:
        subprocess.run(["install_name_tool", *args], check=True)
    except FileNotFoundError as exc:
        raise RuntimeError(
            "install_name_tool is required to package macOS runtime artifacts"
        ) from exc
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(
            f"install_name_tool failed with exit code {exc.returncode}"
        ) from exc


def _macos_has_rpath(path: Path, rpath: str) -> bool:
    return f"path {rpath} " in _otool_output(path, "-l")


def _macos_dependencies(path: Path) -> list[str]:
    dependencies: list[str] = []
    for line in _otool_output(path, "-L").splitlines()[1:]:
        stripped = line.strip()
        if stripped:
            dependencies.append(stripped.split(" ", 1)[0])
    return dependencies


def _validate_macos_portable_dependencies(files: list[Path]) -> None:
    allowed_prefixes = ("@rpath/", "@loader_path/", "/usr/lib/", "/System/Library/")
    offenders: list[str] = []
    for path in files:
        for dependency in _macos_dependencies(path):
            if dependency.startswith(allowed_prefixes):
                continue
            if dependency == path.name or dependency.endswith(f"/{path.name}"):
                continue
            offenders.append(f"{path.name}: {dependency}")
    if offenders:
        raise RuntimeError(
            "macOS runtime has non-portable absolute dylib dependencies:\n"
            + "\n".join(sorted(offenders))
        )
