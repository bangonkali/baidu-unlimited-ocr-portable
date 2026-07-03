#!/usr/bin/env python3
from __future__ import annotations

import argparse
import datetime as dt
import glob
import hashlib
import json
import os
import stat
import tarfile
import tempfile
import zipfile
from pathlib import Path
from typing import Any

from package_runtime_macos import (
    prepare_macos_runtime_files,
)  # skylos: ignore[SKY-D222] local sibling script module, not a PyPI dependency.

REPO_ROOT = Path(__file__).resolve().parents[1]
PLATFORMS_PATH = REPO_ROOT / "runtime" / "platforms.json"


def load_platforms(repo_root: Path) -> dict[str, Any]:
    path = repo_root / "runtime" / "platforms.json"
    with path.open(
        "r", encoding="utf-8"
    ) as fh:  # skylos: ignore[SKY-D325] path is a fixed repo manifest under repo_root.
        return json.load(fh)


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:  # skylos: ignore[SKY-D325] caller-selected artifact.
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalize_version(raw: str) -> str:
    cleaned = raw.strip().replace("/", "-").replace("\\", "-")
    return cleaned or "dev"


def find_one(build_dir: Path, names: list[str]) -> Path:
    candidates: list[Path] = []
    for name in names:
        direct_candidates = [
            build_dir / name,
            build_dir / "bin" / name,
            build_dir / "bin" / "Release" / name,
            build_dir / "Release" / name,
        ]
        candidates.extend(path for path in direct_candidates if path.exists())
        candidates.extend(
            Path(match) for match in glob.glob(str(build_dir / "**" / name), recursive=True)
        )
    seen: set[Path] = set()
    unique: list[Path] = []
    for candidate in candidates:
        resolved = candidate.resolve(strict=False)
        if resolved not in seen:
            seen.add(resolved)
            unique.append(candidate)
    if not unique:
        die(f"could not find required runtime file under {build_dir}: {', '.join(names)}")
    return unique[0]


def path_dirs() -> list[Path]:
    dirs: list[Path] = []
    for raw in os.environ.get("PATH", "").split(os.pathsep):
        if raw:
            dirs.append(Path(raw.strip('"')))
    return dirs


def dependency_search_dirs(build_dir: Path) -> list[Path]:
    dirs = [
        build_dir,
        build_dir / "bin",
        build_dir / "bin" / "Release",
        build_dir / "Release",
        *path_dirs(),
    ]
    if os.name == "nt":
        for directory in path_dirs():
            if directory.name.lower() in {"bin", "cmd"} and directory.parent.name.lower() == "git":
                dirs.append(directory.parent / "mingw64" / "bin")

    unique: list[Path] = []
    seen: set[Path] = set()
    for directory in dirs:
        resolved = directory.resolve(strict=False)
        if resolved not in seen and resolved.is_dir():
            unique.append(resolved)
            seen.add(resolved)
    return unique


def find_dependency(build_dir: Path, name: str) -> Path:
    candidates = [Path(match) for match in glob.glob(str(build_dir / "**" / name), recursive=True)]
    for directory in dependency_search_dirs(build_dir):
        candidates.append(directory / name)
    for candidate in candidates:
        if candidate.exists():
            return candidate
    die(f"could not find bundled runtime dependency: {name}")


def collect_runtime_files(build_dir: Path, target: dict[str, Any]) -> list[Path]:
    files: list[Path] = []
    executable_paths: list[Path] = []
    for exe in target["executables"]:
        path = find_one(build_dir, [exe])
        executable_paths.append(path)
        files.append(path)
    for library in target.get("required_libraries", []):
        files.append(find_one(build_dir, [library]))
    for dependency in target.get("bundled_dependency_libraries", []):
        files.append(find_dependency(build_dir, dependency))

    search_dirs = {path.parent for path in executable_paths}
    search_dirs.add(build_dir / "bin")
    search_dirs.add(build_dir / "bin" / "Release")
    for directory in sorted(search_dirs):
        if not directory.exists():
            continue
        for pattern in target.get("library_globs", []):
            for match in directory.glob(pattern):
                if match.is_file() or match.is_symlink():
                    files.append(match)

    unique: list[Path] = []
    seen: set[Path] = set()
    for file_path in files:
        key = Path(os.path.abspath(file_path))
        if key not in seen:
            seen.add(key)
            unique.append(file_path)
    return unique


def executable_manifest(files: list[Path], target: dict[str, Any]) -> dict[str, str]:
    by_name = {path.name: f"bin/{path.name}" for path in files}
    return {exe: by_name[exe] for exe in target["executables"] if exe in by_name}


def library_manifest(files: list[Path], target: dict[str, Any]) -> dict[str, str]:
    by_name = {path.name: f"bin/{path.name}" for path in files}
    return {
        library: by_name[library]
        for library in target.get("required_libraries", [])
        if library in by_name
    }


