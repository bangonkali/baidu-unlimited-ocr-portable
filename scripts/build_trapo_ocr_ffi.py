#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import urllib.request
import zipfile
from pathlib import Path
from urllib.parse import urlparse

from onnxruntime_staging import (
    ocr_ffi_ort_platform,
    stage_onnxruntime_files,
)
from trapo_ocr_ffi_build_env import TRUTHY_ENV_VALUES, portable_build_env

REPO_ROOT = Path(__file__).resolve().parents[1]
NATIVE_SOURCE = REPO_ROOT / "src" / "trapo-ocr-native"
USER_AGENT = "trapo-ocr-ffi-builder"
ALLOWED_HOSTS = {"api.nuget.org", "github.com"}
DIRECTML_VERSION = "1.15.4"
OPENCV_ARCHIVE = "opencv-mobile-4.13.0-windows-vs2022.zip"
WINDOWS_DEPS = (
    {
        "id": "directml",
        "url": f"https://api.nuget.org/v3-flatcontainer/microsoft.ai.directml/{DIRECTML_VERSION}/microsoft.ai.directml.{DIRECTML_VERSION}.nupkg",
        "sha256": "4e7cb7ddce8cf837a7a75dc029209b520ca0101470fcdf275c1f49736a3615b9",
        "archive": f"microsoft.ai.directml.{DIRECTML_VERSION}.nupkg",
    },
    {
        "id": "opencv",
        "url": f"https://github.com/nihui/opencv-mobile/releases/download/v35/{OPENCV_ARCHIVE}",
        "sha256": "a08e31484c2598c88ffad3cc2408fc8b1020a7c354b180fc991ccd9ca5f7ab8d",
        "archive": OPENCV_ARCHIVE,
    },
)
ONNXRUNTIME_RECEIPTS: dict[str, dict[str, object]] = {}


