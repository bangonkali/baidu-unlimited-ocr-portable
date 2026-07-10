from __future__ import annotations

import os
import shutil
from pathlib import Path

REQUIRED_MSVC_RUNTIME_DLLS = (
    "msvcp140.dll",
    "vcruntime140.dll",
    "vcruntime140_1.dll",
)


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def redist_roots() -> list[Path]:
    roots: list[Path] = []
    configured = os.environ.get("VCTOOLSREDISTDIR")
    if configured:
        roots.append(Path(configured))
    program_files = os.environ.get("PROGRAMFILES")
    if program_files:
        visual_studio = Path(program_files) / "Microsoft Visual Studio"
        roots.extend(sorted(visual_studio.glob("*/*/VC/Redist/MSVC/*"), reverse=True))
    unique: list[Path] = []
    seen: set[Path] = set()
    for root in roots:
        resolved = root.resolve(strict=False)
        if resolved.is_dir() and resolved not in seen:
            unique.append(resolved)
            seen.add(resolved)
    return unique


def find_crt_dir(arch: str) -> Path:
    for root in redist_roots():
        for pattern in (f"{arch}/Microsoft.VC*.CRT", f"*/{arch}/Microsoft.VC*.CRT"):
            for candidate in sorted(root.glob(pattern), reverse=True):
                if all((candidate / name).is_file() for name in REQUIRED_MSVC_RUNTIME_DLLS):
                    return candidate
    die(f"Visual C++ redistributable runtime was not found for {arch}")


def stage_windows_runtime(output_dir: Path, platform: str) -> list[Path]:
    if not platform.startswith("windows-"):
        return []
    arch = "arm64" if "arm64" in platform else "x64"
    source_dir = find_crt_dir(arch)
    output_dir.mkdir(parents=True, exist_ok=True)
    staged: list[Path] = []
    for source in sorted(source_dir.glob("*.dll")):
        destination = output_dir / source.name
        shutil.copy2(source, destination)
        staged.append(destination)
    missing = [name for name in REQUIRED_MSVC_RUNTIME_DLLS if not (output_dir / name).is_file()]
    if missing:
        die("staged Visual C++ runtime is missing required files: " + ", ".join(missing))
    print(f"Staged {len(staged)} Visual C++ runtime files for {platform}", flush=True)
    return staged
