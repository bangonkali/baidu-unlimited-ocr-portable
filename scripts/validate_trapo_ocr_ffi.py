#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

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
    command = [
        sys.executable,
        str(REPO_ROOT / "scripts" / "test_ctypes_runtime.py"),
        "--trapo-ocr-ffi-lib",
        str(trapo_ocr_ffi_path(args.platform, args.build_bin)),
        "--abi-only",
    ]
    if args.backend == "cuda":
        command.extend(["--require-generative-backend", "cuda"])
    subprocess.run(command, check=True)


if __name__ == "__main__":
    main()