def dependency_manifest(files: list[Path], target: dict[str, Any]) -> dict[str, str]:
    by_name = {path.name: f"bin/{path.name}" for path in files}
    return {
        library: by_name[library]
        for library in target.get("bundled_dependency_libraries", [])
        if library in by_name
    }


def make_package_manifest(
    *,
    platform_id: str,
    target: dict[str, Any],
    version: str,
    repo_root: Path,
    build_dir: Path,
    files: list[Path],
    archive_name: str,
    archive_sha256: str | None = None,
    archive_size: int | None = None,
) -> dict[str, Any]:
    llama_dir = repo_root / "thirdparty" / "llama.cpp"
    llama_commit = ""
    git_dir = llama_dir / ".git"
    if git_dir.exists():
        try:
            import subprocess

            llama_commit = subprocess.check_output(
                ["git", "-C", str(llama_dir), "rev-parse", "HEAD"],
                text=True,
                stderr=subprocess.DEVNULL,
            ).strip()
        except Exception:
            llama_commit = ""

    required_libraries = library_manifest(files, target)
    dependency_libraries = dependency_manifest(files, target)
    return {
        "schema_version": 1,
        "platform": platform_id,
        "version": version,
        "created_at": dt.datetime.now(dt.UTC)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "archive_name": archive_name,
        "archive_sha256": archive_sha256,
        "archive_size": archive_size,
        "target": {
            "label": target.get("label"),
            "os": target.get("os"),
            "arch": target.get("arch"),
            "backend": target.get("backend"),
            "cuda_major": target.get("cuda_major"),
            "cuda_architectures": target.get("cuda_architectures"),
            "minimum_compute_capability": target.get("minimum_compute_capability"),
            "known_gpu_support": target.get("known_gpu_support", []),
            "support_status": target.get("support_status"),
        },
        "build": {
            "repo_commit": os.environ.get("GITHUB_SHA", ""),
            "llama_cpp_commit": llama_commit,
            "build_dir": str(build_dir),
            "cmake_defines": target.get("cmake_defines", []),
        },
        "layout": {
            "root": f"uocr-runtime-{platform_id}-{version}",
            "bin_dir": "bin",
            "primary_binary": f"bin/{target['primary_binary']}",
            "executables": executable_manifest(files, target),
            "required_libraries": required_libraries,
            "dependency_libraries": dependency_libraries,
            "ffi_library": next(iter(required_libraries.values()), ""),
            "files": sorted(f"bin/{path.name}" for path in files),
        },
    }


def add_tar_member(tar: tarfile.TarFile, source: Path, arcname: str) -> None:
    tar.add(source, arcname=arcname, recursive=False)


def add_zip_member(zipf: zipfile.ZipFile, source: Path, arcname: str) -> None:
    if source.is_symlink():
        target = source.resolve(strict=True)
        zipf.write(target, arcname)
        return
    zipf.write(source, arcname)


