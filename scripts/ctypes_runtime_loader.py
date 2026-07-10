from __future__ import annotations

import contextlib
import ctypes
import os
import sys
from pathlib import Path

DLL_DIRECTORY_HANDLES: list[object] = []
DLL_DIRECTORY_PATHS: set[str] = set()
WINDOWS_RUNTIME_DEPENDENCY_DLLS = (
    "cublas64_13.dll",
    "cublasLt64_13.dll",
    "cudart64_13.dll",
    "cudnn64_9.dll",
    "cufft64_12.dll",
    "libcrypto-3-x64.dll",
    "libssl-3-x64.dll",
    "nvrtc64_130_0.dll",
)


def prepare_dynamic_library_search(path: Path) -> None:
    if os.name == "nt" and hasattr(os, "add_dll_directory"):
        for directory in windows_dependency_dirs(path):
            add_windows_dll_directory(directory)
    elif sys.platform == "darwin":
        load_sibling_libraries(path, "lib*.dylib", ctypes.RTLD_GLOBAL)
    else:
        load_sibling_libraries(path, "lib*.so*", ctypes.RTLD_GLOBAL)


def load_sibling_libraries(path: Path, pattern: str, mode: int) -> None:
    for sibling in sorted(path.parent.glob(pattern)):
        if sibling.resolve() != path.resolve():
            with contextlib.suppress(OSError):
                ctypes.CDLL(str(sibling), mode=mode)


def windows_dependency_dirs(path: Path) -> list[Path]:
    dirs = [path.parent]
    for raw in os.environ.get("PATH", "").split(os.pathsep):
        if not raw:
            continue
        candidate = Path(raw.strip('"'))
        dirs.append(candidate)
        if candidate.name.lower() in {"bin", "cmd"} and candidate.parent.name.lower() == "git":
            dirs.append(candidate.parent / "mingw64" / "bin")

    unique: list[Path] = []
    seen: set[str] = set()
    for directory in dirs:
        try:
            resolved = directory.resolve(strict=False)
        except OSError:
            continue
        key = str(resolved).lower()
        if key in seen or not resolved.is_dir():
            continue
        if resolved == path.parent or any(
            (resolved / name).exists() for name in WINDOWS_RUNTIME_DEPENDENCY_DLLS
        ):
            unique.append(resolved)
            seen.add(key)
    return unique


def add_windows_dll_directory(directory: Path) -> None:
    key = str(directory.resolve(strict=False)).lower()
    if key in DLL_DIRECTORY_PATHS:
        return
    DLL_DIRECTORY_HANDLES.append(os.add_dll_directory(str(directory)))
    DLL_DIRECTORY_PATHS.add(key)
