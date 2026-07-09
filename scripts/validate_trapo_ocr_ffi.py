#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from test_ctypes_runtime import validate_trapo_ocr_exported_symbols, validate_trapo_ocr_runtime

REPO_ROOT = Path(__file__).resolve().parents[1]
PORTABLE_BACKEND_CACHE_FLAGS = (
    "TRAPO_LLAMA_ENABLE_CUDA",
    "GGML_CUDA",
    "TRAPO_LLAMA_ENABLE_VULKAN",
    "GGML_VULKAN",
    "TRAPO_LLAMA_ENABLE_OPENCL",
    "GGML_OPENCL",
)


def trapo_ocr_ffi_path(platform: str, build_bin: Path) -> Path:
    if platform.startswith("windows-"):
        return build_bin / "trapo-ocr-ffi.dll"
    if platform.startswith("macos-"):
        return build_bin / "libtrapo-ocr-ffi.dylib"
    return build_bin / "libtrapo-ocr-ffi.so"


def cmake_cache_path(platform: str) -> Path:
    return REPO_ROOT / "target" / "trapo-ocr-ffi" / platform / "CMakeCache.txt"


def cache_bool(cache: str, name: str) -> bool:
    prefixes = (f"{name}:BOOL=", f"{name}:UNINITIALIZED=")
    for line in cache.splitlines():
        if line.startswith(prefixes):
            value = line.split("=", 1)[1].strip().upper()
            return value in {"1", "ON", "TRUE", "YES"}
    return False


def validate_portable_build_configuration(platform: str) -> dict[str, object]:
    cache_path = cmake_cache_path(platform)
    if not cache_path.is_file():
        raise SystemExit(f"trapo-ocr-ffi CMake cache was not found: {cache_path}")
    cache = cache_path.read_text(encoding="utf-8", errors="replace")
    enabled = [name for name in PORTABLE_BACKEND_CACHE_FLAGS if cache_bool(cache, name)]
    if enabled:
        raise SystemExit(
            "trapo-ocr-ffi portable build unexpectedly enabled hardware backends: "
            + ", ".join(enabled)
        )
    return {
        "cmake_cache": str(cache_path),
        "portable_backend_flags": {name: False for name in PORTABLE_BACKEND_CACHE_FLAGS},
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate staged trapo-ocr-ffi runtime support.")
    parser.add_argument("--platform", required=True)
    parser.add_argument("--backend", required=True)
    parser.add_argument(
        "--build-bin", type=Path, default=REPO_ROOT / "thirdparty/llama.cpp/build/bin"
    )
    parser.add_argument(
        "--probe-runtime",
        action="store_true",
        help=(
            "Also load the FFI and call runtime capability functions. "
            "Do not use on hosted CUDA runners."
        ),
    )
    args = parser.parse_args()
    ffi_path = trapo_ocr_ffi_path(args.platform, args.build_bin)
    payload = validate_trapo_ocr_exported_symbols(ffi_path)
    payload["portable_backend_configuration"] = validate_portable_build_configuration(args.platform)
    if args.probe_runtime:
        payload["runtime_probe"] = validate_trapo_ocr_runtime(ffi_path)
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
