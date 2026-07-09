#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from test_ctypes_runtime import validate_trapo_ocr_runtime

REPO_ROOT = Path(__file__).resolve().parents[1]


def trapo_ocr_ffi_path(platform: str, build_bin: Path) -> Path:
    if platform.startswith("windows-"):
        return build_bin / "trapo-ocr-ffi.dll"
    if platform.startswith("macos-"):
        return build_bin / "libtrapo-ocr-ffi.dylib"
    return build_bin / "libtrapo-ocr-ffi.so"


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate staged trapo-ocr-ffi runtime support.")
    parser.add_argument("--platform", required=True)
    parser.add_argument("--backend", required=True)
    parser.add_argument(
        "--build-bin", type=Path, default=REPO_ROOT / "thirdparty/llama.cpp/build/bin"
    )
    args = parser.parse_args()
    payload = validate_trapo_ocr_runtime(
        trapo_ocr_ffi_path(args.platform, args.build_bin),
        "cuda" if args.backend == "cuda" else "",
    )
    print(json.dumps(payload, indent=2, sort_keys=True))
    if "load_error" in payload and args.backend == "cuda":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
