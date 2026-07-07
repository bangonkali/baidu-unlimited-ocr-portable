#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
import stat
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]

RUNNERS = [
    "trapo-tesseract-rs-runner",
    "trapo-pp-ocrv6-runner",
]


def runner_name(base: str, platform: str) -> str:
    return f"{base}.exe" if platform.startswith("windows-") else base


def candidate_dirs(repo_root: Path) -> list[Path]:
    return [
        repo_root / "target" / "release",
        repo_root / "target" / "aarch64-pc-windows-msvc" / "release",
        repo_root / "target" / "x86_64-pc-windows-msvc" / "release",
        repo_root / "target" / "aarch64-apple-darwin" / "release",
        repo_root / "target" / "x86_64-unknown-linux-gnu" / "release",
        repo_root / "target" / "aarch64-unknown-linux-gnu" / "release",
    ]


def find_runner(repo_root: Path, platform: str, base: str) -> Path | None:
    name = runner_name(base, platform)
    for directory in candidate_dirs(repo_root):
        candidate = directory / name
        if candidate.is_file():
            return candidate
    return None


def stage(args: argparse.Namespace) -> None:
    repo_root = args.repo_root.resolve()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    missing: list[str] = []
    copied: list[str] = []
    for base in RUNNERS:
        source = find_runner(repo_root, args.platform, base)
        if source is None:
            missing.append(runner_name(base, args.platform))
            continue
        destination = output_dir / source.name
        shutil.copy2(source, destination)
        if not args.platform.startswith("windows-"):
            destination.chmod(
                destination.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH
            )
        copied.append(str(destination))
    if missing and args.required:
        raise SystemExit(f"missing native runner binaries: {', '.join(missing)}")
    for path in copied:
        print(path)


def main() -> None:
    parser = argparse.ArgumentParser(description="Stage Trapo native OCR runner wrappers.")
    parser.add_argument("--repo-root", type=Path, default=REPO_ROOT)
    parser.add_argument("--platform", required=True)
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=REPO_ROOT / "thirdparty" / "llama.cpp" / "build" / "bin",
    )
    parser.add_argument("--required", action="store_true")
    stage(parser.parse_args())


if __name__ == "__main__":
    main()
