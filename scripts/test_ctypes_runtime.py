#!/usr/bin/env python3
from __future__ import annotations

import argparse
import contextlib
import ctypes
import json
import os
import struct
import subprocess
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
ABI_VERSION = 1
REQUIRED_SYMBOLS = {
    "uocr_ffi_abi_version",
    "uocr_ffi_build_info",
    "uocr_ffi_create",
    "uocr_ffi_destroy",
    "uocr_ffi_last_error",
    "uocr_ffi_last_status",
    "uocr_ffi_media_marker",
    "uocr_ffi_run_count",
    "uocr_ffi_run_image",
}
DLL_DIRECTORY_HANDLES: list[object] = []
DLL_DIRECTORY_PATHS: set[str] = set()
WINDOWS_RUNTIME_DEPENDENCY_DLLS = (
    "cublas64_13.dll",
    "cublasLt64_13.dll",
    "cudart64_13.dll",
    "libcrypto-3-x64.dll",
    "libssl-3-x64.dll",
    "nvrtc64_130_0.dll",
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


def read_c_string(blob: bytes, offset: int) -> str:
    end = blob.find(b"\x00", offset)
    if end < 0:
        end = len(blob)
    return blob[offset:end].decode("ascii", errors="replace")


def pe_exports(path: Path) -> set[str]:
    blob = path.read_bytes()
    if len(blob) < 0x40 or blob[:2] != b"MZ":
        return set()
    pe_offset = struct.unpack_from("<I", blob, 0x3C)[0]
    if blob[pe_offset : pe_offset + 4] != b"PE\x00\x00":
        return set()
    coff = pe_offset + 4
    number_of_sections = struct.unpack_from("<H", blob, coff + 2)[0]
    optional_size = struct.unpack_from("<H", blob, coff + 16)[0]
    optional = coff + 20
    magic = struct.unpack_from("<H", blob, optional)[0]
    data_dir = optional + (112 if magic == 0x20B else 96)
    export_rva, _export_size = struct.unpack_from("<II", blob, data_dir)
    sections = []
    section_offset = optional + optional_size
    for index in range(number_of_sections):
        offset = section_offset + index * 40
        virtual_size, virtual_address, raw_size, raw_pointer = struct.unpack_from(
            "<IIII", blob, offset + 8
        )
        sections.append((virtual_address, max(virtual_size, raw_size), raw_pointer))

    def rva_to_offset(rva: int) -> int:
        for virtual_address, size, raw_pointer in sections:
            if virtual_address <= rva < virtual_address + size:
                return raw_pointer + (rva - virtual_address)
        return rva

    if not export_rva:
        return set()
    export_offset = rva_to_offset(export_rva)
    fields = struct.unpack_from("<IIHHIIIIIII", blob, export_offset)
    number_of_names = fields[7]
    names_rva = fields[9]
    names_offset = rva_to_offset(names_rva)
    names: set[str] = set()
    for index in range(number_of_names):
        name_rva = struct.unpack_from("<I", blob, names_offset + index * 4)[0]
        names.add(read_c_string(blob, rva_to_offset(name_rva)))
    return names


def exported_symbols(path: Path) -> set[str]:
    if path.suffix.lower() == ".dll":
        return pe_exports(path)
    commands = (
        ["nm", "-D", str(path)],
        ["nm", "-gU", str(path)],
        ["llvm-nm", "-D", str(path)],
    )
    for command in commands:
        try:
            output = subprocess.check_output(command, text=True, stderr=subprocess.STDOUT)
        except Exception:
            continue
        symbols: set[str] = set()
        for line in output.splitlines():
            parts = line.split()
            if parts:
                name = parts[-1]
                symbols.add(name)
                if name.startswith("_"):
                    symbols.add(name[1:])
        if symbols:
            return symbols
    return set()


def validate_exported_symbols(path: Path) -> dict[str, Any]:
    symbols = exported_symbols(path)
    missing = sorted(REQUIRED_SYMBOLS - symbols)
    if missing:
        raise SystemExit(f"FFI library is missing required exports: {', '.join(missing)}")
    return {
        "ffi_library": str(path),
        "abi_version": ABI_VERSION,
        "symbol_validation": True,
        "required_symbols": sorted(REQUIRED_SYMBOLS),
    }


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


def load_ffi_for_abi(path: Path) -> dict[str, Any]:
    if os.name == "nt" and hasattr(os, "add_dll_directory"):
        for directory in windows_dependency_dirs(path):
            add_windows_dll_directory(directory)
    elif sys.platform == "darwin":
        for sibling in sorted(path.parent.glob("lib*.dylib")):
            if sibling.resolve() != path.resolve():
                with contextlib.suppress(OSError):
                    ctypes.CDLL(str(sibling), mode=ctypes.RTLD_GLOBAL)
    else:
        for sibling in sorted(path.parent.glob("lib*.so*")):
            if sibling.resolve() != path.resolve():
                with contextlib.suppress(OSError):
                    ctypes.CDLL(str(sibling), mode=ctypes.RTLD_GLOBAL)
    try:
        lib = ctypes.CDLL(str(path))
    except OSError as exc:
        payload = validate_exported_symbols(path)
        payload["load_error"] = str(exc)
        return payload
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


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Validate the Unlimited-OCR ctypes native runtime ABI."
    )
    parser.add_argument("--ffi-lib", default="", help="Path to libuocr-ffi/uocr-ffi.dll.")
    parser.add_argument(
        "--abi-only",
        action="store_true",
        help="Accepted for release workflow compatibility; ABI validation is always used.",
    )
    args = parser.parse_args()

    ffi_lib = find_ffi_lib(args.ffi_lib)
    payload = load_ffi_for_abi(ffi_lib)
    if "load_error" in payload:
        print(json.dumps(payload, indent=2, sort_keys=True))
        raise SystemExit(1)
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
