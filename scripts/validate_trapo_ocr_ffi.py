#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from test_ctypes_runtime import validate_trapo_ocr_exported_symbols, validate_trapo_ocr_runtime
from trapo_ocr_ffi_build_env import is_cuda13_platform

REPO_ROOT = Path(__file__).resolve().parents[1]
CUDA_BACKEND_CACHE_FLAGS = (
    "TRAPO_LLAMA_ENABLE_CUDA",
    "GGML_CUDA",
)
OPTIONAL_BACKEND_CACHE_FLAGS = (
    "TRAPO_LLAMA_ENABLE_VULKAN",
    "GGML_VULKAN",
    "TRAPO_LLAMA_ENABLE_OPENCL",
    "GGML_OPENCL",
)
ALL_BACKEND_CACHE_FLAGS = CUDA_BACKEND_CACHE_FLAGS + OPTIONAL_BACKEND_CACHE_FLAGS


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
    flag_state = {name: cache_bool(cache, name) for name in ALL_BACKEND_CACHE_FLAGS}
    unexpected_on = [name for name in OPTIONAL_BACKEND_CACHE_FLAGS if flag_state[name]]
    if unexpected_on:
        raise SystemExit(
            "trapo-ocr-ffi portable build unexpectedly enabled hardware backends: "
            + ", ".join(unexpected_on)
        )
    if is_cuda13_platform(platform):
        missing_cuda = [name for name in CUDA_BACKEND_CACHE_FLAGS if not flag_state[name]]
        if missing_cuda:
            raise SystemExit(
                "trapo-ocr-ffi cuda13 build is missing required CUDA backends: "
                + ", ".join(missing_cuda)
            )
    else:
        unexpected_cuda = [name for name in CUDA_BACKEND_CACHE_FLAGS if flag_state[name]]
        if unexpected_cuda:
            raise SystemExit(
                "trapo-ocr-ffi non-cuda build unexpectedly enabled CUDA backends: "
                + ", ".join(unexpected_cuda)
            )
    return {
        "cmake_cache": str(cache_path),
        "portable_backend_flags": flag_state,
        "cuda13_platform": is_cuda13_platform(platform),
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
            "Do not use on hosted GitHub Actions runners without a GPU; "
            "compile-time CUDA validation does not require a device."
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