def run(command: list[str], *, cwd: Path = REPO_ROOT, env: dict[str, str] | None = None) -> None:
    print("+ " + " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd, env=env, check=True)


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def download(url: str, destination: Path, expected_sha256: str) -> None:
    if destination.is_file() and sha256_file(destination) == expected_sha256:
        return
    parsed = urlparse(url)
    if parsed.scheme != "https" or (parsed.hostname or "").lower() not in ALLOWED_HOSTS:
        die(f"refusing native dependency download URL: {url}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(
        request, timeout=300
    ) as response:  # skylos: ignore[SKY-D216] host allowlist.
        destination.write_bytes(
            response.read()
        )  # skylos: ignore[SKY-D324] fixed dependency cache path.
    actual = sha256_file(destination)
    if actual != expected_sha256:
        destination.unlink(missing_ok=True)
        die(f"SHA256 mismatch for {destination.name}: {actual}")


def safe_extract_zip(archive: Path, destination: Path) -> None:
    root = destination.resolve()
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    with zipfile.ZipFile(archive) as zipf:
        for member in zipf.namelist():
            target = (destination / member).resolve()
            if root not in (target, *target.parents):
                die(f"refusing to extract outside destination: {member}")
        zipf.extractall(destination)


def prepare_onnxruntime_deps(platform: str) -> dict[str, object]:
    if platform in ONNXRUNTIME_RECEIPTS:
        return ONNXRUNTIME_RECEIPTS[platform]
    command = [
        "cargo",
        "run",
        "--quiet",
        "-p",
        "trapo-xtask",
        "--",
        "native-deps",
        "prepare",
        "--platform",
        platform,
        "--repo-root",
        str(REPO_ROOT),
    ]
    print("+ " + " ".join(command), flush=True)
    output = subprocess.check_output(command, cwd=REPO_ROOT, text=True)
    lines = [line for line in output.splitlines() if line.strip()]
    if not lines:
        die("native dependency helper did not return a receipt")
    receipt = json.loads(lines[-1])
    ONNXRUNTIME_RECEIPTS[platform] = receipt
    return receipt


def prepare_windows_deps(
    platform: str, ort: dict[str, object] | None = None
) -> dict[str, Path | list[Path]]:
    arch = "arm64" if "arm64" in platform else "x64"
    dml_bin = "arm64-win" if arch == "arm64" else "x64-win"
    deps_root = REPO_ROOT / ".deps" / "windows" / arch
    downloads = REPO_ROOT / ".deps" / "downloads"
    for dep in WINDOWS_DEPS:
        archive = downloads / str(dep["archive"])
        download(str(dep["url"]), archive, str(dep["sha256"]))
        safe_extract_zip(archive, deps_root / str(dep["id"]))
    opencv_root = deps_root / "opencv" / OPENCV_ARCHIVE.removesuffix(".zip") / arch
    if ort is None:
        ort = prepare_onnxruntime_deps(platform)
    return {
        "ort_include": Path(str(ort["include_dir"])),
        "ort_lib": Path(str(ort["library"])),
        "ort_runtime_libraries": [Path(str(path)) for path in ort["runtime_libraries"]],
        "ort_notice_files": [Path(str(path)) for path in ort["notice_files"]],
        "directml_include": deps_root / "directml" / "include",
        "directml_bin": deps_root / "directml" / "bin" / dml_bin,
        "opencv": opencv_root,
    }


def configure_args(args: argparse.Namespace, build_dir: Path) -> list[str]:
    command = [
        "cmake",
        "-S",
        str(NATIVE_SOURCE),
        "-B",
        str(build_dir),
        "-DCMAKE_BUILD_TYPE=Release",
        f"-DTRAPO_LLAMA_CPP_ROOT={REPO_ROOT / 'thirdparty' / 'llama.cpp'}",
    ]
    if args.platform.startswith("windows-"):
        deps = prepare_windows_deps(
            args.platform, prepare_onnxruntime_deps(ocr_ffi_ort_platform(args.platform))
        )
        command.extend(
            [
                f"-DTRAPO_ORT_INCLUDE_DIR={deps['ort_include']}",
                f"-DTRAPO_ORT_LIB={deps['ort_lib']}",
                f"-DTRAPO_DIRECTML_INCLUDE_DIR={deps['directml_include']}",
                f"-DOpenCV_DIR={deps['opencv']}",
            ]
        )
        if "arm64" in args.platform:
            command.extend(["-A", "ARM64", "-T", "ClangCL"])
    return command


def reset_stale_cmake_cache(build_dir: Path, env: dict[str, str]) -> None:
    cache = build_dir / "CMakeCache.txt"
    if not cache.is_file():
        return
    text = cache.read_text(encoding="utf-8", errors="ignore")
    stale_reason = ""
    for line in text.splitlines():
        if line.startswith("CMAKE_HOME_DIRECTORY:INTERNAL="):
            source = Path(line.partition("=")[2])
            if source.resolve() != NATIVE_SOURCE.resolve():
                stale_reason = f"generated from a different source directory: {source}"
            break
    stale_flags = []
    cache_flags = {
        "TRAPO_LLAMA_ENABLE_CUDA": ("TRAPO_LLAMA_ENABLE_CUDA", "GGML_CUDA"),
        "TRAPO_LLAMA_ENABLE_VULKAN": ("TRAPO_LLAMA_ENABLE_VULKAN", "GGML_VULKAN"),
        "TRAPO_LLAMA_ENABLE_OPENCL": ("TRAPO_LLAMA_ENABLE_OPENCL", "GGML_OPENCL"),
    }
    if not stale_reason:
        for env_name, cmake_names in cache_flags.items():
            want_enabled = env.get(env_name, "").upper() in TRUTHY_ENV_VALUES
            cache_enabled = any(f"{cmake_name}:BOOL=ON" in text for cmake_name in cmake_names)
            if want_enabled != cache_enabled:
                stale_flags.append(env_name)
        if stale_flags:
            stale_reason = "with mismatched llama.cpp backends: " + ", ".join(stale_flags)
    if not stale_reason:
        return
    target_root = (REPO_ROOT / "target" / "trapo-ocr-ffi").resolve()
    resolved_build_dir = build_dir.resolve()
    if target_root not in (resolved_build_dir, *resolved_build_dir.parents):
        die(f"refusing to remove stale build cache outside {target_root}: {resolved_build_dir}")
    print(f"Removing stale trapo-ocr-ffi CMake cache {stale_reason}", flush=True)
    shutil.rmtree(resolved_build_dir)  # skylos: ignore[SKY-D215] bounded target build cache.


def build(args: argparse.Namespace) -> Path | None:
    build_dir = args.build_dir.resolve()
    env = portable_build_env(args.platform)
    reset_stale_cmake_cache(build_dir, env)
    try:
        run(configure_args(args, build_dir), env=env)
        run(
            [
                "cmake",
                "--build",
                str(build_dir),
                "--config",
                "Release",
                "--target",
                "trapo_ocr_native",
            ],
            env=env,
        )
    except subprocess.CalledProcessError as error:
        if args.required:
            raise
        print(f"warning: trapo-ocr-ffi build failed: {error}", flush=True)
        return None
    return find_native_library(build_dir, args.platform)


def find_native_library(build_dir: Path, platform: str) -> Path:
    names = native_library_names(platform)
    for name in names:
        for candidate in build_dir.rglob(name):
            if candidate.is_file():
                return candidate
    die(f"native OCR FFI build did not produce one of: {', '.join(names)}")


def native_library_names(platform: str) -> list[str]:
    if platform.startswith("windows-"):
        return ["trapo-ocr-ffi.dll"]
    if platform.startswith("macos-"):
        return ["libtrapo-ocr-ffi.dylib"]
    return ["libtrapo-ocr-ffi.so"]


def staged_name(platform: str) -> str:
    if platform.startswith("windows-"):
        return "trapo-ocr-ffi.dll"
    if platform.startswith("macos-"):
        return "libtrapo-ocr-ffi.dylib"
    return "libtrapo-ocr-ffi.so"


def stage_library(source: Path, args: argparse.Namespace) -> None:
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, output_dir / staged_name(args.platform))
    stage_onnxruntime_files(output_dir, args.platform, prepare_onnxruntime_deps)
    if args.platform.startswith("windows-"):
        deps = prepare_windows_deps(
            args.platform, prepare_onnxruntime_deps(ocr_ffi_ort_platform(args.platform))
        )
        directml_bin = deps["directml_bin"]
        assert isinstance(directml_bin, Path)
        shutil.copy2(directml_bin / "DirectML.dll", output_dir / "DirectML.dll")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description="Build and stage trapo-ocr-ffi.")
    parser.add_argument("--platform", required=True)
    parser.add_argument("--build-dir", type=Path, default=REPO_ROOT / "target" / "trapo-ocr-ffi")
    parser.add_argument(
        "--output-dir", type=Path, default=REPO_ROOT / "thirdparty" / "llama.cpp" / "build" / "bin"
    )
    parser.add_argument("--required", action="store_true")
    args = parser.parse_args()
    library = build(args)
    if library is not None:
        stage_library(library, args)
        print(args.output_dir.resolve() / staged_name(args.platform))


if __name__ == "__main__":
    main()
