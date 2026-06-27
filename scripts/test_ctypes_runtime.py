#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import ctypes
import json
import os
import sys
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]
ABI_VERSION = 1
TINY_PNG = (
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAFgwJ/"
    "l6mZ4QAAAABJRU5ErkJggg=="
)


def platform_ffi_names() -> list[str]:
    if os.name == "nt":
        return ["uocr-ffi.dll", "libuocr-ffi.dll"]
    if sys.platform == "darwin":
        return ["libuocr-ffi.dylib"]
    return ["libuocr-ffi.so"]


def find_ffi_lib(explicit: str = "") -> Path:
    if explicit:
        return Path(explicit).expanduser().resolve()
    if os.environ.get("UOCR_FFI_LIB"):
        return Path(os.environ["UOCR_FFI_LIB"]).expanduser().resolve()

    names = platform_ffi_names()
    candidates: list[Path] = []
    for name in names:
        candidates.extend(sorted((REPO_ROOT / "thirdparty" / "uocr-runtime").glob(f"*/bin/{name}")))
        candidates.extend(
            [
                REPO_ROOT / "thirdparty" / "llama.cpp" / "build" / "bin" / name,
                REPO_ROOT / "thirdparty" / "llama.cpp" / "build" / "bin" / "Release" / name,
            ]
        )
    for candidate in candidates:
        if candidate.exists():
            return candidate.resolve()
    raise SystemExit(f"could not find FFI library; checked names: {', '.join(names)}")


def load_ffi_for_abi(path: Path) -> dict[str, Any]:
    if os.name == "nt" and hasattr(os, "add_dll_directory"):
        os.add_dll_directory(str(path.parent))
    elif sys.platform == "darwin":
        for sibling in sorted(path.parent.glob("lib*.dylib")):
            if sibling.resolve() != path.resolve():
                try:
                    ctypes.CDLL(str(sibling), mode=ctypes.RTLD_GLOBAL)
                except OSError:
                    pass
    else:
        for sibling in sorted(path.parent.glob("lib*.so*")):
            if sibling.resolve() != path.resolve():
                try:
                    ctypes.CDLL(str(sibling), mode=ctypes.RTLD_GLOBAL)
                except OSError:
                    pass
    lib = ctypes.CDLL(str(path))
    lib.uocr_ffi_abi_version.argtypes = []
    lib.uocr_ffi_abi_version.restype = ctypes.c_uint32
    lib.uocr_ffi_build_info.argtypes = []
    lib.uocr_ffi_build_info.restype = ctypes.c_char_p
    lib.uocr_ffi_media_marker.argtypes = []
    lib.uocr_ffi_media_marker.restype = ctypes.c_char_p
    version = int(lib.uocr_ffi_abi_version())
    if version != ABI_VERSION:
        raise SystemExit(f"unexpected ABI version {version}; expected {ABI_VERSION}")
    return {
        "ffi_library": str(path),
        "abi_version": version,
        "build_info": (lib.uocr_ffi_build_info() or b"").decode("utf-8", errors="replace"),
        "media_marker": (lib.uocr_ffi_media_marker() or b"").decode("utf-8", errors="replace"),
    }


def default_image() -> Path:
    handle = tempfile.NamedTemporaryFile(prefix="uocr-ctypes-", suffix=".png", delete=False)
    path = Path(handle.name)
    with handle:
        handle.write(base64.b64decode(TINY_PNG))
    return path


def run_live(args: argparse.Namespace, ffi_lib: Path) -> dict[str, Any]:
    sys.path.insert(0, str(REPO_ROOT / "src"))
    os.environ["UOCR_FFI_LIB"] = str(ffi_lib)

    from baidu_unlimited_ocr_portable.native_runner import RuntimePaths, profile_by_key, stream_ocr

    image = Path(args.image).expanduser().resolve() if args.image else default_image()
    paths = RuntimePaths.from_env()
    missing = paths.missing("ffi")
    if missing:
        raise SystemExit("missing live runtime files:\n" + "\n".join(missing))

    profile = profile_by_key(args.profile)
    runs: list[dict[str, Any]] = []
    previous_run_count = 0
    for index in range(args.runs):
        done = None
        token_text = []
        for event in stream_ocr(
            paths=paths,
            image_path=image,
            prompt=args.prompt,
            profile=profile,
            max_tokens=args.max_tokens,
            runtime_backend="ffi",
        ):
            if event.kind == "token":
                token_text.append(event.text)
            elif event.kind == "done":
                done = event
            elif event.kind == "error":
                raise SystemExit(json.dumps(event.metadata or {"error": event.text}, indent=2))
        if done is None:
            raise SystemExit(f"run {index + 1} did not emit a done event")
        metadata = done.metadata or {}
        run_count = int(metadata.get("ffi_run_count") or 0)
        if run_count <= previous_run_count:
            raise SystemExit(f"FFI session was not reused continuously: previous={previous_run_count}, current={run_count}")
        previous_run_count = run_count
        runs.append(
            {
                "run": index + 1,
                "ffi_run_count": run_count,
                "tokens": len(token_text),
                "elapsed_ms": metadata.get("elapsed_ms"),
            }
        )

    return {"ffi_library": str(ffi_lib), "image": str(image), "runs": runs}


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate the Unlimited-OCR ctypes native runtime.")
    parser.add_argument("--ffi-lib", default="", help="Path to libuocr-ffi/uocr-ffi.dll.")
    parser.add_argument("--abi-only", action="store_true", help="Only load the library and validate exported ABI symbols.")
    parser.add_argument("--image", default="", help="Image to OCR in live mode. Defaults to a temporary 1x1 PNG.")
    parser.add_argument("--profile", default="best-zero-empty-q4")
    parser.add_argument("--prompt", default="document parsing.")
    parser.add_argument("--max-tokens", type=int, default=1)
    parser.add_argument("--runs", type=int, default=2)
    args = parser.parse_args()

    ffi_lib = find_ffi_lib(args.ffi_lib)
    payload = load_ffi_for_abi(ffi_lib)
    if not args.abi_only:
        payload["live"] = run_live(args, ffi_lib)
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
