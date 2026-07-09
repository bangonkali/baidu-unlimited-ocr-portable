#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from test_ctypes_runtime import validate_trapo_ocr_runtime

REPO_ROOT = Path(__file__).resolve().parents[1]
CUDA_CACHE_FLAGS = ("TRAPO_LLAMA_ENABLE_CUDA", "GGML_CUDA")


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


def validate_cuda_build_configuration(platform: str) -> dict[str, object]:
    cache_path = cmake_cache_path(platform)
    if not cache_path.is_file():
        raise SystemExit(f"trapo-ocr-ffi CMake cache was not found: {cache_path}")
    cache = cache_path.read_text(encoding="utf-8", errors="replace")
    missing = [name for name in CUDA_CACHE_FLAGS if not cache_bool(cache, name)]
    if missing:
        raise SystemExit("trapo-ocr-ffi CUDA build flags were not enabled: " + ", ".join(missing))
    return {
        "cmake_cache": str(cache_path),
        "cuda_build_flags": {name: True for name in CUDA_CACHE_FLAGS},
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate staged trapo-ocr-ffi runtime support.")
    parser.add_argument("--platform", required=True)
    parser.add_argument("--backend", required=True)
    parser.add_argument(
        "--build-bin", type=Path, default=REPO_ROOT / "thirdparty/llama.cpp/build/bin"
    )
    args = parser.parse_args()
    required_backend = "cuda" if args.backend == "cuda" else ""
    payload = validate_trapo_ocr_runtime(
        trapo_ocr_ffi_path(args.platform, args.build_bin),
        required_backend,
    )
    if required_backend:
        payload["required_backend_configuration"] = validate_cuda_build_configuration(args.platform)
    print(json.dumps(payload, indent=2, sort_keys=True))
    if "load_error" in payload and args.backend != "cuda":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