def package_runtime(args: argparse.Namespace) -> None:
    repo_root = args.repo_root.resolve()
    platforms = load_platforms(repo_root)
    targets = platforms["targets"]
    if args.platform not in targets:
        die(f"unsupported platform label: {args.platform}")

    target = targets[args.platform]
    version = normalize_version(args.version)
    build_dir = args.build_dir.resolve()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    runtime_files = collect_runtime_files(build_dir, target)
    if not runtime_files:
        die("no runtime files were collected")
    if target.get("os") == "macos":
        try:
            prepare_macos_runtime_files(runtime_files, target)
        except RuntimeError as error:
            die(str(error))

    archive_ext = target["archive_ext"]
    archive_name = f"{platforms['asset_prefix']}-{args.platform}-{version}.{archive_ext}"
    archive_path = output_dir / archive_name
    root_name = f"uocr-runtime-{args.platform}-{version}"

    initial_manifest = make_package_manifest(
        platform_id=args.platform,
        target=target,
        version=version,
        repo_root=repo_root,
        build_dir=build_dir,
        files=runtime_files,
        archive_name=archive_name,
    )

    with tempfile.TemporaryDirectory(prefix="uocr-runtime-meta-") as tmp:
        tmp_path = Path(tmp)
        manifest_path = tmp_path / "manifest.json"
        readme_path = tmp_path / "README.txt"
        manifest_path.write_text(
            json.dumps(initial_manifest, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        readme_path.write_text(
            "\n".join(
                [
                    f"Unlimited-OCR native runtime: {args.platform}",
                    f"Version: {version}",
                    "",
                    "This archive contains native llama.cpp runtime binaries only.",
                    "GGUF model files are downloaded separately by the setup scripts.",
                    "",
                ]
            ),
            encoding="utf-8",
        )

        if archive_ext == "zip":
            with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as zipf:
                for source in runtime_files:
                    add_zip_member(zipf, source, f"{root_name}/bin/{source.name}")
                zipf.write(manifest_path, f"{root_name}/manifest.json")
                zipf.write(readme_path, f"{root_name}/README.txt")
        elif archive_ext == "tar.gz":
            with tarfile.open(archive_path, "w:gz") as tar:
                for source in runtime_files:
                    add_tar_member(tar, source, f"{root_name}/bin/{source.name}")
                add_tar_member(tar, manifest_path, f"{root_name}/manifest.json")
                add_tar_member(tar, readme_path, f"{root_name}/README.txt")
        else:
            die(f"unsupported archive extension for {args.platform}: {archive_ext}")

    archive_hash = sha256_file(archive_path)
    archive_size = archive_path.stat().st_size
    final_manifest = make_package_manifest(
        platform_id=args.platform,
        target=target,
        version=version,
        repo_root=repo_root,
        build_dir=build_dir,
        files=runtime_files,
        archive_name=archive_name,
        archive_sha256=archive_hash,
        archive_size=archive_size,
    )
    sidecar_path = output_dir / f"{archive_name}.runtime.json"
    sidecar_path.write_text(
        json.dumps(final_manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    sha_path = output_dir / f"{archive_name}.sha256"
    sha_path.write_text(f"{archive_hash}  {archive_name}\n", encoding="utf-8")

    # Keep local package outputs executable even if a filesystem lost mode bits.
    for file_path in runtime_files:
        if file_path.name in target["executables"] and not file_path.name.endswith(".exe"):
            file_path.chmod(file_path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    print(
        json.dumps(
            {
                "archive": str(archive_path),
                "sha256": archive_hash,
                "manifest": str(sidecar_path),
            },
            indent=2,
        )
    )


def merge_manifests(args: argparse.Namespace) -> None:
    input_dir = args.input_dir.resolve()
    output_path = args.output.resolve()
    repo_root = args.repo_root.resolve()
    platforms = load_platforms(repo_root)

    sidecars = sorted(input_dir.rglob("*.runtime.json"))
    if not sidecars:
        die(f"no per-platform runtime manifests found under {input_dir}")

    merged: dict[str, Any] = {
        "schema_version": 1,
        "created_at": dt.datetime.now(dt.UTC)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "asset_prefix": platforms["asset_prefix"],
        "platforms": {},
    }
    for sidecar in sidecars:
        data = json.loads(sidecar.read_text(encoding="utf-8"))
        platform_id = data["platform"]
        merged["platforms"][platform_id] = {
            "platform": platform_id,
            "version": data["version"],
            "archive_name": data["archive_name"],
            "archive_sha256": data["archive_sha256"],
            "archive_size": data["archive_size"],
            "target": data["target"],
            "layout": data["layout"],
            "build": data["build"],
        }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(merged, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(output_path)


def write_version_env(args: argparse.Namespace) -> None:
    version = ""
    if args.ref_type == "tag" and args.ref_name:
        version = args.ref_name
    elif args.input_version:
        version = args.input_version
    elif args.sha:
        version = f"dev-{args.sha[:12]}"
    else:
        version = "dev"
    version = normalize_version(version)
    with args.github_env.open(
        "a", encoding="utf-8"
    ) as fh:  # skylos: ignore[SKY-D324] GitHub Actions env file.
        fh.write(f"UOCR_RUNTIME_VERSION={version}\n")
    print(version)


def main() -> None:
    parser = argparse.ArgumentParser(description="Package Unlimited-OCR native runtime binaries.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    package_parser = subparsers.add_parser("package", help="Create a platform runtime archive.")
    package_parser.add_argument("--repo-root", type=Path, default=REPO_ROOT)
    package_parser.add_argument("--platform", required=True)
    package_parser.add_argument(
        "--build-dir",
        type=Path,
        default=REPO_ROOT / "thirdparty" / "llama.cpp" / "build",
    )
    package_parser.add_argument("--output-dir", type=Path, default=REPO_ROOT / "dist")
    package_parser.add_argument("--version", required=True)
    package_parser.set_defaults(func=package_runtime)

    merge_parser = subparsers.add_parser(
        "merge", help="Merge per-platform manifests into a release manifest."
    )
    merge_parser.add_argument("--repo-root", type=Path, default=REPO_ROOT)
    merge_parser.add_argument("--input-dir", type=Path, required=True)
    merge_parser.add_argument("--output", type=Path, required=True)
    merge_parser.set_defaults(func=merge_manifests)

    version_parser = subparsers.add_parser(
        "write-version-env", help="Write UOCR_RUNTIME_VERSION to a GitHub env file."
    )
    version_parser.add_argument("--github-env", type=Path, required=True)
    version_parser.add_argument("--ref-type", default="")
    version_parser.add_argument("--ref-name", default="")
    version_parser.add_argument("--sha", default="")
    version_parser.add_argument("--input-version", default="")
    version_parser.set_defaults(func=write_version_env)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
