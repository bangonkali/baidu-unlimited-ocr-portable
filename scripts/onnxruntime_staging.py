from __future__ import annotations

import shutil
from collections.abc import Callable
from pathlib import Path
from typing import Any

ReceiptLoader = Callable[[str], dict[str, Any]]


def ocr_ffi_ort_platform(platform: str) -> str:
    if platform.endswith("-cuda13"):
        return platform.removesuffix("-cuda13") + "-cpu"
    return platform


def provider_ort_platforms(platform: str) -> list[str]:
    return [platform] if platform.endswith("-cuda13") else []


def is_provider_runtime_library(path: Path) -> bool:
    return "providers_" in path.name.lower()


def onnxruntime_runtime_libraries(platform: str, load_receipt: ReceiptLoader) -> list[Path]:
    libraries = [
        Path(str(item))
        for item in load_receipt(ocr_ffi_ort_platform(platform))["runtime_libraries"]
    ]
    for provider_platform in provider_ort_platforms(platform):
        provider_receipt = load_receipt(provider_platform)
        libraries.extend(
            Path(str(item))
            for item in provider_receipt["runtime_libraries"]
            if is_provider_runtime_library(Path(str(item)))
        )
    return libraries


def onnxruntime_notice_files(platform: str, load_receipt: ReceiptLoader) -> list[Path]:
    return [
        Path(str(item)) for item in load_receipt(ocr_ffi_ort_platform(platform))["notice_files"]
    ]


def stage_onnxruntime_files(output_dir: Path, platform: str, load_receipt: ReceiptLoader) -> None:
    for path in onnxruntime_runtime_libraries(platform, load_receipt):
        shutil.copy2(path, output_dir / path.name)
    for path in onnxruntime_notice_files(platform, load_receipt):
        shutil.copy2(path, output_dir / f"onnxruntime-{path.name}")
