#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from pathlib import Path

from build_parallelism import cmake_build_parallel_args
from nvidia_redist_staging import stage_nvidia_redist
from onnxruntime_staging import (
    ocr_ffi_ort_platform,
    stage_onnxruntime_files,
)
from trapo_ocr_ffi_build_env import TRUTHY_ENV_VALUES, portable_build_env
from trapo_ocr_ffi_deps import prepare_linux_deps, prepare_windows_deps
from windows_runtime_staging import stage_windows_runtime

REPO_ROOT = Path(__file__).resolve().parents[1]
NATIVE_SOURCE = REPO_ROOT / "src" / "trapo-ocr-native"
ONNXRUNTIME_RECEIPTS: dict[str, dict[str, object]] = {}


def run(command: list[str], *, cwd: Path = REPO_ROOT, env: dict[str, str] | None = None) -> None:
    print("+ " + " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd, env=env, check=True)


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


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
    elif args.platform.startswith("linux-x86_64-"):
        deps = prepare_linux_deps(
            args.platform, prepare_onnxruntime_deps(ocr_ffi_ort_platform(args.platform))
        )
        command.extend(
            [
                "-DTRAPO_ENABLE_DESKTOP_NATIVE_PIPELINE=ON",
                f"-DTRAPO_ORT_INCLUDE_DIR={deps['ort_include']}",
                f"-DTRAPO_ORT_LIB={deps['ort_lib']}",
                f"-DOpenCV_DIR={deps['opencv']}",
            ]
        )
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
                *cmake_build_parallel_args(env),
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
    stage_nvidia_redist(output_dir, args.platform)
    stage_windows_runtime(output_dir, args.platform)
    if args.platform.startswith("windows-"):
        deps = prepare_windows_deps(
            args.platform, prepare_onnxruntime_deps(ocr_ffi_ort_platform(args.platform))
        )
        directml_bin = deps["directml_bin"]
        assert isinstance(directml_bin, Path)
        shutil.copy2(directml_bin / "DirectML.dll", output_dir / "DirectML.dll")


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
